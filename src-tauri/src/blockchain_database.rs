use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sled::{Db, Tree};
use tokio::sync::RwLock;
use log::info;

use bincode::{Decode, Encode};

/// Block data structure
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Block {
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub nonce: u64,
    pub difficulty: u64,
    pub transactions: Vec<Transaction>,
    pub merkle_root: String,
}

/// Transaction data structure
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Transaction {
    pub txid: String,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub timestamp: u64,
    pub fee: u64,
}

/// Transaction input
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct TransactionInput {
    pub previous_txid: String,
    pub previous_output_index: u32,
    pub script_sig: String,
    pub sequence: u32,
}

/// Transaction output
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct TransactionOutput {
    pub value: u64,
    pub script_pubkey: String,
    pub address: String,
}

/// UTXO (Unspent Transaction Output)
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct UTXO {
    pub txid: String,
    pub output_index: u32,
    pub value: u64,
    pub script_pubkey: String,
    pub address: String,
    pub block_height: u64,
}

/// Blockchain database service using Sled
pub struct BlockchainDatabase {
    db: Db,
    blocks: Tree,
    transactions: Tree,
    utxos: Tree,
    addresses: Tree,
    metadata: Tree,
}

impl BlockchainDatabase {    /// Create new blockchain database
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let db_path = data_dir.join("blockchain.db");
        
        println!("Initializing blockchain database at: {:?}", db_path);
        
        // Ensure the data directory exists
        if let Some(parent) = db_path.parent() {
            println!("Creating directory: {:?}", parent);
            match std::fs::create_dir_all(parent) {
                Ok(_) => println!("Directory created successfully"),
                Err(e) => {
                    println!("Failed to create directory: {}", e);
                    return Err(anyhow::anyhow!("Failed to create blockchain data directory: {}", e));
                }
            }
        }

        println!("Opening sled database...");
        let db = match sled::open(&db_path) {
            Ok(db) => {
                println!("Sled database opened successfully");
                db
            },
            Err(e) => {
                println!("Failed to open sled database: {}", e);
                println!("Attempted path: {:?}", db_path);
                println!("Path exists: {}", db_path.exists());
                println!("Parent exists: {}", db_path.parent().map(|p| p.exists()).unwrap_or(false));
                return Err(anyhow::anyhow!("Failed to open blockchain database: {}", e));
            }
        };

        println!("Opening database trees...");
        let blocks = db.open_tree("blocks")
            .context("Failed to open blocks tree")?;
        let transactions = db.open_tree("transactions")
            .context("Failed to open transactions tree")?;
        let utxos = db.open_tree("utxos")
            .context("Failed to open UTXOs tree")?;
        let addresses = db.open_tree("addresses")
            .context("Failed to open addresses tree")?;
        let metadata = db.open_tree("metadata")
            .context("Failed to open metadata tree")?;
        println!("All database trees opened successfully");

