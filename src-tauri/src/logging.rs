use std::sync::Once;
use log::{LevelFilter, Record, Level, Metadata};
use chrono::Local;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Custom logger for B-Rad Coin application
pub struct AppLogger {
    log_file: Mutex<Option<File>>,
}

// Use a static reference instead of Lazy
static APP_LOGGER: AppLogger = AppLogger {
    log_file: Mutex::new(None),
};

static LOGGER_INIT: Once = Once::new();

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = Local::now();
            let level_str = match record.level() {
                Level::Error => "ERROR",
                Level::Warn => "WARN ",
                Level::Info => "INFO ",
                Level::Debug => "DEBUG",
                Level::Trace => "TRACE",
            };
            
            let log_message = format!(
                "{} [{}] [{}] {}\n",
                now.format("%Y-%m-%d %H:%M:%S"),
                level_str,
                record.target(),
                record.args()
            );
            
            // Always print to console
            print!("{}", log_message);

            // Also log to file if available
            if let Ok(mut log_file) = self.log_file.lock() {
                if let Some(ref mut file) = *log_file {
                    let _ = file.write_all(log_message.as_bytes());
                    let _ = file.flush();
                }
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut log_file) = self.log_file.lock() {
            if let Some(ref mut file) = *log_file {
                let _ = file.flush();
            }
        }
    }
}

/// Initialize the logging system
pub fn init(log_dir: Option<PathBuf>, level: LevelFilter) -> Result<(), String> {
    LOGGER_INIT.call_once(|| {
        // Set up file logging if a directory is provided
        if let Some(dir) = log_dir {
            if let Err(e) = initialize_log_file(&dir) {
                eprintln!("Failed to initialize log file: {}", e);
            }
        }
        
        // Register our logger
        if let Err(e) = log::set_logger(&APP_LOGGER).map(|()| log::set_max_level(level)) {
            eprintln!("Failed to set logger: {}", e);
        }
    });
    
    Ok(())
}

/// Initialize the log file
fn initialize_log_file(log_dir: &PathBuf) -> Result<(), String> {
    // Create logs directory if it doesn't exist
    if let Err(e) = create_dir_all(log_dir) {
        return Err(format!("Failed to create log directory: {}", e));
    }
    
    // Create log file with timestamp
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let log_file_path = log_dir.join(format!("b_rad_coin_{}.log", timestamp));
    
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_file_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;
    
    // Store the file handle in our logger
    if let Ok(mut logger_file) = APP_LOGGER.log_file.lock() {
        *logger_file = Some(file);
    }
    
    Ok(())
}

/// Log a startup message with application version and build info
pub fn log_app_startup(app_version: &str) {
    log::info!("==================================================");
    log::info!("B-Rad Coin Application Starting");
    log::info!("Version: {}", app_version);
    log::info!("Build date: {}", env!("CARGO_PKG_VERSION"));
    log::info!("==================================================");
}

/// Log an application shutdown message
pub fn log_app_shutdown() {
    log::info!("==================================================");
    log::info!("B-Rad Coin Application Shutting Down");
    log::info!("==================================================");
}

/// Helper macro for logging within the application
#[macro_export]
macro_rules! app_log {
    (error, $($arg:tt)+) => {
        log::error!($($arg)+);
    };
    (warn, $($arg:tt)+) => {
        log::warn!($($arg)+);
    };
    (info, $($arg:tt)+) => {
        log::info!($($arg)+);
    };
    (debug, $($arg:tt)+) => {
        log::debug!($($arg)+);
    };
    (trace, $($arg:tt)+) => {
        log::trace!($($arg)+);
    };
}