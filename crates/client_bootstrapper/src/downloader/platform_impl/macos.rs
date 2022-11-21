use std::{fs, path::Path};

use anyhow::{bail, Context};
use async_trait::async_trait;
use deploy_history::{
    client_version_info::ClientVersionInfo,
    domain::{BinaryType, Channel},
};
use futures::future;
use reqwest::Client;

use super::{util::download_file, ClientDownloader};

/// Mac has its own CDN path compared to Windows, so for now we'll just hardcode this.
const CDN_PATH: &str = "https://setup.rbxcdn.com/mac";

/// `Roblox.zip` is only the bootstrapper, so we want `RobloxPlayer.zip`.
const PLAYER_FILE: &str = "RobloxPlayer.zip";

const TARGET_CONCURRENT_DOWNLOADS: u32 = 10;

pub struct MacDownloader;

#[async_trait]
impl ClientDownloader for MacDownloader {
    async fn get_latest_client_version(client: &Client) -> anyhow::Result<ClientVersionInfo> {
        let version_info = ClientVersionInfo::get(client, &Channel::Live, &BinaryType::MacPlayer)
            .await
            .context("Failed to get latest version info")?;

        Ok(version_info)
    }

    /// In the case of Mac, we already know the download paths beforehand because it's only two files.
    /// Still, we need the version info to generate the paths.
    async fn get_file_download_paths(
        _client: &Client,
        version_info: &ClientVersionInfo,
    ) -> anyhow::Result<Vec<String>> {
        let download_path = format!("{CDN_PATH}/{}-{PLAYER_FILE}", version_info.version_guid);
        Ok(vec![download_path])
    }

    async fn download_files_and_write_to_path(
        client: &Client,
        download_paths: Vec<String>,
        write_to: &Path,
    ) -> anyhow::Result<()> {
        // Clear out any old client files that may exist if we're updating.
        if write_to.exists() {
            fs::remove_dir_all(write_to)
                .context(format!("Failed to delete client directory: {write_to:?}"))?
        }

        fs::create_dir_all(write_to)
            .context(format!("Failed to create client directory: {write_to:?}"))?;

        let temp_dir = write_to.join("temp/");
        fs::create_dir(&temp_dir).context("Failed to create temp directory")?;

        let mut download_tasks = Vec::new();
        for download_path in &download_paths {
            download_tasks.push(download_file(
                client,
                download_path,
                &temp_dir,
                TARGET_CONCURRENT_DOWNLOADS,
            ))
        }

        let client_files = future::try_join_all(download_tasks)
            .await
            .context("Failed to download one or more client files")?;

        // In the case of Mac, there should only be one file downloaded.
        if client_files.len() != 1 {
            bail!(
                "Expected 1 client file to be downloaded for Mac, got {}",
                client_files.len()
            );
        }

        let player_path = temp_dir.join("RobloxPlayer.app");
        if !player_path.exists() {
            bail!("Roblox Player could not be found at: {player_path:?}");
        }

        log::info!("Got Roblox Player: {player_path:?}");

        // Move the player from the temp dir into the client dir
        fs::rename(player_path, write_to.join("RobloxPlayer.app"))
            .context("Failed to move RobloxPlayer.app out of temp/ dir")?;

        // Delete the temp directory and everything in it
        fs::remove_dir_all(&temp_dir).context("Failed to delete temp/ directory")?;

        log::info!("Cleaned up temp/ directory");

        Ok(())
    }
}
