use std::path::PathBuf;

use async_trait::async_trait;
use deploy_history::client_version_info::ClientVersionInfo;
use reqwest::Client;

#[cfg(target_os = "windows")]
pub use windows::WindowsDownloader as Downloader;

#[cfg(target_os = "macos")]
pub use macos::MacDownloader as Downloader;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

mod util;

/// Structure for OS-specific client downloaders.
#[async_trait]
pub trait ClientDownloader {
    async fn get_latest_client_version(client: &Client) -> anyhow::Result<ClientVersionInfo>;

    async fn get_file_download_paths(
        client: &Client,
        version_info: &ClientVersionInfo,
    ) -> anyhow::Result<Vec<String>>;

    async fn download_files_and_write_to_path(
        client: &Client,
        download_paths: Vec<String>,
        write_to: &PathBuf,
    ) -> anyhow::Result<()>;
}
