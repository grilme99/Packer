use std::{
    fs::{self, canonicalize},
    path::Path,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
};

use anyhow::Context;
use reqwest::header::{HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE};
use wry::{
    application::{
        accelerator::Accelerator,
        dpi::{LogicalSize, PhysicalPosition},
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        keyboard::{KeyCode, ModifiersState},
        menu::{MenuBar, MenuItemAttributes},
        platform::macos::WindowBuilderExtMacOS,
        window::WindowBuilder,
    },
    http::Response,
    webview::{WebContext, WebViewBuilder},
};

use crate::{async_runtime::Message, manifest::ProjectManifest};

#[derive(Debug)]
enum UserEvent {}

pub struct Application<'a> {
    manifest: &'a ProjectManifest,
    root_path: &'a Path,
}

impl<'a> Application<'a> {
    pub fn new(root_path: &'a Path, manifest: &'a ProjectManifest) -> anyhow::Result<Self> {
        Ok(Self {
            manifest,
            root_path,
        })
    }

    /// Creates the event loop and runs the application.
    ///
    /// **WARNING**: This will consume the thread it is called from.
    pub fn run(
        &self,
        _input_tx: Sender<Message>,
        output_rx: Receiver<Message>,
    ) -> anyhow::Result<()> {
        let event_loop = EventLoop::<UserEvent>::with_user_event();

        let game_name = self.manifest.game.name.to_owned();
        let assets_path = self.root_path.join("assets");

        let current_task = Arc::new(Mutex::new(Message::CheckingForUpdates));

        let mut menu = MenuBar::new();
        let mut file_menu = MenuBar::new();
        file_menu.add_item(
            MenuItemAttributes::new(format!("Quit {game_name}").as_str()).with_accelerators(
                &Accelerator::new(Some(ModifiersState::SUPER), KeyCode::KeyQ),
            ),
        );
        menu.add_submenu("File", true, file_menu);

        let window_width = self.manifest.design.width;
        let window_height = self.manifest.design.height;

        let window = WindowBuilder::new()
            .with_always_on_top(true)
            .with_decorations(false)
            .with_has_shadow(true)
            .with_movable_by_window_background(true)
            .with_inner_size(LogicalSize::new(window_width, window_height))
            .with_max_inner_size(LogicalSize::new(window_width, window_height))
            .with_menu(menu)
            // There are actually three layer of background color when creating WebView window.
            // The first is window background...
            .with_transparent(true)
            .build(&event_loop)
            .context("Failed to build window")?;

        let mut web_context = WebContext::new(None);

        let current_task2 = Arc::clone(&current_task);
        let webview = WebViewBuilder::new(window)?
            .with_web_context(&mut web_context)
            // The second is on webview...
            .with_transparent(true)
            // And the last is in html.
            .with_url("bootstrapper://assets/bootstrapper.html")?
            .with_hotkeys_zoom(false)
            .with_clipboard(true)
            .with_accept_first_mouse(true)
            .with_custom_protocol("bootstrapper".into(), move |request| {
                let name = &request.uri().path()[1..];

                if name == "current_task" {
                    let current_task = current_task2.lock().unwrap();
                    let current_task = current_task.to_string();

                    return Response::builder()
                        .header(CONTENT_TYPE, "text/plain")
                        .header("x-current-task", current_task.as_str())
                        .body(vec![])
                        .map_err(Into::into);
                }

                let path = assets_path.join(name);
                let path = canonicalize(path)?;

                // TODO: Come up with a better way of handling errors in this closure.
                let content_type = mime_guess::from_path(&path).first().expect("mime type");

                Response::builder()
                    .header(CONTENT_TYPE, content_type.to_string())
                    .header(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"))
                    .body(fs::read(&path).unwrap())
                    .map_err(Into::into)
            })
            .build()
            .context("Failed to build webview")?;

        let current_task3 = Arc::new(current_task);
        event_loop.run(move |event, _, control_flow| {
            // We need to poll at a constant rate so that we can poll for messages from the async thread without waiting
            // for user interaction.
            *control_flow = ControlFlow::Poll;

            let window = webview.window();

            // Poll for messages from the async thread
            if let Ok(message) = output_rx.try_recv() {
                log::debug!("Got message from async thread: {message:?}");
                *current_task3.lock().unwrap() = message;
            }

            match event {
                Event::NewEvents(StartCause::Init) => {
                    window.set_title(&game_name);
                    window.set_resizable(false);

                    #[cfg(feature = "devtools")]
                    webview.open_devtools();

                    // Center the window
                    let monitor = window.current_monitor().expect("current monitor");
                    let size = monitor.size();

                    log::debug!(
                        "Current monitor is {:?} with size {}x{}",
                        monitor.name(),
                        size.width,
                        size.height
                    );

                    window.set_outer_position(PhysicalPosition::new(
                        (size.width / 2) - (window_width),
                        (size.height / 2) - (window_height),
                    ));

                    log::info!("Application started successfully");
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::MouseInput { .. } => window.drag_window().unwrap(),
                    _ => (),
                },
                Event::MenuEvent { .. } => {
                    // There is only one menu item, so we can safely close without checking which menu this came from.
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            }
        });
    }

    // /// Checks if a new client download is required. See [`DownloadContext`].
    // pub async fn is_client_download_required(&mut self) -> anyhow::Result<bool> {
    //     self.download_context.require_client_download().await
    // }

    // /// Start downloading the client!
    // pub async fn initiate_client_download(&mut self, write_to: &PathBuf) -> anyhow::Result<()> {
    //     self.download_context
    //         .initiate_client_download(write_to)
    //         .await?;

    //     Ok(())
    // }

    // pub async fn launch_roblox_client(
    //     &self,
    //     place_id: &u64,
    //     client_root: &Path,
    // ) -> anyhow::Result<()> {
    //     self.gamejoin_context
    //         .launch_roblox_client(place_id, client_root)
    //         .await
    // }
}
