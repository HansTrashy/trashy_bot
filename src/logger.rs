use fern::{self, InitError, Dispatch};
use log::LevelFilter;
use chrono::Utc;
use std::io;

pub fn setup() -> Result<(), InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                Utc::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .level_for("tokio_reactor", LevelFilter::Info)
        .level_for("tokio_io", LevelFilter::Info)
        .level_for("rustls", LevelFilter::Info)
        .level_for("h2", LevelFilter::Info)
        .level_for("tungstenite", LevelFilter::Info)
        .level_for("mio", LevelFilter::Info)
        .chain(io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
