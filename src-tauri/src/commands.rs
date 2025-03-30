use tauri::{State, command};
use log::{info, error, debug};

use crate::wallet_manager::AsyncWalletManager;
use crate::config::{AppSettings, ConfigManager};
// AsyncSecurityManager is used in open_wallet command
use crate::security::AsyncSecurityManager;

/// Response type for commands with proper error handling
type CommandResult<T> = Result<T, String>;

/// Convert Application errors to string responses for Tauri
fn format_error<E: std::fmt::Display>(e: E) -> String {
    format!("{}", e)
}

/// Command to check if a wallet is currently open
#[command]
pub async fn check_wallet_status(wallet_manager: State<'_, AsyncWalletManager>) -> CommandResult<bool> {
    debug!("Command: check_wallet_status");
    let manager = wallet_manager.get_manager().await;
    
    // A wallet is open if get_current_wallet returns Some
    let result = manager.get_current_wallet().is_some();
    
    if result {
        debug!("Wallet status: open");
    } else {
        debug!("Wallet status: closed");
    }
    
    Ok(result)
}

/// Command to close the currently open wallet
#[command]
pub async fn close_wallet(wallet_manager: State<'_, AsyncWalletManager>) -> CommandResult<bool> {
    info!("Command: close_wallet");
    let mut manager = wallet_manager.get_manager().await;
    
    // Close the wallet
    manager.close_wallet();
    
    Ok(true)
}

/// Command to get a list of available wallets
#[command]
pub async fn get_available_wallets(wallet_manager: State<'_, AsyncWalletManager>) -> CommandResult<Vec<String>> {
    debug!("Command: get_available_wallets");
    let manager = wallet_manager.get_manager().await;
    
    // Get wallets and extract names
    let wallets = manager.list_wallets()
        .into_iter()
        .map(|w| w.name.clone())
        .collect();
    
    Ok(wallets)
}

/// Wallet details for the frontend
#[derive(serde::Serialize)]
pub struct WalletDetails {
    name: String,
    secured: bool,
}

/// Command to get detailed information about all wallets
#[command]
pub async fn get_wallet_details(wallet_manager: State<'_, AsyncWalletManager>) -> CommandResult<Vec<WalletDetails>> {
    debug!("Command: get_wallet_details");
    let manager = wallet_manager.get_manager().await;
    
    // Get wallets and convert to WalletDetails
    let wallets = manager.list_wallets()
        .into_iter()
        .map(|w| WalletDetails {
            name: w.name.clone(),
            secured: w.secured,
        })
        .collect();
    
    Ok(wallets)
}

/// Command to check if the current wallet is secured (password protected)
#[command]
pub async fn is_current_wallet_secured(wallet_manager: State<'_, AsyncWalletManager>) -> CommandResult<Option<bool>> {
    debug!("Command: is_current_wallet_secured");
    let manager = wallet_manager.get_manager().await;
    
    Ok(manager.is_current_wallet_secured())
}

/// Command to create a new wallet with optional password protection
#[command]
pub async fn create_wallet(
    wallet_name: String, 
    password: String, 
    use_password: bool,
    wallet_manager: State<'_, AsyncWalletManager>
) -> CommandResult<bool> {
    info!("Command: create_wallet with name: {}", wallet_name);
    
    // If password protection is disabled, use empty password
    let effective_password = if use_password { password } else { String::new() };
    
    let mut manager = wallet_manager.get_manager().await;
    match manager.create_wallet(&wallet_name, &effective_password) {
        Ok(_) => {
            info!("Successfully created wallet: {}", wallet_name);
            // Now open the newly created wallet
            match manager.open_wallet(&wallet_name, if use_password { Some(&effective_password) } else { None }) {
                Ok(_) => {
                    info!("Successfully opened new wallet: {}", wallet_name);
                    Ok(true)
                },
                Err(e) => {
                    error!("Created wallet but failed to open it: {}", e);
                    Err(format_error(e))
                }
            }
        },
        Err(e) => {
            error!("Failed to create wallet: {}", e);
            Err(format_error(e))
        }
    }
}

