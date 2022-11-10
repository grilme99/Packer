use time::{macros::format_description, Date};

use crate::domain::Channel;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct DeployLog {
    pub channel: Channel,
    pub version_guid: String,
    pub timestamp: Date,
}

impl ToString for DeployLog {
    fn to_string(&self) -> String {
        let format = format_description!("[month repr:short] [day]");
        let date = self.timestamp.format(&format).expect("valid date");

        format!(
            "{} {} ({date})",
            self.channel.to_string(),
            self.version_guid
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{deploy_log::DeployLog, domain::Channel};
    use time::macros::date;

    fn get_deploy_log() -> DeployLog {
        let channel = Channel::Live;

        DeployLog {
            version_guid: "version-d780cbcde4ab4f52".into(),
            timestamp: date!(2022 - 1 - 1),
            channel,
        }
    }

    #[test]
    fn valid_formatted_string() {
        let deploy_log = get_deploy_log();
        assert_eq!(
            deploy_log.to_string(),
            "live version-d780cbcde4ab4f52 (Jan 01)"
        );
    }
}
