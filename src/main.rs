use anyhow::Context;
use client_bootstrapper::project::Project;
use env_logger::Env;
use reqwest::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = Env::default().filter_or("RUST_LOG", "debug");
    env_logger::init_from_env(env);

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36")
        .referer(false)
        .build()?;

    let mut project = Project::new(&client).context("Failed to create project")?;

    let download_required = project
        .is_client_download_required()
        .await
        .context("Failed to check if download is required")?;

    log::info!("Download required: {download_required}");

    let root = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("client/");
    log::info!("Root directory: {root:?}");

    if download_required {
        project
            .initiate_client_download(&root)
            .await
            .context("Failed to initiate client download")?;
    }

    project
        .launch_roblox_client(&4483381587, &root.join("RobloxPlayer.app"))
        .await
        .context("Failed to launch Roblox client")?;

    Ok(())
}
