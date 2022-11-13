use std::path::{Path, PathBuf};

use anyhow::Context;
use reqwest::Client;

use crate::{downloader::DownloadContext, gamejoin::GamejoinContext};

use self::manifest::ProjectManifest;

pub mod manifest;

#[derive(Debug)]
pub struct Project<'a> {
    pub manifest: ProjectManifest,
    download_context: DownloadContext<'a>,
    gamejoin_context: GamejoinContext<'a>,
}

impl<'a> Project<'a> {
    pub fn new(client: &'a Client) -> anyhow::Result<Self> {
        let download_context = DownloadContext::new(client);
        let gamejoin_context = GamejoinContext::new(client);

        let manifest = ProjectManifest::get().context("Failed to get manifest contents")?;

        Ok(Self {
            manifest,
            download_context,
            gamejoin_context,
        })
    }

    /// Checks if a new client download is required. See [`DownloadContext`].
    pub async fn is_client_download_required(&mut self) -> anyhow::Result<bool> {
        self.download_context.require_client_download().await
    }

    /// Start downloading the client!
    pub async fn initiate_client_download(&mut self, write_to: &PathBuf) -> anyhow::Result<()> {
        self.download_context
            .initiate_client_download(write_to)
            .await?;

        Ok(())
    }

    pub async fn launch_roblox_client(
        &self,
        place_id: &u64,
        client_root: &Path,
    ) -> anyhow::Result<()> {
        self.gamejoin_context
            .launch_roblox_client(place_id, client_root)
            .await
    }
}
