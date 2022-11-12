use anyhow::Context;
use async_trait::async_trait;
use deploy_history::{
    client_version_info::ClientVersionInfo,
    domain::{BinaryType, Channel},
};
use reqwest::Client;

use super::ClientDownloader;

pub struct WindowsDownloader;

#[async_trait]
impl ClientDownloader for WindowsDownloader {
    async fn get_latest_client_version(client: &Client) -> anyhow::Result<ClientVersionInfo> {
        let version_info =
            ClientVersionInfo::get(client, &Channel::Live, &BinaryType::WindowsPlayer)
                .await
                .context("Failed to get latest version info")?;

        Ok(version_info)
    }

    /// In the case of Mac, we already know the download paths beforehand because it's only two files.
    /// Still, we need the version info to generate the paths.
    async fn get_file_download_paths(
        _client: &Client,
        _version_info: &ClientVersionInfo,
    ) -> anyhow::Result<Vec<String>> {
        todo!("Windows support is a WIP.");
    }
}
