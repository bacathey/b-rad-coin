use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::{RwLock, Mutex};
use sha2::{Sha256, Digest};

use crate::blockchain_database::{AsyncBlockchainDatabase, Block, Transaction, TransactionOutput};
use crate::errors::*;

/// Mining status for a wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningStatus {
    pub wallet_id: String,
    pub is_mining: bool,
    pub hash_rate: f64, // hashes per second
    pub blocks_mined: u32,
    pub last_block_time: Option<u64>,
    pub mining_address: String,
    pub current_difficulty: u64,
}

/// Mining service for individual wallet mining
pub struct MiningService {
    blockchain_db: Arc<AsyncBlockchainDatabase>,
    active_miners: Arc<RwLock<HashMap<String, MiningStatus>>>,
    app_handle: Option<AppHandle>,
    target_block_time: Duration, // Target time between blocks
}

impl MiningService {    /// Create new mining service
    pub fn new(blockchain_db: Arc<AsyncBlockchainDatabase>) -> Self {
        Self {
            blockchain_db,
            active_miners: Arc::new(RwLock::new(HashMap::new())),
            app_handle: None,
            target_block_time: Duration::from_secs(30), // 30 second target block time
        }
    }

    /// Initialize with app handle for event emission
    pub async fn initialize(&mut self, app_handle: AppHandle) -> AppResult<()> {
        self.app_handle = Some(app_handle);
        Ok(())
    }

    /// Start mining for a wallet
    pub async fn start_mining(&self, wallet_id: String, mining_address: String) -> AppResult<()> {
        info!("Starting mining for wallet: {} at address: {}", wallet_id, mining_address);

        // Check if already mining
        {
            let active_miners = self.active_miners.read().await;
            if let Some(status) = active_miners.get(&wallet_id) {
                if status.is_mining {
                    warn!("Wallet {} is already mining", wallet_id);
                    return Ok(());
                }
            }
        }

        // Get current difficulty
        let current_difficulty = self.calculate_difficulty().await?;

        // Initialize mining status
        let mining_status = MiningStatus {
            wallet_id: wallet_id.clone(),
            is_mining: true,
            hash_rate: 0.0,
            blocks_mined: 0,
            last_block_time: None,
            mining_address: mining_address.clone(),
            current_difficulty,
        };

        {
            let mut active_miners = self.active_miners.write().await;
            active_miners.insert(wallet_id.clone(), mining_status);
        }

        // Emit initial status
        self.emit_mining_status(&wallet_id).await;

        // Start mining process in background
        let blockchain_db = self.blockchain_db.clone();
        let active_miners = self.active_miners.clone();
        let app_handle = self.app_handle.clone();        tokio::spawn(async move {
            let active_miners_clone = active_miners.clone();
            if let Err(e) = Self::perform_mining(
                wallet_id.clone(),
                mining_address,
                blockchain_db,
                active_miners,
                app_handle,
            ).await {
                error!("Mining failed for {}: {}", wallet_id, e);
                
                // Mark mining as stopped
                let mut miners = active_miners_clone.write().await;
                if let Some(status) = miners.get_mut(&wallet_id) {
                    status.is_mining = false;
                }
            }
        });

        Ok(())
    }

    /// Stop mining for a wallet
    pub async fn stop_mining(&self, wallet_id: &str) -> AppResult<()> {
        info!("Stopping mining for wallet: {}", wallet_id);

        let mut active_miners = self.active_miners.write().await;
        if let Some(status) = active_miners.get_mut(wallet_id) {
            status.is_mining = false;
        }

        Ok(())
    }

    /// Get mining status for a wallet
    pub async fn get_mining_status(&self, wallet_id: &str) -> Option<MiningStatus> {
        let active_miners = self.active_miners.read().await;
        active_miners.get(wallet_id).cloned()
    }

    /// Get all active mining statuses
    pub async fn get_all_mining_statuses(&self) -> HashMap<String, MiningStatus> {
        let active_miners = self.active_miners.read().await;
        active_miners.clone()
    }

