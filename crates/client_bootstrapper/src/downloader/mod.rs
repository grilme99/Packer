use std::path::Path;

use anyhow::Context;
use deploy_history::client_version_info::ClientVersionInfo;
use reqwest::Client;

use crate::downloader::platform_impl::ClientDownloader;

use self::client_lock::ClientLock;
use self::platform_impl::Downloader;

mod client_lock;
mod platform_impl;

/// Stateful object that handles the actual downloading of the Roblox client.
///
/// Tracks and reports progress of any asynchronous download tasks.
#[derive(Debug)]
pub struct DownloadContext {
    pub client_lock: Option<ClientLock>,
    client: Client,
    /// Cached latest client version. Saved lazily.
    cached_client_version: Option<ClientVersionInfo>,
}

impl DownloadContext {
    pub fn new(root_dir: &Path) -> anyhow::Result<Self> {
        // FIXME: Eating the error like this silences any parsing errors which could be helpful.
        let client_lock = ClientLock::get(root_dir).ok();
        log::debug!("Existing client.lock: {client_lock:?}");

        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36")
            .referer(false)
            .build()?;

        Ok(Self {
            client,
            client_lock,
            cached_client_version: None,
        })
    }

    /// Start downloading the client! This mostly branches out to OS-specific download
    /// implementations because Roblox packages the client up different for Windows and Mac.
    pub async fn initiate_client_download(
        &mut self,
        root_dir: &Path,
    ) -> anyhow::Result<()> {
        let write_to = root_dir.join("client");

        let latest_version = self
            .get_latest_client_version()
            .await
            .context("Failed to get latest client version")?;

        let download_paths = Downloader::get_file_download_paths(&self.client, &latest_version)
            .await
            .context("Failed to get client download paths")?;

        log::debug!("Got download paths:\n{}", download_paths.join(",\n"));

        Downloader::download_files_and_write_to_path(&self.client, download_paths, &write_to)
            .await
            .context("Failed to download files or write to path")?;

        self.update_client_lock(root_dir)
            .await
            .context("Failed to update client.lock")?;

        Ok(())
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
            // Roblox version strings don't follow semver rules, which makes comparing
            // versions a bit of a pain. This is probably the most robust way to do it.
            // Will also catch cases where we're somehow ahead of the latest client
            // version (maybe we downloaded a test branch?).
            let lock_version = &client_lock.version;
            if lock_version.version != latest_version.version
                || lock_version.patch != latest_version.patch
                || lock_version.change_list != latest_version.change_list
            {
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
    pub async fn update_client_lock(&mut self, root_dir: &Path) -> anyhow::Result<()> {
        let latest_version = self
            .get_latest_client_version()
            .await
            .context("Failed to get latest client version")?;

        let new_lock = ClientLock {
            version: latest_version,
        };

        new_lock
            .write_lock_to_path(&new_lock, root_dir)
            .context("Failed to write new ClientLock to path")?;

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
            let version_info = Downloader::get_latest_client_version(&self.client)
                .await
                .context("Failed to get latest client version")?;

            // FIXME PERF: Don't clone here.
            self.cached_client_version = Some(version_info.clone());

            Ok(version_info)
        }
    }
}
