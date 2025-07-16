mod bip39_words;

// B-Rad Coin Application
use log::{debug, error, info, warn, LevelFilter};
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
pub mod network_constants;
pub mod dns_seeder;

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
            force_sync,
            is_blockchain_ready,
            // Blockchain setup commands
            check_blockchain_database_exists,
            get_blockchain_database_path,
            get_default_blockchain_database_path,
            get_blockchain_database_size,
            open_folder_picker,
            create_blockchain_database_at_location,
            set_blockchain_database_location,
            start_blockchain_services,
            stop_blockchain_services,
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
            get_wallet_private_key,
            get_current_wallet_info,
            get_cpu_cores,
            // Mining address commands
            get_all_wallet_addresses,
            get_mining_configuration
        ])        .setup(|app| {
            info!("Setting up application");
            
            // Initialize basic app components first to access configuration
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                info!("Initializing basic application components");
                match initialize_basic_app().await {
                    Ok(basic_state) => {
                        info!("Basic application components initialized successfully");
                        
                        // Check if system tray should be enabled from configuration
                        let should_enable_tray = basic_state.config_manager.get_config().app_settings.minimize_to_system_tray;
                        info!("System tray setting: {}", should_enable_tray);
                        
                        // Add basic components to Tauri state
                        app_handle.manage(basic_state.wallet_manager);
                        app_handle.manage(basic_state.security_manager);
                        app_handle.manage(basic_state.config_manager);
                        
                        // Create system tray if enabled in settings
                        if should_enable_tray {
                            info!("Setting up system tray (enabled in settings)");
                            if let Err(e) = setup_system_tray_after_init(&app_handle) {
                                error!("Failed to setup system tray: {}", e);
                            }
                        } else {
                            info!("System tray disabled in settings, skipping initialization");
                        }
                        
                        // Check if blockchain database exists
                        info!("Checking for blockchain database");
                        let config_manager = app_handle.state::<Arc<ConfigManager>>();
                        let blockchain_exists = check_blockchain_exists(&config_manager).await;
                        
                        // Check if developer mode is enabled
                        let config = config_manager.get_config();
                        let is_developer_mode = config.app_settings.developer_mode;
                        info!("Developer mode enabled: {}", is_developer_mode);
                        
                        // Start blockchain services if database exists
                        if blockchain_exists {
                            info!("Blockchain database found, starting all services");
                            // Start blockchain services since database exists
                            match commands::start_blockchain_services(app_handle.clone()).await {
                                Ok(_) => {
                                    info!("All blockchain services started successfully");
                                    // Notify frontend that blockchain services are ready
                                    if let Some(window) = app_handle.get_webview_window("main") {
                                        let _ = window.emit("blockchain-services-ready", ());
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to start blockchain services: {}", e);
                                    // Notify frontend about the error
                                    if let Some(window) = app_handle.get_webview_window("main") {
                                        let _ = window.emit("blockchain-setup-error", e);
                                    }
                                }
                            }
                        } else {
                            info!("Blockchain database not found, waiting for user setup");
                            // Notify frontend that blockchain setup is needed
                            info!("Emitting blockchain-setup-required event to frontend");
                            if let Some(window) = app_handle.get_webview_window("main") {
                                info!("Main window found, emitting blockchain-setup-required event to frontend");
                                match window.emit("blockchain-setup-required", ()) {
                                    Ok(_) => info!("Successfully emitted blockchain-setup-required event"),
                                    Err(e) => error!("Failed to emit blockchain-setup-required event: {}", e),
                                }
                            } else {
                                warn!("Main window not found, cannot emit blockchain-setup-required event");
                                // Try to list all windows for debugging
                                let windows = app_handle.webview_windows();
                                info!("Available windows: {:?}", windows.keys().collect::<Vec<_>>());
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to initialize basic application components: {}", e);
                        // Notify frontend about the error
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.emit("app-initialization-error", e.to_string());
                        }
                    }
                }
            });
            
            Ok(())
        }).on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Check if system tray is enabled before deciding what to do
                let app_handle = window.app_handle();
                if let Some(config_manager) = app_handle.try_state::<Arc<ConfigManager>>() {
                    let config = config_manager.get_config();
                    if config.app_settings.minimize_to_system_tray {
                        debug!("Window close requested - minimizing to tray (tray enabled)");
                        // Hide the window instead of closing the app
                        let _ = window.hide();
                        // Prevent the default close behavior
                        api.prevent_close();
                    } else {
                        debug!("Window close requested - closing application (tray disabled)");
                        // Allow the window to close normally, which will exit the app
                        // No need to call api.prevent_close()
                    }
                } else {
                    // Fallback: if we can't access config, use default behavior (minimize to tray)
                    debug!("Window close requested - minimizing to tray (config not available)");
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .build(generate_context!())
        .expect("Error while building tauri application");    // Run the app
    info!("Running application");
    app.run(|app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            info!("Exit requested, cleaning up resources");
            
            // Set shutdown flag
            SHUTDOWN_IN_PROGRESS.store(true, Ordering::SeqCst);
            
            // Prevent the default exit to allow cleanup
            api.prevent_exit();
            
            // Perform cleanup in a blocking manner to ensure completion
            let app_handle_clone = app_handle.clone();
            let cleanup_task = tauri::async_runtime::spawn(async move {
                info!("Starting shutdown cleanup process");
                
                // Stop all services first
                if let Err(e) = commands::stop_blockchain_services(app_handle_clone.clone()).await {
                    error!("Error stopping blockchain services during shutdown: {}", e);
                }
                
                // Close database if it exists
                if let Some(blockchain_db) = app_handle_clone.try_state::<Arc<AsyncBlockchainDatabase>>() {
                    info!("Closing blockchain database during shutdown");
                    if let Err(e) = flush_and_release_database(blockchain_db.inner().clone()).await {
                        error!("Error closing blockchain database during shutdown: {}", e);
                    }
                }
                
                info!("Shutdown cleanup completed");
            });
            
            // Wait for cleanup to complete with a reasonable timeout
            tauri::async_runtime::spawn(async move {
                let timeout_duration = std::time::Duration::from_secs(5); // 5 second timeout
                
                match tokio::time::timeout(timeout_duration, cleanup_task).await {
                    Ok(Ok(())) => {
                        info!("Cleanup completed successfully, exiting application");
                    }
                    Ok(Err(e)) => {
                        error!("Cleanup task panicked: {:?}", e);
                    }
                    Err(_) => {
                        error!("Cleanup timed out after 5 seconds, forcing exit");
                    }
                }
                
                // Exit the application after cleanup
                std::process::exit(0);
            });
        }
        tauri::RunEvent::Exit => {
            info!("Application exiting");
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

/// Basic application state container (without blockchain services)
struct BasicAppState {
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

/// Initialize basic application components (config, wallet, security - no blockchain services)
async fn initialize_basic_app() -> AppResult<BasicAppState> {
    debug!("Initializing basic application components");

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

    info!("Basic application components initialized successfully");

    // Return the basic application state
    Ok(BasicAppState {
        config_manager,
        wallet_manager: async_wallet_manager,
        security_manager: async_security_manager,
    })
}

/// Check if blockchain database exists at configured or default location
async fn check_blockchain_exists(config_manager: &Arc<ConfigManager>) -> bool {
    let config = config_manager.get_config();
    
    // Check if there's a custom location configured
    if let Some(custom_location) = &config.app_settings.local_blockchain_file_location {
        let custom_path = std::path::Path::new(custom_location);
        let exists = custom_path.exists() && custom_path.is_dir();
        info!("Checking custom blockchain location: {:?}, exists: {}", custom_path, exists);
        return exists;
    }
    
    // Check default location
    let blockchain_data_dir = match dirs::data_dir() {
        Some(dir) => dir.join("com.b-rad-coin.app").join("blockchain"),
        None => {
            error!("Failed to determine blockchain data directory");
            return false;
        }
    };
    
    let exists = blockchain_data_dir.exists() && blockchain_data_dir.is_dir();
    
    info!("Checking default blockchain location: {:?}, exists: {}", blockchain_data_dir, exists);
    exists
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

/// Setup system tray after application initialization (when we have access to AppHandle)
fn setup_system_tray_after_init(app_handle: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
    
    info!("Setting up system tray after initialization");
    
    // Create tray menu items
    let wallet_status_item = MenuItem::with_id(app_handle, "wallet_status", "No wallet open", false, None::<&str>)?;
    let network_status_item = MenuItem::with_id(app_handle, "network_status", "Network: Disconnected", false, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app_handle)?;
    
    let show_item = MenuItem::with_id(app_handle, "show", "Show Window", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app_handle, "hide", "Hide Window", true, None::<&str>)?;
    let separator2 = PredefinedMenuItem::separator(app_handle)?;
    
    let open_wallet_item = MenuItem::with_id(app_handle, "open_wallet", "Open Wallet...", true, None::<&str>)?;
    let create_wallet_item = MenuItem::with_id(app_handle, "create_wallet", "Create Wallet...", true, None::<&str>)?;
    let close_wallet_item = MenuItem::with_id(app_handle, "close_wallet", "Close Wallet", false, None::<&str>)?;
    let separator3 = PredefinedMenuItem::separator(app_handle)?;
    
    let quit_item = MenuItem::with_id(app_handle, "quit", "Quit", true, None::<&str>)?;
    
    let menu = Menu::with_items(app_handle, &[
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
    
    // Get the default icon (we need to handle this differently for AppHandle)
    let icon = app_handle.default_window_icon().cloned().unwrap_or_else(|| {
        // Fallback: create a simple default icon
        let rgba_data: &[u8] = &[0; 32 * 32 * 4]; // 32x32 RGBA pixels (all transparent)
        tauri::image::Image::new(rgba_data, 32, 32)
    });
    
    // Create tray icon
    let _tray = tauri::tray::TrayIconBuilder::with_id("main-tray")
        .tooltip("B-Rad Coin Wallet")
        .icon(icon)
        .menu(&menu)
        .on_tray_icon_event(|tray, event| {
            match event {
                tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, button_state: tauri::tray::MouseButtonState::Up, .. } => {
                    debug!("Tray icon left clicked - showing window");
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                tauri::tray::TrayIconEvent::DoubleClick { button: tauri::tray::MouseButton::Left, .. } => {
                    debug!("Tray icon double clicked - showing window");
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            }
        })
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "quit" => {
                    info!("Quit selected from tray menu");
                    // Set shutdown flag and exit the application
                    SHUTDOWN_IN_PROGRESS.store(true, std::sync::atomic::Ordering::SeqCst);
                    
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
                }
                "close_wallet" => {
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
        .build(app_handle)?;
    
    info!("System tray created successfully after initialization");
    Ok(())
}

/// Setup resource cleanup handler for proper database flushing on shutdown
fn setup_resource_cleanup_handler(app_handle: tauri::AppHandle, blockchain_db: Arc<AsyncBlockchainDatabase>) {
    // Set up a handler that will be called during shutdown
    let cleanup_db = blockchain_db.clone();
    
    // Register cleanup to be called on various exit scenarios
    tauri::async_runtime::spawn(async move {
        // Wait for shutdown signal
        tokio::signal::ctrl_c().await.ok();
        info!("Received shutdown signal, flushing database to disk and releasing resources");
        let _ = flush_and_release_database(cleanup_db).await;
        std::process::exit(0);
    });
    
    // Also set up cleanup for panic scenarios
    let panic_cleanup_db = blockchain_db.clone();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("Application panic detected: {:?}", panic_info);
        
        // Attempt to cleanup blockchain database in a blocking manner
        let rt = tokio::runtime::Runtime::new();
        if let Ok(runtime) = rt {
            runtime.block_on(async {
                let _ = flush_and_release_database(panic_cleanup_db.clone()).await;
            });
        }
        
        // Re-enable default panic behavior
        std::process::abort();
    }));
}

/// Flush blockchain database to disk and release resources
async fn flush_and_release_database(blockchain_db: Arc<AsyncBlockchainDatabase>) -> Result<(), String> {
    info!("Flushing blockchain database to disk and releasing resources...");
    
    // Flush all data to disk and release database resources
    match blockchain_db.close().await {
        Ok(_) => {
            info!("Blockchain database successfully flushed to disk and resources released");
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to flush and release blockchain database resources: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
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