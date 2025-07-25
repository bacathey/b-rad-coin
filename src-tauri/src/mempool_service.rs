//! Transaction Mempool Service
//! Manages pending transactions before they are included in blocks

use crate::blockchain_database::{AsyncBlockchainDatabase, Transaction, TransactionInput, TransactionOutput};
use crate::errors::*;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

/// Maximum number of transactions to keep in mempool
const MAX_MEMPOOL_SIZE: usize = 10000;

/// Maximum transaction size in bytes
const MAX_TRANSACTION_SIZE: usize = 100000; // 100KB

/// Transaction fee rate (satoshis per byte)
const MIN_FEE_RATE: u64 = 1;

/// Transaction replacement reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplacementReason {
    RbfFlag,      // Transaction explicitly signaled RBF
    HigherFee,    // Higher fee rate
    UserRequest,  // User requested replacement
}

/// RBF replacement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementResult {
    pub success: bool,
    pub old_tx_hash: String,
    pub new_tx_hash: String,
    pub reason: ReplacementReason,
    pub old_fee: u64,
    pub new_fee: u64,
    pub fee_increase: u64,
}

/// Mempool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolTransaction {
    pub transaction: Transaction,
    pub received_time: u64,
    pub fee_rate: u64, // satoshis per byte
    pub size: usize,   // transaction size in bytes
    pub dependencies: Vec<String>, // txids this transaction depends on
}

/// Mempool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolStats {
    pub transaction_count: usize,
    pub total_size_bytes: usize,
    pub min_fee_rate: u64,
    pub max_fee_rate: u64,
    pub avg_fee_rate: u64,
}

/// Transaction mempool service
pub struct MempoolService {
    transactions: Arc<RwLock<HashMap<String, MempoolTransaction>>>,
    blockchain_db: Arc<AsyncBlockchainDatabase>,
    app_handle: Option<AppHandle>,
}

