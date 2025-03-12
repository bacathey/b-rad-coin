// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::Mutex;
use std::fs;
use std::path::PathBuf;
use std::error::Error;
use serde::{Deserialize, Serialize};
use tauri::State;

// Re-export all the types and commands from main.rs
// This ensures mobile and desktop versions have the same functionality

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
            wallets: vec![
                WalletInfo { name: "Main Wallet".to_string() },
                WalletInfo { name: "Trading Wallet".to_string() },
                WalletInfo { name: "Cold Storage".to_string() },
            ],
            current_wallet: None,  // Explicitly set to None to ensure wallet starts closed
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
        let config: WalletConfig = serde_json::from_str(&config_content)?;
        Ok((config, config_path))
    } else {
        // Create a default config if it doesn't exist
        let default_config = WalletConfig::default();
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

#[tauri::command]
fn check_wallet_status(state: State<AppState>) -> bool {
    let config = state.config.lock().unwrap();
    // Only return true if we have both a current wallet set AND it exists in the wallets list
    match &config.current_wallet {
        Some(wallet_name) => config.wallets.iter().any(|w| w.name == *wallet_name),
        None => false
    }
}

#[tauri::command]
fn close_wallet(state: State<AppState>) -> Result<bool, String> {
    let mut config = state.config.lock().unwrap();
    config.current_wallet = None;
    // Save the updated config
    let config_path = state.config_path.lock().unwrap();
    save_config(&config, &config_path).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
fn get_available_wallets(state: State<AppState>) -> Vec<String> {
    let config = state.config.lock().unwrap();
    config.wallets.iter().map(|w| w.name.clone()).collect()
}

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

#[tauri::command]
fn create_wallet(wallet_name: String, _password: String, state: State<AppState>) -> Result<bool, String> {
    let mut config = state.config.lock().unwrap();
    
    // Check if a wallet with the same name already exists
    if config.wallets.iter().any(|w| w.name == wallet_name) {
        return Err(format!("Wallet '{}' already exists", wallet_name));
    }
    
    let wallet_name_clone = wallet_name.clone();
    
    // Add the wallet to the config
    config.wallets.push(WalletInfo {
        name: wallet_name,
    });
    
    // Set it as the current wallet
    config.current_wallet = Some(wallet_name_clone.clone());
    
    // Save the updated config
    let config_path = state.config_path.lock().unwrap();
    save_config(&config, &config_path).map_err(|e| e.to_string())?;
    
    println!("Created and opened wallet: {}", wallet_name_clone);
    Ok(true)
}

#[tauri::command]
fn get_current_wallet_name(state: State<AppState>) -> Option<String> {
    let config = state.config.lock().unwrap();
    config.current_wallet.clone()
}

#[tauri::command]
fn update_app_settings(
    theme: Option<String>, 
    auto_backup: Option<bool>, 
    notifications_enabled: Option<bool>,
    state: State<AppState>
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

#[tauri::command]
fn get_app_settings(state: State<AppState>) -> AppSettings {
    let config = state.config.lock().unwrap();
    config.app_settings.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load or create the config file
    let (config, config_path) = load_config().expect("Failed to load config");
    
    // Create the app state with the loaded config
    let app_state = AppState {
        config: Mutex::new(config),
        config_path: Mutex::new(config_path),
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
