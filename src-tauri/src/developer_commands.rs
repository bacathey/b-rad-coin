use crate::errors::AppError;
use log::{debug, error, info};
use std::path::PathBuf;
use std::fs;
use tauri::command;

/// Get recent log entries for the developer page
#[command]
pub async fn get_recent_logs() -> Result<String, String> {
    info!("Command: get_recent_logs");
    
    // Get the app data directory where logs are stored
    let log_dir = match dirs::data_dir() {
        Some(dir) => dir.join("com.b-rad-coin.app").join("logs"),
        None => return Err("Failed to determine log directory".to_string()),
    };
    
    debug!("Looking for logs in directory: {}", log_dir.display());
    
    // Check if the directory exists
    if !log_dir.exists() {
        return Err(format!("Log directory does not exist: {}", log_dir.display()));
    }
    
    // Get a list of all log files sorted by modification time (most recent first)
    let mut log_files: Vec<PathBuf> = Vec::new();
    match fs::read_dir(&log_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("log") {
                        log_files.push(path);
                    }
                }
            }
        },
        Err(e) => {
            error!("Failed to read log directory: {}", e);
            return Err(format!("Failed to read log directory: {}", e));
        }
    }
    
    // Sort log files by modification time (newest first)
    log_files.sort_by(|a, b| {
        let a_meta = fs::metadata(a).ok();
        let b_meta = fs::metadata(b).ok();
        
        match (a_meta, b_meta) {
            (Some(a_meta), Some(b_meta)) => {
                b_meta.modified().unwrap_or_default().cmp(&a_meta.modified().unwrap_or_default())
            },
            _ => std::cmp::Ordering::Equal,
        }
    });
    
    // Get the most recent log file, if any
    let recent_log = match log_files.first() {
        Some(path) => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Get the last 100 lines (or all if fewer)
                    let lines: Vec<&str> = content.lines().collect();
                    let start_idx = if lines.len() > 100 { lines.len() - 100 } else { 0 };
                    lines[start_idx..].join("\n")
                },
                Err(e) => {
                    error!("Failed to read log file: {}", e);
                    return Err(format!("Failed to read log file: {}", e));
                }
            }
        },
        None => "No log files found.".to_string()
    };
    
    Ok(recent_log)
}

/// Echo a command for the developer page
#[command]
pub fn echo_command(command: String) -> Result<String, String> {
    info!("Command: echo_command - {}", command);
    Ok(format!("Command received: {}\nTimestamp: {}", command, chrono::Local::now().format("%Y-%m-%d %H:%M:%S")))
}

/// Command to get the configuration directory path
#[command]
pub fn get_config_directory() -> Result<String, String> {
    info!("Command: get_config_directory");
    
    // Get the app data directory
    let config_dir = match dirs::data_dir() {
        Some(dir) => dir.join("com.b-rad-coin.app").join("config"),
        None => return Err("Failed to determine config directory".to_string()),
    };
    
    debug!("Configuration directory path: {}", config_dir.display());
    Ok(config_dir.to_string_lossy().into_owned())
}
