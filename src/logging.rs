use std::{fs, path::Path};

use anyhow::Context;
use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    append::{console::Target, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
    Handle,
};

pub fn init_logging(root_dir: &Path) -> anyhow::Result<Handle> {
    let level = log::LevelFilter::Info;

    let log_dir = root_dir.join("log");
    if !log_dir.exists() {
        fs::create_dir(&log_dir).context("Failed to create log directory")?;
    }

    let log_path = log_dir.join("bootstrapper.log");

    // Build a stderr logger.
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();

    // Logging to log file.
    let log_file = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(log_path)
        .context("Failed to create file appender")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(log_file)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )
        .context("Failed to build config")?;

    let handle = log4rs::init_config(config).context("failed to initiate config")?;

    Ok(handle)
}
