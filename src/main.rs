use anyhow::Context;
use client_bootstrapper::project::Project;
use env_logger::Env;
use reqwest::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = Env::default().filter_or("RUST_LOG", "debug");
    env_logger::init_from_env(env);

    let client = Client::new();
    let mut project = Project::new(&client).context("Failed to create project")?;

    let download_required = project
        .is_client_download_required()
        .await
        .context("Failed to check if download is required")?;

    log::info!("Download required: {download_required}");
    if download_required {
        project
            .initiate_client_download()
            .await
            .context("Failed to initiate client download")?;
    }

    Ok(())
}
