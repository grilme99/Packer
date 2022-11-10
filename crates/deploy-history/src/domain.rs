#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinaryType {
    WindowsPlayer,
    MacPlayer,
}

impl ToString for BinaryType {
    fn to_string(&self) -> String {
        let str = match self {
            BinaryType::WindowsPlayer => "WindowsPlayer",
            BinaryType::MacPlayer => "MacPlayer",
        };

        str.to_owned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Channel {
    Live,
    ZCanary,
    ZIntegration,
}

impl Channel {
    pub fn base_url(&self) -> String {
        match self {
            Channel::Live => "https://setup.rbxcdn.com".to_owned(),
            _ => format!("https://setup.rbxcdn.com/channel/{}", self.to_string()),
        }
    }
}

impl ToString for Channel {
    fn to_string(&self) -> String {
        let str = match self {
            Channel::Live => "live",
            Channel::ZCanary => "zcanary",
            Channel::ZIntegration => "zintegration",
        };

        str.to_owned()
    }
}
