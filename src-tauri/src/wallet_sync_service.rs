use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::{RwLock, Mutex};

use crate::blockchain_database::AsyncBlockchainDatabase;
use crate::wallet_manager::AsyncWalletManager;
use crate::wallet_data::Utxo;
use crate::config::ConfigManager;
use crate::errors::*;

/// Wallet sync status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSyncStatus {
    pub wallet_id: String,
    pub is_syncing: bool,
    pub sync_progress: f64, // 0.0 to 1.0
    pub last_sync_block: u64,
    pub current_balance: u64,
    pub transaction_count: u32,
    pub utxo_count: u32,
}

/// Wallet sync service for individual wallet synchronization
pub struct WalletSyncService {
    blockchain_db: Arc<AsyncBlockchainDatabase>,
    wallet_manager: Option<AsyncWalletManager>,
    config_manager: Option<Arc<ConfigManager>>,
    active_syncs: Arc<RwLock<HashMap<String, WalletSyncStatus>>>,
    app_handle: Option<AppHandle>,
}

impl WalletSyncService {    /// Create new wallet sync service
    pub fn new(blockchain_db: Arc<AsyncBlockchainDatabase>) -> Self {
        Self {
            blockchain_db,
            wallet_manager: None,
            config_manager: None,
            active_syncs: Arc::new(RwLock::new(HashMap::new())),
            app_handle: None,
        }
    }/// Initialize with app handle for event emission
    pub async fn initialize(&mut self, app_handle: AppHandle) -> AppResult<()> {
        self.app_handle = Some(app_handle);
        Ok(())
    }    /// Set wallet manager for updating wallet data
    pub async fn set_wallet_manager(&mut self, wallet_manager: AsyncWalletManager) {
        self.wallet_manager = Some(wallet_manager);
    }

    /// Set config manager for updating wallet configuration
    pub async fn set_config_manager(&mut self, config_manager: Arc<ConfigManager>) {
        self.config_manager = Some(config_manager);
    }

    /// Start syncing a wallet
    pub async fn start_wallet_sync(&self, wallet_id: String, addresses: Vec<String>) -> AppResult<()> {
        info!("Starting wallet sync for wallet: {}", wallet_id);

        // Check if already syncing
        {
            let active_syncs = self.active_syncs.read().await;
            if let Some(status) = active_syncs.get(&wallet_id) {
                if status.is_syncing {
                    warn!("Wallet {} is already syncing", wallet_id);
                    return Ok(());
                }
            }
        }

        // Initialize sync status
        let sync_status = WalletSyncStatus {
            wallet_id: wallet_id.clone(),
            is_syncing: true,
            sync_progress: 0.0,
            last_sync_block: 0,
            current_balance: 0,
            transaction_count: 0,
            utxo_count: 0,
        };

        {
            let mut active_syncs = self.active_syncs.write().await;
            active_syncs.insert(wallet_id.clone(), sync_status);
        }

        // Emit initial status
        self.emit_wallet_sync_status(&wallet_id).await;        // Start sync process in background
        let blockchain_db = self.blockchain_db.clone();
        let wallet_manager = match &self.wallet_manager {
            Some(manager) => manager.clone(),
            None => {
                error!("No wallet manager available for sync");
                return Err(AppError::Generic("No wallet manager available".to_string()));
            }
        };
        let config_manager = self.config_manager.clone();
        let active_syncs = self.active_syncs.clone();
        let app_handle = self.app_handle.clone();tokio::spawn(async move {
            let active_syncs_clone = active_syncs.clone();            if let Err(e) = Self::perform_wallet_sync(                wallet_id.clone(),
                addresses,
                blockchain_db,
                wallet_manager,
                config_manager,
                active_syncs,
                app_handle,
            ).await {
                error!("Wallet sync failed for {}: {}", wallet_id, e);
                
                // Mark sync as failed
                let mut syncs = active_syncs_clone.write().await;
                if let Some(status) = syncs.get_mut(&wallet_id) {
                    status.is_syncing = false;
                }
            }
        });

        Ok(())
    }

    /// Stop syncing a wallet
    pub async fn stop_wallet_sync(&self, wallet_id: &str) -> AppResult<()> {
        info!("Stopping wallet sync for wallet: {}", wallet_id);

        let mut active_syncs = self.active_syncs.write().await;
        if let Some(status) = active_syncs.get_mut(wallet_id) {
            status.is_syncing = false;
        }

        Ok(())
    }

