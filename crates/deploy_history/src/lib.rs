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

const LOG_PATTERN: &str = r"New (Studio6?4?) (version-.+) at (\d+/\d+/\d+ \d+:\d+:\d+ [A,P]M), file version: (\d+), (\d+), (\d+), (\d+)";

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
        let is_64_bit = capture[1].ends_with("64");
        let version_guid = capture[2].to_string();
        let timestamp = Date::parse(&capture[3], &format).unwrap();

        let major_rev: usize = capture[4].parse().unwrap();
        let version: usize = capture[5].parse().unwrap();
        let patch: usize = capture[6].parse().unwrap();
        let change_list: usize = capture[7].parse().unwrap();

        let deploy_log = DeployLog {
            channel: channel.to_owned(),

            is_64_bit,
            version_guid,
            timestamp,

            major_rev,
            version,
            patch,
            change_list,
        };

        // We're only interested in including studio builds for the current CPU architecture, but
        // only if we're not running tests
        if cfg!(test) {
            logs.push(deploy_log);
        } else {
            let running_x64 = cfg!(target_pointer_width = "64");
            if (running_x64 && is_64_bit) || (!running_x64 && !is_64_bit) {
                logs.push(deploy_log);
            }
        }
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

    let latest_log = deploy_logs.into_iter().find(|i| {
        i.channel == *channel
            && i.version_id() == version_info.version
            && i.version_guid == version_info.version_guid
    });

    Ok(latest_log)
}

#[cfg(test)]
mod tests {
    use crate::{domain::Channel, get_logs_from_string};

    #[test]
    fn captures_single_line_old_format() {
        let test = "New Studio64 version-32f890bd512d4b6a at 4/19/2021 6:38:02 PM, file version: 0, 475, 0, 420862...Done!";
        let logs = get_logs_from_string(&Channel::Live, test.to_string());
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn captures_multi_line_old_format() {
        let test = "New Studio64 version-44076729f5ea4827 at 2/8/2021 4:18:10 PM, file version: 0, 465, 0, 417678...Done!\n
New Studio64 version-44076729f5ea4827 at 2/8/2021 4:18:10 PM, file version: 0, 465, 0, 417678...Done!\n
New Studio64 version-44076729f5ea4827 at 2/8/2021 4:18:10 PM, file version: 0, 465, 0, 417678...Done!\n
New Studio64 version-44076729f5ea4827 at 2/8/2021 4:18:10 PM, file version: 0, 465, 0, 417678...Done!\n
New Studio64 version-44076729f5ea4827 at 2/8/2021 4:18:10 PM, file version: 0, 465, 0, 417678...Done!";
        let logs = get_logs_from_string(&Channel::Live, test.to_string());
        assert_eq!(logs.len(), 5);
    }

    #[test]
    fn captures_single_line_new_format() {
        let test = "New Studio64 version-a357245614884bf5 at 5/26/2021 2:24:55 PM, file version: 0, 480, 1, 423489, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...";
        let logs = get_logs_from_string(&Channel::Live, test.to_string());
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn captures_multi_line_new_format() {
        let test = "New Studio64 version-a357245614884bf5 at 5/26/2021 2:24:55 PM, file version: 0, 480, 1, 423489, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...\n
New Studio version-a9bf28344668488d at 5/28/2021 3:05:57 PM, file version: 0, 481, 0, 423686, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...\n
New Studio64 version-04ce9261158d4b0f at 5/28/2021 3:06:54 PM, file version: 0, 481, 0, 423686, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...\n
New Studio version-1eff506b67a94c05 at 5/28/2021 3:17:17 PM, file version: 0, 481, 0, 423686, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...\n
New Studio64 version-8444aba87ee74d93 at 5/28/2021 3:18:16 PM, file version: 0, 481, 0, 423686, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...\n
New Studio version-6a089dcca5644f8e at 6/3/2021 11:56:25 AM, file version: 0, 481, 1, 423973, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...\n
New Studio64 version-a7431ddc8dfe4a7d at 6/3/2021 11:57:24 AM, file version: 0, 481, 1, 423973, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...\n
New Studio version-348f7d9c46cd4892 at 6/3/2021 12:02:45 PM, file version: 0, 481, 1, 423973, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...\n
New Studio64 version-f8db636ab0834da8 at 6/3/2021 12:03:55 PM, file version: 0, 481, 1, 423973, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...\n
New Studio version-c2037653e0a446ac at 6/7/2021 4:26:20 PM, file version: 0, 482, 0, 424268, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...\n
New Studio64 version-51bb1cea7fd9483a at 6/7/2021 4:27:10 PM, file version: 0, 482, 0, 424268, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...\n
New Studio version-879389fd96a942b1 at 6/7/2021 4:33:52 PM, file version: 0, 482, 0, 424268, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...";
        let logs = get_logs_from_string(&Channel::Live, test.to_string());
        assert_eq!(logs.len(), 12);
    }

    #[test]
    fn skips_bad_types() {
        let test = "New WindowsPlayer version-4981d7ba0b92417b at 6/14/2021 3:26:16 PM, file version: 0, 483, 0, 424775, git hash: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ...";
        let logs = get_logs_from_string(&Channel::Live, test.to_string());
        assert!(logs.is_empty());
    }
}
