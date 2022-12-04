use std::{process, thread};

use anyhow::{bail, Context};
use client_bootstrapper::{
    application::Application, async_runtime::initiate_application_tasks, manifest::ProjectManifest,
};

use libpacker::{logging::init_logging, util::get_root_directory};
fn main() -> anyhow::Result<()> {
    let root_directory = get_root_directory().context("Failed to get current directory")?;
    if !root_directory.exists() {
        bail!("Root directory does not exist: {root_directory:?}")
    }

    init_logging(&root_directory).context("Failed to initiate logging")?;

    log::info!("Preparing bootstrapper");
    log::info!("Root directory: {root_directory:?}");

    let manifest =
        ProjectManifest::get(&root_directory).context("Failed to get project manifest")?;

    let (async_thread_send, async_thread_receive) = crossbeam::channel::unbounded();

    // Drive the async thread
    let client_dir = root_directory.to_owned();

    let async_manifest = manifest.clone();
    thread::spawn(move || {
        if let Err(e) = initiate_application_tasks(&client_dir, async_thread_send, &async_manifest)
        {
            // Async thread errored, report error and exit application
            log::error!("Async thread error:\n{e:?}");
            process::exit(1);
        }
    });

    let application =
        Application::new(&root_directory, &manifest).context("Failed to create project")?;

    application
        .run(async_thread_receive)
        .context("Failed to create and run application")?;

    // Nothing can run beyond this point. The main thread is consumed by the event loop.
    // The event loop is required to run on the main thread on MacOS, so cannot be moved to another thread.

    Ok(())
}
