use anyhow::Context;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{domain::BinaryType, domain::Channel};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClientVersionResponse {
    version: String,
    client_version_upload: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientVersionInfo {
    pub channel: Channel,
    pub version_guid: String,

    pub major_rev: usize,
    pub version: usize,
    pub patch: usize,
    pub change_list: usize,
}

impl ClientVersionInfo {
    pub fn new(channel: Channel, version: String, version_guid: String) -> Self {
        let (major_rev, version, patch, change_list) = parts_from_version(&version);

        Self {
            channel,
            version_guid,

            major_rev,
            version,
            patch,
            change_list,
        }
    }

    pub async fn get(
        client: &Client,
        channel: &Channel,
        binary_type: &BinaryType,
    ) -> anyhow::Result<Self> {
        let response = client
            .get(format!(
                "https://clientsettings.roblox.com/v2/client-version/{}/channel/{}",
                binary_type.to_string(),
                channel.to_string()
            ))
            .send()
            .await
            .context("Failed to send request for client version info")?
            .json::<ClientVersionResponse>()
            .await
            .context("Failed to parse response for client version info into JSON")?;

        let (major_rev, version, patch, change_list) = parts_from_version(&response.version);

        Ok(Self {
            channel: channel.to_owned(),
            version_guid: response.client_version_upload,

            major_rev,
            version,
            patch,
            change_list,
        })
    }
}

fn parts_from_version(version: &str) -> (usize, usize, usize, usize) {
    let mut version_parts = version.split(".");
    let major_rev = version_parts.nth(0).expect("major rev").parse().unwrap();
    let version = version_parts.nth(0).expect("major rev").parse().unwrap();
    let patch = version_parts.nth(0).expect("major rev").parse().unwrap();
    let change_list = version_parts.nth(0).expect("major rev").parse().unwrap();

    (major_rev, version, patch, change_list)
}
