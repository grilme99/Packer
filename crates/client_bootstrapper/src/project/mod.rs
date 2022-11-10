use anyhow::Context;
use reqwest::Client;

use crate::downloader::DownloadContext;

use self::manifest::ProjectManifest;

pub mod manifest;

#[derive(Debug)]
pub struct Project<'a> {
    pub manifest: ProjectManifest,
    download_context: DownloadContext<'a>,
    client: &'a Client,
}

impl<'a> Project<'a> {
    pub fn new(client: &'a Client) -> anyhow::Result<Self> {
        let download_context = DownloadContext::new(client);

        let manifest = ProjectManifest::get().context("Failed to get manifest contents")?;

        Ok(Self {
            manifest,
            client,
            download_context,
        })
    }

    /// Checks if a new client download is required. See [`DownloadContext`].
    pub async fn is_client_download_required(&mut self) -> anyhow::Result<bool> {
        self.download_context.require_client_download().await
    }

    /// Start downloading the client!
    pub async fn initiate_client_download(&mut self) -> anyhow::Result<()> {
        self.download_context
            .update_client_lock()
            .await
            .context("Failed to update client lock")?;

        Ok(())
    }
}