    /// Get sync status for a wallet
    pub async fn get_wallet_sync_status(&self, wallet_id: &str) -> Option<WalletSyncStatus> {
        let active_syncs = self.active_syncs.read().await;
        active_syncs.get(wallet_id).cloned()
    }    /// Get all active sync statuses
    pub async fn get_all_sync_statuses(&self) -> HashMap<String, WalletSyncStatus> {
        let active_syncs = self.active_syncs.read().await;
        active_syncs.clone()
    }    /// Perform the actual wallet synchronization
    async fn perform_wallet_sync(
        wallet_id: String,
        addresses: Vec<String>,
        blockchain_db: Arc<AsyncBlockchainDatabase>,
        wallet_manager: AsyncWalletManager,
        config_manager: Option<Arc<ConfigManager>>,
        active_syncs: Arc<RwLock<HashMap<String, WalletSyncStatus>>>,
        app_handle: Option<AppHandle>,
    ) -> AppResult<()> {
        info!("Performing wallet sync for {} with {} addresses", wallet_id, addresses.len());

        let current_height = blockchain_db.get_block_height().await
            .map_err(|e| AppError::Generic(format!("Failed to get block height: {}", e)))?;

        let mut total_balance = 0u64;
        let mut total_utxos = 0u32;
        let mut all_utxos = Vec::new();
        let transaction_count = 0u32;
        let _processed_blocks = 0u64;

        // Sync each address
        for (addr_index, address) in addresses.iter().enumerate() {
            debug!("Syncing address {}: {}", addr_index + 1, address);

            // Get UTXOs for this address
            let utxos = blockchain_db.get_address_utxos(address).await
                .map_err(|e| AppError::Generic(format!("Failed to get UTXOs for address {}: {}", address, e)))?;            let address_balance = utxos.iter().map(|utxo| utxo.value).sum::<u64>();
            total_balance += address_balance;
            total_utxos += utxos.len() as u32;
            all_utxos.extend(utxos);

            debug!("Address {} has {} UTXOs with total value {}", address, all_utxos.len(), address_balance);

            // Update progress
            let progress = (addr_index + 1) as f64 / addresses.len() as f64;
            {
                let mut syncs = active_syncs.write().await;
                if let Some(status) = syncs.get_mut(&wallet_id) {
                    if !status.is_syncing {
                        info!("Wallet sync cancelled for {}", wallet_id);
                        return Ok(());
                    }
                    
                    status.sync_progress = progress;
                    status.current_balance = total_balance;
                    status.utxo_count = total_utxos;
                    status.last_sync_block = current_height;
                }
            }

            // Emit progress update
            if let Some(ref app) = app_handle {
                let status = {
                    let syncs = active_syncs.read().await;
                    syncs.get(&wallet_id).cloned()
                };
                
                if let Some(status) = status {
                    if let Err(e) = app.emit("wallet-sync-status", &status) {
                        warn!("Failed to emit wallet sync status: {}", e);
                    }
                }
            }

            // Small delay to prevent overwhelming the system
            tokio::time::sleep(Duration::from_millis(100)).await;
        }        // Mark sync as completed
        {
            let mut syncs = active_syncs.write().await;
            if let Some(status) = syncs.get_mut(&wallet_id) {                status.is_syncing = false;
                status.sync_progress = 1.0;
                status.current_balance = total_balance;
                status.utxo_count = total_utxos;
                status.last_sync_block = current_height;
                status.transaction_count = transaction_count;
            }
        }

        // Update wallet data in memory and save to disk
        info!("Updating wallet data for {} with balance: {}, UTXOs: {}", wallet_id, total_balance, total_utxos);
        
        let mut manager = wallet_manager.get_manager().await;
        if let Some(wallet) = manager.get_current_wallet_mut() {
            if wallet.name == wallet_id {
                // Convert blockchain UTXOs to wallet UTXOs
                let wallet_utxos: Vec<Utxo> = all_utxos.into_iter().map(|blockchain_utxo| {
                    Utxo {
                        txid: blockchain_utxo.txid,
                        vout: blockchain_utxo.output_index,
                        value: blockchain_utxo.value,
                        script_pubkey: blockchain_utxo.script_pubkey,
                        address: blockchain_utxo.address,
                        is_change: false, // Assume not change for now
                        height: Some(blockchain_utxo.block_height as u32),
                    }
                }).collect();

                // Update wallet data
                wallet.data.balance = total_balance;
                wallet.data.utxos = wallet_utxos;
                wallet.data.block_height = current_height as u32;
                wallet.data.modified_at = chrono::Utc::now().timestamp();

                // Save wallet data to disk
                let wallet_data_path = wallet.path.join("wallet.dat");
                let password = if wallet.data.is_encrypted { 
                    // In a real implementation, we'd need to securely get the password
                    // For now, we'll skip saving encrypted wallets during sync to avoid password issues
                    warn!("Skipping disk save for encrypted wallet {} during sync", wallet_id);
                    None
                } else { 
                    None 
                };                if !wallet.data.is_encrypted {
                    if let Err(e) = wallet.data.save(&wallet_data_path, password) {
                        warn!("Failed to save wallet data to disk: {}", e);
                    } else {
                        info!("Successfully saved updated wallet data for {}", wallet_id);
                    }
                }

                // Update wallet addresses and block height in config
                if let Some(ref config_mgr) = config_manager {
                    let wallet_addresses: Vec<String> = wallet.data.addresses.iter()
                        .map(|addr_info| addr_info.address.clone())
                        .collect();
                    
                    if let Err(e) = config_mgr.update_wallet_sync_info(
                        &wallet_id,
                        wallet_addresses,
                        current_height,
                        Some(chrono::Utc::now().timestamp()),
                    ).await {
                        warn!("Failed to update wallet config: {}", e);
                    } else {
                        info!("Successfully updated wallet config for {}", wallet_id);
                    }
                }
            } else {
                warn!("Current wallet name mismatch: expected {}, found {}", wallet_id, wallet.name);
            }
        } else {
            warn!("No current wallet found to update after sync");
        }

        // Emit final status
        if let Some(ref app) = app_handle {
            let status = {
                let syncs = active_syncs.read().await;
                syncs.get(&wallet_id).cloned()
            };
            
            if let Some(status) = status {
                if let Err(e) = app.emit("wallet-sync-status", &status) {
                    warn!("Failed to emit final wallet sync status: {}", e);
                }
            }
        }

        info!("Wallet sync completed for {}: {} balance, {} UTXOs", wallet_id, total_balance, total_utxos);
        Ok(())
    }

