use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{RwLock, Mutex};
use sha2::{Sha256, Digest};

use crate::blockchain_database::{AsyncBlockchainDatabase, Block, Transaction, TransactionOutput};
use crate::errors::*;

// Bitcoin-compatible constants
const MAX_BLOCK_SIZE: usize = 1_000_000; // 1MB like Bitcoin
const MAX_BLOCK_WEIGHT: usize = 4_000_000; // 4MB weight units like Bitcoin
const TARGET_BLOCK_TIME: u64 = 60; // 1 minute instead of Bitcoin's 10 minutes
const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 144; // Adjust every 144 blocks (2.4 hours at 1 min/block)
const INITIAL_DIFFICULTY_TARGET: u64 = 0x00000000FFFF0000; // Simplified target that fits in u64
const COINBASE_REWARD: u64 = 5000000000; // 50 BTC in satoshis (will halve every 210,000 blocks)
const HALVING_INTERVAL: u64 = 210000; // Halve reward every 210,000 blocks

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
    pub current_target: String,
    pub network_hash_rate: f64,
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
            target_block_time: Duration::from_secs(TARGET_BLOCK_TIME), // 1 minute target block time
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

        // Get current difficulty and target
        let (current_difficulty, current_target) = self.calculate_current_difficulty().await?;

        // Initialize mining status
        let mining_status = MiningStatus {
            wallet_id: wallet_id.clone(),
            is_mining: true,
            hash_rate: 0.0,
            blocks_mined: 0,
            last_block_time: None,
            mining_address: mining_address.clone(),
            current_difficulty,
            current_target: format!("{:064x}", current_target),
            network_hash_rate: 0.0,
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

    /// Calculate current mining difficulty using Bitcoin-style algorithm
    async fn calculate_current_difficulty(&self) -> AppResult<(u64, u64)> {
        let current_height = self.blockchain_db.get_block_height().await
            .map_err(|e| AppError::Generic(format!("Failed to get block height: {}", e)))?;

        // For the first block, use initial difficulty
        if current_height == 0 {
            return Ok((bits_to_difficulty(0x1d00ffff), INITIAL_DIFFICULTY_TARGET));
        }

        // Check if we need to adjust difficulty
        if current_height % DIFFICULTY_ADJUSTMENT_INTERVAL == 0 && current_height > 0 {
            self.adjust_difficulty(current_height).await
        } else {
            // Use previous block's difficulty
            if let Some(previous_block) = self.blockchain_db.get_block_by_height(current_height).await
                .map_err(|e| AppError::Generic(format!("Failed to get previous block: {}", e)))? {
                let target = difficulty_to_target(previous_block.difficulty);
                Ok((previous_block.difficulty, target))
            } else {
                // Fallback to initial difficulty
                Ok((bits_to_difficulty(0x1d00ffff), INITIAL_DIFFICULTY_TARGET))
            }
        }
    }

    /// Adjust difficulty based on block times (Bitcoin-style difficulty adjustment)
    async fn adjust_difficulty(&self, current_height: u64) -> AppResult<(u64, u64)> {
        let adjustment_start_height = current_height - DIFFICULTY_ADJUSTMENT_INTERVAL;
        
        // Get the first block of the adjustment period
        let first_block = self.blockchain_db.get_block_by_height(adjustment_start_height).await
            .map_err(|e| AppError::Generic(format!("Failed to get adjustment start block: {}", e)))?
            .ok_or_else(|| AppError::Generic("Adjustment start block not found".to_string()))?;

        // Get the last block (previous block)
        let last_block = self.blockchain_db.get_block_by_height(current_height - 1).await
            .map_err(|e| AppError::Generic(format!("Failed to get last block: {}", e)))?
            .ok_or_else(|| AppError::Generic("Last block not found".to_string()))?;

        // Calculate actual time taken for the adjustment period
        let actual_timespan = last_block.timestamp - first_block.timestamp;
        let target_timespan = DIFFICULTY_ADJUSTMENT_INTERVAL * TARGET_BLOCK_TIME;

        // Limit adjustment to 4x increase or 1/4 decrease (Bitcoin rule)
        let adjusted_timespan = actual_timespan.max(target_timespan / 4).min(target_timespan * 4);

        // Calculate new difficulty
        let old_target = difficulty_to_target(last_block.difficulty);
        let new_target = (old_target as u128 * adjusted_timespan as u128 / target_timespan as u128) as u64;
        
        // Ensure target doesn't exceed the maximum (minimum difficulty)
        let new_target = new_target.min(INITIAL_DIFFICULTY_TARGET);
        let new_difficulty = target_to_difficulty(new_target);

        info!(
            "Difficulty adjustment at height {}: actual_timespan={}s, target_timespan={}s, old_difficulty={}, new_difficulty={}",
            current_height, actual_timespan, target_timespan, last_block.difficulty, new_difficulty
        );

        Ok((new_difficulty, new_target))
    }

    /// Calculate mining reward based on block height (with halving)
    fn calculate_block_reward(height: u64) -> u64 {
        let halvings = height / HALVING_INTERVAL;
        if halvings >= 64 {
            0 // After 64 halvings, reward becomes 0
        } else {
            COINBASE_REWARD >> halvings
        }
    }

    /// Calculate network hash rate estimate
    async fn estimate_network_hash_rate(&self) -> AppResult<f64> {
        let current_height = self.blockchain_db.get_block_height().await
            .map_err(|e| AppError::Generic(format!("Failed to get block height: {}", e)))?;

        if current_height < 10 {
            return Ok(0.0);
        }

        // Look at last 10 blocks to estimate hash rate
        let mut total_work = 0.0;
        let mut time_span = 0u64;

        if let Some(latest_block) = self.blockchain_db.get_block_by_height(current_height).await
            .map_err(|e| AppError::Generic(format!("Failed to get latest block: {}", e)))? {
            
            if let Some(older_block) = self.blockchain_db.get_block_by_height(current_height.saturating_sub(10)).await
                .map_err(|e| AppError::Generic(format!("Failed to get older block: {}", e)))? {
                
                time_span = latest_block.timestamp - older_block.timestamp;
                
                // Estimate work done (simplified)
                for height in (current_height.saturating_sub(10))..=current_height {
                    if let Some(block) = self.blockchain_db.get_block_by_height(height).await
                        .map_err(|e| AppError::Generic(format!("Failed to get block: {}", e)))? {
                        total_work += block.difficulty as f64;
                    }
                }
            }
        }

        if time_span > 0 {
            Ok(total_work / time_span as f64)
        } else {
            Ok(0.0)
        }
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
            if let Ok(true) = Self::try_mine_block_with_app_handle(&wallet_id, &mining_address, &blockchain_db, &active_miners, &app_handle).await {
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

    /// Try to mine a single block using Bitcoin-style Proof of Work
    async fn try_mine_block_with_app_handle(
        wallet_id: &str,
        mining_address: &str,
        blockchain_db: &Arc<AsyncBlockchainDatabase>,
        active_miners: &Arc<RwLock<HashMap<String, MiningStatus>>>,
        app_handle: &Option<AppHandle>,
    ) -> AppResult<bool> {
        // Get current block height and last block hash
        let current_height = blockchain_db.get_block_height().await
            .map_err(|e| AppError::Generic(format!("Failed to get block height: {}", e)))?;

        let previous_hash = if current_height > 0 {
            let previous_block = blockchain_db.get_block_by_height(current_height).await
                .map_err(|e| AppError::Generic(format!("Failed to get previous block: {}", e)))?;
            previous_block.map(|block| block.hash).unwrap_or_else(|| "0".repeat(64))
        } else {
            "0".repeat(64)
        };

        // Get current difficulty and target
        let (difficulty, target) = {
            let miners = active_miners.read().await;
            if let Some(status) = miners.get(wallet_id) {
                let target = if let Ok(target_val) = u64::from_str_radix(&status.current_target, 16) {
                    target_val
                } else {
                    INITIAL_DIFFICULTY_TARGET
                };
                (status.current_difficulty, target)
            } else {
                (bits_to_difficulty(0x1d00ffff), INITIAL_DIFFICULTY_TARGET)
            }
        };

        // Calculate mining reward with halving
        let block_reward = Self::calculate_block_reward(current_height + 1);

        // Create coinbase transaction (mining reward)
        let coinbase_tx = Transaction {
            txid: format!("coinbase_{}", current_height + 1),
            inputs: vec![],
            outputs: vec![TransactionOutput {
                value: block_reward,
                script_pubkey: format!("OP_DUP OP_HASH160 {} OP_EQUALVERIFY OP_CHECKSIG", mining_address),
                address: mining_address.to_string(),
            }],
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)
                .unwrap_or_default().as_secs(),
            fee: 0,
        };

        // Get pending transactions from mempool if available
        let mut transactions = vec![coinbase_tx.clone()];
        
        // Try to get mempool transactions through app handle
        if let Some(app) = app_handle {
            if let Some(mempool) = app.try_state::<crate::mempool_service::AsyncMempoolService>() {
                let mempool_txs = mempool.get_transactions_for_mining(100, MAX_BLOCK_SIZE - 1000).await;
                if !mempool_txs.is_empty() {
                    info!("Including {} transactions from mempool in block", mempool_txs.len());
                    transactions.extend(mempool_txs);
                } else {
                    debug!("No transactions available in mempool");
                }
            } else {
                debug!("Mempool service not available, mining with coinbase only");
            }
        }
        
        // Calculate merkle root
        let merkle_root = calculate_merkle_root(&transactions);

        // Create block header for mining
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_default().as_secs();

        // Start mining with random nonce
        let mut nonce = rand::random::<u64>();

        // Try a few nonces before yielding control (to prevent blocking)
        for _ in 0..1000 {
            // Create block header data
            let block_header = create_block_header(
                current_height + 1,
                &previous_hash,
                &merkle_root,
                timestamp,
                target_to_bits(target),
                nonce,
            );

            // Calculate double SHA256 hash (Bitcoin-style)
            let hash_bytes = double_sha256(&block_header);
            let hash_hex = format_hash(&hash_bytes);

            // Check if hash meets target (Bitcoin-style difficulty check)
            if hash_meets_target(&hash_hex, target) {
                // Block found! Create the complete block
                let new_block = Block {
                    height: current_height + 1,
                    hash: hash_hex,
                    previous_hash,
                    timestamp,
                    transactions,
                    nonce,
                    difficulty,
                    merkle_root,
                };

                // Verify block size constraints
                let block_json = serde_json::to_string(&new_block).unwrap_or_default();
                if block_json.len() > MAX_BLOCK_SIZE {
                    warn!("Block exceeds maximum size limit: {} > {}", block_json.len(), MAX_BLOCK_SIZE);
                    return Ok(false);
                }

                // Store the mined block
                blockchain_db.store_block(&new_block).await
                    .map_err(|e| AppError::Generic(format!("Failed to store mined block: {}", e)))?;

                // Submit block to network
                if let Some(app_handle) = app_handle {
                    if let Some(network_service) = app_handle.try_state::<crate::network_service::AsyncNetworkService>() {
                        if let Err(e) = network_service.announce_new_block(new_block.hash.clone()).await {
                            warn!("Failed to announce mined block to network: {}", e);
                        } else {
                            info!("Successfully announced mined block {} to network", new_block.hash);
                        }
                    }
                }

                info!(
                    "Block {} mined successfully! Hash: {}, Difficulty: {}, Reward: {} satoshis",
                    new_block.height, new_block.hash, difficulty, block_reward
                );
                return Ok(true);
            }

            nonce = nonce.wrapping_add(1);
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

// Bitcoin-style difficulty conversion functions

/// Convert Bitcoin compact bits representation to difficulty value
fn bits_to_difficulty(bits: u32) -> u64 {
    let max_target = INITIAL_DIFFICULTY_TARGET;
    let target = bits_to_target(bits);
    if target == 0 {
        return u64::MAX;
    }
    (max_target / target).max(1)
}

/// Convert difficulty to target value
fn difficulty_to_target(difficulty: u64) -> u64 {
    if difficulty == 0 {
        return INITIAL_DIFFICULTY_TARGET;
    }
    INITIAL_DIFFICULTY_TARGET / difficulty
}

/// Convert target to difficulty value
fn target_to_difficulty(target: u64) -> u64 {
    if target == 0 {
        return u64::MAX;
    }
    INITIAL_DIFFICULTY_TARGET / target
}

/// Convert Bitcoin compact bits to target value
fn bits_to_target(bits: u32) -> u64 {
    let exponent = ((bits >> 24) & 0xff) as u32;
    let mantissa = bits & 0x00ffffff;
    
    if exponent <= 3 {
        mantissa as u64 >> (8 * (3 - exponent))
    } else {
        (mantissa as u64) << (8 * (exponent - 3))
    }
}

/// Convert target to Bitcoin compact bits representation
fn target_to_bits(target: u64) -> u32 {
    if target == 0 {
        return 0;
    }
    
    let target_bytes = target.to_be_bytes();
    let mut exponent = 8;
    
    // Find the most significant byte
    while exponent > 0 && target_bytes[8 - exponent] == 0 {
        exponent -= 1;
    }
    
    if exponent <= 3 {
        let mantissa = (target as u32) << (8 * (3 - exponent));
        mantissa | (exponent as u32) << 24
    } else {
        let mantissa = (target >> (8 * (exponent - 3))) as u32;
        (mantissa & 0x00ffffff) | (exponent as u32) << 24
    }
}

/// Check if a hash meets the target difficulty
fn hash_meets_target(hash: &str, target: u64) -> bool {
    // Convert hash to numeric value for comparison
    if let Ok(hash_value) = u64::from_str_radix(&hash[0..16], 16) {
        hash_value <= target
    } else {
        false
    }
}

/// Calculate double SHA256 hash (Bitcoin-style)
fn double_sha256(data: &[u8]) -> [u8; 32] {
    let first_hash = Sha256::digest(data);
    let second_hash = Sha256::digest(&first_hash);
    second_hash.into()
}

/// Create Bitcoin-style block header for hashing
fn create_block_header(
    height: u64,
    previous_hash: &str,
    merkle_root: &str,
    timestamp: u64,
    bits: u32,
    nonce: u64,
) -> Vec<u8> {
    let header_data = format!(
        "{}{}{}{}{}{}",
        height,
        previous_hash,
        merkle_root,
        timestamp,
        bits,
        nonce
    );
    header_data.into_bytes()
}

/// Calculate merkle root from transactions (simplified implementation)
fn calculate_merkle_root(transactions: &[Transaction]) -> String {
    if transactions.is_empty() {
        return "0".repeat(64);
    }

    // For now, use a simple hash of all transaction IDs
    // In a full implementation, this would build a proper merkle tree
    let mut hasher = Sha256::new();
    for tx in transactions {
        hasher.update(tx.txid.as_bytes());
    }
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Format hash as hex string
fn format_hash(hash: &[u8]) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}
