use std::{env, fs, path::PathBuf};

use anyhow::Context;
use deploy_history::client_version_info::ClientVersionInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientLock {
    pub version: ClientVersionInfo,
}

impl ClientLock {
    pub fn get() -> anyhow::Result<ClientLock> {
        let lock_path = get_lock_path().context("Failed to get lock path")?;

        let lock = fs::read_to_string(lock_path).context("Failed to read client.lock")?;
        let lock = toml::from_str::<ClientLock>(&lock)
            .context("Failed to parse client.lock to ClientLock format")?;

        Ok(lock)
    }

    pub fn write_lock_to_path(&self, lock: &ClientLock) -> anyhow::Result<()> {
        let lock_path = get_lock_path().context("Failed to get lock path")?;

        let lock =
            toml::to_string_pretty(lock).context("Failed to convert ClientLock to string")?;

        fs::write(lock_path, lock).context("Failed to write client.lock")?;

        Ok(())
    }
}

fn get_lock_path() -> anyhow::Result<PathBuf> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let lock_path = current_dir.join("client.lock");

    Ok(lock_path)
}
