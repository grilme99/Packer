use reqwest::Client;

/// Stateful object that handles the actual downloading of the Roblox client.
///
/// Tracks and reports progress of any asynchronous download tasks.
#[derive(Debug)]
pub struct DownloaderContext {
    client: Client,
}

impl DownloaderContext {
    pub fn new() -> Self {
        let client = Client::default();

        Self { client }
    }
}
