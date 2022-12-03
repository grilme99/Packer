use std::{env, path::PathBuf};

use anyhow::Context;

#[cfg(debug_assertions)]
pub fn get_root_directory() -> anyhow::Result<PathBuf> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    let current_dir = current_dir.canonicalize().context(format!(
        "Failed to canonicalize current executable: {current_dir:?}"
    ))?;

    Ok(current_dir)
}

#[cfg(all(not(debug_assertions), target_os = "windows"))]
pub fn get_root_directory() -> anyhow::Result<PathBuf> {
    todo!("Windows support is not completed")
}

#[cfg(all(not(debug_assertions), target_os = "macos"))]
pub fn get_root_directory() -> anyhow::Result<PathBuf> {
    let current_dir = env::current_exe()
        .context("Failed to get current executable")?
        .join("../../Resources");

    let current_dir = current_dir.canonicalize().context(format!(
        "Failed to canonicalize current executable: {current_dir:?}"
    ))?;

    Ok(current_dir)
}
