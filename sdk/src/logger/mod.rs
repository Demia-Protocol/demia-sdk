use log::{LevelFilter, SetLoggerError};

use crate::configuration::LoggingConfiguration;

pub fn init(config: &LoggingConfiguration) -> Result<(), SetLoggerError> {
    let log_level = match config.level.as_str() {
        "debug" => LevelFilter::Debug,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    fern::Dispatch::new()
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
        .level_for("sqlx", LevelFilter::Warn)
        .level_for("vaultrs", LevelFilter::Warn)
        .chain(std::io::stdout())
        // .chain(OpenOptions::new()
        // .write(true)
        // .create(true)
        // .truncate(true)
        // .open(&configuration.logging.debug_location)
        // .map_err(|e| Error::LoggerFormattingError(e))?
        // )
        .apply()
}