/// Command to get the name of the currently open wallet
#[command]
pub async fn get_current_wallet_name(wallet_manager: State<'_, AsyncWalletManager>) -> CommandResult<Option<String>> {
    debug!("Command: get_current_wallet_name");
    
    let manager = wallet_manager.get_manager().await;
    
    // Extract name from current wallet if one is open
    let result = manager.get_current_wallet()
        .map(|wallet| wallet.name.clone());
    
    Ok(result)
}

/// Command to update application settings
#[command]
pub async fn update_app_settings(
    theme: Option<String>, 
    auto_backup: Option<bool>, 
    notifications_enabled: Option<bool>,
    log_level: Option<String>,
    config_manager: State<'_, ConfigManager>,
) -> CommandResult<bool> {
    info!("Command: update_app_settings");
    
    // Get a copy of the current config
    let mut config = config_manager.get_config().clone();
    
    // Update only the provided settings
    if let Some(theme) = theme {
        info!("Updating theme to: {}", theme);
        config.app_settings.theme = theme;
    }
    
    if let Some(auto_backup) = auto_backup {
        info!("Updating auto_backup to: {}", auto_backup);
        config.app_settings.auto_backup = auto_backup;
    }
    
    if let Some(notifications) = notifications_enabled {
        info!("Updating notifications_enabled to: {}", notifications);
        config.app_settings.notifications_enabled = notifications;
    }
    
    if let Some(log_level) = log_level {
        info!("Updating log_level to: {}", log_level);
        config.app_settings.log_level = log_level;
        
        // TODO: Update actual log level at runtime if needed
    }
    
    // Save the updated config
    match config_manager.update_app_settings(config.app_settings).await {
        Ok(_) => {
            info!("Settings updated successfully");
            Ok(true)
        },
        Err(e) => {
            error!("Failed to update settings: {}", e);
            Err(format_error(e))
        }
    }
}

/// Command to get current application settings
#[command]
pub async fn get_app_settings(config_manager: State<'_, ConfigManager>) -> CommandResult<AppSettings> {
    debug!("Command: get_app_settings");
    
    // Get config is now returning a reference directly, not a Result
    let config = config_manager.get_config();
    Ok(config.app_settings.clone())
}

/// Command to open a wallet
#[command]
pub async fn open_wallet(
    wallet_name: String, 
    password: Option<String>, 
    wallet_manager: State<'_, AsyncWalletManager>,
    security_manager: State<'_, AsyncSecurityManager>,
) -> CommandResult<bool> {
    info!("Command: open_wallet for wallet: {}", wallet_name);
    
    // Get the wallet to check if it's secured
    let manager = wallet_manager.get_manager().await;
    let wallet_info = manager.find_wallet_by_name(&wallet_name)
        .ok_or_else(|| format!("Wallet '{}' not found", wallet_name))?;
    
    if wallet_info.secured {
        // If wallet is secured, password is required
        let password = match password {
            Some(pwd) if !pwd.is_empty() => pwd,
            _ => return Err("Password is required for this secured wallet".to_string())
        };
        
        // Authenticate with security manager
        let mut sec_manager = security_manager.get_manager().await;
        match sec_manager.authenticate(&password) {
            Ok(_) => {
                // Authentication successful, attempt to open wallet
                let mut manager = wallet_manager.get_manager().await;
                match manager.open_wallet(&wallet_name, Some(&password)) {
                    Ok(_) => {
                        info!("Successfully opened secured wallet: {}", wallet_name);
                        Ok(true)
                    },
                    Err(e) => {
                        error!("Failed to open secured wallet: {}", e);
                        Err(format_error(e))
                    }
                }
            },
            Err(e) => {
                error!("Authentication failed: {}", e);
                Err(format_error(e))
            }
        }
    } else {
        // For unsecured wallets, we don't need password authentication
        let mut manager = wallet_manager.get_manager().await;
        match manager.open_wallet(&wallet_name, None) {
            Ok(_) => {
                info!("Successfully opened unsecured wallet: {}", wallet_name);
                Ok(true)
            },
            Err(e) => {
                error!("Failed to open unsecured wallet: {}", e);
                Err(format_error(e))
            }
        }
    }
}
