// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::path::PathBuf;
use std::error::Error;
use serde::{Deserialize, Serialize};
use tauri::{State, Manager};  // Only keep needed imports
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex; // Use Tokio's async-friendly Mutex
use anyhow::Result;

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
async fn get_config_path() -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let exe_dir = std::env::current_exe()?
        .parent()
        .ok_or_else(|| "Failed to get executable directory".to_string())?
        .to_path_buf();
    
    let config_dir = exe_dir.join("config");
    fs::create_dir_all(&config_dir).await?;
    Ok(config_dir.join("wallet_config.json"))
}

// Helper function to load the configuration
async fn load_config() -> Result<(WalletConfig, PathBuf), Box<dyn Error + Send + Sync>> {
    let config_path = get_config_path().await?;
    
    // Check if the config file exists
    if fs::try_exists(&config_path).await? {
        // Read and parse the config file
        let mut file = fs::File::open(&config_path).await?;
        let mut config_content = String::new();
        file.read_to_string(&mut config_content).await?;
        
        let mut config: WalletConfig = serde_json::from_str(&config_content)?;
        
        // IMPORTANT: Always ensure no wallet is open at startup
        // This enforces that the app always starts with no wallet open
        config.current_wallet = None;
        
        // Save the updated config with no wallet open
        let config_json = serde_json::to_string_pretty(&config)?;
        let mut file = fs::File::create(&config_path).await?;
        file.write_all(config_json.as_bytes()).await?;
        
        Ok((config, config_path))
    } else {
        // Create a default config if it doesn't exist
        let default_config = WalletConfig::default();
        let config_json = serde_json::to_string_pretty(&default_config)?;
        let mut file = fs::File::create(&config_path).await?;
        file.write_all(config_json.as_bytes()).await?;
        Ok((default_config, config_path))
    }
}

// Helper function to save the configuration
async fn save_config(config: &WalletConfig, path: &PathBuf) -> Result<(), Box<dyn Error + Send + Sync>> {
    let config_json = serde_json::to_string_pretty(config)?;
    let mut file = fs::File::create(path).await?;
    file.write_all(config_json.as_bytes()).await?;
    Ok(())
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_config_directory() -> Result<String, String> {
    get_config_path()
        .await
        .map(|path| path.parent().unwrap_or(&path).to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_wallet_status(state: State<'_, AppState>) -> Result<bool, String> {
    let config = state.config.lock().await;
    // Only return true if we have both a current wallet set AND it exists in the wallets list
    let result = match &config.current_wallet {
        Some(wallet_name) => config.wallets.iter().any(|w| w.name == *wallet_name),
        None => false
    };
    Ok(result)
}

#[tauri::command]
async fn close_wallet(state: State<'_, AppState>) -> Result<bool, String> {
    // Use tokio mutex which is compatible with async
    let config_path;
    let config_clone;
    {
        let mut config = state.config.lock().await;
        config.current_wallet = None;
        config_clone = config.clone();
        config_path = state.config_path.lock().await.clone();
    }
    
    // Save the updated config after releasing the lock
    save_config(&config_clone, &config_path)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

#[tauri::command]
async fn get_available_wallets(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let config = state.config.lock().await;
    let wallets = config.wallets.iter().map(|w| w.name.clone()).collect();
    Ok(wallets)
}

#[tauri::command]
async fn open_wallet(wallet_name: String, state: State<'_, AppState>) -> Result<bool, String> {
    // Minimize the time we hold the lock
    let (exists, config_path);
    {
        let config = state.config.lock().await;
        exists = config.wallets.iter().any(|w| w.name == wallet_name);
        config_path = state.config_path.lock().await.clone();
    }
    
    // Check if wallet exists
    if !exists {
        return Err(format!("Wallet '{}' not found", wallet_name));
    }
    
    let config_clone;
    // Set the current wallet
    {
        let mut config = state.config.lock().await;
        config.current_wallet = Some(wallet_name.clone());
        config_clone = config.clone();
    }
    
    // Save the updated config
    save_config(&config_clone, &config_path)
        .await
        .map_err(|e| e.to_string())?;
    
    println!("Opening wallet: {}", wallet_name);
    Ok(true)
}

#[tauri::command]
async fn create_wallet(wallet_name: String, _password: String, state: State<'_, AppState>) -> Result<bool, String> {
    // Minimize the time we hold the lock
    let (exists, config_path);
    {
        let config = state.config.lock().await;
        exists = config.wallets.iter().any(|w| w.name == wallet_name);
        config_path = state.config_path.lock().await.clone();
    }
    
    // Check if a wallet with the same name already exists
    if exists {
        return Err(format!("Wallet '{}' already exists", wallet_name));
    }
    
    let wallet_name_clone = wallet_name.clone();
    let config_clone;
    
    // Add the wallet to the config and set as current
    {
        let mut config = state.config.lock().await;
        config.wallets.push(WalletInfo {
            name: wallet_name,
        });
        config.current_wallet = Some(wallet_name_clone.clone());
        config_clone = config.clone();
    }
    
    // Save the updated config
    save_config(&config_clone, &config_path)
        .await
        .map_err(|e| e.to_string())?;
    
    println!("Created and opened wallet: {}", wallet_name_clone);
    Ok(true)
}

#[tauri::command]
async fn get_current_wallet_name(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let config = state.config.lock().await;
    Ok(config.current_wallet.clone())
}

#[tauri::command]
async fn update_app_settings(
    theme: Option<String>, 
    auto_backup: Option<bool>, 
    notifications_enabled: Option<bool>,
    state: State<'_, AppState>
) -> Result<bool, String> {
    // Minimize the time we hold the lock
    let config_path;
    let config_clone;
    {
        let mut config = state.config.lock().await;
        
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
        
        config_path = state.config_path.lock().await.clone();
        config_clone = config.clone();
    }
    
    // Save the updated config
    save_config(&config_clone, &config_path)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

#[tauri::command]
async fn get_app_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let config = state.config.lock().await;
    Ok(config.app_settings.clone())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    
    // Load or create the config file in the runtime
    let (config, config_path) = rt.block_on(async {
        load_config().await.expect("Failed to load config")
    });
    
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
        // Handle app exit with a proper event handler
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                println!("Application closing, resetting wallet state");
                
                // Get the app state and run the close_wallet command with the runtime
                if let Some(state) = window.app_handle().try_state::<AppState>() {
                    // Create a new runtime for this async operation
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
                    let _ = rt.block_on(async {
                        close_wallet(state).await
                    });
                }
                
                // Allow the window to close normally
                api.prevent_close();
                window.app_handle().exit(0);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
