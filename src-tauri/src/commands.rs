use crate::logging;
use log::{debug, error, info, warn};
use std::sync::Arc;  // Add this import for Arc
use tauri::Emitter;
use tauri::{command, Manager, State};
use serde::{Serialize, Deserialize};

use crate::config::{AppSettings, ConfigManager}; // Ensure WalletInfo is imported if not already
use crate::security::AsyncSecurityManager;
use crate::wallet_manager::AsyncWalletManager;
use bip39::Mnemonic;
use rand::Rng;
use crate::blockchain_sync::{AsyncBlockchainSyncService, NetworkStatus};
use crate::wallet_sync_service::{AsyncWalletSyncService, WalletSyncStatus};
use crate::mining_service::{AsyncMiningService, MiningStatus};

/// Response type for commands with proper error handling
type CommandResult<T> = Result<T, String>;

/// Convert Application errors to string responses for Tauri
fn format_error<E: std::fmt::Display>(e: E) -> String {
    format!("{}", e)
}

/// Wallet details for the frontend
#[derive(serde::Serialize)]
pub struct WalletDetails {
    name: String,
    secured: bool,
}


/// Command to check if a wallet is currently open
#[command]
pub async fn check_wallet_status(
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<bool> {
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

/// Command to get the number of available CPU cores
#[command]
pub async fn get_cpu_cores() -> CommandResult<u32> {
    debug!("Command: get_cpu_cores");
    
    let cores = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1);
    
    debug!("Available CPU cores: {}", cores);
    Ok(cores)
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
pub async fn get_available_wallets(
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<Vec<String>> {    debug!("Command: get_available_wallets");
    let mut manager = wallet_manager.get_manager().await;

    // Get wallets and extract names
    let wallets = manager
        .list_wallets()
        .into_iter()
        .map(|w| w.name.clone())
        .collect();

    Ok(wallets)
}

/// Command to get detailed information about all wallets
#[command]
pub async fn get_wallet_details(
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<Vec<WalletDetails>> {    debug!("Command: get_wallet_details");
    let mut manager = wallet_manager.get_manager().await;
    
    // Get wallets and convert to WalletDetails
    let wallets: Vec<WalletDetails> = manager
        .list_wallets()
        .into_iter()
        .map(|w| WalletDetails {
            name: w.name.clone(),
            secured: w.secured,
        })
        .collect();

    debug!("get_wallet_details: Found {} wallets", wallets.len());if !wallets.is_empty() {
        debug!("Available wallets: {}", wallets.iter().map(|w| w.name.as_str()).collect::<Vec<_>>().join(", "));
    }

    Ok(wallets)
}

/// Command to check if the current wallet is secured (password protected)
#[command]
pub async fn is_current_wallet_secured(
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<Option<bool>> {
    debug!("Command: is_current_wallet_secured");
    let manager = wallet_manager.get_manager().await;

    Ok(manager.is_current_wallet_secured())
}

/// Command to create a new wallet with optional password protection and a specific seed phrase
#[command]
pub async fn create_wallet(
    wallet_name: String,
    password: String,
    use_password: bool,
    seed_phrase: Option<String>,
    wallet_manager: State<'_, AsyncWalletManager>,
    config_manager_arc: State<'_, Arc<ConfigManager>>,
) -> CommandResult<bool> {
    info!("Command: create_wallet with name: {}", wallet_name);

    // If password protection is disabled, use empty password
    let effective_password = if use_password {
        password
    } else {
        String::new()
    };

    // Get the actual seed phrase or generate one if not provided
    let actual_seed_phrase = if let Some(phrase) = &seed_phrase {
        debug!("Using provided seed phrase (first word: {}, last word: {})",
               phrase.split(' ').next().unwrap_or(""),
               phrase.split(' ').last().unwrap_or(""));
        phrase.clone()
    } else {
        // Seed phrase is required
        error!("No seed phrase provided");
        return Err("Seed phrase is required for wallet creation.".to_string());
    };

    let mut manager = wallet_manager.get_manager().await;
    
    // Call the synchronous create_wallet_with_seed function
    match manager.create_wallet_with_seed(&wallet_name, &effective_password, &actual_seed_phrase, use_password) {
        Ok(_) => {
            info!("Wallet created successfully: {}", wallet_name);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to create wallet: {}", e);
            Err(e.to_string())
        }
    }
}

/// Command to get the name of the currently open wallet
#[command]
pub async fn get_current_wallet_name(
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<Option<String>> {
    debug!("Command: get_current_wallet_name");

    let manager = wallet_manager.get_manager().await;

    // Extract name from current wallet if one is open
    let result = manager
        .get_current_wallet()
        .map(|wallet| wallet.name.clone());

    Ok(result)
}

/// Command to get the path of the currently open wallet
#[command]
pub async fn get_current_wallet_path(
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<Option<String>> {
    debug!("Command: get_current_wallet_path");
    
    let manager = wallet_manager.get_manager().await;
    
    // Get the current wallet name
    let current_wallet_name = match manager.get_current_wallet() {
        Some(wallet) => wallet.name.clone(),
        None => {
            info!("No wallet is currently open");
            return Ok(None);
        }
    };
    
    // Find the wallet info to get the path
    let wallet_info = manager.find_wallet_by_name(&current_wallet_name);
    
    match wallet_info {
        Some(info) => {
            debug!("Found path for wallet '{}': {}", current_wallet_name, info.path);
            
            // Get the base wallets directory
            let wallets_dir = manager.get_wallets_dir();
            debug!("Base wallets directory: {}", wallets_dir.display());
            
            // Check if the path is relative or absolute
            let path_str = &info.path;
            let path = std::path::Path::new(path_str);
            debug!("Wallet path from config: {}", path.display());
            debug!("Is absolute path: {}", path.is_absolute());
            
            let wallet_dir = if path.is_absolute() {
                // If the path is already absolute, use it directly
                path.to_path_buf()
            } else {
                // Check if the path already starts with "wallets/" to avoid duplication
                if path_str.starts_with("wallets/") || path_str.starts_with("wallets\\") {
                    // The path already includes the wallets directory, so use it as relative to the parent of wallets_dir
                    let parent_dir = wallets_dir.parent().unwrap_or(&wallets_dir);
                    parent_dir.join(path_str)
                } else {
                    // Otherwise, join it with the wallets directory
                    wallets_dir.join(path_str)
                }
            };
            
            debug!("Constructed wallet directory path: {}", wallet_dir.display());
            
            // Verify the path
            let exists = wallet_dir.exists();
            let is_dir = if exists { wallet_dir.is_dir() } else { false };
            
            debug!("Wallet path exists: {}, Is directory: {}", exists, is_dir);
            
            if !exists {
                warn!("Wallet directory does not exist: {}", wallet_dir.display());
            } else if !is_dir {
                warn!("Wallet path is not a directory: {}", wallet_dir.display());
            }
            
            // Try to canonicalize the path
            let canonical_result = wallet_dir.canonicalize();
            
            match canonical_result {
                Ok(canonical_path) => {
                    debug!("Canonical wallet path: {}", canonical_path.display());
                    
                    // Convert to string with platform-specific separators
                    match canonical_path.to_str() {
                        Some(path_str) => {
                            let final_path = path_str.to_string();
                            debug!("Returning wallet path: {}", final_path);
                            Ok(Some(final_path))
                        },
                        None => {
                            warn!("Could not convert canonical path to string");
                            // Fall back to non-canonical path
                            match wallet_dir.to_str() {
                                Some(dir_str) => Ok(Some(dir_str.to_string())),
                                None => {
                                    warn!("Could not convert path to string, using original path");
                                    Ok(Some(info.path.clone()))
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    warn!("Could not canonicalize path: {}", e);
                    // Fall back to non-canonical path
                    match wallet_dir.to_str() {
                        Some(dir_str) => Ok(Some(dir_str.to_string())),
                        None => {
                            warn!("Could not convert path to string, using original path");
                            Ok(Some(info.path.clone()))
                        }
                    }
                }
            }
        },
        None => {
            error!("Failed to find path for current wallet: {}", current_wallet_name);
            Err(format!("Could not find path information for wallet '{}'", current_wallet_name))
        }
    }
}

/// Command to update application settings
#[derive(Debug, serde::Deserialize)]
pub struct UpdateSettingsRequest {
    theme: Option<String>,
    auto_backup: Option<bool>,
    notifications_enabled: Option<bool>,
    log_level: Option<String>,
    developer_mode: Option<bool>,
    skip_seed_phrase_dialogs: Option<bool>,
    minimize_to_system_tray: Option<bool>,
    mining_threads: Option<u32>,
}

#[command]
pub async fn update_app_settings(
    request: UpdateSettingsRequest,
    config_manager_arc: State<'_, Arc<ConfigManager>>,
) -> CommandResult<bool> {
    info!("Command: update_app_settings - {:?}", request);

    // Get the inner ConfigManager from the Arc
    let config_manager = config_manager_arc.inner();

    // Get a copy of the current config
    let mut config = config_manager.get_config().clone();

    // Update only the provided settings
    if let Some(theme_val) = request.theme {
        info!("Updating theme to: {}", theme_val);
        config.app_settings.theme = theme_val;
    }

    if let Some(auto_backup_val) = request.auto_backup {
        info!("Updating auto_backup to: {}", auto_backup_val);
        config.app_settings.auto_backup = auto_backup_val;
    }    
    
    if let Some(notifications_val) = request.notifications_enabled {
        info!("Updating notifications_enabled to: {}", notifications_val);
        config.app_settings.notifications_enabled = notifications_val;
    }
    
    if let Some(log_level_val) = request.log_level {
        info!("Updating log_level to: {}", log_level_val);
        config.app_settings.log_level = log_level_val;
        // TODO: Update actual log level at runtime if needed
    }
    
    if let Some(dev_mode) = request.developer_mode {
        info!("Updating developer_mode to: {}", dev_mode);
        config.app_settings.developer_mode = dev_mode;
        
        // If developer mode is being turned off, also turn off skip_seed_phrase_dialogs
        if !dev_mode && config.app_settings.skip_seed_phrase_dialogs {
            info!("Developer mode disabled, disabling skip_seed_phrase_dialogs");
            config.app_settings.skip_seed_phrase_dialogs = false;
        }
    }
    
    if let Some(skip_dialogs) = request.skip_seed_phrase_dialogs {
        // Only allow skip_seed_phrase_dialogs to be enabled if developer_mode is enabled
        if skip_dialogs && !config.app_settings.developer_mode {
            error!("Cannot enable skip_seed_phrase_dialogs when developer_mode is disabled");
            return Err("Developer mode must be enabled to skip seed phrase dialogs".to_string());
        }
        
        info!("Updating skip_seed_phrase_dialogs to: {}", skip_dialogs);
        config.app_settings.skip_seed_phrase_dialogs = skip_dialogs;
    }

    if let Some(minimize_to_tray) = request.minimize_to_system_tray {
        info!("Updating minimize_to_system_tray to: {}", minimize_to_tray);
        config.app_settings.minimize_to_system_tray = minimize_to_tray;
        // Note: System tray changes will take effect on next application restart
        if minimize_to_tray {
            info!("System tray will be enabled on next application restart");
        } else {
            info!("System tray will be disabled on next application restart");
        }
    }

    if let Some(threads) = request.mining_threads {
        // Validate thread count (should be 1 to available CPU cores)
        let max_cores = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1);
        
        if threads == 0 {
            error!("Mining threads cannot be 0");
            return Err("Mining threads must be at least 1".to_string());
        }
        
        if threads > max_cores {
            error!("Mining threads {} exceeds available CPU cores {}", threads, max_cores);
            return Err(format!("Mining threads cannot exceed {} (available CPU cores)", max_cores));
        }
        
        info!("Updating mining_threads to: {}", threads);
        config.app_settings.mining_threads = threads;
    }

    // Save the updated config using the inner ConfigManager
    match config_manager
        .update_app_settings(config.app_settings.clone())
        .await
    {
        Ok(_) => {
            info!("Settings updated successfully - final developer_mode: {}", config.app_settings.developer_mode);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to update settings: {}", e);
            Err(format_error(e))
        }
    }
}

/// Command to get current application settings
#[command]
pub async fn get_app_settings(
    config_manager_arc: State<'_, Arc<ConfigManager>>, // Change type to State<'_, Arc<ConfigManager>>
) -> CommandResult<AppSettings> {    debug!("Command: get_app_settings");

    // Access the inner ConfigManager through the Arc
    let config = config_manager_arc.inner().get_config();
    info!("Current developer_mode value: {}", config.app_settings.developer_mode);
    Ok(config.app_settings.clone())
}

/// Command to open a wallet
#[command]
pub async fn open_wallet(
    wallet_name: String,
    password: Option<String>,
    wallet_manager: State<'_, AsyncWalletManager>,
    security_manager: State<'_, AsyncSecurityManager>,
    wallet_sync: State<'_, AsyncWalletSyncService>,
) -> CommandResult<bool> {
    info!("Command: open_wallet for wallet: {}", wallet_name);

    // First, determine if the wallet exists and if it's secured
    let is_wallet_secured = {
        let manager = wallet_manager.get_manager().await;
        match manager.find_wallet_by_name(&wallet_name) {
            Some(info) => {
                debug!(
                    "Found wallet info for '{}', secured: {}",
                    wallet_name, info.secured
                );
                info.secured
            }
            None => {
                error!("Wallet '{}' not found", wallet_name);
                return Err(format!("Wallet '{}' not found", wallet_name));
            }
        }
    }; // Release the mutex lock here

    // Handle secured vs unsecured wallets separately
    if is_wallet_secured {
        // For secured wallets, validate the password
        let password = match password {
            Some(pwd) if !pwd.is_empty() => pwd,
            _ => {
                error!("Password is required for secured wallet '{}'", wallet_name);
                return Err("Password is required for this secured wallet".to_string());
            }
        };

        // Authenticate with security manager first
        let mut sec_manager = security_manager.get_manager().await;
        match sec_manager.authenticate(&password) {
            Ok(_) => {
                debug!(
                    "Authentication succeeded for secured wallet: {}",
                    wallet_name
                );
                drop(sec_manager); // Explicitly release security manager lock                // Now open the wallet with the validated password
                let mut manager = wallet_manager.get_manager().await;
                match manager.open_wallet(&wallet_name, Some(&password)) {
                    Ok(_) => {
                        info!("Successfully opened secured wallet: {}", wallet_name);
                        
                        // Automatically start wallet synchronization
                        if let Some(wallet) = manager.get_current_wallet() {
                            let addresses: Vec<String> = wallet.data.addresses.iter()
                                .map(|addr| addr.address.clone())
                                .collect();
                            
                            if !addresses.is_empty() {
                                info!("Starting automatic sync for wallet: {} with {} addresses", wallet_name, addresses.len());
                                if let Err(e) = wallet_sync.start_wallet_sync(wallet_name.clone(), addresses).await {
                                    warn!("Failed to start automatic wallet sync: {}", e);
                                }
                            } else {
                                info!("No addresses found in wallet: {}, skipping sync", wallet_name);
                            }
                        }
                        
                        Ok(true)
                    }
                    Err(e) => {
                        error!("Failed to open secured wallet: {}", e);
                        Err(format_error(e))
                    }
                }
            }
            Err(e) => {
                error!("Authentication failed: {}", e);
                Err(format_error(e))
            }
        }
    } else {        // For unsecured wallets, just open directly
        let mut manager = wallet_manager.get_manager().await;
        match manager.open_wallet(&wallet_name, None) {
            Ok(_) => {
                info!("Successfully opened unsecured wallet: {}", wallet_name);
                
                // Automatically start wallet synchronization
                if let Some(wallet) = manager.get_current_wallet() {
                    let addresses: Vec<String> = wallet.data.addresses.iter()
                        .map(|addr| addr.address.clone())
                        .collect();
                    
                    if !addresses.is_empty() {
                        info!("Starting automatic sync for wallet: {} with {} addresses", wallet_name, addresses.len());
                        if let Err(e) = wallet_sync.start_wallet_sync(wallet_name.clone(), addresses).await {
                            warn!("Failed to start automatic wallet sync: {}", e);
                        }
                    } else {
                        info!("No addresses found in wallet: {}, skipping sync", wallet_name);
                    }
                }
                
                Ok(true)
            }
            Err(e) => {
                error!("Failed to open unsecured wallet: {}", e);
                Err(format_error(e))
            }
        }
    }
}

/// Command to initiate application shutdown
#[command]
pub async fn shutdown_application(app: tauri::AppHandle) -> CommandResult<bool> {
    info!("Command: shutdown_application received");

    // Set the shutdown flag to prevent infinite loops
    crate::SHUTDOWN_IN_PROGRESS.store(true, std::sync::atomic::Ordering::SeqCst);

    // Run shutdown process in another thread to avoid blocking
    let app_handle = app.clone();
    tokio::spawn(async move {
        info!("Starting application shutdown sequence");

        // Close any open wallet first
        if let Some(wallet_manager) = app_handle.try_state::<AsyncWalletManager>() {
            match wallet_manager.shutdown().await {
                Ok(_) => info!("Wallet manager shutdown completed successfully"),
                Err(e) => error!("Wallet manager shutdown error: {}", e),
            }
        }

        // Log application shutdown
        logging::log_app_shutdown();

        // Wait a moment to ensure logs are written
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Send shutdown complete event to frontend
        if let Some(main_window) = app_handle.get_webview_window("main") {
            let _ = main_window.emit("app-shutdown-complete", ());

            // Give the frontend a moment to react
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        // Exit the application
        app_handle.exit(0);
    });

    // Return immediately, the actual shutdown happens in the background
    Ok(true)
}

/// Command to secure an existing wallet with a password
#[command]
pub async fn secure_wallet(
    wallet_name: String,
    password: String,
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<bool> {
    info!("Command: secure_wallet for wallet: {}", wallet_name);

    let mut manager = wallet_manager.get_manager().await;
    match manager.secure_wallet(&wallet_name, &password) {
        Ok(_) => {
            info!("Successfully secured wallet: {}", wallet_name);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to secure wallet: {}", e);
            Err(format_error(e))
        }
    }
}

/// Command to recover a wallet using a seed phrase
#[command]
pub async fn recover_wallet(
    wallet_name: String,
    _seed_phrase: String,
    password: String,
    use_password: bool,
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<bool> {
    info!("Command: recover_wallet with name: {}", wallet_name);
    debug!("Recovering wallet using seed phrase");

    // TODO: In the future, implement proper recovery from seed phrase
    // For now, we'll reuse the create_wallet logic as a placeholder

    // If password protection is disabled, use empty password
    let effective_password = if use_password {
        password
    } else {
        String::new()
    };

    let mut manager = wallet_manager.get_manager().await;
    match manager.create_wallet(&wallet_name, &effective_password) {
        Ok(_) => {
            info!("Successfully recovered wallet: {}", wallet_name);
            // Now open the newly created wallet
            match manager.open_wallet(
                &wallet_name,
                if use_password {
                    Some(&effective_password)
                } else {
                    None
                },
            ) {
                Ok(_) => {
                    info!("Successfully opened recovered wallet: {}", wallet_name);
                    Ok(true)
                }
                Err(e) => {
                    error!("Recovered wallet but failed to open it: {}", e);
                    Err(format_error(e))
                }
            }
        }
        Err(e) => {
            error!("Failed to recover wallet: {}", e);
            Err(format_error(e))
        }
    }
}

/// Command to get the application version
#[command]
pub fn get_app_version() -> CommandResult<String> {
    debug!("Command: get_app_version");
    // Get version from the Cargo.toml via environment variable
    let version = crate::APP_VERSION;
    debug!("App version: {}", version);
    Ok(version.to_string())
}

/// Command to generate a new 12-word BIP-39 seed phrase using cryptographically secure methods
#[command]
pub async fn generate_seed_phrase() -> CommandResult<String> {
    debug!("Command: generate_seed_phrase using BIP39 standard");
      // Generate entropy for 128-bit security (12 words)
    let mut entropy = [0u8; 16];
    rand::rng().fill(&mut entropy);
    
    // Create mnemonic from entropy using BIP39 standard
    let mnemonic = Mnemonic::from_entropy(&entropy)
        .map_err(|e| format!("Failed to generate BIP39 mnemonic: {}", e))?;
    
    let phrase = mnemonic.to_string();
    
    // Get first and last words for safe logging (never log the full phrase)
    let words: Vec<&str> = phrase.split_whitespace().collect();
    let first_word = words.first().unwrap_or(&"");
    let last_word = words.last().unwrap_or(&"");
    
    debug!("Generated BIP39 seed phrase (first word: {}, last word: {})", first_word, last_word);
    info!("Successfully generated secure BIP39 mnemonic with {} words", words.len());
    
    Ok(phrase)
}

/// Command to open a folder in the system's file explorer
#[command]
pub async fn open_folder_in_explorer(path: String) -> CommandResult<bool> {
    info!("Command: open_folder_in_explorer with path: {}", path);
    
    // Create a PathBuf from the path string
    let path_buf = std::path::PathBuf::from(&path);
    info!("Converted path to PathBuf: {}", path_buf.display());
    
    // Check if the path exists
    let exists = path_buf.exists();
    info!("Path exists check: {}", exists);
    
    if !exists {
        error!("Path does not exist: {}", path);
        return Err(format!("The path '{}' does not exist.", path));
    }
    
    // Log file or directory status
    let is_file = path_buf.is_file();
    let is_dir = path_buf.is_dir();
    info!("Path is file: {}, Path is directory: {}", is_file, is_dir);
    
    // Determine if this is a file or directory
    let target_path = if is_file {
        // If it's a file, we want to open its parent directory
        match path_buf.parent() {
            Some(parent) => {
                info!("Path is a file, opening parent directory: {}", parent.display());
                parent.to_path_buf()
            },
            None => {
                error!("Could not determine parent directory for: {}", path);
                return Err("Could not determine the directory to open.".to_string());
            }
        }
    } else {
        // It's a directory, use it directly
        info!("Path is a directory, using directly");
        path_buf
    };
    
    // Try to get canonical path
    info!("About to open path: {}", target_path.display());
    let canonical_result = target_path.canonicalize();
    
    if let Ok(canonical_path) = &canonical_result {
        info!("Canonical path: {}", canonical_path.display());
    } else if let Err(e) = &canonical_result {
        warn!("Failed to canonicalize path: {}", e);
    }
    
    // Use the canonical path if available, otherwise use the target path
    let final_path = canonical_result.unwrap_or_else(|_| target_path);
    
    // Open the directory with the system file explorer
    info!("Using opener to open path: {}", final_path.display());
    match opener::open(&final_path) {
        Ok(_) => {
            info!("Successfully opened directory: {}", final_path.display());
            Ok(true)
        },
        Err(e) => {
            error!("Failed to open directory: {}", e);
            Err(format!("Failed to open directory: {}", e))
        }
    }
}

/// Command to open a folder using platform-specific shell commands
/// This is a fallback method if the opener crate fails
#[command]
pub async fn open_folder_with_shell_command(path: String) -> CommandResult<bool> {
    info!("Command: open_folder_with_shell_command with path: {}", path);
    
    // Create a PathBuf from the path string
    let path_buf = std::path::PathBuf::from(&path);
    
    // Check if the path exists
    if !path_buf.exists() {
        error!("Path does not exist: {}", path);
        return Err(format!("The path '{}' does not exist.", path));
    }
    
    // Determine if this is a file or directory
    let target_path = if path_buf.is_file() {
        // If it's a file, we want to open its parent directory
        match path_buf.parent() {
            Some(parent) => {
                info!("Path is a file, opening parent directory: {}", parent.display());
                parent.to_path_buf()
            },
            None => {
                error!("Could not determine parent directory for: {}", path);
                return Err("Could not determine the directory to open.".to_string());
            }
        }
    } else {
        // It's a directory, use it directly
        path_buf
    };
    
    // Log the final path we're trying to open
    info!("Attempting to open directory with shell command: {}", target_path.display());
    
    // Use platform-specific commands to open the folder
    let result = if cfg!(target_os = "windows") {
        // On Windows, use explorer.exe
        let path_str = match target_path.to_str() {
            Some(s) => s.to_string(),
            None => {
                error!("Failed to convert path to string");
                return Err("Failed to convert path to string".to_string());
            }
        };
        
        // Use explorer.exe to open the folder
        match std::process::Command::new("explorer")
            .arg(&path_str)
            .spawn() {
                Ok(_) => {
                    info!("Successfully opened Windows Explorer with path: {}", path_str);
                    true
                },
                Err(e) => {
                    error!("Failed to open Windows Explorer: {}", e);
                    false
                }
            }
    } else if cfg!(target_os = "macos") {
        // On macOS, use open command
        match std::process::Command::new("open")
            .arg(target_path)
            .spawn() {
                Ok(_) => {
                    info!("Successfully opened macOS Finder with path");
                    true
                },
                Err(e) => {
                    error!("Failed to open macOS Finder: {}", e);
                    false
                }
            }
    } else if cfg!(target_os = "linux") {
        // On Linux, try xdg-open
        match std::process::Command::new("xdg-open")
            .arg(target_path)
            .spawn() {
                Ok(_) => {
                    info!("Successfully opened Linux file browser with path");
                    true
                },
                Err(e) => {
                    error!("Failed to open Linux file browser: {}", e);
                    false
                }
            }
    } else {
        error!("Unsupported operating system for shell command folder opening");
        false
    };
    
    if result {
        Ok(true)
    } else {
        Err("Failed to open folder with shell command".to_string())
    }
}

/// Command to delete a wallet by name
#[command]
pub async fn delete_wallet(
    wallet_name: String,
    wallet_manager_state: State<'_, AsyncWalletManager>, // Changed param name for clarity in thought process, will use original if needed
    config_manager_arc: State<'_, Arc<ConfigManager>>,
) -> CommandResult<bool> {
    info!("Command: delete_wallet for wallet: {}", wallet_name);

    // --- Step 1: Close the wallet if it's the one being deleted and is open ---
    { // Scope for first WalletManager lock
        let mut manager = wallet_manager_state.get_manager().await;
        if manager.get_current_wallet().map_or(false, |w| w.name == wallet_name) {
            info!("Wallet '{}' is currently open. Closing it before deletion.", wallet_name);
            manager.close_wallet(); // Assumes WalletManager::close_wallet() returns ()
            info!("Successfully closed wallet '{}'.", wallet_name);
        }
        // WalletManager lock (manager) is released here
    }

    // --- Step 2: Get the relative path of the wallet from configuration ---
    let relative_wallet_path = { // Scope for ConfigManager access
        let config_access = config_manager_arc.inner();
        let current_config = config_access.get_config(); // Assumes get_config() returns &Config or similar
        match current_config.wallets.iter().find(|w| w.name == wallet_name) {
            Some(info) => info.path.clone(), // This is String, assumed relative path
            None => {
                error!("Wallet '{}' not found in configuration.", wallet_name);
                return Err(format!("Wallet '{}' not found in configuration", wallet_name));
            }
        }
    };

    // --- Step 3: Get WalletManager's base directory for wallets to construct full path ---
    let full_wallet_path_to_delete = { // Scope for another WalletManager lock (read-only part)
        let manager = wallet_manager_state.get_manager().await;
        let wallets_base_dir = manager.get_wallets_dir(); // Returns PathBuf
        wallets_base_dir.join(&relative_wallet_path) // Join to get full PathBuf
        // WalletManager lock (manager) is released here
    };      // --- Step 4: Remove wallet entry from configuration using WalletManager's method ---
    // This was the original location of this logic in the old delete_wallet.
    { // Scope for WalletManager lock (modifying config part)
        let mut manager = wallet_manager_state.get_manager().await;
        if let Err(e) = manager.remove_wallet_from_config(&wallet_name).await {
            error!("Failed to remove wallet '{}' from config: {}", wallet_name, e);
            // If this fails, we haven't deleted files yet, which is safer.
            return Err(format!("Failed to remove wallet from config: {}", e));
        }
        // WalletManager lock (manager) is released here
    }

    // --- Step 5: Delete the wallet directory from filesystem ---
    if full_wallet_path_to_delete.exists() {
        match tokio::fs::remove_dir_all(&full_wallet_path_to_delete).await {
            Ok(_) => {
                info!("Deleted wallet directory at {}", full_wallet_path_to_delete.display());
            },
            Err(e) => {
                error!("Failed to delete wallet directory {}: {}", full_wallet_path_to_delete.display(), e);
                // CRITICAL: Wallet is removed from config, but files still exist.
                // This is an inconsistent state. This error should be handled carefully by the user.
                return Err(format!("Wallet config removed, but failed to delete wallet files: {}. Manual cleanup may be required at {}", e, full_wallet_path_to_delete.display()));
            }
        }
    } else {
        // If directory doesn't exist, but config removal was successful, log as warning.
        warn!("Wallet directory {} does not exist, skipping deletion. Wallet was already removed from config.", full_wallet_path_to_delete.display());
    }
    
    info!("Successfully deleted wallet '{}'", wallet_name);
    Ok(true)
}

/// Command to get a fully qualified wallet path
#[command]
pub async fn get_fully_qualified_wallet_path(
    relative_path: String,
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<String> {
    debug!("Command: get_fully_qualified_wallet_path for path '{}'", relative_path);
    
    let manager = wallet_manager.get_manager().await;
    
    // Get the base wallets directory
    let wallets_dir = manager.get_wallets_dir();
    debug!("Base wallets directory: {}", wallets_dir.display());
    
    // Join the relative path with the base directory
    let full_path = wallets_dir.join(relative_path);
    debug!("Fully qualified path: {}", full_path.display());
    
    // Convert to string for return
    match full_path.to_str() {
        Some(path_str) => Ok(path_str.to_string()),
        None => Err("Failed to convert path to string".to_string())
    }
}

/// Simple greeting command for demo purposes
#[command]
pub fn greet(name: String) -> String {
    info!("Command: greet - {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Command to clean up orphaned wallet directories
/// Deletes all wallet files/folders in the wallets directory that are not present in the app configuration
#[command]
pub async fn cleanup_orphaned_wallets(
    wallet_manager: State<'_, AsyncWalletManager>,
    config_manager: State<'_, Arc<ConfigManager>>,
) -> CommandResult<Vec<String>> {
    info!("Command: cleanup_orphaned_wallets - Starting cleanup process");
    
    let manager = wallet_manager.get_manager().await;
    let config = config_manager.get_config();
      // Get the base wallets directory
    let wallets_dir = manager.get_wallets_dir();
    info!("Scanning wallets directory: {}", wallets_dir.display());
    
    // Ensure the wallets directory exists
    if !wallets_dir.exists() {
        info!("Wallets directory does not exist, nothing to clean up");
        return Ok(vec![]);
    }
    
    // Get list of wallet names from config
    let configured_wallets: std::collections::HashSet<String> = config
        .wallets
        .iter()
        .map(|w| w.name.clone())
        .collect();
      debug!("Configured wallets: {:?}", configured_wallets);
    
    let mut deleted_items = Vec::new();
    
    // Read the wallets directory
    match std::fs::read_dir(&wallets_dir) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(dir_entry) => {
                        let path = dir_entry.path();
                        let file_name = match path.file_name() {
                            Some(name) => name.to_string_lossy().to_string(),
                            None => continue,
                        };
                        
                        // Skip if this is a configured wallet
                        if configured_wallets.contains(&file_name) {
                            continue;
                        }
                        
                        // This is an orphaned wallet directory/file
                        info!("Found orphaned wallet item: {}", file_name);
                        
                        // Attempt to delete it
                        if path.is_dir() {
                            match std::fs::remove_dir_all(&path) {
                                Ok(()) => {
                                    info!("Deleted orphaned wallet directory: {}", file_name);
                                    deleted_items.push(format!("Directory: {}", file_name));
                                }
                                Err(e) => {
                                    error!("Failed to delete orphaned wallet directory {}: {}", file_name, e);
                                    return Err(format!("Failed to delete directory {}: {}", file_name, e));
                                }
                            }
                        } else {
                            match std::fs::remove_file(&path) {
                                Ok(()) => {
                                    info!("Deleted orphaned wallet file: {}", file_name);
                                    deleted_items.push(format!("File: {}", file_name));
                                }
                                Err(e) => {
                                    error!("Failed to delete orphaned wallet file {}: {}", file_name, e);
                                    return Err(format!("Failed to delete file {}: {}", file_name, e));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading directory entry: {}", e);
                        return Err(format!("Error reading directory entry: {}", e));
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to read wallets directory: {}", e);
            return Err(format!("Failed to read wallets directory: {}", e));
        }
    }
    
    if deleted_items.is_empty() {
        info!("No orphaned wallet items found to clean up");
    } else {
        info!("Cleaned up {} orphaned wallet items", deleted_items.len());
    }
    
    Ok(deleted_items)
}

/// Command to delete all wallets from both config and disk
/// Deletes all wallets listed in the config file and removes all wallet directories from the wallets folder
#[command]
pub async fn delete_all_wallets(
    wallet_manager: State<'_, AsyncWalletManager>,
    config_manager: State<'_, Arc<ConfigManager>>,
    app: tauri::AppHandle,
) -> CommandResult<Vec<String>> {    info!("Command: delete_all_wallets - Starting deletion process");
    debug!("Command: delete_all_wallets");
      // Close any currently open wallet first - do this separately to avoid deadlock
    {
        let manager = wallet_manager.get_manager().await;
        if manager.get_current_wallet().is_some() {
            info!("Closing currently open wallet before deletion");
            drop(manager); // Release the lock explicitly
            let mut manager_mut = wallet_manager.get_manager().await;
            manager_mut.close_wallet();
        }
    } // Ensure the manager lock is dropped here
    
    // Now get a fresh lock for the deletion operations
    let manager = wallet_manager.get_manager().await;
    let config = config_manager.get_config();
    
    let mut deleted_items = Vec::new();
      // Step 1: Delete wallets from their configured paths
    for wallet_info in &config.wallets {
        info!("Processing wallet from config: {}", wallet_info.name);
        
        // Get the full path to the wallet
        let wallet_path = if std::path::Path::new(&wallet_info.path).is_absolute() {
            std::path::PathBuf::from(&wallet_info.path)
        } else {
            // If relative path, join with the wallets directory
            manager.get_wallets_dir().join(&wallet_info.path)
        };
        
        debug!("Attempting to delete wallet at path: {}", wallet_path.display());
        
        if wallet_path.exists() {
            if wallet_path.is_dir() {
                match std::fs::remove_dir_all(&wallet_path) {
                    Ok(()) => {
                        info!("Deleted wallet directory: {}", wallet_info.name);
                        deleted_items.push(format!("Config wallet (dir): {} at {}", wallet_info.name, wallet_path.display()));
                    }
                    Err(e) => {
                        error!("Failed to delete wallet directory {}: {}", wallet_info.name, e);
                        return Err(format!("Failed to delete wallet directory {}: {}", wallet_info.name, e));
                    }
                }
            } else if wallet_path.is_file() {
                match std::fs::remove_file(&wallet_path) {
                    Ok(()) => {
                        info!("Deleted wallet file: {}", wallet_info.name);
                        deleted_items.push(format!("Config wallet (file): {} at {}", wallet_info.name, wallet_path.display()));
                    }
                    Err(e) => {
                        error!("Failed to delete wallet file {}: {}", wallet_info.name, e);
                        return Err(format!("Failed to delete wallet file {}: {}", wallet_info.name, e));
                    }
                }
            }        } else {
            debug!("Wallet path does not exist, skipping: {}", wallet_path.display());
            deleted_items.push(format!("Config wallet (missing): {} (path not found: {})", wallet_info.name, wallet_path.display()));
        }
    }
      // Step 2: Delete any remaining items in the wallets directory
    let wallets_dir = manager.get_wallets_dir();
    info!("Cleaning up remaining items in wallets directory: {}", wallets_dir.display());
    
    if wallets_dir.exists() {
        match std::fs::read_dir(&wallets_dir) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(dir_entry) => {
                            let path = dir_entry.path();
                            let file_name = match path.file_name() {
                                Some(name) => name.to_string_lossy().to_string(),
                                None => continue,
                            };
                            
                            debug!("Found remaining item in wallets directory: {}", file_name);
                            
                            if path.is_dir() {
                                match std::fs::remove_dir_all(&path) {
                                    Ok(()) => {
                                        info!("Deleted remaining wallet directory: {}", file_name);
                                        deleted_items.push(format!("Remaining directory: {}", file_name));
                                    }
                                    Err(e) => {
                                        error!("Failed to delete remaining directory {}: {}", file_name, e);
                                        return Err(format!("Failed to delete remaining directory {}: {}", file_name, e));
                                    }
                                }
                            } else {
                                match std::fs::remove_file(&path) {
                                    Ok(()) => {
                                        info!("Deleted remaining wallet file: {}", file_name);
                                        deleted_items.push(format!("Remaining file: {}", file_name));
                                    }
                                    Err(e) => {
                                        error!("Failed to delete remaining file {}: {}", file_name, e);
                                        return Err(format!("Failed to delete remaining file {}: {}", file_name, e));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error reading directory entry: {}", e);
                            return Err(format!("Error reading directory entry: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to read wallets directory: {}", e);
                return Err(format!("Failed to read wallets directory: {}", e));
            }
        }
    }
      // Step 3: Clear the wallets from the config file
    info!("Clearing wallets from config file");
    let mut new_config = config.clone();
    new_config.wallets.clear();
    
    match config_manager.update_config(new_config).await {
        Ok(()) => {
            info!("Successfully cleared wallets from config file");
            deleted_items.push("Config file: Cleared all wallet entries".to_string());
        }
        Err(e) => {
            error!("Failed to clear wallets from config: {}", e);
            return Err(format!("Failed to clear wallets from config: {}", e));
        }
    }
      if deleted_items.is_empty() {
        info!("No wallets found to delete");
    } else {
        info!("Successfully deleted {} wallet items", deleted_items.len());
        
        // Emit an event to notify frontend that all wallets have been deleted
        if let Some(main_window) = app.get_webview_window("main") {
            let _ = main_window.emit("wallets-deleted", ());
        }
    }
    
    Ok(deleted_items)
}

/// Structure containing current wallet information for the Account page
#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentWalletInfo {
    pub name: String,
    pub addresses: Vec<AddressDetails>,
    pub master_public_key: String,
    pub balance: u64,
    pub is_secured: bool,
}

/// Detailed address information
#[derive(Debug, Serialize, Deserialize)]
pub struct AddressDetails {
    pub address: String,
    pub public_key: String,
    pub derivation_path: String,
    pub address_type: String,
    pub label: Option<String>,
}

/// Command to get current wallet information for the Account page
#[command]
pub async fn get_current_wallet_info(
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<Option<CurrentWalletInfo>> {
    info!("Command: get_current_wallet_info");

    let manager = wallet_manager.get_manager().await;

    // Check if a wallet is currently open
    let current_wallet = match manager.get_current_wallet() {
        Some(wallet) => wallet,
        None => {
            debug!("No wallet is currently open");
            return Ok(None);
        }
    };

    let wallet_name = current_wallet.name.clone();
    debug!("Getting wallet info for: {}", wallet_name);

    // Get addresses with detailed information
    let mut addresses = Vec::new();
    for (_address_str, key_pair) in &current_wallet.data.keys {
        addresses.push(AddressDetails {
            address: key_pair.address.clone(),
            public_key: key_pair.public_key.clone(),
            derivation_path: key_pair.derivation_path.clone(),
            address_type: match key_pair.key_type {
                crate::wallet_data::KeyType::Legacy => "Legacy (P2PKH)".to_string(),
                crate::wallet_data::KeyType::SegWit => "SegWit (P2SH-P2WPKH)".to_string(),
                crate::wallet_data::KeyType::NativeSegWit => "Native SegWit (P2WPKH)".to_string(),
                crate::wallet_data::KeyType::Taproot => "Taproot (P2TR)".to_string(),
            },
            label: None, // Can be extended in the future
        });
    }

    // If no keys in the main keys map, fall back to addresses list
    if addresses.is_empty() {
        for (index, addr_info) in current_wallet.data.addresses.iter().enumerate() {
            addresses.push(AddressDetails {
                address: addr_info.address.clone(),
                public_key: "Unknown".to_string(),
                derivation_path: format!("m/44'/0'/0'/0/{}", index),
                address_type: "Unknown".to_string(),
                label: addr_info.label.clone(),
            });
        }
    }

    let wallet_info = CurrentWalletInfo {
        name: wallet_name.clone(),
        addresses,
        master_public_key: current_wallet.data.master_public_key.clone(),
        balance: current_wallet.data.balance,
        is_secured: manager.is_current_wallet_secured().unwrap_or(false),
    };

    info!("Successfully retrieved wallet info for: {}", wallet_name);
    Ok(Some(wallet_info))
}

/// Command to get the private key of the currently open wallet
#[command]
pub async fn get_wallet_private_key(
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<String> {
    info!("Command: get_wallet_private_key");

    let manager = wallet_manager.get_manager().await;

    // Check if a wallet is currently open
    let current_wallet = match manager.get_current_wallet() {
        Some(wallet) => wallet,
        None => {
            error!("No wallet is currently open");
            return Err("No wallet is currently open".to_string());
        }
    };    let wallet_name = current_wallet.name.clone();
    debug!("Getting private key for wallet: {}", wallet_name);

    // Get the private key from the wallet data that's already loaded in memory
    match &current_wallet.data.master_private_key {
        Some(private_key) => {
            info!("Successfully retrieved private key for wallet: {}", wallet_name);
            Ok(private_key.clone())
        }
        None => {
            error!("No private key found in wallet data for: {}", wallet_name);
            Err("No private key found in wallet data".to_string())
        }
    }
}

/// Command to show the main window (used by tray)
#[command]
pub async fn show_main_window(app_handle: tauri::AppHandle) -> CommandResult<()> {
    debug!("Command: show_main_window");
    
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show().map_err(format_error)?;
        window.set_focus().map_err(format_error)?;
        info!("Main window shown and focused");
        Ok(())
    } else {
        error!("Main window not found");
        Err("Main window not found".to_string())
    }
}

/// Command to hide the main window (minimize to tray)
#[command]
pub async fn hide_to_tray(app_handle: tauri::AppHandle) -> CommandResult<()> {
    debug!("Command: hide_to_tray");
    
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().map_err(format_error)?;
        info!("Main window hidden to tray");
        Ok(())
    } else {
        error!("Main window not found");
        Err("Main window not found".to_string())
    }
}

/// Command to update the system tray menu with current wallet status
#[command]
pub async fn update_tray_wallet_status(
    _app_handle: tauri::AppHandle,
    wallet_name: Option<String>,
) -> CommandResult<()> {
    debug!("Command: update_tray_wallet_status for wallet: {:?}", wallet_name);
    
    // For now, just log the update - actual menu item updating in Tauri 2 is complex
    let wallet_text = match wallet_name {
        Some(name) => format!("Wallet: {}", name),
        None => "No wallet open".to_string(),
    };
    
    info!("Tray wallet status updated to: {}", wallet_text);
    
    // TODO: Implement actual menu item text update when Tauri 2 API supports it
    // Currently, Tauri 2 doesn't have a straightforward way to update menu item text
    // This is a placeholder for future implementation
    
    Ok(())
}

/// Command to update the system tray menu with network status
#[command]
pub async fn update_tray_network_status(
    _app_handle: tauri::AppHandle,
    is_connected: bool,
    peer_count: Option<u32>,
) -> CommandResult<()> {
    debug!("Command: update_tray_network_status - connected: {}, peers: {:?}", is_connected, peer_count);
    
    let status_text = if is_connected {
        match peer_count {
            Some(count) => format!("Network: Connected ({} peers)", count),
            None => "Network: Connected".to_string(),
        }
    } else {
        "Network: Disconnected".to_string()
    };
    
    info!("Tray network status updated to: {}", status_text);
    
    // TODO: Implement actual menu item text update when Tauri 2 API supports it
    // Currently, Tauri 2 doesn't have a straightforward way to update menu item text
    // This is a placeholder for future implementation
    
    Ok(())
}

/// Command to check the synchronization status of the blockchain
#[command]
pub async fn check_sync_status(
    blockchain_sync_service: State<'_, AsyncBlockchainSyncService>,
) -> CommandResult<NetworkStatus> {    info!("Command: check_sync_status");

    // Get the current network status from the blockchain sync service
    let status = blockchain_sync_service.get_network_status().await;

    info!("Current network status: {:?}", status);

    Ok(status)
}

/// Command to force synchronization with the blockchain
#[command]
pub async fn force_sync(
    app: tauri::AppHandle,
    blockchain_sync_service: State<'_, AsyncBlockchainSyncService>,
) -> CommandResult<bool> {
    info!("Command: force_sync");

    // Trigger a manual sync with the blockchain
    blockchain_sync_service.trigger_sync(&app).await.map_err(format_error)?;

    info!("Manual synchronization with the blockchain has been triggered");

    Ok(true)
}

/// Command to get current blockchain network status
#[command]
pub async fn get_network_status(
    app: tauri::AppHandle,
    blockchain_sync: State<'_, AsyncBlockchainSyncService>,
) -> CommandResult<NetworkStatus> {
    info!("Command: get_network_status");
    let status = blockchain_sync.get_network_status_with_network_height(&app).await;
    info!("Network status: connected={}, local_height={}, network_height={}, syncing={}, peers={}", 
           status.is_connected, status.current_height, status.network_height, status.is_syncing, status.peer_count);
    Ok(status)
}

/// Command to get current block height
#[command]
pub async fn get_block_height(
    blockchain_sync: State<'_, AsyncBlockchainSyncService>,
) -> CommandResult<i32> {
    debug!("Command: get_block_height");
    let height = blockchain_sync.get_block_height().await;
    debug!("Current block height: {}", height);
    Ok(height)
}

/// Command to check if blockchain is currently syncing
#[command]
pub async fn is_blockchain_syncing(
    blockchain_sync: State<'_, AsyncBlockchainSyncService>,
) -> CommandResult<bool> {
    debug!("Command: is_blockchain_syncing");
    let syncing = blockchain_sync.is_syncing().await;
    debug!("Blockchain syncing: {}", syncing);
    Ok(syncing)
}

/// Command to check network connection status
#[command]
pub async fn is_network_connected(
    blockchain_sync: State<'_, AsyncBlockchainSyncService>,
) -> CommandResult<bool> {
    debug!("Command: is_network_connected");
    let connected = blockchain_sync.is_connected().await;
    debug!("Network connected: {}", connected);
    Ok(connected)
}

/// Command to get peer count
#[command]
pub async fn get_peer_count(
    blockchain_sync: State<'_, AsyncBlockchainSyncService>,
) -> CommandResult<i32> {
    debug!("Command: get_peer_count");
    let count = blockchain_sync.get_peer_count().await;
    debug!("Peer count: {}", count);
    Ok(count)
}

// ============================================================================
// Wallet Sync Commands
// ============================================================================

/// Command to start syncing a wallet
#[command]
pub async fn start_wallet_sync(
    wallet_sync: State<'_, AsyncWalletSyncService>,
    wallet_id: String,
    addresses: Vec<String>,
) -> CommandResult<()> {
    debug!("Command: start_wallet_sync for wallet: {}", wallet_id);
    
    wallet_sync.start_wallet_sync(wallet_id.clone(), addresses).await
        .map_err(format_error)?;
    
    info!("Started wallet sync for: {}", wallet_id);
    Ok(())
}

/// Command to stop syncing a wallet
#[command]
pub async fn stop_wallet_sync(
    wallet_sync: State<'_, AsyncWalletSyncService>,
    wallet_id: String,
) -> CommandResult<()> {
    debug!("Command: stop_wallet_sync for wallet: {}", wallet_id);
    
    wallet_sync.stop_wallet_sync(&wallet_id).await
        .map_err(format_error)?;
    
    info!("Stopped wallet sync for: {}", wallet_id);
    Ok(())
}

/// Command to get wallet sync status
#[command]
pub async fn get_wallet_sync_status(
    wallet_sync: State<'_, AsyncWalletSyncService>,
    wallet_id: String,
) -> CommandResult<Option<WalletSyncStatus>> {
    debug!("Command: get_wallet_sync_status for wallet: {}", wallet_id);
    
    let status = wallet_sync.get_wallet_sync_status(&wallet_id).await;
    debug!("Wallet sync status for {}: {:?}", wallet_id, status.is_some());
    Ok(status)
}

/// Command to get all wallet sync statuses
#[command]
pub async fn get_all_wallet_sync_statuses(
    wallet_sync: State<'_, AsyncWalletSyncService>,
) -> CommandResult<std::collections::HashMap<String, WalletSyncStatus>> {
    debug!("Command: get_all_wallet_sync_statuses");
    
    let statuses = wallet_sync.get_all_sync_statuses().await;
    debug!("Retrieved {} wallet sync statuses", statuses.len());
    Ok(statuses)
}

// ============================================================================
// Mining Commands
// ============================================================================

/// Command to start mining for a wallet
#[command]
pub async fn start_mining(
    mining_service: State<'_, AsyncMiningService>,
    wallet_id: String,
    mining_address: String,
) -> CommandResult<()> {
    debug!("Command: start_mining for wallet: {} at address: {}", wallet_id, mining_address);
    
    mining_service.start_mining(wallet_id.clone(), mining_address).await
        .map_err(format_error)?;
    
    info!("Started mining for wallet: {}", wallet_id);
    Ok(())
}

/// Command to stop mining for a wallet
#[command]
pub async fn stop_mining(
    mining_service: State<'_, AsyncMiningService>,
    wallet_id: String,
) -> CommandResult<()> {
    debug!("Command: stop_mining for wallet: {}", wallet_id);
    
    mining_service.stop_mining(&wallet_id).await
        .map_err(format_error)?;
    
    info!("Stopped mining for wallet: {}", wallet_id);
    Ok(())
}

/// Command to get mining status for a wallet
#[command]
pub async fn get_mining_status(
    mining_service: State<'_, AsyncMiningService>,
    wallet_id: String,
) -> CommandResult<Option<MiningStatus>> {
    debug!("Command: get_mining_status for wallet: {}", wallet_id);
    
    let status = mining_service.get_mining_status(&wallet_id).await;
    debug!("Mining status for {}: {:?}", wallet_id, status.is_some());
    Ok(status)
}

/// Command to get all mining statuses
#[command]
pub async fn get_all_mining_statuses(
    mining_service: State<'_, AsyncMiningService>,
) -> CommandResult<std::collections::HashMap<String, MiningStatus>> {
    debug!("Command: get_all_mining_statuses");
    
    let statuses = mining_service.get_all_mining_statuses().await;
    debug!("Retrieved {} mining statuses", statuses.len());
    Ok(statuses)
}

/// Check if blockchain database exists at configured or default location
#[command]
pub async fn check_blockchain_database_exists(
    config_manager: State<'_, Arc<ConfigManager>>,
) -> CommandResult<bool> {
    info!("Command: check_blockchain_database_exists");
    
    let config = config_manager.get_config();
    
    // Get the default location for fallback
    let default_blockchain_data_dir = match dirs::data_dir() {
        Some(dir) => dir.join("com.b-rad-coin.app").join("blockchain"),
        None => {
            error!("Failed to determine default blockchain data directory");
            return Ok(false);
        }
    };
    
    // Check if there's a custom location configured
    if let Some(custom_location) = &config.app_settings.local_blockchain_file_location {
        let custom_path = std::path::Path::new(custom_location);
        let custom_exists = custom_path.exists() && custom_path.is_dir();
        info!("Checking custom blockchain location: {:?}, exists: {}", custom_path, custom_exists);
        
        if custom_exists {
            return Ok(true);
        }
        
        // Custom location doesn't exist, check default location
        info!("Custom location not found, checking default location");
        let default_exists = default_blockchain_data_dir.exists() && default_blockchain_data_dir.is_dir();
        info!("Checking default blockchain location: {:?}, exists: {}", default_blockchain_data_dir, default_exists);
        
        if default_exists {
            // Database found at default location, update config to point there
            info!("Database found at default location, updating config");
            let mut new_config = config.clone();
            new_config.app_settings.local_blockchain_file_location = Some(default_blockchain_data_dir.to_string_lossy().to_string());
            
            if let Err(e) = config_manager.update_config(new_config).await {
                error!("Failed to update configuration to default location: {}", e);
                // Still return true since database exists, even if config update failed
                return Ok(true);
            }
            
            info!("Configuration updated to point to default blockchain location");
            return Ok(true);
        }
        
        // Database not found in either location, update config to default for future use
        info!("Database not found in custom or default location, updating config to default location");
        let mut new_config = config.clone();
        new_config.app_settings.local_blockchain_file_location = Some(default_blockchain_data_dir.to_string_lossy().to_string());
        
        if let Err(e) = config_manager.update_config(new_config).await {
            error!("Failed to update configuration to default location: {}", e);
        } else {
            info!("Configuration updated to default blockchain location for future use");
        }
        
        return Ok(false);
    }
    
    // No custom location configured, check default location
    let default_exists = default_blockchain_data_dir.exists() && default_blockchain_data_dir.is_dir();
    info!("Checking default blockchain location: {:?}, exists: {}", default_blockchain_data_dir, default_exists);
    
    if !default_exists {
        // Database not found at default location, update config to explicitly point to default
        info!("Database not found at default location, updating config to default location");
        let mut new_config = config.clone();
        new_config.app_settings.local_blockchain_file_location = Some(default_blockchain_data_dir.to_string_lossy().to_string());
        
        if let Err(e) = config_manager.update_config(new_config).await {
            error!("Failed to update configuration to default location: {}", e);
        } else {
            info!("Configuration updated to default blockchain location for future use");
        }
    }
    
    Ok(default_exists)
}

/// Get the blockchain database path (custom or default)
#[command]
pub async fn get_blockchain_database_path(
    config_manager: State<'_, Arc<ConfigManager>>,
) -> CommandResult<String> {
    info!("Command: get_blockchain_database_path");
    
    let config = config_manager.get_config();
    
    // Return custom location if configured
    if let Some(custom_location) = &config.app_settings.local_blockchain_file_location {
        return Ok(custom_location.clone());
    }
    
    // Return default location
    let blockchain_data_dir = match dirs::data_dir() {
        Some(dir) => dir.join("com.b-rad-coin.app").join("blockchain"),
        None => {
            return Err("Failed to determine blockchain data directory".to_string());
        }
    };
    
    Ok(blockchain_data_dir.to_string_lossy().to_string())
}

/// Get the default blockchain database path (ignoring any custom configuration)
#[command]
pub async fn get_default_blockchain_database_path() -> CommandResult<String> {
    info!("Command: get_default_blockchain_database_path");
    
    // Always return the default system location, ignoring config
    let blockchain_data_dir = match dirs::data_dir() {
        Some(dir) => dir.join("com.b-rad-coin.app").join("blockchain"),
        None => {
            return Err("Failed to determine default blockchain data directory".to_string());
        }
    };
    
    Ok(blockchain_data_dir.to_string_lossy().to_string())
}

/// Open folder picker dialog
#[command]
pub async fn open_folder_picker(title: String) -> CommandResult<Option<String>> {
    info!("Command: open_folder_picker with title: {}", title);
    
    use rfd::AsyncFileDialog;
    
    let folder = AsyncFileDialog::new()
        .set_title(&title)
        .pick_folder()
        .await;
    
    match folder {
        Some(path) => {
            let path_str = path.path().to_string_lossy().to_string();
            info!("User selected folder: {}", path_str);
            Ok(Some(path_str))
        }
        None => {
            info!("User cancelled folder selection");
            Ok(None)
        }
    }
}

/// Create blockchain database at specified location and update config
#[command]
pub async fn create_blockchain_database_at_location(
    location: String,
    config_manager: State<'_, Arc<ConfigManager>>,
    app_handle: tauri::AppHandle,
) -> CommandResult<bool> {
    info!("Command: create_blockchain_database_at_location at: {}", location);
    
    // First, stop all existing blockchain services to release database locks
    info!("Stopping existing blockchain services before creating new database");
    if let Err(e) = stop_blockchain_services_internal(&app_handle).await {
        error!("Failed to stop blockchain services: {}", e);
        return Err(format!("Failed to stop existing services: {}", e));
    }
    
    // Wait longer for network resources to be fully released
    info!("Waiting for network resources to be released...");
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    let blockchain_path = std::path::Path::new(&location).join("blockchain.db");
    
    // Create the blockchain database
    match crate::blockchain_database::AsyncBlockchainDatabase::new(blockchain_path.clone()).await {
        Ok(_) => {
            info!("Blockchain database created successfully at: {:?}", blockchain_path);
            
            // Update configuration with the new location
            let mut config = config_manager.get_config().clone();
            config.app_settings.local_blockchain_file_location = Some(location);
            
            if let Err(e) = config_manager.update_config(config).await {
                error!("Failed to update configuration: {}", e);
                return Err(format!("Created database but failed to update config: {}", e));
            }
            
            info!("Configuration updated with new blockchain location");
            Ok(true)
        }
        Err(e) => {
            error!("Failed to create blockchain database: {}", e);
            Err(format!("Failed to create blockchain database: {}", e))
        }
    }
}

/// Set existing blockchain database location and update config
#[command]
pub async fn set_blockchain_database_location(
    location: String,
    config_manager: State<'_, Arc<ConfigManager>>,
    app_handle: tauri::AppHandle,
) -> CommandResult<bool> {
    info!("Command: set_blockchain_database_location to: {}", location);
    
    let blockchain_path = std::path::Path::new(&location);
    
    // Verify the location exists and contains a valid blockchain database
    if !blockchain_path.exists() || !blockchain_path.is_dir() {
        return Err("Selected location does not exist or is not a directory".to_string());
    }
    
    // Check if it looks like a blockchain database directory
    // Look for any files that suggest this is a Sled database directory
    let mut has_db_files = false;
    if let Ok(entries) = std::fs::read_dir(blockchain_path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            // Look for common Sled database files/directories
            if file_name_str == "conf" || 
               file_name_str == "db" || 
               file_name_str.starts_with("snap") ||
               entry.path().is_dir() && (
                   file_name_str == "blocks" || 
                   file_name_str == "transactions" || 
                   file_name_str == "utxos" ||
                   file_name_str == "addresses" ||
                   file_name_str == "metadata"
               ) {
                has_db_files = true;
                break;
            }
        }
    }
    
    if !has_db_files {
        return Err("Selected location does not appear to contain a valid blockchain database".to_string());
    }
    
    // First, stop all existing blockchain services to release database locks
    info!("Stopping existing blockchain services before switching database location");
    if let Err(e) = stop_blockchain_services_internal(&app_handle).await {
        error!("Failed to stop blockchain services: {}", e);
        return Err(format!("Failed to stop existing services: {}", e));
    }
    
    // Wait longer for all resources to be fully released, including file locks
    info!("Waiting for all resources and file locks to be released...");
    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
    
    // Force garbage collection to help release any remaining references
    std::hint::black_box(());
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Update configuration with the location
    let mut config = config_manager.get_config().clone();
    config.app_settings.local_blockchain_file_location = Some(location);
    
    if let Err(e) = config_manager.update_config(config).await {
        error!("Failed to update configuration: {}", e);
        return Err(format!("Failed to update configuration: {}", e));
    }
    
    info!("Configuration updated with blockchain location");
    Ok(true)
}

/// Start blockchain services after database setup is complete
#[command]
pub async fn start_blockchain_services(
    app_handle: tauri::AppHandle,
) -> CommandResult<bool> {
    info!("Command: start_blockchain_services");
    
    // First, ensure any existing services are properly stopped
    // This is important when switching database locations
    if let Err(e) = stop_blockchain_services_internal(&app_handle).await {
        warn!("Failed to stop existing services (this might be normal): {}", e);
    }
    
    // Wait a moment for resources to be fully released
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Initialize blockchain database with the configured location
    let config_manager = app_handle.state::<Arc<ConfigManager>>();
    
    // Force reload configuration to ensure we have the latest blockchain location
    (**config_manager).reload_config().await.map_err(|e| {
        error!("Failed to reload configuration: {}", e);
        format!("Failed to reload configuration: {}", e)
    })?;
    
    let config = config_manager.get_config();
    
    let blockchain_data_dir = if let Some(custom_location) = &config.app_settings.local_blockchain_file_location {
        std::path::PathBuf::from(custom_location)
    } else {
        match dirs::data_dir() {
            Some(dir) => dir.join("com.b-rad-coin.app").join("blockchain"),
            None => {
                return Err("Failed to determine blockchain data directory".to_string());
            }
        }
    };
    
    // Initialize blockchain database
    let blockchain_db = match crate::blockchain_database::AsyncBlockchainDatabase::new(blockchain_data_dir).await {
        Ok(db) => Arc::new(db),
        Err(e) => {
            error!("Failed to initialize blockchain database: {}", e);
            return Err(format!("Failed to initialize blockchain database: {}", e));
        }
    };
    
    // Store blockchain database in app state
    app_handle.manage(blockchain_db.clone());
    
    // Initialize and store blockchain sync service
    let blockchain_sync = crate::blockchain_sync::AsyncBlockchainSyncService::new(blockchain_db.clone());
    app_handle.manage(blockchain_sync);
    
    // Initialize and store wallet sync service
    let wallet_sync = crate::wallet_sync_service::AsyncWalletSyncService::new(blockchain_db.clone());
    app_handle.manage(wallet_sync);
    
    // Initialize and store mining service
    let mining_service = crate::mining_service::AsyncMiningService::new(blockchain_db.clone());
    app_handle.manage(mining_service);
    
    // Initialize and store network service
    let network_service = crate::network_service::AsyncNetworkService::new(blockchain_db.clone(), None);
    app_handle.manage(network_service);
    
    // Start network service with retry logic for port binding
    let network_service = app_handle.state::<crate::network_service::AsyncNetworkService>();
    let mut retries = 3;
    let mut last_error_msg = None;
    
    while retries > 0 {
        match network_service.start().await {
            Ok(()) => {
                info!("Network service started successfully");
                break;
            }
            Err(e) => {
                let error_str = e.to_string();
                last_error_msg = Some(error_str.clone());
                if error_str.contains("Only one usage of each socket address") || error_str.contains("10048") {
                    retries -= 1;
                    if retries > 0 {
                        warn!("Port 8333 still in use, waiting and retrying... ({} attempts left)", retries);
                        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                    }
                } else {
                    // For other errors, don't retry
                    error!("Failed to start network service: {}", e);
                    return Err(format!("Failed to start network service: {}", e));
                }
            }
        }
    }
    
    if retries == 0 {
        if let Some(err_msg) = last_error_msg {
            error!("Failed to start network service after retries: {}", err_msg);
            return Err(format!("Failed to start network service after retries: {}. The network port (8333) may still be in use by another process or a previous instance. Please wait a few moments and try again.", err_msg));
        }
    }
    
    // Start blockchain sync service
    let blockchain_sync = app_handle.state::<crate::blockchain_sync::AsyncBlockchainSyncService>();
    if let Err(e) = blockchain_sync.initialize(app_handle.clone()).await {
        error!("Failed to initialize blockchain sync service: {}", e);
        return Err(format!("Failed to initialize blockchain sync service: {}", e));
    }
    
    if let Err(e) = blockchain_sync.start_sync().await {
        error!("Failed to start blockchain sync: {}", e);
        return Err(format!("Failed to start blockchain sync: {}", e));
    }
    
    info!("All blockchain services started successfully");
    
    // Notify frontend that blockchain services are ready
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.emit("blockchain-services-ready", ());
    }
    
    Ok(true)
}

/// Internal function to stop blockchain services (used by other functions)
async fn stop_blockchain_services_internal(app_handle: &tauri::AppHandle) -> Result<(), String> {
    info!("Stopping blockchain services internally");
    
    // Stop network service if it exists
    if let Some(network_service) = app_handle.try_state::<crate::network_service::AsyncNetworkService>() {
        info!("Stopping network service");
        if let Err(e) = network_service.stop().await {
            error!("Failed to stop network service: {}", e);
        } else {
            info!("Network service stopped successfully");
        }
    }
    
    // Wait for network service to fully stop and release the port
    info!("Waiting for network service to fully release resources...");
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Stop blockchain sync service if it exists
    if let Some(blockchain_sync) = app_handle.try_state::<crate::blockchain_sync::AsyncBlockchainSyncService>() {
        info!("Stopping blockchain sync service");
        // Note: Add explicit stop method to blockchain sync service if available
    }
    
    // Stop wallet sync service if it exists
    if let Some(wallet_sync) = app_handle.try_state::<crate::wallet_sync_service::AsyncWalletSyncService>() {
        info!("Stopping wallet sync service");
        // Note: Add explicit stop method to wallet sync service if available
    }
    
    // Close blockchain database if it exists
    if let Some(blockchain_db) = app_handle.try_state::<Arc<crate::blockchain_database::AsyncBlockchainDatabase>>() {
        info!("Closing blockchain database");
        if let Err(e) = blockchain_db.close().await {
            error!("Failed to close blockchain database: {}", e);
            return Err(format!("Failed to close blockchain database: {}", e));
        } else {
            info!("Blockchain database closed successfully");
        }
        
        // Wait additional time for database locks to be fully released
        info!("Waiting for database locks to be fully released...");
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    }
    
    info!("Blockchain services stopped successfully");
    Ok(())
}

/// Stop blockchain services to allow database operations
#[command]
pub async fn stop_blockchain_services(
    app_handle: tauri::AppHandle,
) -> CommandResult<bool> {
    info!("Command: stop_blockchain_services");
    
    match stop_blockchain_services_internal(&app_handle).await {
        Ok(()) => Ok(true),
        Err(e) => Err(e),
    }
}

/// Check if blockchain services are ready
#[command]
pub async fn is_blockchain_ready(
    app_handle: tauri::AppHandle,
) -> CommandResult<bool> {
    info!("Command: is_blockchain_ready");
    
    // Check if blockchain database service exists and is initialized
    let blockchain_db_exists = app_handle.try_state::<Arc<crate::blockchain_database::AsyncBlockchainDatabase>>().is_some();
    
    // Check if blockchain sync service exists and is initialized
    let blockchain_sync_exists = app_handle.try_state::<crate::blockchain_sync::AsyncBlockchainSyncService>().is_some();
    
    // Check if network service exists and is running
    let network_service_exists = app_handle.try_state::<crate::network_service::AsyncNetworkService>().is_some();
    
    let is_ready = blockchain_db_exists && blockchain_sync_exists && network_service_exists;
    
    info!("Blockchain ready check: db={}, sync={}, network={}, ready={}", 
          blockchain_db_exists, blockchain_sync_exists, network_service_exists, is_ready);
    
    Ok(is_ready)
}

/// Get the estimated size of blockchain database files
#[command]
pub async fn get_blockchain_database_size(
    config_manager: State<'_, Arc<ConfigManager>>,
) -> CommandResult<u64> {
    info!("Command: get_blockchain_database_size");
    
    let config = config_manager.get_config();
    
    // Get current blockchain location
    let current_location = if let Some(custom_location) = &config.app_settings.local_blockchain_file_location {
        std::path::PathBuf::from(custom_location)
    } else {
        match dirs::data_dir() {
            Some(dir) => dir.join("com.b-rad-coin.app").join("blockchain"),
            None => {
                return Err("Failed to determine blockchain data directory".to_string());
            }
        }
    };
    
    if !current_location.exists() {
        return Ok(0); // No database exists yet
    }
    
    // Calculate total size of blockchain database files
    let total_size = calculate_directory_size(&current_location, &[
        "blocks",        // Sled tree for blocks
        "transactions",  // Sled tree for transactions
        "utxos",        // Sled tree for UTXOs
        "addresses",    // Sled tree for addresses
        "metadata",     // Sled tree for metadata
        "conf",         // Sled configuration file
        "db",           // Sled database file
        "snap",         // Sled snapshot files
    ]);
    
    info!("Blockchain database size: {} bytes", total_size);
    Ok(total_size)
}

/// Helper function to calculate the size of database files in a directory
fn calculate_directory_size(dir: &std::path::Path, allowed_files: &[&str]) -> u64 {
    use std::fs;
    
    let mut total_size = 0u64;
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            // Check if this is a blockchain database file we should count
            let should_count = if path.is_dir() {
                // For directories, check if the name matches our allowed list
                allowed_files.iter().any(|&allowed| {
                    file_name_str == allowed || file_name_str.starts_with(&format!("{}_", allowed))
                })
            } else {
                // For files, be permissive with Sled database files
                allowed_files.iter().any(|&allowed| {
                    file_name_str == allowed || 
                    file_name_str.starts_with(&format!("{}_", allowed)) ||
                    file_name_str.starts_with(allowed) ||
                    file_name_str.ends_with(".sled") ||
                    file_name_str.ends_with(".db") ||
                    file_name_str == "conf" ||
                    file_name_str.starts_with("snap")
                })
            };
            
            if !should_count {
                continue;
            }
            
            if path.is_dir() {
                // Recursively calculate directory size
                total_size += calculate_directory_size(&path, allowed_files);
            } else if let Ok(metadata) = fs::metadata(&path) {
                total_size += metadata.len();
            }
        }
    }
    
    total_size
}

/// Structure to represent a wallet address for mining selection
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletAddress {
    pub wallet_name: String,
    pub address: String,
    pub label: Option<String>,
    pub derivation_path: String,
}

/// Command to get all addresses from all available wallets
#[command]
pub async fn get_all_wallet_addresses(
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<Vec<WalletAddress>> {
    debug!("Command: get_all_wallet_addresses");
    
    let mut manager = wallet_manager.get_manager().await;
    let mut all_addresses = Vec::new();
    
    // Get all available wallets
    let wallet_list = manager.list_wallets();
    let wallet_count = wallet_list.len();
    
    for wallet_info in &wallet_list {
        // Use addresses from wallet info (stored in config)
        for (index, address) in wallet_info.addresses.iter().enumerate() {
            all_addresses.push(WalletAddress {
                wallet_name: wallet_info.name.clone(),
                address: address.clone(),
                label: None, // Labels are not stored in WalletInfo, set to None
                derivation_path: format!("m/44'/0'/0'/0/{}", index), // Generate standard BIP44 path
            });
        }
    }
    
    info!("Found {} addresses across {} wallets", all_addresses.len(), wallet_count);
    Ok(all_addresses)
}

/// Command to get the current mining configuration including status and reward address
#[command]
pub async fn get_mining_configuration(
    wallet_manager: State<'_, AsyncWalletManager>,
    mining_service: State<'_, AsyncMiningService>,
) -> CommandResult<Option<MiningConfiguration>> {
    debug!("Command: get_mining_configuration");
    
    let manager = wallet_manager.get_manager().await;
    
    // Check if there's a current wallet
    if let Some(current_wallet) = manager.get_current_wallet() {
        let wallet_id = current_wallet.name.clone();
        
        // Get mining status
        let mining_status = mining_service.get_mining_status(&wallet_id).await;
        
        match mining_status {
            Some(status) => {
                Ok(Some(MiningConfiguration {
                    wallet_id: status.wallet_id,
                    is_mining: status.is_mining,
                    mining_address: status.mining_address,
                    hash_rate: status.hash_rate,
                    blocks_mined: status.blocks_mined,
                    current_difficulty: status.current_difficulty,
                }))
            }
            None => {
                // No mining status found, but wallet is open
                // Return default configuration with first address if available
                if let Some(first_address) = current_wallet.data.addresses.first() {
                    Ok(Some(MiningConfiguration {
                        wallet_id: wallet_id.clone(),
                        is_mining: false,
                        mining_address: first_address.address.clone(),
                        hash_rate: 0.0,
                        blocks_mined: 0,
                        current_difficulty: 0,
                    }))
                } else {
                    Ok(None) // No addresses in wallet
                }
            }
        }
    } else {
        Ok(None) // No wallet is currently open
    }
}

/// Mining configuration structure for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct MiningConfiguration {
    pub wallet_id: String,
    pub is_mining: bool,
    pub mining_address: String,
    pub hash_rate: f64,
    pub blocks_mined: u32,
    pub current_difficulty: u64,
}

/// Command to update the label of an address in the current wallet
#[command]
pub async fn update_address_label(
    address: String,
    label: Option<String>,
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<bool> {
    info!("Command: update_address_label for address: {}", address);

    let mut manager = wallet_manager.get_manager().await;

    // First get the wallet name and secured status
    let (_wallet_name, is_secured, wallet_path) = {
        let current_wallet = match manager.get_current_wallet() {
            Some(wallet) => wallet,
            None => {
                error!("No wallet is currently open");
                return Err("No wallet is currently open".to_string());
            }
        };

        let wallet_name = current_wallet.name.clone();
        let wallet_path = current_wallet.path.clone();
        
        let is_secured = if let Some(wallet_info) = manager.find_wallet_by_name(&wallet_name) {
            wallet_info.secured
        } else {
            false
        };

        (wallet_name, is_secured, wallet_path)
    };

    // Now get mutable access to update the wallet
    let current_wallet = match manager.get_current_wallet_mut() {
        Some(wallet) => wallet,
        None => {
            error!("No wallet is currently open");
            return Err("No wallet is currently open".to_string());
        }
    };

    // Update the label in the addresses list
    let mut address_found = false;
    for addr_info in &mut current_wallet.data.addresses {
        if addr_info.address == address {
            addr_info.label = label.clone();
            address_found = true;
            break;
        }
    }

    if !address_found {
        error!("Address not found in current wallet: {}", address);
        return Err(format!("Address '{}' not found in current wallet", address));
    }

    // Update the modified timestamp
    current_wallet.data.modified_at = chrono::Utc::now().timestamp();

    // Get the wallet data file path
    let wallet_data_path = wallet_path.join("wallet.dat");

    // Save the wallet data to disk
    // Note: Since this is an open wallet, if it's secured, it would have been unlocked already
    match current_wallet.data.save(&wallet_data_path, if is_secured { Some("") } else { None }) {
        Ok(_) => {
            info!("Successfully updated label for address: {}", address);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to save wallet data: {}", e);
            Err(format!("Failed to save wallet data: {}", e))
        }
    }
}

/// Command to derive a new address for the current wallet
#[command]
pub async fn derive_new_address(
    label: Option<String>,
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<String> {
    info!("Command: derive_new_address with label: {:?}", label);

    let mut manager = wallet_manager.get_manager().await;

    // First get the wallet name and secured status
    let (_wallet_name, is_secured, wallet_path, master_private_key, next_index) = {
        let current_wallet = match manager.get_current_wallet() {
            Some(wallet) => wallet,
            None => {
                error!("No wallet is currently open");
                return Err("No wallet is currently open".to_string());
            }
        };

        let wallet_name = current_wallet.name.clone();
        let wallet_path = current_wallet.path.clone();
        
        let master_private_key = match &current_wallet.data.master_private_key {
            Some(key) => key.clone(),
            None => {
                error!("No master private key available for key derivation");
                return Err("Master private key not available for key derivation".to_string());
            }
        };

        let next_index = current_wallet.data.addresses.len() as u32;
        
        let is_secured = if let Some(wallet_info) = manager.find_wallet_by_name(&wallet_name) {
            wallet_info.secured
        } else {
            false
        };

        (wallet_name, is_secured, wallet_path, master_private_key, next_index)
    };

    // Determine the derivation path
    let derivation_path = format!("m/44'/0'/0'/0/{}", next_index);

    info!("Deriving new address at path: {}", derivation_path);

    // Parse the master private key
    use bitcoin::bip32::{Xpriv, DerivationPath};
    use bitcoin::secp256k1::Secp256k1;
    use std::str::FromStr;

    let secp = Secp256k1::new();
    
    let master_xpriv = Xpriv::from_str(&master_private_key)
        .map_err(|e| format!("Failed to parse master private key: {}", e))?;

    let derivation_path_parsed = DerivationPath::from_str(&derivation_path)
        .map_err(|e| format!("Failed to parse derivation path: {}", e))?;

    // Derive the new key pair
    let derived_xpriv = master_xpriv.derive_priv(&secp, &derivation_path_parsed)
        .map_err(|e| format!("Failed to derive private key: {}", e))?;

    let private_key = derived_xpriv.private_key;
    let public_key = private_key.public_key(&secp);

    // Create address (using legacy P2PKH for now)
    use bitcoin::{Address, Network, PublicKey};
    let bitcoin_public_key = PublicKey::new(public_key);
    let address = Address::p2pkh(&bitcoin_public_key, Network::Bitcoin);
    let address_string = address.to_string();

    // Create the new key pair
    let key_pair = crate::wallet_data::KeyPair {
        private_key: bitcoin::PrivateKey::new(private_key, Network::Bitcoin).to_wif(),
        public_key: bitcoin_public_key.to_string(),
        address: address_string.clone(),
        key_type: crate::wallet_data::KeyType::Legacy,
        derivation_path: derivation_path.clone(),
    };

    // Create the new address info
    let address_info = crate::wallet_data::AddressInfo {
        address: address_string.clone(),
        key_type: crate::wallet_data::KeyType::Legacy,
        derivation_path: derivation_path.clone(),
        label: label.clone(),
    };

    // Now get mutable access to update the wallet
    let current_wallet = match manager.get_current_wallet_mut() {
        Some(wallet) => wallet,
        None => {
            error!("No wallet is currently open");
            return Err("No wallet is currently open".to_string());
        }
    };

    // Add to wallet data
    current_wallet.data.keys.insert(address_string.clone(), key_pair);
    current_wallet.data.addresses.push(address_info);

    // Update the modified timestamp
    current_wallet.data.modified_at = chrono::Utc::now().timestamp();

    // Get the wallet data file path
    let wallet_data_path = wallet_path.join("wallet.dat");

    // Save the wallet data to disk
    // Note: Since this is an open wallet, if it's secured, it would have been unlocked already
    match current_wallet.data.save(&wallet_data_path, if is_secured { Some("") } else { None }) {
        Ok(_) => {
            info!("Successfully derived new address: {}", address_string);
            Ok(address_string)
        }
        Err(e) => {
            error!("Failed to save wallet data: {}", e);
            Err(format!("Failed to save wallet data: {}", e))
        }
    }
}
