use crate::logging;
use log::{debug, error, info, warn};
use std::sync::Arc;  // Add this import for Arc
use tauri::Emitter;
use tauri::{command, Manager, State};

use crate::config::{AppSettings, ConfigManager}; // Ensure WalletInfo is imported if not already
use crate::security::AsyncSecurityManager;
use crate::wallet_manager::AsyncWalletManager;
use rand::prelude::IndexedRandom;
use crate::bip39_words::WORD_LIST;

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
        // Check if developer mode is enabled
        let config = config_manager_arc.inner().get_config();
        let dev_mode_enabled = config.app_settings.developer_mode;
        
        if dev_mode_enabled {
            // In developer mode, generate a placeholder seed phrase
            info!("Developer mode enabled. Generating placeholder seed phrase for wallet: {}", wallet_name);
            // Generate a deterministic placeholder seed phrase
            let placeholder = format!("test word wallet {} developer mode enabled skip verify testing phrase", wallet_name);
            placeholder
        } else {
            // Not in developer mode, seed phrase is required
            error!("No seed phrase provided and developer mode is disabled");
            return Err("Seed phrase is required for wallet creation. Developer mode must be enabled to skip seed phrase.".to_string());
        }
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
                // Otherwise, join it with the wallets directory
                wallets_dir.join(path_str)
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
#[command]
pub async fn update_app_settings(
    theme: Option<String>,
    auto_backup: Option<bool>,
    notifications_enabled: Option<bool>,
    log_level: Option<String>,
    developer_mode: Option<bool>,
    skip_seed_phrase_dialogs: Option<bool>,
    config_manager_arc: State<'_, Arc<ConfigManager>>,
) -> CommandResult<bool> {
    info!("Command: update_app_settings");

    // Get the inner ConfigManager from the Arc
    let config_manager = config_manager_arc.inner();

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
    
    if let Some(dev_mode) = developer_mode {
        info!("Updating developer_mode to: {}", dev_mode);
        config.app_settings.developer_mode = dev_mode;
        
        // If developer mode is being turned off, also turn off skip_seed_phrase_dialogs
        if !dev_mode && config.app_settings.skip_seed_phrase_dialogs {
            info!("Developer mode disabled, disabling skip_seed_phrase_dialogs");
            config.app_settings.skip_seed_phrase_dialogs = false;
        }
    }
    
    if let Some(skip_dialogs) = skip_seed_phrase_dialogs {
        // Only allow skip_seed_phrase_dialogs to be enabled if developer_mode is enabled
        if skip_dialogs && !config.app_settings.developer_mode {
            error!("Cannot enable skip_seed_phrase_dialogs when developer_mode is disabled");
            return Err("Developer mode must be enabled to skip seed phrase dialogs".to_string());
        }
        
        info!("Updating skip_seed_phrase_dialogs to: {}", skip_dialogs);
        config.app_settings.skip_seed_phrase_dialogs = skip_dialogs;
    }

    // Save the updated config using the inner ConfigManager
    match config_manager
        .update_app_settings(config.app_settings)
        .await
    {
        Ok(_) => {
            info!("Settings updated successfully");
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
                drop(sec_manager); // Explicitly release security manager lock

                // Now open the wallet with the validated password
                let mut manager = wallet_manager.get_manager().await;
                match manager.open_wallet(&wallet_name, Some(&password)) {
                    Ok(_) => {
                        info!("Successfully opened secured wallet: {}", wallet_name);
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
    } else {
        // For unsecured wallets, just open directly
        let mut manager = wallet_manager.get_manager().await;
        match manager.open_wallet(&wallet_name, None) {
            Ok(_) => {
                info!("Successfully opened unsecured wallet: {}", wallet_name);
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

/// Command to generate a new 12-word BIP-39 seed phrase
#[command]
pub async fn generate_seed_phrase() -> CommandResult<String> {
    debug!("Command: generate_seed_phrase");
    let mut rng = rand::rng();    // Select 12 unique words randomly from the BIP-39 English list
    let words: Vec<&str> = WORD_LIST
        .choose_multiple(&mut rng, 12)
        .cloned()
        .collect();
        
    // Join the words with spaces
    let phrase = words.join(" ");
    
    debug!("Generated seed phrase (first word: {}, last word: {})", words.first().unwrap_or(&""), words.last().unwrap_or(&"")); // Avoid logging the full phrase
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