    /// Calculate current mining difficulty
    async fn calculate_difficulty(&self) -> AppResult<u64> {
        // Simple difficulty adjustment based on block height
        // In a real implementation, this would be based on block times
        let current_height = self.blockchain_db.get_block_height().await
            .map_err(|e| AppError::Generic(format!("Failed to get block height: {}", e)))?;

        // Start with base difficulty and increase every 100 blocks
        let base_difficulty = 1000u64;
        let difficulty_adjustment = current_height / 100;
        
        Ok(base_difficulty + (difficulty_adjustment * 100))
    }

    /// Perform the actual mining
    async fn perform_mining(
        wallet_id: String,
        mining_address: String,
        blockchain_db: Arc<AsyncBlockchainDatabase>,
        active_miners: Arc<RwLock<HashMap<String, MiningStatus>>>,
        app_handle: Option<AppHandle>,
    ) -> AppResult<()> {
        info!("Starting mining process for wallet: {}", wallet_id);

        let mut hash_count = 0u64;
        let mut last_hash_rate_update = std::time::Instant::now();

        loop {
            // Check if mining should continue
            {
                let miners = active_miners.read().await;
                if let Some(status) = miners.get(&wallet_id) {
                    if !status.is_mining {
                        info!("Mining stopped for wallet: {}", wallet_id);
                        break;
                    }
                } else {
                    break;
                }
            }

            // Try to mine a block
            if let Ok(true) = Self::try_mine_block(&wallet_id, &mining_address, &blockchain_db, &active_miners).await {
                info!("Block successfully mined by wallet: {}", wallet_id);
                
                // Update blocks mined count
                {
                    let mut miners = active_miners.write().await;
                    if let Some(status) = miners.get_mut(&wallet_id) {
                        status.blocks_mined += 1;
                        status.last_block_time = Some(
                            SystemTime::now().duration_since(UNIX_EPOCH)
                                .unwrap_or_default().as_secs()
                        );
                    }
                }

                // Emit mining status update
                if let Some(ref app) = app_handle {
                    let status = {
                        let miners = active_miners.read().await;
                        miners.get(&wallet_id).cloned()
                    };
                    
                    if let Some(status) = status {
                        if let Err(e) = app.emit("mining-status", &status) {
                            warn!("Failed to emit mining status: {}", e);
                        }
                    }
                }
            }

            hash_count += 1;

            // Update hash rate every second
            if last_hash_rate_update.elapsed() >= Duration::from_secs(1) {
                let hash_rate = hash_count as f64 / last_hash_rate_update.elapsed().as_secs_f64();
                
                {
                    let mut miners = active_miners.write().await;
                    if let Some(status) = miners.get_mut(&wallet_id) {
                        status.hash_rate = hash_rate;
                    }
                }

                hash_count = 0;
                last_hash_rate_update = std::time::Instant::now();

                // Emit hash rate update
                if let Some(ref app) = app_handle {
                    let status = {
                        let miners = active_miners.read().await;
                        miners.get(&wallet_id).cloned()
                    };
                    
                    if let Some(status) = status {
                        if let Err(e) = app.emit("mining-status", &status) {
                            warn!("Failed to emit mining status: {}", e);
                        }
                    }
                }
            }

            // Small delay to prevent overwhelming the CPU
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Ok(())
    }

    /// Try to mine a single block
    async fn try_mine_block(
        wallet_id: &str,
        mining_address: &str,
        blockchain_db: &Arc<AsyncBlockchainDatabase>,
        active_miners: &Arc<RwLock<HashMap<String, MiningStatus>>>,
    ) -> AppResult<bool> {        // Get current block height and last block hash
        let current_height = blockchain_db.get_block_height().await
            .map_err(|e| AppError::Generic(format!("Failed to get block height: {}", e)))?;

        let previous_hash = if current_height > 0 {
            let previous_block = blockchain_db.get_block_by_height(current_height).await
                .map_err(|e| AppError::Generic(format!("Failed to get previous block: {}", e)))?;
            previous_block.map(|block| block.hash).unwrap_or_else(|| "0".repeat(64))
        } else {
            "0".repeat(64)
        };

        // Get current difficulty
        let difficulty = {
            let miners = active_miners.read().await;
            miners.get(wallet_id)
                .map(|status| status.current_difficulty)
                .unwrap_or(1000)
        };        // Create coinbase transaction (mining reward)
        let coinbase_tx = Transaction {
            txid: format!("coinbase_{}", current_height + 1),
            inputs: vec![],
            outputs: vec![TransactionOutput {
                value: 50000000, // 0.5 coin reward
                script_pubkey: format!("OP_DUP OP_HASH160 {} OP_EQUALVERIFY OP_CHECKSIG", mining_address),
                address: mining_address.to_string(),
            }],
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            fee: 0,
        };

        // Create block candidate
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_default().as_secs();

        let nonce = rand::random::<u64>();

        // Create block data for hashing
        let block_data = format!(
            "{}{}{}{}{}",
            current_height + 1,
            previous_hash,
            timestamp,
            serde_json::to_string(&vec![coinbase_tx.clone()]).unwrap_or_default(),
            nonce
        );

        // Hash the block
        let mut hasher = Sha256::new();
        hasher.update(block_data.as_bytes());
        let hash_result = hasher.finalize();
        let hash_hex = format!("{:x}", hash_result);        // Check if hash meets difficulty (simple leading zeros check)
        let leading_zeros = hash_hex.chars().take_while(|&c| c == '0').count();
        let required_zeros = (difficulty / 1000) as usize; // Scale difficulty to leading zeros

        if leading_zeros >= required_zeros {
            // Block found! Add to blockchain
            let new_block = Block {
                height: current_height + 1,
                hash: hash_hex,
                previous_hash,
                timestamp,
                transactions: vec![coinbase_tx],                nonce,
                difficulty,
                merkle_root: "0".repeat(64), // Simplified merkle root
            };

            blockchain_db.store_block(&new_block).await
                .map_err(|e| AppError::Generic(format!("Failed to store mined block: {}", e)))?;

            info!("Block {} mined with hash: {}", new_block.height, new_block.hash);
            return Ok(true);
        }

        Ok(false)
    }

    /// Emit mining status event
    async fn emit_mining_status(&self, wallet_id: &str) {
        if let Some(ref app) = self.app_handle {
            let status = self.get_mining_status(wallet_id).await;
            if let Some(status) = status {
                if let Err(e) = app.emit("mining-status", &status) {
                    warn!("Failed to emit mining status: {}", e);
                }
            }
        }
    }
}

/// Thread-safe wrapper for MiningService
pub struct AsyncMiningService {
    inner: Arc<Mutex<MiningService>>,
}

impl AsyncMiningService {
    /// Create new async mining service
    pub fn new(blockchain_db: Arc<AsyncBlockchainDatabase>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MiningService::new(blockchain_db))),
        }
    }

    /// Initialize with app handle
    pub async fn initialize(&self, app_handle: AppHandle) -> AppResult<()> {
        let mut service = self.inner.lock().await;
        service.initialize(app_handle).await
    }

    /// Start mining for a wallet
    pub async fn start_mining(&self, wallet_id: String, mining_address: String) -> AppResult<()> {
        let service = self.inner.lock().await;
        service.start_mining(wallet_id, mining_address).await
    }

    /// Stop mining for a wallet
    pub async fn stop_mining(&self, wallet_id: &str) -> AppResult<()> {
        let service = self.inner.lock().await;
        service.stop_mining(wallet_id).await
    }

    /// Get mining status for a wallet
    pub async fn get_mining_status(&self, wallet_id: &str) -> Option<MiningStatus> {
        let service = self.inner.lock().await;
        service.get_mining_status(wallet_id).await
    }

    /// Get all active mining statuses
    pub async fn get_all_mining_statuses(&self) -> HashMap<String, MiningStatus> {
        let service = self.inner.lock().await;
        service.get_all_mining_statuses().await
    }
}
