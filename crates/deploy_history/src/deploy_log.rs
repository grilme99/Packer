use time::{macros::format_description, Date};

use crate::domain::Channel;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct DeployLog {
    pub is_64_bit: bool,
    pub channel: Channel,
    pub version_guid: String,
    pub timestamp: Date,

    pub major_rev: usize,
    pub version: usize,
    pub patch: usize,
    pub change_list: usize,
}

impl DeployLog {
    pub fn version_id(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.major_rev, self.version, self.patch, self.change_list
        )
    }
}

impl ToString for DeployLog {
    fn to_string(&self) -> String {
        let format = format_description!("[month repr:short] [day]");
        let date = self.timestamp.format(&format).expect("valid date");

        format!("{} ({date})", self.version_id())
    }
}

#[cfg(test)]
mod tests {
    use crate::{deploy_log::DeployLog, domain::Channel};
    use time::macros::date;

    fn get_deploy_log() -> DeployLog {
        let channel = Channel::Live;

        DeployLog {
            is_64_bit: true,
            version_guid: "version-d780cbcde4ab4f52".into(),
            timestamp: date!(2022 - 1 - 1),
            channel,

            major_rev: 1,
            version: 2,
            patch: 3,
            change_list: 4,
        }
    }

    #[test]
    fn valid_version_id() {
        let deploy_log = get_deploy_log();
        let version_id = deploy_log.version_id();
        assert_eq!(version_id, "1.2.3.4");
    }

    #[test]
    fn valid_formatted_string() {
        let deploy_log = get_deploy_log();
        assert_eq!(deploy_log.to_string(), "1.2.3.4 (Jan 01)");
    }
}
