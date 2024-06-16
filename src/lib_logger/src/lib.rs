use log::{LevelFilter, SetLoggerError};
use std::fs::{self};
use file_rotate::{FileRotate, ContentLimit, compression::Compression, suffix::AppendCount, TimeFrequency};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::time::SystemTime;
use syslog::{BasicLogger, Formatter3164, Facility};
use chrono::Datelike;
use env_logger::{Builder};

pub struct LoggerConfig {
    pub log_to_file: bool,
    pub log_to_console: bool,
    pub log_to_syslog: bool,
    pub log_dir: Option<PathBuf>,
    pub syslog_facility: Facility,
    pub syslog_process_name: String,
    pub syslog_remote_addr: Option<String>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            log_to_file: false,
            log_to_console: true,
            log_to_syslog: false,
            log_dir: Some(PathBuf::from("logs")),
            syslog_facility: Facility::LOG_USER,
            syslog_process_name: "pgShield".to_string(),
            syslog_remote_addr: None,
        }
    }
}

static INIT_LOGGER: Once = Once::new();

pub fn init_logger(config: &LoggerConfig) -> Result<(), SetLoggerError> {
    INIT_LOGGER.call_once(|| {
        let result = init_logger_inner(config);
        if let Err(err) = result {
            eprintln!("Failed to initialize pgShield logger: {}", err);
        }
    });

    Ok(())
}

fn init_logger_inner(config: &LoggerConfig) -> Result<(), SetLoggerError> {
    if config.log_to_syslog {
        let formatter = Formatter3164 {
            facility: config.syslog_facility,
            hostname: None,
            process: config.syslog_process_name.clone(),
            pid: 0,
        };

        let logger = match &config.syslog_remote_addr {
            Some(addr) => syslog::tcp(formatter, addr.as_str()).unwrap(),
            None => syslog::unix(formatter).unwrap(),
        };

        let _ = log::set_boxed_logger(Box::new(BasicLogger::new(logger)))?;
        log::set_max_level(LevelFilter::Info);
    }

    if config.log_to_file {
        let log_dir = config.log_dir.clone().unwrap_or_else(|| PathBuf::from("logs"));
        let now = SystemTime::now();
        let log_file_prefix = "pgshield-";
        let log_file_suffix = format!("{}.log", chrono::Local::now().format("%d%m%Y").to_string());
        let log_file_name = format!("{}{}", log_file_prefix, log_file_suffix);
    
        let log_path = create_log_dir_and_file(&log_dir, &log_file_name, now).unwrap();
    
        let mut log = FileRotate::new(
            log_path.to_str().unwrap(),
            AppendCount::new(2),
            ContentLimit::Time(TimeFrequency::Daily),
            Compression::None,
        );
        
        
    
        let mut builder = Builder::new();
        builder.format(move |buf, record| {
            writeln!(buf, "[{}] - {}", record.level(), record.args()).unwrap();
            Ok(())
        });
        builder.filter(None, LevelFilter::Info);
    
        let logger = builder.build();
        let _ = log::set_boxed_logger(Box::new(logger));
        log::set_max_level(LevelFilter::Info);
    
        // Write log messages to the file
        let _ = log::max_level();
        let _ = log::set_max_level(LevelFilter::Info);
        log::info!("Log message");
        writeln!(log, "Log message").unwrap();
        writeln!(log, "PgShield Initialized").unwrap();
    }


    if config.log_to_console {
        let mut builder = Builder::new();
        builder.format(move |buf, record| {
            writeln!(buf, "[{}] - {}", record.level(), record.args()).unwrap();
            Ok(())
        });
        builder.filter(None, LevelFilter::Info);
       // builder.init();
    }

    Ok(())
}

fn create_log_dir_and_file(
    log_dir: &Path,
    log_file_name: &str,
    now: SystemTime,
) -> Result<PathBuf, std::io::Error> {
    let log_date_dir = format!(
        "{:04}{:02}{:02}",
        now.elapsed()
            .unwrap_or_default()
            .as_secs()
            .checked_div(86400)
            .unwrap_or(0)
            / 86400
            + 719528, // Unix epoch for 1970-01-01
        chrono::Local::now().month(),
        chrono::Local::now().day()
    );
    let log_path = log_dir.join(&log_date_dir).join(log_file_name);

    if let Some(parent) = log_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid log path",
        ));
    }

    Ok(log_path)
}