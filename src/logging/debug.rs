use std::{fs::create_dir, path::Path};

use super::Logger;
use log::{debug, error, info, warn};

pub struct Log {}

impl Logger for Log {
    fn setup_logging() {
        if !Path::new("log").exists() {
            if let Err(e) = create_dir("log") {
                eprintln!("create log dir failed: {e}");
                return;
            }
        };
        let base_config = fern::Dispatch::new().level(log::LevelFilter::Debug);
        let file_config = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .chain(fern::DateBased::new("log/log.", "%Y-%m-%d"));
        if let Err(e) = base_config.chain(file_config).apply() {
            eprintln!("apply log config failed: {e}");
        }
    }
    fn info(info: &str) {
        info!("{info}");
    }
    fn debug(debug: &str) {
        debug!("{debug}");
    }

    fn warn(warn: &str) {
        warn!("{warn}");
    }

    fn error(error: &str) {
        error!("{error}");
    }
}