    /// Emit wallet sync status event
    async fn emit_wallet_sync_status(&self, wallet_id: &str) {
        if let Some(ref app) = self.app_handle {
            let status = self.get_wallet_sync_status(wallet_id).await;
            if let Some(status) = status {
                if let Err(e) = app.emit("wallet-sync-status", &status) {
                    warn!("Failed to emit wallet sync status: {}", e);
                }
            }
        }
    }
}

/// Thread-safe wrapper for WalletSyncService
pub struct AsyncWalletSyncService {
    inner: Arc<Mutex<WalletSyncService>>,
}

impl AsyncWalletSyncService {
    /// Create new async wallet sync service
    pub fn new(blockchain_db: Arc<AsyncBlockchainDatabase>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(WalletSyncService::new(blockchain_db))),
        }
    }

    /// Initialize with app handle
    pub async fn initialize(&self, app_handle: AppHandle) -> AppResult<()> {
        let mut service = self.inner.lock().await;
        service.initialize(app_handle).await
    }

    /// Start syncing a wallet
    pub async fn start_wallet_sync(&self, wallet_id: String, addresses: Vec<String>) -> AppResult<()> {
        let service = self.inner.lock().await;
        service.start_wallet_sync(wallet_id, addresses).await
    }

    /// Stop syncing a wallet
    pub async fn stop_wallet_sync(&self, wallet_id: &str) -> AppResult<()> {
        let service = self.inner.lock().await;
        service.stop_wallet_sync(wallet_id).await
    }

    /// Get sync status for a wallet
    pub async fn get_wallet_sync_status(&self, wallet_id: &str) -> Option<WalletSyncStatus> {
        let service = self.inner.lock().await;
        service.get_wallet_sync_status(wallet_id).await
    }    /// Get all active sync statuses
    pub async fn get_all_sync_statuses(&self) -> HashMap<String, WalletSyncStatus> {
        let service = self.inner.lock().await;
        service.get_all_sync_statuses().await
    }

    /// Set wallet manager for updating wallet data
    pub async fn set_wallet_manager(&self, wallet_manager: AsyncWalletManager) {
        let mut service = self.inner.lock().await;
        service.set_wallet_manager(wallet_manager).await;
    }

    /// Set config manager for updating wallet configuration
    pub async fn set_config_manager(&self, config_manager: Arc<ConfigManager>) {
        let mut service = self.inner.lock().await;
        service.set_config_manager(config_manager).await;
    }
}
