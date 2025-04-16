use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use log::{debug, error, info};
use ring::pbkdf2;
use ring::aead::{self, Aad, BoundKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use std::num::NonZeroU32;
use std::fs;
use thiserror::Error;

/// Error type for wallet data operations
#[derive(Error, Debug)]
pub enum WalletDataError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    
    #[error("Invalid password")]
    InvalidPassword,
}

/// A transaction output that hasn't been spent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    /// Transaction ID
    pub txid: String,
    /// Output index
    pub vout: u32,
    /// Value in satoshis
    pub value: u64,
    /// Script public key
    pub script_pubkey: String,
    /// Address
    pub address: String,
    /// If this is a change output
    pub is_change: bool,
    /// Block height where this UTXO was confirmed (None if unconfirmed)
    pub height: Option<u32>,
}

/// A transaction with its details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction ID
    pub txid: String,
    /// Transaction version
    pub version: u32,
    /// Block height where this transaction was confirmed (None if unconfirmed)
    pub block_height: Option<u32>,
    /// Timestamp of the transaction
    pub timestamp: i64,
    /// Fee paid in satoshis
    pub fee: u64,
    /// Inputs
    pub inputs: Vec<TransactionInput>,
    /// Outputs
    pub outputs: Vec<TransactionOutput>,
    /// Transaction memo or note
    pub memo: Option<String>,
}

/// Transaction input data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    /// Previous transaction ID
    pub prev_txid: String,
    /// Previous output index
    pub prev_vout: u32,
    /// Value in satoshis
    pub value: u64,
    /// Address
    pub address: String,
    /// Sequence number
    pub sequence: u32,
}

/// Transaction output data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutput {
    /// Value in satoshis
    pub value: u64,
    /// Address
    pub address: String,
    /// Script public key
    pub script_pubkey: String,
    /// If this output belongs to the wallet
    pub is_mine: bool,
    /// If this is a change output
    pub is_change: bool,
}

/// Represents the type of key used
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyType {
    /// Standard
    Legacy,
    /// SegWit
    SegWit,
    /// Native SegWit
    NativeSegWit,
    /// Taproot
    Taproot,
}

/// Address with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    /// The address string
    pub address: String,
    /// Type of address/key
    pub key_type: KeyType,
    /// Path in BIP44 (e.g. m/44'/0'/0'/0/0)
    pub derivation_path: String,
    /// Address label
    pub label: Option<String>,
}

/// Key pair for a specific address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    /// The private key in WIF format
    pub private_key: String,
    /// The public key in hex format
    pub public_key: String,
    /// The address derived from the public key
    pub address: String,
    /// Type of key
    pub key_type: KeyType,
    /// Path in BIP44 (e.g. m/44'/0'/0'/0/0)
    pub derivation_path: String,
}

/// Core wallet data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    /// Wallet name/ID
    pub name: String,
    /// Creation time (Unix timestamp)
    pub created_at: i64,
    /// Last modified time (Unix timestamp)
    pub modified_at: i64,
    /// Last synced block height
    pub block_height: u32,
    /// BIP39 seed phrase (12/24 words), encrypted if wallet is secured
    pub seed_phrase: Option<String>,
    /// Master private key (xpriv), encrypted if wallet is secured
    pub master_private_key: Option<String>,
    /// Master public key (xpub)
    pub master_public_key: String,
    /// Key pairs in the wallet (address -> key pair)
    pub keys: HashMap<String, KeyPair>,
    /// Addresses in the wallet with metadata
    pub addresses: Vec<AddressInfo>,
    /// List of UTXOs
    pub utxos: Vec<Utxo>,
    /// Transaction history
    pub transactions: Vec<Transaction>,
    /// Current balance in satoshis
    pub balance: u64,
    /// Account indexes for BIP44 paths
    pub account_indexes: HashMap<u32, u32>,
    /// Is this wallet password protected
    pub is_encrypted: bool,
}

// Encryption related constants
const PBKDF2_ITERATIONS: u32 = 100_000; // Higher is more secure but slower
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32; // AES-256
const TAG_LEN: usize = 16; // GCM authentication tag

impl WalletData {
    /// Create a new wallet with default values
    pub fn new(name: &str, master_public_key: &str, is_encrypted: bool) -> Self {
        let now = chrono::Utc::now().timestamp();
        
        Self {
            name: name.to_string(),
            created_at: now,
            modified_at: now,
            block_height: 0,
            seed_phrase: None,
            master_private_key: None,
            master_public_key: master_public_key.to_string(),
            keys: HashMap::new(),
            addresses: Vec::new(),
            utxos: Vec::new(),
            transactions: Vec::new(),
            balance: 0,
            account_indexes: HashMap::new(),
            is_encrypted: is_encrypted,
        }
    }
    
