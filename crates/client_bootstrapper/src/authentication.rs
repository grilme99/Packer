use anyhow::bail;
use cookie::Cookie;
use secrecy::{ExposeSecret, SecretString};

/// Stores state about Roblox account authentication and handles all behavior around getting authenticated.
#[derive(Debug)]
pub struct AuthenticationContext {
    pub roblosecurity: Option<SecretString>,
}

impl AuthenticationContext {
    pub fn new() -> Self {
        let roblosecurity = Self::get_roblosecurity();

        if roblosecurity.is_some() {
            log::info!("Found existing Roblox authentication");
        } else {
            log::info!("Could not find existing Roblox authentication");
        }

        Self { roblosecurity }
    }

    fn get_roblosecurity() -> Option<SecretString> {
        // rbx_cookie is a crate which attempts to retrieve the `.ROBLOSECURITY` cookie from various places,
        // depending on the underlying OS.
        if let Some(cookie) = rbx_cookie::get_value() {
            Some(SecretString::new(cookie))
        } else {
            None
        }
    }

    pub fn get_roblosecurity_cookie(&self) -> anyhow::Result<Cookie> {
        if let Some(cookie_str) = &self.roblosecurity {
            let cookie = Cookie::build(".ROBLOSECURITY", cookie_str.expose_secret())
                .domain(".roblox.com")
                .finish();

            Ok(cookie)
        } else {
            bail!("No existing authentication found");
        }
    }
}
