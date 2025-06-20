//! Blockchain synchronization service

use crate::core::*;
use crate::errors::*;
use log::{debug, error, info, warn};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

/// Blockchain synchronization service
pub struct BlockchainSyncService {
    server: Option<Server>,
    current_height: Arc<AtomicI32>,
    is_syncing: Arc<AtomicBool>,
    is_connected: Arc<AtomicBool>,
    peer_count: Arc<AtomicI32>,
    app_handle: Option<AppHandle>,
}

/// Network status information
#[derive(serde::Serialize, Clone, Debug)]
pub struct NetworkStatus {
    pub is_connected: bool,
    pub current_height: i32,
    pub is_syncing: bool,
    pub peer_count: i32,
}

impl BlockchainSyncService {
    /// Create a new blockchain sync service
    pub fn new() -> Self {
        Self {
            server: None,
            current_height: Arc::new(AtomicI32::new(-1)),
            is_syncing: Arc::new(AtomicBool::new(false)),
            is_connected: Arc::new(AtomicBool::new(false)),
            peer_count: Arc::new(AtomicI32::new(0)),
            app_handle: None,
        }
    }

    /// Initialize the blockchain sync service
    pub async fn initialize(&mut self, app_handle: AppHandle) -> AppResult<()> {
        info!("Initializing blockchain sync service");
        
        self.app_handle = Some(app_handle.clone());

        // Try to create or load existing blockchain
        let blockchain = match Blockchain::new() {
            Ok(bc) => {
                info!("Loaded existing blockchain");
                bc
            },
            Err(_) => {
                info!("No existing blockchain found, creating new one");
                // Create a temporary wallet address for the genesis block
                let genesis_address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string();
                Blockchain::create_blockchain(genesis_address)?
            }
        };

        // Get the current height
        let height = blockchain.get_best_height().unwrap_or(-1);
        self.current_height.store(height, Ordering::Relaxed);
        
        info!("Current blockchain height: {}", height);

        // Create UTXO set
        let utxo_set = UTXOSet { blockchain };

        // Create server for network communication
        let server = Server::new("3001", "", utxo_set)?;
        self.server = Some(server);

        // Start the sync process
        self.start_sync_process().await?;

        Ok(())
    }

