use std::{path::Path, process, sync::mpsc::Sender, time::Duration};

use anyhow::Context;
use sysinfo::{System, SystemExt};
use tokio::time::sleep;

use crate::{downloader::DownloadContext, gamejoin::GamejoinContext, manifest::ProjectManifest};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    CheckingForUpdates,
    DownloadingClient,
    PreparingFiles,
    LaunchingGame,
}

impl ToString for Message {
    fn to_string(&self) -> String {
        let str = match &self {
            Message::CheckingForUpdates => "CheckingForUpdates",
            Message::DownloadingClient => "DownloadingClient",
            Message::PreparingFiles => "PreparingFiles",
            Message::LaunchingGame => "LaunchingGame",
        };

        str.to_string()
    }
}

/// Starts asynchronously working through bootstrapper steps and passes messages to the UI as tasks are completed or
/// updated.
#[tokio::main]
pub async fn initiate_application_tasks(
    client_dir: &Path,
    output_tx: Sender<Message>,
    manifest: &ProjectManifest,
) -> anyhow::Result<()> {
    log::info!("Initiated async application tasks");

    let mut download_context =
        DownloadContext::new().context("Failed to construct DownloadContext")?;
    let gamejoin_context = GamejoinContext::new().context("Failed to construct GamejoinContext")?;

    log::info!("Checking for updates");
    output_tx.send(Message::CheckingForUpdates)?;

    let download_required = download_context
        .require_client_download()
        .await
        .context("Failed to check if download is required")?;

    if download_required {
        log::info!("Updating client");
        output_tx.send(Message::DownloadingClient)?;

        download_context
            .initiate_client_download(client_dir)
            .await
            .context("Failed to update client")?;
    }

    log::info!("Loading game");
    output_tx.send(Message::LaunchingGame)?;

    // Launch the game!
    let place_id = &manifest.game.place_id;
    gamejoin_context
        // FIXME: This is coupled to MacOS.
        .launch_roblox_client(place_id, &client_dir.join("RobloxPlayer.app"))
        .await
        .context("Failed to launch Roblox client")?;

    // Wait until the Roblox player has started and exit this process
    while System::new_all()
        .processes_by_name("Roblox")
        .next()
        .is_none()
    {
        log::trace!("Roblox client has not started yet");
    }

    // Roblox player has started, exit the launcher after some delay
    log::info!("Roblox started, closing launcher");

    sleep(Duration::from_secs(2)).await;
    process::exit(0);
}
