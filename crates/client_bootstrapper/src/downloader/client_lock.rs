use std::{fs, path::Path};

use anyhow::Context;
use deploy_history::client_version_info::ClientVersionInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientLock {
    pub version: ClientVersionInfo,
}

impl ClientLock {
    pub fn get(root_dir: &Path) -> anyhow::Result<ClientLock> {
        let lock_path = root_dir.join("client/client.lock");

        let lock = fs::read_to_string(lock_path).context("Failed to read client.lock")?;
        let lock = toml::from_str::<ClientLock>(&lock)
            .context("Failed to parse client.lock to ClientLock format")?;

        Ok(lock)
    }

    pub fn write_lock_to_path(&self, lock: &ClientLock, root_dir: &Path) -> anyhow::Result<()> {
        let lock_path = root_dir.join("client/client.lock");

        let lock =
            toml::to_string_pretty(lock).context("Failed to convert ClientLock to string")?;

        fs::write(lock_path, lock).context("Failed to write client.lock")?;

        Ok(())
    }
}
