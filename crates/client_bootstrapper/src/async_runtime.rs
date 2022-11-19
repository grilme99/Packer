use std::{path::Path, sync::mpsc::Sender};

use anyhow::Context;

use crate::{downloader::DownloadContext, gamejoin::GamejoinContext};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    CheckingForUpdates,
    UpdatingClient,
    LoadingGame,
}

impl ToString for Message {
    fn to_string(&self) -> String {
        let str = match &self {
            Message::CheckingForUpdates => "CheckingForUpdates",
            Message::LoadingGame => "LoadingGame",
            Message::UpdatingClient => "UpdatingClient",
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
) -> anyhow::Result<()> {
    log::info!("Initiated async application tasks");

    let mut download_context =
        DownloadContext::new().context("Failed to construct DownloadContext")?;
    let _gamejoin_context =
        GamejoinContext::new().context("Failed to construct GamejoinContext")?;

    log::info!("Checking for updates");
    output_tx.send(Message::CheckingForUpdates)?;

    let download_required = download_context
        .require_client_download()
        .await
        .context("Failed to check if download is required")?;

    if download_required {
        log::info!("Updating client");
        output_tx.send(Message::UpdatingClient)?;

        download_context
            .initiate_client_download(client_dir)
            .await
            .context("Failed to update client")?;
    }

    log::info!("Loading game");
    output_tx.send(Message::LoadingGame)?;

    Ok(())
}