    /// Start the blockchain synchronization process
    async fn start_sync_process(&self) -> AppResult<()> {
        info!("Starting blockchain synchronization process");

        let server = self.server.as_ref().unwrap().clone();
        let current_height = Arc::clone(&self.current_height);
        let is_syncing = Arc::clone(&self.is_syncing);
        let is_connected = Arc::clone(&self.is_connected);
        let peer_count = Arc::clone(&self.peer_count);
        let app_handle = self.app_handle.clone().unwrap();

        // Start the server in a separate thread
        let server_clone = server.clone();
        thread::spawn(move || {
            if let Err(e) = server_clone.start_server() {
                error!("Failed to start blockchain server: {}", e);
            }
        });

        // Start the sync monitoring task
        let sync_monitor = tokio::spawn(async move {
            let mut sync_interval = tokio::time::interval(Duration::from_secs(30));
            let mut status_update_interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                tokio::select! {
                    _ = sync_interval.tick() => {
                        // Perform sync check
                        Self::check_sync_status(&server, &current_height, &is_syncing, &is_connected, &peer_count).await;
                    }
                    _ = status_update_interval.tick() => {
                        // Emit status update to frontend
                        Self::emit_network_status(&app_handle, &current_height, &is_syncing, &is_connected, &peer_count).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// Check synchronization status
    async fn check_sync_status(
        server: &Server,
        current_height: &Arc<AtomicI32>,
        is_syncing: &Arc<AtomicBool>,
        is_connected: &Arc<AtomicBool>,
        peer_count: &Arc<AtomicI32>,
    ) {
        debug!("Checking blockchain sync status");

        // Update current height
        if let Ok(height) = server.get_best_height() {
            let old_height = current_height.load(Ordering::Relaxed);
            if height != old_height {
                current_height.store(height, Ordering::Relaxed);
                info!("Blockchain height updated: {} -> {}", old_height, height);
            }
        }

        // TODO: Implement actual peer discovery and connection checking
        // For now, simulate network status
        let known_nodes = server.get_known_nodes();
        let peer_count_val = known_nodes.len() as i32;
        peer_count.store(peer_count_val, Ordering::Relaxed);
        
        // Consider connected if we have peers
        let connected = peer_count_val > 0;
        is_connected.store(connected, Ordering::Relaxed);

        // Set syncing status (true if we're actively requesting blocks)
        // This is a simplified implementation
        is_syncing.store(connected && current_height.load(Ordering::Relaxed) < 100, Ordering::Relaxed);
    }

    /// Emit network status to frontend
    async fn emit_network_status(
        app_handle: &AppHandle,
        current_height: &Arc<AtomicI32>,
        is_syncing: &Arc<AtomicBool>,
        is_connected: &Arc<AtomicBool>,
        peer_count: &Arc<AtomicI32>,
    ) {
        let status = NetworkStatus {
            current_height: current_height.load(Ordering::Relaxed),
            is_syncing: is_syncing.load(Ordering::Relaxed),
            is_connected: is_connected.load(Ordering::Relaxed),
            peer_count: peer_count.load(Ordering::Relaxed),
        };

        if let Err(e) = app_handle.emit("blockchain-status", &status) {
            debug!("Failed to emit blockchain status: {}", e);
        }
    }

    /// Get current network status
    pub fn get_network_status(&self) -> NetworkStatus {
        NetworkStatus {
            current_height: self.current_height.load(Ordering::Relaxed),
            is_syncing: self.is_syncing.load(Ordering::Relaxed),
            is_connected: self.is_connected.load(Ordering::Relaxed),
            peer_count: self.peer_count.load(Ordering::Relaxed),
        }
    }

    /// Get current block height
    pub fn get_block_height(&self) -> i32 {
        self.current_height.load(Ordering::Relaxed)
    }

    /// Check if the blockchain is currently syncing
    pub fn is_syncing(&self) -> bool {
        self.is_syncing.load(Ordering::Relaxed)
    }

    /// Check if connected to network
    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::Relaxed)
    }    /// Get peer count
    pub fn get_peer_count(&self) -> i32 {
        self.peer_count.load(Ordering::Relaxed)
    }

    /// Initialize and start the blockchain sync process
    pub async fn initialize(&mut self, app_handle: AppHandle) -> AppResult<()> {
        self.app_handle = Some(app_handle);
        Ok(())
    }

    /// Start the sync process
    pub async fn start_sync_process(&mut self) -> AppResult<()> {
        if let Some(app_handle) = &self.app_handle {
            info!("Starting blockchain synchronization process");
            
            let current_height = self.current_height.clone();
            let is_syncing = self.is_syncing.clone();
            let is_connected = self.is_connected.clone();
            let peer_count = self.peer_count.clone();
            let app_handle_clone = app_handle.clone();
            
            // Start the sync background task
            tokio::spawn(async move {
                Self::run_sync_loop(
                    app_handle_clone,
                    current_height,
                    is_syncing,
                    is_connected,
                    peer_count,
                ).await;
            });
        }
        
        Ok(())
    }
}

/// Thread-safe wrapper for BlockchainSyncService
pub struct AsyncBlockchainSyncService {
    inner: Arc<RwLock<BlockchainSyncService>>,
}

impl AsyncBlockchainSyncService {
    /// Create new async blockchain sync service
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BlockchainSyncService::new())),
        }
    }

    /// Initialize the service
    pub async fn initialize(&self, app_handle: AppHandle) -> AppResult<()> {
        let mut service = self.inner.write().await;
        service.initialize(app_handle).await
    }

    /// Get network status
    pub async fn get_network_status(&self) -> NetworkStatus {
        let service = self.inner.read().await;
        service.get_network_status()
    }

    /// Get block height
    pub async fn get_block_height(&self) -> i32 {
        let service = self.inner.read().await;
        service.get_block_height()
    }

    /// Check if syncing
    pub async fn is_syncing(&self) -> bool {
        let service = self.inner.read().await;
        service.is_syncing()
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        let service = self.inner.read().await;
        service.is_connected()
    }    /// Get peer count
    pub async fn get_peer_count(&self) -> i32 {
        let service = self.inner.read().await;
        service.get_peer_count()
    }

    /// Start blockchain synchronization
    pub async fn start_sync(&self) -> AppResult<()> {
        let mut service = self.inner.write().await;
        service.start_sync_process().await
    }

    /// Start event emission to frontend
    pub async fn start_event_emission(&self, app_handle: AppHandle) {
        let service = self.inner.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                let status = {
                    let service_guard = service.read().await;
                    service_guard.get_network_status()
                };
                
                if let Err(e) = app_handle.emit("blockchain-status", &status) {
                    error!("Failed to emit blockchain status: {}", e);
                }
            }
        });
    }
}
