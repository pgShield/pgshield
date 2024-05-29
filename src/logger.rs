// logger.rs
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref INFO_LOGGER: Mutex<std::fs::File> = Mutex::new(
        OpenOptions::new()
            .append(true)
            .create(true)
            .open("proxy_info.log")
            .unwrap()
    );

    static ref ERROR_LOGGER: Mutex<std::fs::File> = Mutex::new(
        OpenOptions::new()
            .append(true)
            .create(true)
            .open("proxy_error.log")
            .unwrap()
    );
}

pub fn log_info(message: &str) {
    let mut file = INFO_LOGGER.lock().unwrap();
    writeln!(file, "INFO: {}", message).unwrap();
}

pub fn log_error(message: &str) {
    let mut file = ERROR_LOGGER.lock().unwrap();
    writeln!(file, "ERROR: {}", message).unwrap();
}

pub fn init_logger() {
    log_info("PGShield Logger initialized");
}
