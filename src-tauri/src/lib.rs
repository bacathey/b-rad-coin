mod bip39_words;

// B-Rad Coin Application
use log::{debug, error, info, LevelFilter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{generate_context, generate_handler, Manager, Emitter};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
//use tauri_plugin_updater::UpdaterExt;

// Add static flag to track shutdown state
static SHUTDOWN_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

// Import modules
pub mod commands;
pub mod config;
pub mod developer_commands;
pub mod errors;
pub mod logging;
pub mod security;
pub mod wallet_data;
pub mod wallet_manager;
// pub mod core;  // Temporarily commented out due to missing dependencies
pub mod blockchain_sync;
pub mod blockchain_database;
pub mod wallet_sync_service;
pub mod mining_service;
pub mod network_service;

use commands::*;
use developer_commands::*;
use config::ConfigManager;
use errors::AppResult;
use security::{AsyncSecurityManager, SecurityManager};
use wallet_manager::{AsyncWalletManager, WalletManager};
use blockchain_sync::AsyncBlockchainSyncService;
use blockchain_database::AsyncBlockchainDatabase;
use wallet_sync_service::AsyncWalletSyncService;
use mining_service::AsyncMiningService;
use network_service::AsyncNetworkService;

/// Application version
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Authentication timeout in seconds
const AUTH_TIMEOUT_SECONDS: u64 = 1800; // 30 minutes

/// Application entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Setup logging first
    setup_logging().expect("Failed to set up logging");    // Log application startup
    logging::log_app_startup(APP_VERSION);

    // Build and run Tauri application
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(generate_handler![
            check_wallet_status,
            close_wallet,
            get_available_wallets,
            get_wallet_details,
            is_current_wallet_secured,
            open_wallet,
            create_wallet,
            generate_seed_phrase,
            get_current_wallet_path,
            get_fully_qualified_wallet_path,
            open_folder_in_explorer,
            open_folder_with_shell_command,
            delete_wallet,
            recover_wallet,
            get_current_wallet_name,
            update_app_settings,
            get_app_settings,
            secure_wallet,
            shutdown_application,
            show_main_window,
            hide_to_tray,
            update_tray_wallet_status,
            update_tray_network_status,
            get_app_version,
            greet,
            // Blockchain commands
            get_network_status,
            get_block_height,
            is_blockchain_syncing,
            is_network_connected,
            get_peer_count,
            // Wallet sync commands
            start_wallet_sync,
            stop_wallet_sync,
            get_wallet_sync_status,
            get_all_wallet_sync_statuses,
            // Mining commands
            start_mining,
            stop_mining,
            get_mining_status,
            get_all_mining_statuses,
            // Developer commands
            get_recent_logs,
            echo_command,
            get_config_directory,
            cleanup_orphaned_wallets,
            delete_all_wallets,
            get_wallet_private_key
        ])        .setup(|app| {
            info!("Setting up application - DELAYED INIT");
            
            // Create system tray
            setup_system_tray(app)?;
            
            // Initialize app components with a small delay to ensure runtime is ready
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(100));
                tauri::async_runtime::spawn(async move {
                    info!("DELAYED: Initializing application components");
                    match initialize_app().await {
                        Ok(app_state) => {
                            info!("DELAYED: Application components initialized successfully");
                            
                            // Add all components to Tauri state
                            app_handle.manage(app_state.wallet_manager);
                            app_handle.manage(app_state.security_manager);
                            app_handle.manage(app_state.config_manager);
                            app_handle.manage(app_state.blockchain_sync);
                            app_handle.manage(app_state.blockchain_db);
                            app_handle.manage(app_state.wallet_sync);
                            app_handle.manage(app_state.mining_service);
                            app_handle.manage(app_state.network_service);
                            
                            info!("DELAYED: Starting blockchain sync service");
                            
                            // Initialize and start blockchain sync in the background
                            let app_handle_for_sync = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                let blockchain_sync = app_handle_for_sync.state::<AsyncBlockchainSyncService>();
                                
                                info!("SYNC: Initializing blockchain sync service with app handle");
                                if let Err(e) = blockchain_sync.initialize(app_handle_for_sync.clone()).await {
                                    error!("SYNC: Failed to initialize blockchain sync service: {}", e);
                                    return;
                                }
                                info!("SYNC: Blockchain sync service initialized successfully");
                                
                                info!("SYNC: Starting blockchain sync process");
                                if let Err(e) = blockchain_sync.start_sync().await {
                                    error!("SYNC: Failed to start blockchain sync: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("DELAYED: Failed to initialize application components: {}", e);
                        }
                    }
                });
            });
            
            Ok(())
        }).on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                debug!("Window close requested - minimizing to tray");
                
                // Hide the window instead of closing the app
                let _ = window.hide();
                // Prevent the default close behavior
                api.prevent_close();
            }
        })
        .build(generate_context!())
        .expect("Error while building tauri application");    // Run the app
    info!("Running application");
    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            debug!("Exit requested");
            api.prevent_exit();
        }
        _ => {}
    });
}

