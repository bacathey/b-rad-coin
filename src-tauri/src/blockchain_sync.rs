//! Blockchain synchronization service

use crate::blockchain_database::AsyncBlockchainDatabase;
use crate::errors::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::RwLock;

/// Blockchain synchronization service
pub struct BlockchainSyncService {
    blockchain_db: Arc<AsyncBlockchainDatabase>,
    current_height: Arc<AtomicI32>,
    is_syncing: Arc<AtomicBool>,
    is_connected: Arc<AtomicBool>,
    peer_count: Arc<AtomicI32>,
    app_handle: Option<AppHandle>,
}

/// Network status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub current_height: i32,
    pub is_syncing: bool,
    pub is_connected: bool,
    pub peer_count: i32,
}

impl BlockchainSyncService {
    /// Create a new blockchain sync service
    pub fn new(blockchain_db: Arc<AsyncBlockchainDatabase>) -> Self {
        Self {
            blockchain_db,
            current_height: Arc::new(AtomicI32::new(0)),
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

        // Get current blockchain height from database
        match self.blockchain_db.get_block_height().await {
            Ok(height) => {
                let height_i32 = height as i32;
                self.current_height.store(height_i32, Ordering::Relaxed);
                info!("Current blockchain height: {}", height_i32);
            },
            Err(e) => {
                warn!("Failed to get blockchain height: {}", e);
                self.current_height.store(0, Ordering::Relaxed);
            }
        }

        // Start the sync process
        self.start_sync_process().await?;

        Ok(())
    }    /// Start the blockchain synchronization process
    async fn start_sync_process(&self) -> AppResult<()> {
        info!("Starting blockchain synchronization process");

        let blockchain_db = Arc::clone(&self.blockchain_db);
        let current_height = Arc::clone(&self.current_height);
        let is_syncing = Arc::clone(&self.is_syncing);
        let is_connected = Arc::clone(&self.is_connected);
        let peer_count = Arc::clone(&self.peer_count);
        let app_handle = self.app_handle.clone().unwrap();

        // Start the sync monitoring task
        tokio::spawn(async move {
            let mut sync_interval = tokio::time::interval(Duration::from_secs(30));
            let mut status_update_interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                tokio::select! {
                    _ = sync_interval.tick() => {
                        // Perform sync check and request blocks if needed
                        Self::check_sync_status_and_request_blocks(&app_handle, &blockchain_db, &current_height, &is_syncing, &is_connected, &peer_count).await;
                    }
                    _ = status_update_interval.tick() => {
                        // Emit status update to frontend
                        Self::emit_network_status(&app_handle, &current_height, &is_syncing, &is_connected, &peer_count).await;
                    }
                }
            }
        });

        Ok(())
    }    /// Check synchronization status and request blocks if needed (integrated with network service)
    async fn check_sync_status_and_request_blocks(
        app_handle: &AppHandle,
        blockchain_db: &Arc<AsyncBlockchainDatabase>,
        current_height: &Arc<AtomicI32>,
        is_syncing: &Arc<AtomicBool>,
        is_connected: &Arc<AtomicBool>,
        peer_count: &Arc<AtomicI32>,
    ) {
        debug!("Checking blockchain sync status and requesting blocks if needed");

        // Update current height from database
        let local_height = if let Ok(height) = blockchain_db.get_block_height().await {
            let old_height = current_height.load(Ordering::Relaxed);
            let new_height = height as i32;
            if new_height != old_height {
                current_height.store(new_height, Ordering::Relaxed);
                info!("Blockchain height updated: {} -> {}", old_height, new_height);
            }
            new_height as u64
        } else {
            0
        };

        // Get network service stats to check peer status and network height
        let (network_height, connected_peers) = if let Some(network_service) = app_handle.try_state::<crate::network_service::AsyncNetworkService>() {
            let stats = network_service.get_stats().await;
            (stats.network_height, stats.connected_peers)
        } else {
            warn!("Network service not available for sync check");
            (local_height, 0) // Fallback values
        };
        
        peer_count.store(connected_peers as i32, Ordering::Relaxed);
        let connected = connected_peers > 0;
        is_connected.store(connected, Ordering::Relaxed);

        // Check if we need to sync (network height is higher than local height)
        let needs_sync = connected && network_height > local_height;
        
        if needs_sync && !is_syncing.load(Ordering::Relaxed) {
            info!("Starting blockchain sync: local height {} < network height {}", local_height, network_height);
            is_syncing.store(true, Ordering::Relaxed);
            
            // Request blocks using the network service (Bitcoin-style)
            if let Some(network_service) = app_handle.try_state::<crate::network_service::AsyncNetworkService>() {
                if let Err(e) = network_service.sync_blockchain().await {
                    error!("Failed to start blockchain sync: {}", e);
                } else {
                    info!("Blockchain sync request sent to network service");
                }
            }
            
            is_syncing.store(false, Ordering::Relaxed);
        }
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
    }

    /// Get peer count
    pub fn get_peer_count(&self) -> i32 {
        self.peer_count.load(Ordering::Relaxed)
    }
}

/// Thread-safe wrapper for BlockchainSyncService
pub struct AsyncBlockchainSyncService {
    inner: Arc<RwLock<BlockchainSyncService>>,
}

impl AsyncBlockchainSyncService {
    /// Create new async blockchain sync service
    pub fn new(blockchain_db: Arc<AsyncBlockchainDatabase>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(BlockchainSyncService::new(blockchain_db))),
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
    }

    /// Get peer count
    pub async fn get_peer_count(&self) -> i32 {
        let service = self.inner.read().await;
        service.get_peer_count()
    }

    /// Start blockchain synchronization
    pub async fn start_sync(&self) -> AppResult<()> {
        // The sync process is already started in initialize, so this is a no-op
        Ok(())
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
