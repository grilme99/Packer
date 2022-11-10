use anyhow::Context;
use client_version_info::ClientVersionInfo;
use deploy_log::DeployLog;
use domain::{BinaryType, Channel};
use regex::Regex;
use reqwest::Client;
use time::{macros::format_description, Date};

pub mod client_version_info;
pub mod deploy_log;
pub mod domain;

const LOG_PATTERN: &str = r"New Client (version-.+) at (\d+/\d+/\d+ \d+:\d+:\d+ [A,P]M)";

/// Pull raw deployment history from Roblox S3 bucket
async fn get_deploy_history(client: &Client, channel: &Channel) -> anyhow::Result<String> {
    let url = format!("{}/DeployHistory.txt", channel.base_url());

    let history = client
        .get(url)
        .send()
        .await
        .context("Failed to get deploy logs from rbxcdn")?
        .text()
        .await
        .context("Failed to parse deploy logs into string")?;

    Ok(history)
}

fn get_logs_from_string(channel: &Channel, deploy_history: String) -> Vec<DeployLog> {
    let regex = Regex::new(LOG_PATTERN).unwrap();
    let format =
        format_description!("[month padding:none]/[day padding:none]/[year] [hour padding:none]:[minute]:[second] [period]");

    let mut logs = vec![];
    for capture in regex.captures_iter(&*deploy_history) {
        let version_guid = capture[1].to_string();
        let timestamp = Date::parse(&capture[2], &format).unwrap();

        let deploy_log = DeployLog {
            channel: channel.to_owned(),

            version_guid,
            timestamp,
        };

        logs.push(deploy_log);
    }

    logs
}

pub async fn get_deploy_logs_for_channel(
    client: &Client,
    channel: &Channel,
) -> anyhow::Result<Vec<DeployLog>> {
    let deploy_history = get_deploy_history(client, channel).await?;
    let build_logs = get_logs_from_string(channel, deploy_history);
    Ok(build_logs)
}

pub async fn get_latest_deploy_log_for_channel(
    client: &Client,
    channel: &Channel,
    binary_type: &BinaryType,
) -> anyhow::Result<Option<DeployLog>> {
    let version_info = ClientVersionInfo::get(client, channel, binary_type)
        .await
        .context("Failed to get version info")?;

    let deploy_logs = get_deploy_logs_for_channel(client, channel)
        .await
        .context("Failed to get latest deploy logs")?;

    let latest_log = deploy_logs
        .into_iter()
        .find(|i| i.channel == *channel && i.version_guid == version_info.version_guid);

    Ok(latest_log)
}

#[cfg(test)]
mod tests {
    use crate::{domain::Channel, get_logs_from_string};

    #[test]
    fn captures_multi_line() {
        let test = "New RccService version-336605f55f6847b4 at 11/10/2009 3:35:29 PM... Done!
        New RccService version-4fb818c8a9004e56 at 11/10/2009 11:32:43 PM... Done!
        New Client version-133721681a5245bb at 11/10/2009 11:39:38 PM... Done!
        New Client version-a00995fdd72842a2 at 11/11/2009 12:22:26 AM... Done!
        New RccService version-0e918a958ffd4782 at 11/18/2009 1:24:07 AM... Done!
        New RccService version-ddc196b8bcd249a5 at 11/25/2009 12:54:50 AM... Done!
        New Client version-901e02f7d95046c7 at 11/25/2009 1:03:43 AM... Done!
        New RccService version-7d5db4f118c04c32 at 12/4/2009 4:16:00 PM... New RccService version-0c5ffd6455e548fd at 12/5/2009 1:17:34 AM... Done!
        New RccService version-c2266dad804f4be7 at 12/7/2009 12:57:29 PM... Done!
        New RccService version-22f8550585f344fe at 12/11/2009 12:34:35 AM... Done!
        New RccService version-80ab88f82c004848 at 12/17/2009 12:34:42 AM... Done!
        New Client version-de75ac4e180244b4 at 12/17/2009 12:50:56 AM... Done!
        New Client version-a10672c987274c2e at 12/19/2009 12:18:29 AM... Done!
        New RccService version-ae2ebf93ac594514 at 1/8/2010 11:31:45 AM... Done!
        New RccService version-18f76f9455204d6b at 1/8/2010 12:18:28 PM... Done!
        New Client version-29d1896c5e90402b at 1/8/2010 1:16:46 PM... Done!";
        let logs = get_logs_from_string(&Channel::Live, test.to_string());
        assert_eq!(logs.len(), 6);
    }

    #[test]
    fn captures_single_line() {
        let test = "New Client version-133721681a5245bb at 11/10/2009 11:39:38 PM... Done!";
        let logs = get_logs_from_string(&Channel::Live, test.to_string());
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn skips_bad_types() {
        let test = "New WindowsPlayer version-4981d7ba0b92417b at 6/14/2021 3:26:16 PM, file version: 0, 483, 0, 424775, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...";
        let logs = get_logs_from_string(&Channel::Live, test.to_string());
        assert!(logs.is_empty());
    }
}
