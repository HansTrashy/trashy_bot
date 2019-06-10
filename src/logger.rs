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
        .chain(io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
