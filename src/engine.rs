extern crate lib_logger;

use lib_logger::LoggerConfig;

fn initialize() {
    let config = LoggerConfig {
        log_to_file: true,
        log_to_console: true,
        log_to_syslog: false,
        log_dir: Some("logs".into()),
        syslog_facility: syslog::Facility::LOG_USER,
        syslog_process_name: "pgShield".into(),
        syslog_remote_addr: None,
    };

    lib_logger::init_logger(&config).unwrap();

    log::info!("pgShield initialized");
}