    /// Set sensitive data for wallet (only for newly created wallets, before saving)
    pub fn set_sensitive_data(&mut self, seed_phrase: &str, master_private_key: &str) {
        self.seed_phrase = Some(seed_phrase.to_string());
        self.master_private_key = Some(master_private_key.to_string());
    }
    
    /// Add a new key pair to the wallet
    pub fn add_key_pair(&mut self, key_pair: KeyPair) {
        let address = key_pair.address.clone();
        
        // Add the address info
        self.addresses.push(AddressInfo {
            address: address.clone(),
            key_type: key_pair.key_type.clone(),
            derivation_path: key_pair.derivation_path.clone(),
            label: None,
        });
        
        // Add the key pair
        self.keys.insert(address, key_pair);
        
        // Update modified time
        self.modified_at = chrono::Utc::now().timestamp();
    }
    
    /// Save wallet data to file, encrypting if necessary
    pub fn save(&self, path: &PathBuf, password: Option<&str>) -> Result<(), WalletDataError> {
        let serialized = serde_json::to_string_pretty(&self)?;
        
        // If wallet is encrypted but no password provided, return error
        if self.is_encrypted && password.is_none() {
            return Err(WalletDataError::EncryptionError(
                "Password required for encrypted wallet".to_string()
            ));
        }
        
        // If the wallet is encrypted, encrypt the data
        let file_data = if self.is_encrypted {
            let password = password.unwrap(); // Safe because we checked above
            self.encrypt_data(&serialized, password)?
        } else {
            serialized.into_bytes()
        };
        
        // Create directory if it doesn't exist
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        
        // Write the data to file
        fs::write(path, file_data)?;
        info!("Wallet data saved to {}", path.display());
        
        Ok(())
    }
    
    /// Load wallet data from file
    pub fn load(path: &PathBuf, password: Option<&str>) -> Result<Self, WalletDataError> {
        info!("Loading wallet data from {}", path.display());
        
        // Read the file
        let file_data = fs::read(path)?;
        
        // Try to parse as JSON first (unencrypted wallet)
        match serde_json::from_slice::<WalletData>(&file_data) {
            Ok(wallet) => {
                // If the wallet is encrypted but no password provided, return error
                if wallet.is_encrypted && password.is_none() {
                    return Err(WalletDataError::DecryptionError(
                        "Password required for encrypted wallet".to_string()
                    ));
                }
                
                // If the wallet isn't encrypted, we're done
                if !wallet.is_encrypted {
                    return Ok(wallet);
                }
                
                // Otherwise, we parsed an encrypted wallet as unencrypted JSON, which should never happen
                return Err(WalletDataError::DecryptionError(
                    "Wallet is marked as encrypted but data is not encrypted".to_string()
                ));
            },
            Err(_) => {
                // If we can't parse as JSON, assume it's encrypted
                if password.is_none() {
                    return Err(WalletDataError::DecryptionError(
                        "Password required for encrypted wallet".to_string()
                    ));
                }
                
                // Try to decrypt
                let password = password.unwrap(); // Safe because we checked above
                let decrypted_data = Self::decrypt_data(&file_data, password)?;
                
                // Parse the decrypted data
                let wallet = serde_json::from_str(&decrypted_data)?;
                Ok(wallet)
            }
        }
    }
    
    /// Encrypt data using password-based AES-256-GCM
    fn encrypt_data(&self, data: &str, password: &str) -> Result<Vec<u8>, WalletDataError> {
        let rand = SystemRandom::new();
        
        // Generate a random salt for PBKDF2
        let mut salt = [0u8; SALT_LEN];
        rand.fill(&mut salt)
            .map_err(|_| WalletDataError::EncryptionError("Failed to generate salt".to_string()))?;
        
        // Generate a random nonce
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand.fill(&mut nonce_bytes)
            .map_err(|_| WalletDataError::EncryptionError("Failed to generate nonce".to_string()))?;
        
        // Derive encryption key from password using PBKDF2
        let mut key_bytes = [0u8; KEY_LEN];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
            &salt,
            password.as_bytes(),
            &mut key_bytes,
        );
        
