use crate::logging;
use log::{debug, error, info};
use std::sync::Arc;  // Add this import for Arc
use tauri::Emitter;
use tauri::{command, Manager, State};

use crate::config::{AppSettings, ConfigManager, WalletInfo}; // Ensure WalletInfo is imported if not already
use crate::security::AsyncSecurityManager;
use crate::wallet_manager::AsyncWalletManager;
use rand::seq::SliceRandom;
use crate::bip39_words::WORD_LIST;

/// Response type for commands with proper error handling
type CommandResult<T> = Result<T, String>;

/// Convert Application errors to string responses for Tauri
fn format_error<E: std::fmt::Display>(e: E) -> String {
    format!("{}", e)
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
) -> CommandResult<Vec<String>> {
    debug!("Command: get_available_wallets");
    let manager = wallet_manager.get_manager().await;

    // Get wallets and extract names
    let wallets = manager
        .list_wallets()
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
pub async fn get_wallet_details(
    wallet_manager: State<'_, AsyncWalletManager>,
) -> CommandResult<Vec<WalletDetails>> {
    debug!("Command: get_wallet_details");
    let manager = wallet_manager.get_manager().await;

    // Get wallets and convert to WalletDetails
    let wallets = manager
        .list_wallets()
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
        // This should never happen if UI is working correctly
        error!("No seed phrase provided, this shouldn't happen!");
        return Err("No seed phrase provided".to_string());
    };

    let mut manager = wallet_manager.get_manager().await;
    
    // Call the synchronous create_wallet_with_seed function
    // Remove the .await as the underlying function is now sync
    match manager.create_wallet_with_seed(&wallet_name, &effective_password, &actual_seed_phrase, use_password) {
        Ok(_) => {
            info!("Wallet created successfully: {}", wallet_name);
            // Return Ok(true) to match the function signature CommandResult<bool>
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

/// Command to update application settings
#[command]
pub async fn update_app_settings(
    theme: Option<String>,
    auto_backup: Option<bool>,
    notifications_enabled: Option<bool>,
    log_level: Option<String>,
    config_manager_arc: State<'_, Arc<ConfigManager>>, // Change type to State<'_, Arc<ConfigManager>>
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
) -> CommandResult<AppSettings> {
    debug!("Command: get_app_settings");

    // Access the inner ConfigManager through the Arc
    let config = config_manager_arc.inner().get_config();
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
    let mut rng = rand::thread_rng();
    
    // Select 12 unique words randomly from the BIP-39 English list
    let words: Vec<&str> = WORD_LIST
        .choose_multiple(&mut rng, 12)
        .cloned()
        .collect();
        
    // Join the words with spaces
    let phrase = words.join(" ");
    
    debug!("Generated seed phrase (first word: {}, last word: {})", words.first().unwrap_or(&""), words.last().unwrap_or(&"")); // Avoid logging the full phrase
    Ok(phrase)
}
