use std::{path::Path, process, time::Duration};

use anyhow::{bail, Context};
use crossbeam::channel::{Receiver, Sender};
use sysinfo::{System, SystemExt};
use tokio::time::sleep;

use crate::{
    authentication::AuthenticationContext, downloader::DownloadContext, gamejoin::GamejoinContext,
    manifest::ProjectManifest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    CheckingForUpdates,
    DownloadingClient,
    PreparingFiles,
    LaunchingGame,

    PromptForAuth,
    AuthCompleted,
}

impl ToString for Message {
    fn to_string(&self) -> String {
        let str = match &self {
            Message::CheckingForUpdates => "CheckingForUpdates",
            Message::DownloadingClient => "DownloadingClient",
            Message::PreparingFiles => "PreparingFiles",
            Message::LaunchingGame => "LaunchingGame",
            _ => "N/A",
        };

        str.to_string()
    }
}

/// Starts asynchronously working through bootstrapper steps and passes messages to the UI as tasks are completed or
/// updated.
#[tokio::main]
pub async fn initiate_application_tasks(
    root_dir: &Path,
    manifest: &ProjectManifest,
    async_thread_sender: Sender<Message>,
    application_thread_receiver: Receiver<Message>,
) -> anyhow::Result<()> {
    log::info!("Initiated async application tasks");

    let mut download_context =
        DownloadContext::new(root_dir).context("Failed to construct DownloadContext")?;
    let auth_context = AuthenticationContext::new();
    let gamejoin_context =
        GamejoinContext::new(&auth_context).context("Failed to construct GamejoinContext")?;

    log::info!("Checking for updates");
    async_thread_sender.send(Message::CheckingForUpdates)?;

    let download_required = download_context
        .require_client_download()
        .await
        .context("Failed to check if download is required")?;

    if download_required {
        log::info!("Updating client");
        async_thread_sender.send(Message::DownloadingClient)?;

        download_context
            .initiate_client_download(root_dir)
            .await
            .context("Failed to update client")?;
    }

    // FIXME: This is coupled to MacOS.
    let roblox_player = root_dir.join("client/RobloxPlayer.app/Contents/MacOS/RobloxPlayer");
    if !roblox_player.exists() {
        bail!("Roblox Player does not exist at path: {roblox_player:?}");
    }

    // Once we have a client, make sure authentication is all good
    let already_authenticated = auth_context.already_authenticated();
    if !already_authenticated {
        log::info!("Authentication not available, prompting for auth");

        async_thread_sender.send(Message::PromptForAuth)?;

        // Wait for the application thread to tell us authentication is complete
        while application_thread_receiver
            .recv()
            .is_ok_and(|message| message != Message::AuthCompleted)
        {
            sleep(Duration::from_secs(1)).await;
        }

        log::info!("Got authentication complete message from application");

        // Ensure the cookie exists. We do't actually need it here, though.
        // TODO: Come up with a more robust method of waiting for the cookie to be written to disk.
        sleep(Duration::from_secs(5)).await;
        auth_context
            .get_webview_roblosecurity()
            .context("Error while getting ROBLOSECURITY from WebView")?
            .context("No cookie found in WebView storage")?;
    }

    log::info!("Loading game");
    async_thread_sender.send(Message::LaunchingGame)?;

    // Launch the game!
    let place_id = &manifest.game.place_id;
    gamejoin_context
        .launch_roblox_client(place_id, &roblox_player)
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