        // Set up AES-GCM for encryption
        let unbound_key = UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
            .map_err(|_| WalletDataError::EncryptionError("Failed to create encryption key".to_string()))?;
        
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut sealing_key = aead::SealingKey::new(unbound_key, nonce);
        
        // Encrypt the data
        let mut in_out = data.as_bytes().to_vec();
        let tag = sealing_key.seal_in_place_separate_tag(Aad::empty(), &mut in_out)
            .map_err(|_| WalletDataError::EncryptionError("Failed to encrypt data".to_string()))?;
        
        // Construct the final output: salt + nonce + ciphertext + tag
        let mut result = Vec::with_capacity(SALT_LEN + NONCE_LEN + in_out.len() + TAG_LEN);
        result.extend_from_slice(&salt);
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&in_out);
        result.extend_from_slice(tag.as_ref());
        
        Ok(result)
    }
    
    /// Decrypt data using password-based AES-256-GCM
    fn decrypt_data(encrypted_data: &[u8], password: &str) -> Result<String, WalletDataError> {
        // Check if the data is large enough to contain all components
        if encrypted_data.len() < SALT_LEN + NONCE_LEN + TAG_LEN {
            return Err(WalletDataError::DecryptionError("Encrypted data is too short".to_string()));
        }
        
        // Extract components
        let salt = &encrypted_data[0..SALT_LEN];
        let nonce_bytes = &encrypted_data[SALT_LEN..(SALT_LEN + NONCE_LEN)];
        let ciphertext_with_tag = &encrypted_data[(SALT_LEN + NONCE_LEN)..];
        
        // The tag is at the end of the ciphertext
        let ciphertext_len = ciphertext_with_tag.len() - TAG_LEN;
        let (ciphertext, tag) = ciphertext_with_tag.split_at(ciphertext_len);
        
        // Derive decryption key from password using PBKDF2
        let mut key_bytes = [0u8; KEY_LEN];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
            salt,
            password.as_bytes(),
            &mut key_bytes,
        );
        
        // Set up AES-GCM for decryption
        let unbound_key = UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
            .map_err(|_| WalletDataError::DecryptionError("Failed to create decryption key".to_string()))?;
        
        let mut nonce_array = [0u8; NONCE_LEN];
        nonce_array.copy_from_slice(nonce_bytes);
        let nonce = Nonce::assume_unique_for_key(nonce_array);
        
        let mut opening_key = aead::OpeningKey::new(unbound_key, nonce);
        
        // Combine ciphertext and tag for decryption
        let mut ciphertext_and_tag = ciphertext.to_vec();
        ciphertext_and_tag.extend_from_slice(tag);
        
        // Decrypt
        let plaintext = opening_key
            .open_in_place(Aad::empty(), &mut ciphertext_and_tag)
            .map_err(|_| WalletDataError::InvalidPassword)?;
        
        // Convert to string
        let plaintext_str = String::from_utf8(plaintext.to_vec())
            .map_err(|_| WalletDataError::DecryptionError("Invalid UTF-8 in decrypted data".to_string()))?;
        
        Ok(plaintext_str)
    }
    
    /// Calculate the current balance from UTXOs
    pub fn calculate_balance(&mut self) -> u64 {
        let balance = self.utxos.iter().map(|utxo| utxo.value).sum();
        self.balance = balance;
        balance
    }
    
    /// Get a list of all addresses in the wallet
    pub fn get_addresses(&self) -> Vec<String> {
        self.addresses.iter().map(|addr| addr.address.clone()).collect()
    }
    
    /// Add a UTXO to the wallet
    pub fn add_utxo(&mut self, utxo: Utxo) {
        // Check if the UTXO already exists
        if !self.utxos.iter().any(|u| u.txid == utxo.txid && u.vout == utxo.vout) {
            self.utxos.push(utxo);
            self.balance = self.calculate_balance();
            self.modified_at = chrono::Utc::now().timestamp();
        }
    }
    
    /// Remove a spent UTXO from the wallet
    pub fn remove_utxo(&mut self, txid: &str, vout: u32) {
        self.utxos.retain(|utxo| !(utxo.txid == txid && utxo.vout == vout));
        self.balance = self.calculate_balance();
        self.modified_at = chrono::Utc::now().timestamp();
    }
    
    /// Add a transaction to the history
    pub fn add_transaction(&mut self, tx: Transaction) {
        // Check if transaction already exists
        if !self.transactions.iter().any(|t| t.txid == tx.txid) {
            self.transactions.push(tx);
            self.modified_at = chrono::Utc::now().timestamp();
        }
    }
}