        Ok(Self {
            db,
            blocks,
            transactions,
            utxos,
            addresses,
            metadata,
        })
    }

    /// Get the current block height
    pub fn get_block_height(&self) -> Result<u64> {
        if let Some(height_bytes) = self.metadata.get("block_height")? {            let height = bincode::decode_from_slice(&height_bytes, bincode::config::standard())?.0;
            Ok(height)
        } else {
            Ok(0)
        }
    }

    /// Set the current block height
    pub fn set_block_height(&self, height: u64) -> Result<()> {        let height_bytes = bincode::encode_to_vec(&height, bincode::config::standard())?;
        self.metadata.insert("block_height", height_bytes)?;
        self.db.flush()?;
        Ok(())
    }

    /// Store a block in the database
    pub fn store_block(&self, block: &Block) -> Result<()> {
        let block_key = format!("height_{}", block.height);        let block_bytes = bincode::encode_to_vec(block, bincode::config::standard())?;
        
        self.blocks.insert(block_key.as_bytes(), block_bytes)?;
        
        // Store by hash as well for quick lookup
        let hash_key = format!("hash_{}", block.hash);
        self.blocks.insert(hash_key.as_bytes(), bincode::encode_to_vec(&block.height, bincode::config::standard())?)?;
        
        // Update block height if this is the newest block
        let current_height = self.get_block_height()?;
        if block.height > current_height {
            self.set_block_height(block.height)?;
        }

        // Store transactions from this block
        for transaction in &block.transactions {
            self.store_transaction(transaction, block.height)?;
        }

        self.db.flush()?;
        Ok(())
    }

    /// Get a block by height
    pub fn get_block_by_height(&self, height: u64) -> Result<Option<Block>> {
        let block_key = format!("height_{}", height);
        if let Some(block_bytes) = self.blocks.get(block_key.as_bytes())? {            let block = bincode::decode_from_slice(&block_bytes, bincode::config::standard())?.0;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    /// Get a block by hash
    pub fn get_block_by_hash(&self, hash: &str) -> Result<Option<Block>> {
        let hash_key = format!("hash_{}", hash);
        if let Some(height_bytes) = self.blocks.get(hash_key.as_bytes())? {            let height: u64 = bincode::decode_from_slice(&height_bytes, bincode::config::standard())?.0;
            self.get_block_by_height(height)
        } else {
            Ok(None)
        }
    }

    /// Store a transaction
    pub fn store_transaction(&self, transaction: &Transaction, block_height: u64) -> Result<()> {        let tx_bytes = bincode::encode_to_vec(transaction, bincode::config::standard())?;
        
        self.transactions.insert(transaction.txid.as_bytes(), tx_bytes)?;

        // Update UTXOs
        self.update_utxos(transaction, block_height)?;

        Ok(())
    }

    /// Get a transaction by ID
    pub fn get_transaction(&self, txid: &str) -> Result<Option<Transaction>> {
        if let Some(tx_bytes) = self.transactions.get(txid.as_bytes())? {            let transaction = bincode::decode_from_slice(&tx_bytes, bincode::config::standard())?.0;
            Ok(Some(transaction))
        } else {
            Ok(None)
        }
    }

    /// Update UTXOs based on a transaction
    fn update_utxos(&self, transaction: &Transaction, block_height: u64) -> Result<()> {
        // Remove spent UTXOs
        for input in &transaction.inputs {
            let utxo_key = format!("{}:{}", input.previous_txid, input.previous_output_index);
            self.utxos.remove(utxo_key.as_bytes())?;
        }

        // Add new UTXOs
        for (index, output) in transaction.outputs.iter().enumerate() {
            let utxo = UTXO {
                txid: transaction.txid.clone(),
                output_index: index as u32,
                value: output.value,
                script_pubkey: output.script_pubkey.clone(),
                address: output.address.clone(),
                block_height,
            };

            let utxo_key = format!("{}:{}", transaction.txid, index);            let utxo_bytes = bincode::encode_to_vec(&utxo, bincode::config::standard())?;
            
            self.utxos.insert(utxo_key.as_bytes(), utxo_bytes)?;

            // Index by address
            self.add_address_utxo(&output.address, &utxo_key)?;
        }

        Ok(())
    }

    /// Add UTXO to address index
    fn add_address_utxo(&self, address: &str, utxo_key: &str) -> Result<()> {
        let address_key = format!("addr_{}", address);
          let mut utxo_list: Vec<String> = if let Some(list_bytes) = self.addresses.get(address_key.as_bytes())? {
            bincode::decode_from_slice(&list_bytes, bincode::config::standard())?.0
        } else {
            Vec::new()
        };

        if !utxo_list.contains(&utxo_key.to_string()) {
            utxo_list.push(utxo_key.to_string());
            let list_bytes = bincode::encode_to_vec(&utxo_list, bincode::config::standard())?;
            self.addresses.insert(address_key.as_bytes(), list_bytes)?;
        }

        Ok(())
    }

    /// Get UTXOs for an address
    pub fn get_address_utxos(&self, address: &str) -> Result<Vec<UTXO>> {
        let address_key = format!("addr_{}", address);
        let mut utxos = Vec::new();        if let Some(list_bytes) = self.addresses.get(address_key.as_bytes())? {
            let utxo_keys: Vec<String> = bincode::decode_from_slice(&list_bytes, bincode::config::standard())?.0;

            for utxo_key in utxo_keys {
                if let Some(utxo_bytes) = self.utxos.get(utxo_key.as_bytes())? {
                    let utxo = bincode::decode_from_slice(&utxo_bytes, bincode::config::standard())?.0;
                    utxos.push(utxo);
                }
            }
        }

        Ok(utxos)
    }

    /// Get balance for an address
    pub fn get_address_balance(&self, address: &str) -> Result<u64> {
        let utxos = self.get_address_utxos(address)?;
        let balance = utxos.iter().map(|utxo| utxo.value).sum();
        Ok(balance)
    }

    /// Check if a UTXO exists and is unspent
    pub fn is_utxo_unspent(&self, txid: &str, output_index: u32) -> Result<bool> {
        let utxo_key = format!("{}:{}", txid, output_index);
        Ok(self.utxos.contains_key(utxo_key.as_bytes())?)
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<HashMap<String, u64>> {
        let mut stats = HashMap::new();
        
        stats.insert("block_height".to_string(), self.get_block_height()?);
        stats.insert("blocks_count".to_string(), self.blocks.len() as u64 / 2); // Divided by 2 because we store by height and hash
        stats.insert("transactions_count".to_string(), self.transactions.len() as u64);
        stats.insert("utxos_count".to_string(), self.utxos.len() as u64);
        
        Ok(stats)
    }

    /// Flush all pending writes to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    /// Close the database and release all resources
    /// This flushes pending writes and ensures data integrity
    pub fn close(&self) -> Result<()> {
        info!("Releasing blockchain database resources");
        
        // Flush all pending writes to ensure data integrity
        self.db.flush()
            .context("Failed to flush database before closing")?;
        
        info!("Blockchain database resources released successfully");
        Ok(())
    }

    // ...existing code...
}

/// Thread-safe wrapper for BlockchainDatabase
pub struct AsyncBlockchainDatabase {
    inner: Arc<RwLock<BlockchainDatabase>>,
}

impl AsyncBlockchainDatabase {
    /// Create new async blockchain database
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        let db = BlockchainDatabase::new(data_dir)?;
        Ok(Self {
            inner: Arc::new(RwLock::new(db)),
        })
    }

    /// Get the current block height
    pub async fn get_block_height(&self) -> Result<u64> {
        let db = self.inner.read().await;
        db.get_block_height()
    }

    /// Store a block
    pub async fn store_block(&self, block: &Block) -> Result<()> {
        let db = self.inner.write().await;
        db.store_block(block)
    }

    /// Get a block by height
    pub async fn get_block_by_height(&self, height: u64) -> Result<Option<Block>> {
        let db = self.inner.read().await;
        db.get_block_by_height(height)
    }

    /// Get a block by hash
    pub async fn get_block_by_hash(&self, hash: &str) -> Result<Option<Block>> {
        let db = self.inner.read().await;
        db.get_block_by_hash(hash)
    }

    /// Get a transaction by ID
    pub async fn get_transaction(&self, txid: &str) -> Result<Option<Transaction>> {
        let db = self.inner.read().await;
        db.get_transaction(txid)
    }

    /// Get UTXOs for an address
    pub async fn get_address_utxos(&self, address: &str) -> Result<Vec<UTXO>> {
        let db = self.inner.read().await;
        db.get_address_utxos(address)
    }

    /// Get balance for an address
    pub async fn get_address_balance(&self, address: &str) -> Result<u64> {
        let db = self.inner.read().await;
        db.get_address_balance(address)
    }

    /// Check if a UTXO exists and is unspent
    pub async fn is_utxo_unspent(&self, txid: &str, output_index: u32) -> Result<bool> {
        let db = self.inner.read().await;
        db.is_utxo_unspent(txid, output_index)
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<HashMap<String, u64>> {
        let db = self.inner.read().await;
        db.get_stats()
    }    /// Flush all pending writes to disk
    pub async fn flush(&self) -> Result<()> {
        let db = self.inner.read().await;
        db.flush()
    }    /// Close the database and release all resources
    pub async fn close(&self) -> Result<()> {
        info!("Closing async blockchain database and releasing resources");
        let db = self.inner.read().await;
        db.close()
    }

    /// Populate blockchain with test data using wallet addresses
    pub async fn populate_test_data(&self, wallet_addresses: Vec<String>) -> Result<()> {
        if wallet_addresses.is_empty() {
            info!("No wallet addresses provided, skipping test data generation");
            return Ok(());
        }

        info!("Populating blockchain with test data using {} wallet addresses", wallet_addresses.len());
        
        let current_height = self.get_block_height().await?;
        if current_height > 0 {
            info!("Blockchain already has data (height: {}), skipping test data generation", current_height);
            return Ok(());
        }

        let db = self.inner.write().await;
        
        // Create some test blocks with transactions to wallet addresses
        for block_height in 1u64..=10u64 {
            let mut transactions = Vec::new();
            
            // Create 2 transactions per block
            for tx_index in 0..2 {
                let txid = format!("tx_{}_{}", block_height, tx_index);
                
                // For the first transaction in early blocks, create coinbase (no inputs)
                let inputs = if block_height <= 3 && tx_index == 0 {
                    vec![] // Coinbase transaction
                } else {
                    vec![TransactionInput {
                        previous_txid: format!("tx_{}_{}", block_height - 1, 0),
                        previous_output_index: 0,
                        script_sig: "signature_placeholder".to_string(),
                        sequence: 0xffffffff,
                    }]
                };

                // Create outputs to wallet addresses
                let mut outputs = Vec::new();
                let addresses_to_use = std::cmp::min(wallet_addresses.len(), 2); // Use up to 2 addresses per transaction
                
                for i in 0..addresses_to_use {
                    let address = &wallet_addresses[i % wallet_addresses.len()];
                    let value = 1000000 + (block_height * 100000) + (tx_index * 10000) + (i as u64 * 5000); // Varying amounts
                    
                    outputs.push(TransactionOutput {
                        value,
                        script_pubkey: format!("OP_DUP OP_HASH160 {} OP_EQUALVERIFY OP_CHECKSIG", address),
                        address: address.clone(),
                    });
                }

                let transaction = Transaction {
                    txid: txid.clone(),
                    inputs,
                    outputs,
                    timestamp: 1640995200 + (block_height * 600), // ~10 minutes per block
                    fee: 1000,
                };

                transactions.push(transaction);
            }

            // Create the block
            let block = Block {
                height: block_height,
                hash: format!("block_hash_{}", block_height),
                previous_hash: if block_height == 1 { 
                    "genesis".to_string() 
                } else { 
                    format!("block_hash_{}", block_height - 1) 
                },
                timestamp: 1640995200 + (block_height * 600),
                nonce: block_height * 12345,
                difficulty: 1000,
                transactions,
                merkle_root: format!("merkle_root_{}", block_height),
            };

            // Store the block and its transactions
            db.store_block(&block)?;
            info!("Created test block {} with {} transactions", block_height, block.transactions.len());
        }

        info!("Test blockchain data populated successfully");
        Ok(())
    }
}
