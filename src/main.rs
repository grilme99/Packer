use std::{sync::mpsc, thread};

use anyhow::Context;
use client_bootstrapper::{application::Application, async_runtime::initiate_application_tasks};
use env_logger::Env;

fn main() -> anyhow::Result<()> {
    let env = Env::default().filter_or("RUST_LOG", "debug");
    env_logger::init_from_env(env);

    let root = std::env::current_dir().unwrap();
    log::info!("Root directory: {root:?}");

    let (async_proc_input_tx, async_proc_input_rx) = mpsc::channel();
    let (async_proc_output_tx, async_proc_output_rx) = mpsc::channel();

    // Drive the async thread
    let client_dir = root.join("client/");

    thread::spawn(move || {
        println!("AAAA");
        initiate_application_tasks(&client_dir, async_proc_output_tx)
            .expect("async thread panicked");
    });

    let application = Application::new(&root).context("Failed to create project")?;
    application
        .run(async_proc_input_tx, async_proc_output_rx)
        .context("Failed to create and run application")?;

    // Nothing can run beyond this point. The main thread is consumed by the event loop.
    // The event loop is required to run on the main thread on MacOS.

    // let download_required = project
    //     .is_client_download_required()
    //     .await
    //     .context("Failed to check if download is required")?;

    // log::info!("Download required: {download_required}");

    // if download_required {
    //     project
    //         .initiate_client_download(&root)
    //         .await
    //         .context("Failed to initiate client download")?;
    // }

    // project
    //     .launch_roblox_client(&4483381587, &root.join("RobloxPlayer.app"))
    //     .await
    //     .context("Failed to launch Roblox client")?;

    Ok(())
}
