use std::{env, fs, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectManifest {
    pub game: GameConfig,
    pub branding: BrandingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameConfig {
    pub name: String,
    pub place_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrandingConfig {
    pub bar_color: String,
    pub text_color: String,
}

impl ProjectManifest {
    pub fn get() -> anyhow::Result<ProjectManifest> {
        let manifest_path = get_manifest_path().context("Failed to get manifest path")?;

        let manifest = fs::read_to_string(&manifest_path).context(format!(
            "Failed to read manifest.toml at path {manifest_path:?}"
        ))?;

        let manifest = toml::from_str::<ProjectManifest>(&manifest)
            .context("Failed to parse manifest.toml to Manifest format")?;

        Ok(manifest)
    }
}

fn get_manifest_path() -> anyhow::Result<PathBuf> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let manifest_path = current_dir.join("manifest.toml");

    Ok(manifest_path)
}
