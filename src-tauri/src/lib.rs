// B-Rad Coin Application
use log::{debug, error, info, LevelFilter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{generate_context, generate_handler, Manager};
use tauri_plugin_updater::UpdaterExt;

// Add static flag to track shutdown state
static SHUTDOWN_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

// Import modules
pub mod commands;
pub mod config;
pub mod errors;
pub mod logging;
pub mod security;
pub mod wallet_manager;

use commands::*;
use config::ConfigManager;
use errors::AppResult;
use security::{AsyncSecurityManager, SecurityManager};
use wallet_manager::{AsyncWalletManager, WalletManager};

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
                    .plugin(tauri_plugin_updater::Builder::new().build())
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
                        recover_wallet,
                        get_current_wallet_name,
                        update_app_settings,
                        get_app_settings,
                        secure_wallet,
                        shutdown_application
                    ])
                    .setup(|app| {
                        info!("Setting up application");
                        let handle = app.handle().clone();
                        tauri::async_runtime::spawn(async move {
                            update(handle).await.unwrap();
                        });
                        Ok(())
                    })
                    .on_window_event(|window, event| {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            debug!("Window close requested");

                            // Only proceed if shutdown is not already in progress
                            if !SHUTDOWN_IN_PROGRESS.load(Ordering::SeqCst) {
                                // Get the app handle to initiate shutdown
                                let app_handle = window.app_handle().clone();

                                // Use our shutdown command directly instead of duplicating code
                                tokio::spawn(async move {
                                    // This will handle all the shutdown steps properly
                                    match commands::shutdown_application(app_handle).await {
                                        Ok(_) => debug!("Shutdown initiated successfully from window close event"),
                                        Err(e) => error!("Failed to initiate shutdown: {}", e),
                                    }
                                });

                                // Prevent immediate close, our shutdown sequence will handle it
                                api.prevent_close();
                            } else {
                                debug!("Shutdown already in progress, allowing close to proceed");
                                // Do not prevent close if shutdown is already in progress
                            }
                        }
                    })
                    .build(generate_context!())
                    .expect("Error while building tauri application");

                // Run the application
                app.run(|app_handle, event| {
                    match event {
                        tauri::RunEvent::Exit => {
                            info!("Application exited cleanly");
                        },
                        tauri::RunEvent::ExitRequested { api, .. } => {
                            debug!("Application exit requested via system event");

                            // Only proceed if shutdown is not already in progress
                            if !SHUTDOWN_IN_PROGRESS.load(Ordering::SeqCst) {
                                // Prevent immediate exit
                                api.prevent_exit();

                                // Get a clone of the app handle to initiate shutdown
                                let handle = app_handle.clone();
                                
                                // Initiate proper shutdown sequence
                                tokio::spawn(async move {
                                    match commands::shutdown_application(handle).await {
                                        Ok(_) => debug!("Shutdown initiated successfully from exit request"),
                                        Err(e) => error!("Failed to initiate shutdown: {}", e),
                                    }
                                });
                            } else {
                                debug!("Shutdown already in progress, allowing exit to proceed");
                                // Allow the exit to proceed if shutdown is already in progress
                            }
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
    // Use platform-specific directories in a way compatible with Tauri 2.0
    let log_dir = match dirs::data_dir() {
        Some(dir) => dir.join("com.b-rad-coin.app").join("logs"),
        None => return Err("Failed to determine log directory".to_string()),
    };

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

    // Connect the wallet manager to the config manager for persistence
    debug!("Connecting wallet manager to config manager for persistence");
    async_wallet_manager
        .set_config_manager(config_manager.clone())
        .await;

    info!("Application components initialized successfully");

    // Return the application state with all components
    Ok(AppState {
        config_manager,
        wallet_manager: async_wallet_manager,
        security_manager: async_security_manager,
    })
}


async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
      let mut downloaded = 0;
  
      // alternatively we could also call update.download() and update.install() separately
      update
        .download_and_install(
          |chunk_length, content_length| {
            downloaded += chunk_length;
            println!("downloaded {downloaded} from {content_length:?}");
          },
          || {
            println!("download finished");
          },
        )
        .await?;
  
      println!("update installed");
      app.restart();
    }
  
    Ok(())
  }