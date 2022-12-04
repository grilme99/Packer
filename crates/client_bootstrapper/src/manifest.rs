use std::{fs, path::Path, process, thread};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectManifest {
    pub game: GameConfig,
    pub design: DesignConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameConfig {
    pub name: String,
    pub place_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DesignConfig {
    pub width: u32,
    pub height: u32,
}

impl ProjectManifest {
    pub fn get(root_dir: &Path) -> anyhow::Result<ProjectManifest> {
        let manifest_path = root_dir.join("manifest.toml");

        let manifest = fs::read_to_string(&manifest_path).context(format!(
            "Failed to read manifest.toml at path {manifest_path:?}"
        ))?;

        let manifest = toml::from_str::<ProjectManifest>(&manifest)
            .context("Failed to parse manifest.toml to Manifest format")?;

        Ok(manifest)
    }
}

impl Drop for ProjectManifest {
    fn drop(&mut self) {
        log::debug!("ProjectManifest dropped");

        if thread::panicking() {
            log::error!("ProjectManifest dropped while thread was unwinding from panic. This is most likely caused by the async thread panicking.");
            process::exit(1);
        }
    }
}
