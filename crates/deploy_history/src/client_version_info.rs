use anyhow::Context;
use reqwest::Client;
use serde::Deserialize;

use crate::{deploy_log::DeployLog, domain::BinaryType, domain::Channel};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClientVersionResponse {
    version: String,
    client_version_upload: String,
}

pub struct ClientVersionInfo {
    pub channel: Channel,
    pub version: String,
    pub version_guid: String,
}

impl ClientVersionInfo {
    pub fn new(channel: Channel, version: String, version_guid: String) -> Self {
        Self {
            channel,
            version,
            version_guid,
        }
    }

    pub fn from_deploy_log(log: DeployLog) -> Self {
        Self {
            channel: log.channel,
            version: log.version_id(),
            version_guid: log.version_guid,
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

        Ok(Self {
            channel: channel.to_owned(),
            version: response.version,
            version_guid: response.client_version_upload,
        })
    }
}
