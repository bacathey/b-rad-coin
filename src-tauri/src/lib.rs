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
pub mod blockchain_sync_simple;
pub mod blockchain_database;
pub mod wallet_sync_service;
pub mod mining_service;

use commands::*;
use developer_commands::*;
use config::ConfigManager;
use errors::AppResult;
use security::{AsyncSecurityManager, SecurityManager};
use wallet_manager::{AsyncWalletManager, WalletManager};
use blockchain_sync_simple::AsyncBlockchainSyncService;
use blockchain_database::AsyncBlockchainDatabase;
use wallet_sync_service::AsyncWalletSyncService;
use mining_service::AsyncMiningService;

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
            Ok(app_state) => {                // Build and run Tauri application with our components
                let app = tauri::Builder::default()
                    .plugin(tauri_plugin_updater::Builder::new().build())
                    .plugin(tauri_plugin_opener::init())                    .manage(app_state.wallet_manager)
                    .manage(app_state.security_manager)
                    .manage(app_state.config_manager)
                    .manage(app_state.blockchain_sync)
                    .manage(app_state.blockchain_db)
                    .manage(app_state.wallet_sync)
                    .manage(app_state.mining_service).invoke_handler(generate_handler![
                        check_wallet_status,
                        close_wallet,
                        get_available_wallets,
                        get_wallet_details,
                        is_current_wallet_secured,
                        open_wallet,
                        create_wallet,
                        generate_seed_phrase,
                        get_current_wallet_path,
                        get_fully_qualified_wallet_path,                        open_folder_in_explorer,
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
                        update_tray_network_status,                        get_app_version,
                        greet,                        // Blockchain commands
                        get_network_status,                        get_block_height,
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
                        get_wallet_private_key                    ]).setup(|app| {
                        info!("Setting up application");
                        
                        // Create system tray
                        setup_system_tray(app)?;
                        
                        Ok(())
                    }).on_window_event(|window, event| {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            debug!("Window close requested - minimizing to tray");
                            
                            // Hide the window instead of closing the app
                            let _ = window.hide();
                            
                            // Prevent the default close behavior
                            api.prevent_close();
                        }
                    })                    .build(generate_context!())
                    .expect("Error while building tauri application");                // Start blockchain sync service after app is built
                info!("Starting blockchain sync service");
                let app_handle_for_sync = app.app_handle().clone();
                
                // Start the blockchain sync service
                tokio::spawn(async move {
                    let app_handle_clone = app_handle_for_sync.clone();
                    let blockchain_sync = app_handle_for_sync.state::<AsyncBlockchainSyncService>();
                    
                    if let Err(e) = blockchain_sync.start_sync(app_handle_clone.clone()).await {
                        error!("Failed to start blockchain sync service: {}", e);
                        return;
                    }
                    
                    // Start periodic event emission
                    blockchain_sync.start_event_emission(app_handle_clone).await;
                });

                // Initialize wallet sync and mining services
                info!("Initializing wallet sync and mining services");
                let app_handle_for_services = app.app_handle().clone();
                  tokio::spawn(async move {
                    let app_handle_clone = app_handle_for_services.clone();
                    
                    // Get references to all services
                    let wallet_sync = app_handle_for_services.state::<AsyncWalletSyncService>();
                    let wallet_manager = app_handle_for_services.state::<AsyncWalletManager>();
                    let config_manager = app_handle_for_services.state::<Arc<ConfigManager>>();
                    
                    // Initialize wallet sync service
                    if let Err(e) = wallet_sync.initialize(app_handle_clone.clone()).await {
                        error!("Failed to initialize wallet sync service: {}", e);
                        return;
                    }
                    
                    // Set wallet manager and config manager for wallet sync service
                    wallet_sync.set_wallet_manager(wallet_manager.inner().clone()).await;
                    wallet_sync.set_config_manager(config_manager.inner().clone()).await;
                    
                    // Initialize mining service
                    let mining_service = app_handle_for_services.state::<AsyncMiningService>();
                    if let Err(e) = mining_service.initialize(app_handle_clone).await {
                        error!("Failed to initialize mining service: {}", e);
                        return;
                    }
                    
                    info!("Wallet sync and mining services initialized successfully");
                });

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
    blockchain_sync: AsyncBlockchainSyncService,
    blockchain_db: Arc<AsyncBlockchainDatabase>,
    wallet_sync: AsyncWalletSyncService,
    mining_service: AsyncMiningService,
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
        .await;    // Initialize and start blockchain sync service
    debug!("Initializing blockchain sync service");
    let blockchain_sync = AsyncBlockchainSyncService::new();    // Initialize blockchain database
    debug!("Initializing blockchain database");
    let blockchain_data_dir = match dirs::data_dir() {
        Some(dir) => dir.join("com.b-rad-coin.app").join("blockchain"),
        None => return Err(errors::AppError::Generic("Failed to determine blockchain data directory".to_string())),
    };
    
    info!("Blockchain data directory: {:?}", blockchain_data_dir);

    let blockchain_db = Arc::new(AsyncBlockchainDatabase::new(blockchain_data_dir).await
        .map_err(|e| errors::AppError::Generic(format!("Failed to initialize blockchain database: {}", e)))?);
    
    info!("Blockchain database initialized successfully");
    debug!("Initializing wallet sync service");
    let wallet_sync = AsyncWalletSyncService::new(blockchain_db.clone());
    
    // Initialize mining service
    debug!("Initializing mining service");
    let mining_service = AsyncMiningService::new(blockchain_db.clone());
    
    // Note: blockchain sync will be started in setup() after app handle is available

    info!("Application components initialized successfully");    // Return the application state with all components
    Ok(AppState {
        config_manager,
        wallet_manager: async_wallet_manager,
        security_manager: async_security_manager,
        blockchain_sync,
        blockchain_db,
        wallet_sync,
        mining_service,
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