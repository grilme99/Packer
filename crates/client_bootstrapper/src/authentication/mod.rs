use std::fs;

use anyhow::{bail, Context};
use cookie::Cookie;
use secrecy::{ExposeSecret, SecretString};

mod utils;

#[cfg(target_os = "macos")]
mod binarycookies;

static COOKIE_NAME: &str = ".ROBLOSECURITY";

/// Stores state about Roblox account authentication and handles all behavior around getting authenticated.
#[derive(Debug)]
pub struct AuthenticationContext {}

impl AuthenticationContext {
    pub fn new() -> Self {
        Self {}
    }

    pub fn already_authenticated(&self) -> bool {
        self.get_roblosecurity()
            .is_ok_and(|cookie| cookie.is_some())
    }

    pub fn get_roblosecurity(&self) -> anyhow::Result<Option<SecretString>> {
        // rbx_cookie is a crate which attempts to retrieve the `.ROBLOSECURITY` cookie from various places,
        // depending on the underlying OS.
        let cookie = if let Some(cookie) = rbx_cookie::get_value() {
            Some(SecretString::new(cookie))
        } else if let Ok(cookie) = self.get_webview_roblosecurity() {
            cookie
        } else {
            None
        };

        Ok(cookie)
    }

    pub fn get_roblosecurity_cookie(&self) -> anyhow::Result<Cookie> {
        if let Some(cookie_str) = self.get_roblosecurity()? {
            let cookie = Cookie::build(".ROBLOSECURITY", cookie_str.expose_secret().to_owned())
                .domain(".roblox.com")
                .finish();

            Ok(cookie)
        } else {
            bail!("No existing authentication found");
        }
    }

    #[cfg(target_os = "macos")]
    pub fn get_webview_roblosecurity(&self) -> anyhow::Result<Option<SecretString>> {
        let cookies_path =
            utils::get_cookie_storage_path().context("Failed to get cookie storage path")?;

        let binary = fs::read(&cookies_path)
            .context(format!("Failed to read cookies content: {cookies_path:?}"))?;

        let mut cookie_store = binarycookies::Cookies::new(false);
        cookie_store
            .parse_content(&binary)
            .context("Failed to parse binary content")?;

        if let Some(cookie) = cookie_store
            .cookies
            .iter()
            .find(|cookie| cookie.name == COOKIE_NAME)
        {
            log::info!("Loaded WebView ROBLOSECURITY cookie from MacOS HTTPStorages");
            Ok(Some(SecretString::new(cookie.value.to_owned())))
        } else {
            Ok(None)
        }
    }

    #[cfg(target_os = "windows")]
    pub fn get_webview_roblosecurity(&self) -> anyhow::Result<SecretString> {
        todo!()
    }
}