/// Application state container
struct AppState {
    config_manager: Arc<ConfigManager>,
    wallet_manager: AsyncWalletManager,
    security_manager: AsyncSecurityManager,
    blockchain_sync: AsyncBlockchainSyncService,
    blockchain_db: Arc<AsyncBlockchainDatabase>,
    wallet_sync: AsyncWalletSyncService,
    mining_service: AsyncMiningService,
    network_service: AsyncNetworkService,
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
    let async_wallet_manager = AsyncWalletManager::new(wallet_manager);    // Connect the wallet manager to the config manager for persistence
    debug!("Connecting wallet manager to config manager for persistence");
    async_wallet_manager
        .set_config_manager(config_manager.clone())
        .await;    // Initialize blockchain database first
    debug!("Initializing blockchain database");
    let blockchain_data_dir = match dirs::data_dir() {
        Some(dir) => dir.join("com.b-rad-coin.app").join("blockchain"),
        None => return Err(errors::AppError::Generic("Failed to determine blockchain data directory".to_string())),
    };
    
    info!("Blockchain data directory: {:?}", blockchain_data_dir);

    let blockchain_db = Arc::new(AsyncBlockchainDatabase::new(blockchain_data_dir).await
        .map_err(|e| errors::AppError::Generic(format!("Failed to initialize blockchain database: {}", e)))?);
    
    info!("Blockchain database initialized successfully");

    // Initialize and start blockchain sync service (now that we have the database)
    debug!("Initializing blockchain sync service");
    let blockchain_sync = AsyncBlockchainSyncService::new(blockchain_db.clone());
    debug!("Initializing wallet sync service");
    let wallet_sync = AsyncWalletSyncService::new(blockchain_db.clone());
      // Initialize mining service
    debug!("Initializing mining service");
    let mining_service = AsyncMiningService::new(blockchain_db.clone());
    
    // Initialize network service
    debug!("Initializing network service");
    let network_service = AsyncNetworkService::new(blockchain_db.clone(), None); // Use default port
    
    // Note: blockchain sync will be started in setup() after app handle is available

    info!("Application components initialized successfully");

    // Return the application state with all components
    Ok(AppState {
        config_manager,
        wallet_manager: async_wallet_manager,
        security_manager: async_security_manager,
        blockchain_sync,
        blockchain_db,
        wallet_sync,
        mining_service,
        network_service,
    })
}

/// Setup system tray with menu and event handlers
fn setup_system_tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
    
    info!("Setting up system tray");
    
    // Create tray menu items
    let wallet_status_item = MenuItem::with_id(app, "wallet_status", "No wallet open", false, None::<&str>)?;
    let network_status_item = MenuItem::with_id(app, "network_status", "Network: Disconnected", false, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    
    let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    
    let open_wallet_item = MenuItem::with_id(app, "open_wallet", "Open Wallet...", true, None::<&str>)?;
    let create_wallet_item = MenuItem::with_id(app, "create_wallet", "Create Wallet...", true, None::<&str>)?;
    let close_wallet_item = MenuItem::with_id(app, "close_wallet", "Close Wallet", false, None::<&str>)?;
    let separator3 = PredefinedMenuItem::separator(app)?;
    
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[
        &wallet_status_item,
        &network_status_item,
        &separator1,
        &show_item,
        &hide_item,
        &separator2,
        &open_wallet_item,
        &create_wallet_item,
        &close_wallet_item,
        &separator3,
        &quit_item,
    ])?;
    
    // Create tray icon
    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("B-Rad Coin Wallet")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_tray_icon_event(|tray, event| {
            match event {
                TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, button_state: tauri::tray::MouseButtonState::Up, .. } => {
                    debug!("Tray icon left clicked - showing window");
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                TrayIconEvent::DoubleClick { button: tauri::tray::MouseButton::Left, .. } => {
                    debug!("Tray icon double clicked - showing window");
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            }
        })        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "quit" => {
                    info!("Quit selected from tray menu");
                    // Set shutdown flag and exit the application
                    SHUTDOWN_IN_PROGRESS.store(true, Ordering::SeqCst);
                    
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        match commands::shutdown_application(app_handle).await {
                            Ok(_) => debug!("Shutdown completed successfully from tray menu"),
                            Err(e) => error!("Failed to shutdown from tray menu: {}", e),
                        }
                    });
                }
                "show" => {
                    debug!("Show selected from tray menu");
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "hide" => {
                    debug!("Hide selected from tray menu");
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
                "open_wallet" => {
                    debug!("Open wallet selected from tray menu");
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        // Emit event to frontend to open wallet dialog
                        let _ = window.emit("tray-open-wallet", ());
                    }
                }
                "create_wallet" => {
                    debug!("Create wallet selected from tray menu");
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        // Emit event to frontend to open create wallet dialog
                        let _ = window.emit("tray-create-wallet", ());
                    }
                }                "close_wallet" => {
                    debug!("Close wallet selected from tray menu");
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let wallet_manager = app_handle.state::<AsyncWalletManager>();
                        match commands::close_wallet(wallet_manager).await {
                            Ok(_) => {
                                debug!("Wallet closed successfully from tray menu");
                                // Update tray menu to reflect wallet closed
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    let _ = window.emit("wallet-closed", ());
                                }
                            },
                            Err(e) => error!("Failed to close wallet from tray menu: {}", e),
                        }
                    });
                }
                _ => {}
            }
        })
        .build(app)?;
    
    info!("System tray created successfully");
    Ok(())
}


//async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
//    if let Some(update) = app.updater()?.check().await? {
//      let mut downloaded = 0;
//  
//      // alternatively we could also call update.download() and update.install() separately
//      update
//        .download_and_install(
//          |chunk_length, content_length| {
//            downloaded += chunk_length;
//            println!("downloaded {downloaded} from {content_length:?}");
//          },
//          || {
//           println!("download finished");
//         },
//        )
//        .await?;
//  
//      println!("update installed");
//      app.restart();
//    }
//  
//    Ok(())
//  }