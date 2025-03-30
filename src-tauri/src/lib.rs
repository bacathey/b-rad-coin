// B-Rad Coin Application
use log::{LevelFilter, info, error, debug};
use std::sync::Arc;
use tauri::{Manager, generate_context, generate_handler, Emitter};

// Import modules
pub mod errors;
pub mod logging;
pub mod config;
pub mod wallet_manager;
pub mod security;
pub mod commands;

use errors::AppResult;
use wallet_manager::{WalletManager, AsyncWalletManager};
use config::ConfigManager;
use security::{SecurityManager, AsyncSecurityManager};
use commands::*;

/// Application version
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Authentication timeout in seconds
const AUTH_TIMEOUT_SECONDS: u64 = 1800; // 30 minutes

/// Application entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    
    // Application initialization inside the runtime
    rt.block_on(async {
        // Setup logging first
        setup_logging().expect("Failed to set up logging");
        
        // Log application startup
        logging::log_app_startup(APP_VERSION);
        
        // Initialize components
        match initialize_app().await {
            Ok(app_state) => {
                // Build and run Tauri application with our components
                let app = tauri::Builder::default()
                    .plugin(tauri_plugin_opener::init())
                    .manage(app_state.wallet_manager)
                    .manage(app_state.security_manager)
                    .manage(app_state.config_manager)
                    .invoke_handler(generate_handler![
                        check_wallet_status,
                        close_wallet,
                        get_available_wallets,
                        get_wallet_details,
                        is_current_wallet_secured,
                        open_wallet,
                        create_wallet,
                        get_current_wallet_name,
                        update_app_settings,
                        get_app_settings
                    ])
                    .setup(|_app| {
                        info!("Setting up application");
                        Ok(())
                    })
                    .on_window_event(|window, event| {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            debug!("Window close requested");
                            

                            // Prevent the window from closing immediately
                            api.prevent_close();
                            

                            // Clone what we need from the window
                            let window_label = window.label().to_string();
                            let app_handle = window.app_handle().clone();
                            

                            // Spawn a new async task to handle the shutdown sequence
                            tokio::spawn(async move {
                                info!("Starting application shutdown sequence");
                                

                                // Get the wallet manager and close any open wallet
                                if let Some(wallet_manager) = app_handle.try_state::<AsyncWalletManager>() {
                                    match wallet_manager.shutdown().await {
                                        Ok(_) => info!("Wallet manager shutdown completed successfully"),
                                        Err(e) => error!("Wallet manager shutdown error: {}", e),
                                    }
                                }
                                

                                // Log application shutdown
                                logging::log_app_shutdown();
                                

                                // Get the main window again and emit the event
                                if let Some(main_window) = app_handle.get_webview_window(&window_label) {
                                    // Send shutdown complete event
                                    let _ = main_window.emit("app-shutdown-complete", ());
                                }
                                

                                // Exit the application
                                std::thread::sleep(std::time::Duration::from_millis(500));
                                app_handle.exit(0);
                            });
                        }
                    })
                    .build(generate_context!())
                    .expect("Error while building tauri application");
                
                // Run the application
                app.run(|_app_handle, event| {
                    match event {
                        tauri::RunEvent::Exit => {
                            info!("Application exited cleanly");
                        },
                        tauri::RunEvent::ExitRequested { api, .. } => {
                            debug!("Application exit requested");
                            api.prevent_exit();
                        },
                        _ => {}
                    }
                });
            },
            Err(e) => {
                error!("Application initialization failed: {}", e);
                std::process::exit(1);
            }
        }
    });
}

/// Application state container
struct AppState {
    config_manager: Arc<ConfigManager>,
    wallet_manager: AsyncWalletManager,
    security_manager: AsyncSecurityManager,
}

/// Set up application logging
fn setup_logging() -> Result<(), String> {
    // Get executable directory for logs
    let exe_dir = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => return Err(format!("Failed to get executable path: {}", e)),
    };
    
    let log_dir = exe_dir.parent()
        .map(|p| p.join("logs"))
        .ok_or_else(|| "Failed to determine log directory".to_string())?;
    
    // Initialize logging with file output
    logging::init(Some(log_dir), LevelFilter::Info)
}

/// Initialize application components
async fn initialize_app() -> AppResult<AppState> {
    debug!("Initializing application components");
    
    // Initialize configuration manager
    debug!("Initializing configuration manager");
    let config_manager = Arc::new(ConfigManager::new().await?);
    
    // Initialize security manager
    debug!("Initializing security manager");
    let security_manager = SecurityManager::new(AUTH_TIMEOUT_SECONDS);
    let async_security_manager = AsyncSecurityManager::new(security_manager);
    
    // Initialize wallet manager with config
    debug!("Initializing wallet manager");
    let wallet_manager = WalletManager::new(config_manager.get_config().clone());
    let async_wallet_manager = AsyncWalletManager::new(wallet_manager);
    
    info!("Application components initialized successfully");
    
    // Return the application state with all components
    Ok(AppState {
        config_manager,
        wallet_manager: async_wallet_manager,
        security_manager: async_security_manager,
    })
}