impl MempoolService {
    /// Create new mempool service
    pub fn new(blockchain_db: Arc<AsyncBlockchainDatabase>) -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            blockchain_db,
            app_handle: None,
        }
    }

    /// Initialize with app handle for event emission
    pub async fn initialize(&mut self, app_handle: AppHandle) -> AppResult<()> {
        self.app_handle = Some(app_handle);
        Ok(())
    }

    /// Add transaction to mempool
    pub async fn add_transaction(&self, mut transaction: Transaction) -> AppResult<String> {
        // Generate transaction hash if not provided
        if transaction.txid.is_empty() {
            transaction.txid = self.calculate_transaction_hash(&transaction)?;
        }
        
        info!("Adding transaction {} to mempool", transaction.txid);

        // Validate transaction
        self.validate_transaction(&transaction).await?;

        // Calculate transaction metadata
        let transaction_size = self.estimate_transaction_size(&transaction)?;
        let fee_rate = self.calculate_fee_rate(&transaction, transaction_size)?;
        let dependencies = self.find_dependencies(&transaction).await;

        let tx_hash = transaction.txid.clone();
        let mempool_tx = MempoolTransaction {
            transaction: transaction.clone(),
            received_time: Self::current_timestamp(),
            fee_rate,
            size: transaction_size,
            dependencies,
        };

        // Add to mempool with eviction if necessary
        {
            let mut txs = self.transactions.write().await;
            
            // Check if we need to evict transactions
            if txs.len() >= MAX_MEMPOOL_SIZE {
                self.evict_low_fee_transactions(&mut txs).await;
            }

            txs.insert(transaction.txid.clone(), mempool_tx);
        }

        // Emit event for frontend
        self.emit_mempool_update().await;

        info!("Transaction {} added to mempool (fee rate: {} sat/byte)", 
              tx_hash, fee_rate);
        Ok(tx_hash)
    }

    /// Remove transaction from mempool (used when included in block)
    pub async fn remove_transaction(&self, txid: &str) -> Option<Transaction> {
        let mut txs = self.transactions.write().await;
        if let Some(mempool_tx) = txs.remove(txid) {
            info!("Removed transaction {} from mempool", txid);
            self.emit_mempool_update().await;
            Some(mempool_tx.transaction)
        } else {
            None
        }
    }

    /// Get transaction from mempool
    pub async fn get_transaction(&self, txid: &str) -> Option<Transaction> {
        let txs = self.transactions.read().await;
        txs.get(txid).map(|mempool_tx| mempool_tx.transaction.clone())
    }

    /// Get transactions for mining (highest fee rate first)
    pub async fn get_transactions_for_mining(&self, max_count: usize, max_size_bytes: usize) -> Vec<Transaction> {
        let txs = self.transactions.read().await;
        
        // Sort by fee rate (highest first)
        let mut sorted_txs: Vec<_> = txs.values().collect();
        sorted_txs.sort_by(|a, b| b.fee_rate.cmp(&a.fee_rate));

        let mut selected = Vec::new();
        let mut total_size = 0;

        for mempool_tx in sorted_txs {
            if selected.len() >= max_count {
                break;
            }
            
            if total_size + mempool_tx.size > max_size_bytes {
                break;
            }

            // Check dependencies are satisfied
            if self.dependencies_satisfied(&mempool_tx.dependencies, &selected) {
                selected.push(mempool_tx.transaction.clone());
                total_size += mempool_tx.size;
            }
        }

        info!("Selected {} transactions for mining (total size: {} bytes)", 
              selected.len(), total_size);
        selected
    }

    /// Get all pending transactions
    pub async fn get_all_transactions(&self) -> Vec<Transaction> {
        let txs = self.transactions.read().await;
        txs.values().map(|mempool_tx| mempool_tx.transaction.clone()).collect()
    }

    /// Get mempool statistics
    pub async fn get_stats(&self) -> MempoolStats {
        let txs = self.transactions.read().await;
        
        if txs.is_empty() {
            return MempoolStats {
                transaction_count: 0,
                total_size_bytes: 0,
                min_fee_rate: 0,
                max_fee_rate: 0,
                avg_fee_rate: 0,
            };
        }

        let fee_rates: Vec<u64> = txs.values().map(|tx| tx.fee_rate).collect();
        let total_size: usize = txs.values().map(|tx| tx.size).sum();

        MempoolStats {
            transaction_count: txs.len(),
            total_size_bytes: total_size,
            min_fee_rate: *fee_rates.iter().min().unwrap_or(&0),
            max_fee_rate: *fee_rates.iter().max().unwrap_or(&0),
            avg_fee_rate: if !fee_rates.is_empty() { 
                fee_rates.iter().sum::<u64>() / fee_rates.len() as u64 
            } else { 0 },
        }
    }

    /// Clear all transactions from mempool
    pub async fn clear(&self) {
        let mut txs = self.transactions.write().await;
        txs.clear();
        info!("Mempool cleared");
        self.emit_mempool_update().await;
    }

    /// Validate transaction before adding to mempool
    async fn validate_transaction(&self, transaction: &Transaction) -> AppResult<()> {
        // Check transaction size
        let size = self.estimate_transaction_size(transaction)?;
        if size > MAX_TRANSACTION_SIZE {
            return Err(AppError::Generic(format!("Transaction too large: {} bytes", size)));
        }

        // Check if transaction already exists in mempool
        let txs = self.transactions.read().await;
        if txs.contains_key(&transaction.txid) {
            return Err(AppError::Generic("Transaction already in mempool".to_string()));
        }

        // Check if transaction already exists in blockchain
        if self.blockchain_db.get_transaction(&transaction.txid).await.is_ok() {
            return Err(AppError::Generic("Transaction already in blockchain".to_string()));
        }

        // Validate inputs and outputs
        if transaction.inputs.is_empty() {
            return Err(AppError::Generic("Transaction has no inputs".to_string()));
        }

        if transaction.outputs.is_empty() {
            return Err(AppError::Generic("Transaction has no outputs".to_string()));
        }

        // Calculate and verify fee
        let fee_rate = self.calculate_fee_rate(transaction, size)?;
        if fee_rate < MIN_FEE_RATE {
            return Err(AppError::Generic(format!(
                "Fee rate too low: {} sat/byte (minimum: {})", 
                fee_rate, MIN_FEE_RATE
            )));
        }

        // TODO: Add more sophisticated validation:
        // - Verify signatures
        // - Check double-spending
        // - Validate input amounts
        // - Check locktime
        
        Ok(())
    }

    /// Calculate transaction hash
    fn calculate_transaction_hash(&self, transaction: &Transaction) -> AppResult<String> {
        use sha2::{Sha256, Digest};
        
        // Create a deterministic string representation of the transaction
        let tx_data = format!(
            "{}:{}:{}:{}",
            serde_json::to_string(&transaction.inputs).map_err(|e| AppError::Generic(e.to_string()))?,
            serde_json::to_string(&transaction.outputs).map_err(|e| AppError::Generic(e.to_string()))?,
            transaction.timestamp,
            transaction.fee
        );
        
        // Calculate SHA256 hash
        let mut hasher = Sha256::new();
        hasher.update(tx_data.as_bytes());
        let result = hasher.finalize();
        
        Ok(format!("{:x}", result))
    }

    /// Estimate transaction size in bytes
    fn estimate_transaction_size(&self, transaction: &Transaction) -> AppResult<usize> {
        // Simple estimation based on JSON serialization
        match serde_json::to_string(transaction) {
            Ok(json) => Ok(json.len()),
            Err(e) => Err(AppError::Generic(format!("Failed to serialize transaction: {}", e))),
        }
    }

    /// Calculate fee rate for transaction
    /// Calculate transaction size (simplified)
    fn calculate_transaction_size(&self, transaction: &Transaction) -> usize {
        // Simplified size calculation - in real implementation would properly serialize transaction
        let base_size = 10; // base transaction overhead
        let input_size = transaction.inputs.len() * 150; // ~150 bytes per input
        let output_size = transaction.outputs.len() * 34; // ~34 bytes per output
        base_size + input_size + output_size
    }

    /// Calculate transaction fee
    async fn calculate_transaction_fee(&self, transaction: &Transaction) -> AppResult<u64> {
        // Simple fee calculation - in real implementation would check input/output values
        let size = self.calculate_transaction_size(transaction);
        let base_fee = 1000; // 1000 satoshis base fee
        let size_fee = size as u64 * MIN_FEE_RATE;
        Ok(base_fee + size_fee)
    }

    fn calculate_fee_rate(&self, transaction: &Transaction, size: usize) -> AppResult<u64> {
        // Simple fee calculation - in real implementation would check input/output values
        let base_fee = 1000; // 1000 satoshis base fee
        let size_fee = size as u64 * MIN_FEE_RATE;
        let total_fee = base_fee + size_fee;
        
        Ok(total_fee / size as u64)
    }

    /// Find transaction dependencies
    async fn find_dependencies(&self, transaction: &Transaction) -> Vec<String> {
        let mut dependencies = Vec::new();
        let txs = self.transactions.read().await;

        for input in &transaction.inputs {
            // Use the correct field name for previous transaction ID
            let prev_txid = &input.previous_txid;
            if txs.contains_key(prev_txid) {
                dependencies.push(prev_txid.clone());
            }
        }

        dependencies
    }

    /// Check if transaction dependencies are satisfied
    fn dependencies_satisfied(&self, dependencies: &[String], selected: &[Transaction]) -> bool {
        for dep_txid in dependencies {
            if !selected.iter().any(|tx| tx.txid == *dep_txid) {
                return false;
            }
        }
        true
    }

    /// Evict low fee rate transactions to make room
    async fn evict_low_fee_transactions(&self, txs: &mut HashMap<String, MempoolTransaction>) {
        let evict_count = txs.len() / 10; // Evict 10% of transactions
        
        // Sort by fee rate (lowest first)
        let mut sorted_txids: Vec<_> = txs.iter()
            .map(|(txid, mempool_tx)| (txid.clone(), mempool_tx.fee_rate))
            .collect();
        sorted_txids.sort_by(|a, b| a.1.cmp(&b.1));

        for (txid, _) in sorted_txids.into_iter().take(evict_count) {
            txs.remove(&txid);
            warn!("Evicted transaction {} due to mempool size limit", txid);
        }
    }

    /// Emit mempool update event to frontend
    async fn emit_mempool_update(&self) {
        if let Some(app_handle) = &self.app_handle {
            let stats = self.get_stats().await;
            if let Err(e) = app_handle.emit("mempool-update", &stats) {
                warn!("Failed to emit mempool update: {}", e);
            }
        }
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// Replace a transaction with higher fee (Replace-By-Fee)
    pub async fn replace_transaction(
        &self,
        old_tx_hash: &str,
        new_transaction: Transaction,
        reason: ReplacementReason,
    ) -> AppResult<ReplacementResult> {
        let mut mempool_txs = self.transactions.write().await;
        
        // Find the old transaction
        let old_entry = mempool_txs.get(old_tx_hash)
            .ok_or_else(|| AppError::Generic("Transaction not found in mempool".to_string()))?
            .clone();
        
        // Validate the replacement
        self.validate_replacement(&old_entry.transaction, &new_transaction, &reason)?;
        
        // Calculate new fee
        let new_fee = self.calculate_transaction_fee(&new_transaction).await?;
        let fee_increase = new_fee.saturating_sub(old_entry.fee_rate * old_entry.size as u64);
        
        // Create new entry
        let new_tx_hash = new_transaction.txid.clone();
        let transaction_size = self.calculate_transaction_size(&new_transaction);
        let new_entry = MempoolTransaction {
            transaction: new_transaction.clone(),
            received_time: Self::current_timestamp(),
            fee_rate: new_fee / transaction_size as u64,
            size: transaction_size,
            dependencies: self.find_dependencies(&new_transaction).await,
        };
        
        // Add new transaction and remove old one
        mempool_txs.insert(new_tx_hash.clone(), new_entry);
        mempool_txs.remove(old_tx_hash);
        
        let result = ReplacementResult {
            success: true,
            old_tx_hash: old_tx_hash.to_string(),
            new_tx_hash: new_tx_hash.clone(),
            reason,
            old_fee: old_entry.fee_rate * old_entry.size as u64,
            new_fee,
            fee_increase,
        };
        
        info!("Transaction replaced: {} -> {} (fee: {} -> {})", 
              old_tx_hash, new_tx_hash, old_entry.fee_rate * old_entry.size as u64, new_fee);
        
        // Emit update
        self.emit_mempool_update().await;
        
        Ok(result)
    }
    
    /// Validate RBF replacement rules
    fn validate_replacement(
        &self,
        old_tx: &Transaction,
        new_tx: &Transaction,
        reason: &ReplacementReason,
    ) -> AppResult<()> {
        // Check if old transaction signals RBF or if this is a user request
        let old_signals_rbf = self.transaction_signals_rbf(old_tx);
        
        match reason {
            ReplacementReason::RbfFlag => {
                if !old_signals_rbf {
                    return Err(AppError::Generic(
                        "Original transaction does not signal RBF".to_string()
                    ));
                }
            }
            ReplacementReason::UserRequest => {
                // User requests are always allowed for their own transactions
            }
            ReplacementReason::HigherFee => {
                // Higher fee replacements require RBF signal
                if !old_signals_rbf {
                    return Err(AppError::Generic(
                        "Original transaction does not signal RBF for fee-based replacement".to_string()
                    ));
                }
            }
        }
        
        // Check that inputs are the same (prevents double spending)
        if old_tx.inputs.len() != new_tx.inputs.len() {
            return Err(AppError::Generic(
                "Replacement transaction must have same inputs".to_string()
            ));
        }
        
        for (old_input, new_input) in old_tx.inputs.iter().zip(new_tx.inputs.iter()) {
            if old_input.previous_txid != new_input.previous_txid {
                return Err(AppError::Generic(
                    "Replacement transaction must use same UTXOs".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Check if transaction signals Replace-By-Fee
    fn transaction_signals_rbf(&self, transaction: &Transaction) -> bool {
        // A transaction signals RBF if any input has sequence number < 0xfffffffe
        transaction.inputs.iter().any(|input| input.sequence < 0xfffffffe)
    }
    
    /// Get transactions that can be replaced
    pub async fn get_replaceable_transactions(&self) -> Vec<(String, Transaction, bool)> {
        let mempool_txs = self.transactions.read().await;
        
        mempool_txs
            .iter()
            .filter_map(|(tx_hash, entry)| {
                let signals_rbf = self.transaction_signals_rbf(&entry.transaction);
                if signals_rbf {
                    Some((
                        tx_hash.clone(),
                        entry.transaction.clone(),
                        signals_rbf,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Thread-safe wrapper for MempoolService
pub struct AsyncMempoolService {
    inner: Arc<RwLock<MempoolService>>,
}

impl AsyncMempoolService {
    /// Create new async mempool service
    pub fn new(blockchain_db: Arc<AsyncBlockchainDatabase>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(MempoolService::new(blockchain_db))),
        }
    }

    /// Initialize the service
    pub async fn initialize(&self, app_handle: AppHandle) -> AppResult<()> {
        let mut service = self.inner.write().await;
        service.initialize(app_handle).await
    }

    /// Add transaction to mempool
    pub async fn add_transaction(&self, transaction: Transaction) -> AppResult<String> {
        let service = self.inner.read().await;
        service.add_transaction(transaction).await
    }

    /// Remove transaction from mempool
    pub async fn remove_transaction(&self, txid: &str) -> Option<Transaction> {
        let service = self.inner.read().await;
        service.remove_transaction(txid).await
    }

    /// Get transaction from mempool
    pub async fn get_transaction(&self, txid: &str) -> Option<Transaction> {
        let service = self.inner.read().await;
        service.get_transaction(txid).await
    }

    /// Get transactions for mining
    pub async fn get_transactions_for_mining(&self, max_count: usize, max_size_bytes: usize) -> Vec<Transaction> {
        let service = self.inner.read().await;
        service.get_transactions_for_mining(max_count, max_size_bytes).await
    }

    /// Get all transactions
    pub async fn get_all_transactions(&self) -> Vec<Transaction> {
        let service = self.inner.read().await;
        service.get_all_transactions().await
    }

    /// Get mempool statistics
    pub async fn get_stats(&self) -> MempoolStats {
        let service = self.inner.read().await;
        service.get_stats().await
    }

    /// Get mempool info (alias for get_stats for command compatibility)
    pub async fn get_mempool_info(&self) -> AppResult<MempoolStats> {
        Ok(self.get_stats().await)
    }

    /// Clear mempool
    pub async fn clear(&self) -> AppResult<()> {
        let service = self.inner.read().await;
        service.clear().await;
        Ok(())
    }
    
    /// Replace a transaction with higher fee (Replace-By-Fee)
    pub async fn replace_transaction(
        &self,
        old_tx_hash: &str,
        new_transaction: Transaction,
        reason: ReplacementReason,
    ) -> AppResult<ReplacementResult> {
        let service = self.inner.read().await;
        service.replace_transaction(old_tx_hash, new_transaction, reason).await
    }
    
    /// Get transactions that can be replaced
    pub async fn get_replaceable_transactions(&self) -> Vec<(String, Transaction, bool)> {
        let service = self.inner.read().await;
        service.get_replaceable_transactions().await
    }
}

impl Clone for AsyncMempoolService {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
