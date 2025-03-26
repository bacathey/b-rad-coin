// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{State, Manager};  // Added Manager trait for state management

// Configuration structure to store wallet data
#[derive(Serialize, Deserialize, Clone, Debug)]
struct WalletConfig {
    wallets: Vec<WalletInfo>,
    current_wallet: Option<String>,
    app_settings: AppSettings,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct WalletInfo {
    name: String,
    // In a real app, you would store other wallet info here:
    // - path to wallet file
    // - creation date
    // - last accessed date
    // - user preferences for this wallet
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppSettings {
    theme: String,
    auto_backup: bool,
    notifications_enabled: bool,
}

// Default implementation for WalletConfig
impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            wallets: Vec::new(), // Empty wallets - will be loaded from wallet_config.json
            current_wallet: None,
            app_settings: AppSettings {
                theme: "system".to_string(),
                auto_backup: true,
                notifications_enabled: true,
            },
        }
    }
}

// App state to store the configuration
struct AppState {
    config: Mutex<WalletConfig>,
    config_path: Mutex<PathBuf>,
}

// Helper function to get the config file path
fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
    let exe_dir = std::env::current_exe()?
        .parent()
        .ok_or_else(|| "Failed to get executable directory".to_string())?
        .to_path_buf();
    
    let config_dir = exe_dir.join("config");
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("wallet_config.json"))
}

// Helper function to load the configuration
fn load_config() -> Result<(WalletConfig, PathBuf), Box<dyn Error>> {
    let config_path = get_config_path()?;

    // Check if the config file exists
    if config_path.exists() {
        // Read and parse the config file
        let config_content = fs::read_to_string(&config_path)?;
        let mut config: WalletConfig = serde_json::from_str(&config_content)?;
        
        // IMPORTANT: Always ensure no wallet is open at startup
        // This enforces that the app always starts with no wallet open
        config.current_wallet = None;
        
        // Save the updated config with no wallet open
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, config_json)?;
        
        Ok((config, config_path))
    } else {
        // Create a default config if it doesn't exist
        let default_config = WalletConfig {
            wallets: vec![
                WalletInfo { name: "Main Wallet".to_string() },
                WalletInfo { name: "Trading Wallet".to_string() },
                WalletInfo { name: "Cold Storage".to_string() },
            ],
            current_wallet: None,
            app_settings: AppSettings {
                theme: "system".to_string(),
                auto_backup: true,
                notifications_enabled: true,
            },
        };
        
        // Save the config
        let config_json = serde_json::to_string_pretty(&default_config)?;
        fs::write(&config_path, config_json)?;
        Ok((default_config, config_path))
    }
}

// Helper function to save the configuration
fn save_config(config: &WalletConfig, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let config_json = serde_json::to_string_pretty(config)?;
    fs::write(path, config_json)?;
    Ok(())
}

// Command to check if the wallet is open
#[tauri::command]
fn check_wallet_status(state: State<AppState>) -> bool {
    let config = state.config.lock().unwrap();
    config.current_wallet.is_some()
}

// Command to close the wallet
#[tauri::command]
fn close_wallet(state: State<AppState>) -> Result<bool, String> {
    let mut config = state.config.lock().unwrap();

    // Clear the current wallet
    config.current_wallet = None;

    // Save the updated config
    let config_path = state.config_path.lock().unwrap();
    save_config(&config, &config_path).map_err(|e| e.to_string())?;

    Ok(true)
}

// Command to get available wallets
#[tauri::command]
fn get_available_wallets(state: State<AppState>) -> Vec<String> {
    let config = state.config.lock().unwrap();
    config.wallets.iter().map(|w| w.name.clone()).collect()
}

// Command to open a wallet
#[tauri::command]
fn open_wallet(wallet_name: String, state: State<AppState>) -> Result<bool, String> {
    let mut config = state.config.lock().unwrap();

    // Check if wallet exists
    if !config.wallets.iter().any(|w| w.name == wallet_name) {
        return Err(format!("Wallet '{}' not found", wallet_name));
    }

    // Set the current wallet
    config.current_wallet = Some(wallet_name.clone());

    // Save the updated config
    let config_path = state.config_path.lock().unwrap();
    save_config(&config, &config_path).map_err(|e| e.to_string())?;

    println!("Opening wallet: {}", wallet_name);
    Ok(true)
}

