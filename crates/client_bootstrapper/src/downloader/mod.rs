use anyhow::Context;
use deploy_history::{
    client_version_info::ClientVersionInfo,
    domain::{BinaryType, Channel},
};
use reqwest::Client;

use self::client_lock::ClientLock;

mod client_lock;

/// Stateful object that handles the actual downloading of the Roblox client.
///
/// Tracks and reports progress of any asynchronous download tasks.
#[derive(Debug)]
pub struct DownloadContext<'a> {
    pub client_lock: Option<ClientLock>,
    client: &'a Client,
    /// Cached latest client version. Saved lazily.
    cached_client_version: Option<ClientVersionInfo>,
}

impl<'a> DownloadContext<'a> {
    pub fn new(client: &'a Client) -> Self {
        // FIXME: Eating the error like this silences any parsing errors which could be helpful.
        let client_lock = ClientLock::get().ok();
        log::debug!("Existing client.lock: {client_lock:?}");

        Self {
            client,
            client_lock,
            cached_client_version: None,
        }
    }

    /// Works out if we need to download a new version of the client.
    /// A new download could be required when:
    ///  1. Could not find an existing client downloaded.
    ///  2. Could not find or parse an existing `client.lock` file.
    ///  3. The client version defined in `client.lock` predates the latest client version.
    ///
    /// Errors out if we failed to get the latest client version.
    pub async fn require_client_download(&mut self) -> anyhow::Result<bool> {
        let latest_version = self
            .get_latest_client_version()
            .await
            .context("Failed to get latest client version")?;

        if let Some(client_lock) = &self.client_lock {
            // FIXME: Roblox version strings don't follow semver rules, which makes comparing
            //  versions a bit of a pain. This check doesn't account for emergency patches or
            //  the `change_list` property (unsure what change_list actually means).
            if client_lock.version.version != latest_version.version {
                return Ok(true);
            }
        } else {
            // No client.lock file, require download. This also includes parse errors.
            return Ok(true);
        }

        Ok(false)
    }

    /// Update the client lock with the currently installed (latest) version of the client.
    /// This should only be called after client installation has completed.
    pub async fn update_client_lock(&mut self) -> anyhow::Result<()> {
        let latest_version = self
            .get_latest_client_version()
            .await
            .context("Failed to get latest client version")?;

        let new_lock = ClientLock {
            version: latest_version,
        };

        new_lock.write_lock_to_path(&new_lock).context("Failed to write new ClientLock to path")?;

        self.client_lock = Some(new_lock);

        log::debug!("Wrote new ClientLock to path");
        Ok(())
    }

    async fn get_latest_client_version(&mut self) -> anyhow::Result<ClientVersionInfo> {
        if let Some(version_info) = &self.cached_client_version {
            log::debug!("Hit cached client version info");
            Ok(version_info.clone()) // FIXME PERF: Don't clone here.
        } else {
            log::debug!("Missed cached client version info");

            // FIXME: Parse the correct `BinaryType`.
            let version_info =
                ClientVersionInfo::get(self.client, &Channel::Live, &BinaryType::MacPlayer)
                    .await
                    .context("Failed to get latest version info")?;

            // FIXME PERF: Don't clone here.
            self.cached_client_version = Some(version_info.clone());

            Ok(version_info)
        }
    }
}
