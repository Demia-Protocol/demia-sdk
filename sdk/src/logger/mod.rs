use fern::Dispatch;
use log::{LevelFilter, SetLoggerError};

use crate::configuration::LoggingConfiguration;

pub fn init(config: &LoggingConfiguration, dispatch: Option<Dispatch>) -> Result<(), SetLoggerError> {
    let log_level = match config.level.as_str() {
        "debug" => LevelFilter::Debug,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    let dispatch = match dispatch {
        None => fern::Dispatch::new(),
        Some(d) => d,
    };

    dispatch
        .format(|out: fern::FormatCallback, message, record| {
            let source = format!("{}:{}", record.target(), record.line().unwrap_or_default());
            let gap = if source.len() < 35 {
                " ".repeat(35 - source.len())
            } else {
                " ".to_string()
            };

            out.finish(format_args!(
                "[{} | {:6}| {}]{} {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                source,
                gap,
                message
            ))
        })
        .level(log_level)
        .level_for("tracing", log::LevelFilter::Info)
        .level_for("hyper", log::LevelFilter::Info)
        .level_for("sqlx", LevelFilter::Warn)
        .level_for("vaultrs", LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()
}