// Command to create a new wallet
#[tauri::command]
fn create_wallet(
    wallet_name: String,
    _password: String,
    state: State<AppState>,
) -> Result<bool, String> {
    let mut config = state.config.lock().unwrap();

    // Check if a wallet with the same name already exists
    if config.wallets.iter().any(|w| w.name == wallet_name) {
        return Err(format!("Wallet '{}' already exists", wallet_name));
    }

    // In a real app, you would:
    // 1. Create an actual wallet file with encryption
    // 2. Store the password securely (hashed and salted)
    // 3. Save metadata about the wallet creation

    let wallet_name_clone = wallet_name.clone();

    // For now, just add the wallet to the config
    config.wallets.push(WalletInfo { name: wallet_name });

    // Set it as the current wallet
    config.current_wallet = Some(wallet_name_clone.clone());

    // Save the updated config
    let config_path = state.config_path.lock().unwrap();
    save_config(&config, &config_path).map_err(|e| e.to_string())?;

    println!("Created and opened wallet: {}", wallet_name_clone);
    Ok(true)
}

// Command to get the name of the currently opened wallet
#[tauri::command]
fn get_current_wallet_name(state: State<AppState>) -> Option<String> {
    let config = state.config.lock().unwrap();
    config.current_wallet.clone()
}

// Command to update app settings
#[tauri::command]
fn update_app_settings(
    theme: Option<String>,
    auto_backup: Option<bool>,
    notifications_enabled: Option<bool>,
    state: State<AppState>,
) -> Result<bool, String> {
    let mut config = state.config.lock().unwrap();

    // Update only provided settings
    if let Some(theme_value) = theme {
        config.app_settings.theme = theme_value;
    }

    if let Some(auto_backup_value) = auto_backup {
        config.app_settings.auto_backup = auto_backup_value;
    }

    if let Some(notifications_value) = notifications_enabled {
        config.app_settings.notifications_enabled = notifications_value;
    }

    // Save the updated config
    let config_path = state.config_path.lock().unwrap();
    save_config(&config, &config_path).map_err(|e| e.to_string())?;

    Ok(true)
}

// Command to get app settings
#[tauri::command]
fn get_app_settings(state: State<AppState>) -> AppSettings {
    let config = state.config.lock().unwrap();
    config.app_settings.clone()
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load or create the config file
    let (config, config_path) = load_config()?;

    // Create the app state with the loaded config
    let app_state = AppState {
        config: Mutex::new(config),
        config_path: Mutex::new(config_path),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            println!("Attempting to launch second instance");
            
            // Focus the main window when a second instance tries to launch
            let window = app.get_webview_window("main");
            
            if let Some(window) = window {
                // Focus the window
                let _ = window.set_focus();
                
                // Restore if minimized
                if let Ok(false) = window.is_visible() {
                    let _ = window.show();
                }
                
                // Unminimize if needed
                if let Ok(true) = window.is_minimized() {
                    let _ = window.unminimize();
                }
            }
        }))
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            get_config_directory,
            check_wallet_status,
            close_wallet,
            get_available_wallets,
            open_wallet,
            create_wallet,
            get_current_wallet_name,
            update_app_settings,
            get_app_settings
        ])
        // Handle app exit with proper event handler
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                println!("Application closing, resetting wallet state");
                
                // Get the app state and call close_wallet directly
                if let Some(state) = window.app_handle().try_state::<AppState>() {
                    let _ = close_wallet(state);
                }
                
                // Allow the window to close normally
                api.prevent_close();
                window.app_handle().exit(0);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

// Existing greet command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_config_directory() -> Result<String, String> {
    get_config_path()
        .map(|path| path.parent().unwrap_or(&path).to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}
