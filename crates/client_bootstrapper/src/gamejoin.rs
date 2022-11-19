use std::{path::Path, process::Command};

use anyhow::{bail, Context};
use rand::{thread_rng, Rng};
use reqwest::{
    header::{HeaderMap, REFERER},
    Client,
};

use crate::authentication::AuthenticationContext;

/// Handles everything around negotiating the game joining process with Roblox (getting an authentication
/// ticket, etc).
#[derive(Debug)]
pub struct GamejoinContext {
    client: Client,
    auth_context: AuthenticationContext,
}

impl GamejoinContext {
    pub fn new() -> anyhow::Result<Self> {
        let auth_context = AuthenticationContext::new();

        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36")
            .referer(false)
            .build()?;

        Ok(Self {
            client,
            auth_context,
        })
    }

    /// Launch the game client into the specified experience!
    pub async fn launch_roblox_client(
        &self,
        place_id: &u64,
        client_root: &Path,
    ) -> anyhow::Result<()> {
        // FIXME: Assumes we're on MacOS.
        let roblox_player = client_root.join("Contents/MacOS/RobloxPlayer");

        if !roblox_player.exists() {
            bail!("Can't launch the Roblox client because the player does not exist at {roblox_player:?}");
        }

        if !roblox_player.is_file() {
            bail!("Can't launch the Roblox client because the player is not a file ({roblox_player:?})");
        }

        log::debug!("Found RobloxPlayer at: {roblox_player:?}");

        let application_args = self
            .generate_application_args(place_id)
            .await
            .context("Failed to generate application args")?;

        Command::new(roblox_player)
            .args(application_args)
            .output()
            .context("Failed to launch RobloxPlayer")?;

        Ok(())
    }

    /// Generate the arguments passed to the Roblox Player that tell it how to start.
    pub async fn generate_application_args(&self, place_id: &u64) -> anyhow::Result<Vec<String>> {
        let authentication_ticket = self
            .create_authentication_ticket()
            .await
            .context("Failed to create authentication ticket")?;

        // The tracker ID doesn't really matter, it just needs to be somewhat random because constant values
        // could trip up the client.
        let tracker_id = thread_rng().gen_range(100_000_000..999_999_999).to_string();

        let script_url = format!("https://assetgame.roblox.com/game/PlaceLauncher.ashx?request=RequestGame&browserTrackerId={tracker_id}&placeId={place_id}&isPlayTogetherGame=false");

        #[rustfmt::skip]
        let application_args: Vec<String> = vec![
            "-ticket".into(), authentication_ticket,
            "-scriptURL".into(), script_url,
            "-browserTrackerId".into(), tracker_id,
            "-rloc".into(), "en_us".into(),
            "-gloc".into(), "en_us".into(),
            "-launchExp".into(), "InApp".into(), // launchExp doesn't do anything anymore. Still required.
        ];

        // TODO-Security: Application args contains auth ticket, is this safe to log?
        // log::debug!(
        //     "Generated application args:\n{}",
        //     application_args.join("\n")
        // );

        Ok(application_args)
    }

    /// An authentication ticket is required for initiating the gamejoin process (handled by the Roblox client).
    pub async fn create_authentication_ticket(&self) -> anyhow::Result<String> {
        let auth_cookie = self
            .auth_context
            .get_roblosecurity_cookie()
            .context("Failed to get authentication cookie")?
            .to_string();

        let mut headers = HeaderMap::new();
        headers.insert(REFERER, "https://www.roblox.com".try_into()?);
        headers.insert(reqwest::header::CONTENT_LENGTH, 0.into());
        headers.insert("Cookie", auth_cookie.try_into()?);

        // First we need to get a X-CSRF token
        let csrf_response = self
            .client
            .post("https://auth.roblox.com/v2/logout")
            .headers(headers.clone())
            .send()
            .await
            .context("Failed to send CSRF token request")?;

        let csrf_token = csrf_response
            .headers()
            .get("x-csrf-token")
            .context("Response does not include an X-CSRF token")?;

        log::debug!("Got X-CSRF token: {csrf_token:?}");
        headers.insert("x-csrf-token", csrf_token.to_owned());

        let ticket_response = self
            .client
            .post("https://auth.roblox.com/v1/authentication-ticket")
            .headers(headers)
            .send()
            .await
            .context("Failed to send request for authentication ticket")?;

        let auth_ticket = ticket_response
            .headers()
            .get("rbx-authentication-ticket")
            .context("Response does not include rbx-authentication-ticket")?
            .to_str()
            .context("Could not parse authentication ticket to string slice")?;

        // TODO-Security: Is it safe to log this? It's useful for debugging everything is correct.
        // log::debug!("Got authentication ticket: {auth_ticket}");

        Ok(auth_ticket.to_owned())
    }
}
