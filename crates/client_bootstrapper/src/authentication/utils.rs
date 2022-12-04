use std::path::PathBuf;

use anyhow::Context;

const BINARY_COOKIES_NAME: &str = "packer.binarycookies";

#[cfg(target_os = "macos")]
pub fn get_cookie_storage_path() -> anyhow::Result<PathBuf> {
    let home_dir = dirs::home_dir().context("Failed to get home dir")?;

    let cookie_storage = home_dir.join(format!("Library/HTTPStorages/{BINARY_COOKIES_NAME}"));

    Ok(cookie_storage)
}

#[cfg(target_os = "windows")]
pub fn get_cookie_storage_path() -> anyhow::Result<PathBuf> {
    todo!()
}
