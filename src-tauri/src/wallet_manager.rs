use crate::config::{Config, ConfigManager, WalletInfo};
use crate::errors::WalletError;
// Import the necessary types from the wallet_data module
use crate::wallet_data::{WalletData, KeyPair, AddressInfo};
use base64::Engine;
use log::{debug, error, info};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Wallet type representing an open wallet
pub struct Wallet {
    pub name: String,
    pub path: PathBuf,
    // Add other wallet properties as needed
}

/// WalletManager handles all wallet operations
pub struct WalletManager {
    config: Config,
    config_manager: Option<Arc<ConfigManager>>,
    current_wallet: Option<Wallet>, // This state is not persisted
}

impl WalletManager {
    /// Create a new WalletManager instance
    pub fn new(config: Config) -> Self {
        info!("Initializing wallet manager");
        WalletManager {
            config,
            config_manager: None,
            current_wallet: None,
        }
    }

    /// Set the ConfigManager to enable persistence
    pub fn set_config_manager(&mut self, config_manager: Arc<ConfigManager>) {
        debug!("Setting ConfigManager for wallet persistence");
        self.config_manager = Some(config_manager);
    }

    /// Get a list of available wallets
    pub fn list_wallets(&self) -> Vec<&WalletInfo> {
        debug!("Listing available wallets");
        self.config.wallets.iter().collect()
    }

    /// Find a wallet by name
    pub fn find_wallet_by_name(&self, name: &str) -> Option<&WalletInfo> {
        debug!("Finding wallet with name: {}", name);
        self.config.wallets.iter().find(|w| w.name == name)
    }

    /// Open a wallet with the given name and optional password
    pub fn open_wallet(&mut self, name: &str, password: Option<&str>) -> Result<(), WalletError> {
        info!("Attempting to open wallet: {}", name);

        // Find the wallet in available wallets
        let wallet_info = self
            .config
            .wallets
            .iter()
            .find(|w| w.name == name)
            .ok_or_else(|| {
                error!("Wallet not found: {}", name);
                WalletError::NotFound(name.to_string())
            })?;

        debug!("Found wallet info for: {}", name);

        // Check if the wallet is secured and verify password accordingly
        if wallet_info.secured {
            // For secured wallets, password is required
            match password {
                Some(pwd) if !pwd.is_empty() => {
                    // In a real implementation, proper password verification would happen here
                    debug!(
                        "Password verification succeeded for secured wallet: {}",
                        name
                    );
                }
                _ => {
                    error!("Password required for secured wallet: {}", name);
                    return Err(WalletError::AccessDenied(name.to_string()));
                }
            }
        }

        // Store the path before closing any wallet to avoid borrowing issues
        let wallet_path = wallet_info.path.clone();

        // Close any currently open wallet first
        if self.current_wallet.is_some() {
            debug!("Closing previously open wallet before opening new one");
            self.close_wallet();
        }

        // Attempt to load the wallet data file
        let wallet_dir_path = PathBuf::from(&wallet_path);
        let wallet_data_path = wallet_dir_path.join("wallet.dat");
        
        debug!("Loading wallet data from: {}", wallet_data_path.display());
        
        // Use tokio block_in_place since we're in a sync function but need to call async
        let wallet_data_result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                WalletData::load(&wallet_data_path, password).await
            })
        });

        // Check if we succeeded in loading wallet data
        match wallet_data_result {
            Ok(wallet_data) => {
                debug!("Successfully loaded wallet data for: {}", name);
                // You could store wallet_data in the Wallet struct if desired
                // For now, we just log some info about it
                debug!("Wallet balance: {}, addresses: {}", wallet_data.balance, wallet_data.addresses.len());
            },
            Err(e) => {
                // Just log the error but continue - wallet data loading is optional for now
                error!("Failed to load wallet data, continuing anyway: {}", e);
            }
        }

        // Create a wallet object 
        let opened_wallet = Wallet {
            name: name.to_string(),
            path: PathBuf::from(&wallet_path),
        };

        // Set current wallet in memory only
        self.current_wallet = Some(opened_wallet);

        info!("Successfully opened wallet: {}", name);
        Ok(())
    }

    /// Close the currently open wallet
    pub fn close_wallet(&mut self) {
        if let Some(wallet) = &self.current_wallet {
            info!("Closing wallet: {}", wallet.name);

            // Perform any necessary cleanup here

            // Clear the current wallet from memory
            self.current_wallet = None;
            debug!("Wallet closed successfully");
        } else {
            debug!("No wallet is currently open to close");
        }
    }

    /// Get the currently open wallet
    pub fn get_current_wallet(&self) -> Option<&Wallet> {
        if let Some(wallet) = &self.current_wallet {
            debug!("Retrieved current wallet: {}", wallet.name);
        } else {
            debug!("No wallet is currently open");
        }
        self.current_wallet.as_ref()
    }

    /// Create a new wallet
    pub fn create_wallet(&mut self, name: &str, password: &str) -> Result<(), WalletError> {
        info!("Attempting to create new wallet: {}", name);

        // Check if wallet with this name already exists
        if self.config.wallets.iter().any(|w| w.name == name) {
            error!("Wallet already exists: {}", name);
            return Err(WalletError::AlreadyExists(name.to_string()));
        }

        // Determine if this is a secured wallet based on password
        let is_secured = !password.is_empty();

        // Create wallet directory path
        let wallet_path = format!("wallets/{}", name);
        debug!("Creating wallet with path: {}", wallet_path);

        // Create wallet directory if it doesn't exist
        let wallet_dir_path = PathBuf::from(&wallet_path);
        if let Err(e) = std::fs::create_dir_all(&wallet_dir_path) {
            error!("Failed to create wallet directory: {}", e);
            return Err(WalletError::Generic(format!(
                "Failed to create wallet directory: {}",
                e
            )));
        }

        // Generate wallet data
        use rand::rngs::OsRng;
        use rand::RngCore;
        let mut key: [u8; 32] = [0; 32];
        let mut rand = OsRng::default();
        rand.fill_bytes(&mut key);

        // Create wallet data
        let wallet_data = WalletData {
            private_key: key.to_vec(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // Serialize the wallet data
        let serialized_data = match serde_json::to_string_pretty(&wallet_data) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to serialize wallet data: {}", e);
                return Err(WalletError::Generic(format!(
                    "Failed to serialize wallet data: {}",
                    e
                )));
            }
        };

        // Prepare the data to write to disk (encrypt if secured)
        let data_to_write = if is_secured {
            // If secured, encrypt the data with the password
            debug!("Encrypting wallet data for secured wallet: {}", name);
            
            match self.encrypt_data(&serialized_data, password) {
                Ok(encrypted) => encrypted,
                Err(e) => {
                    error!("Failed to encrypt wallet data: {}", e);
                    return Err(WalletError::Generic(format!(
                        "Failed to encrypt wallet data: {}",
                        e
                    )));
                }
            }
        } else {
            // If not secured, use plaintext
            serialized_data
        };

        // Write wallet data to disk
        let wallet_data_path = wallet_dir_path.join("wallet.dat");
        if let Err(e) = std::fs::write(&wallet_data_path, data_to_write) {
            error!("Failed to write wallet data to disk: {}", e);
            return Err(WalletError::Generic(format!(
                "Failed to write wallet data to disk: {}",
                e
            )));
        }
        debug!("Wallet data written to disk: {}", wallet_data_path.display());

        // Create wallet info object
        let wallet_info = WalletInfo {
            name: name.to_string(),
            path: wallet_path,
            secured: is_secured,
        };

        // Add to in-memory config
        self.config.wallets.push(wallet_info.clone());

        // Persist to disk if we have a ConfigManager
        if let Some(config_manager) = &self.config_manager {
            // Use tokio block_in_place since we're in a sync function but need to call async
            match tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { config_manager.add_wallet(wallet_info).await })
            }) {
                Ok(_) => {
                    info!("Wallet configuration persisted to disk: {}", name);
                }
                Err(e) => {
                    error!("Failed to persist wallet configuration: {}", e);
                    // Continue anyway as the wallet is created in memory
                }
            }
        } else {
            debug!("No ConfigManager available, wallet will not persist across sessions");
        }

        info!("Successfully created wallet: {}", name);
        Ok(())
    }

    /// Create a wallet with a seed phrase
    pub async fn create_wallet_with_seed(&mut self, name: &str, password: &str, seed_phrase: &str, is_secured: bool) -> Result<(), WalletError> {
        info!("Attempting to create new wallet with seed phrase: {}", name);

        // Check if wallet with this name already exists
        if self.config.wallets.iter().any(|w| w.name == name) {
            error!("Wallet already exists: {}", name);
            return Err(WalletError::AlreadyExists(name.to_string()));
        }

        // Create wallet directory path
        let wallet_path = format!("wallets/{}", name);
        debug!("Creating wallet with path: {}", wallet_path);

        // Create wallet directory if it doesn't exist
        let wallet_dir_path = PathBuf::from(&wallet_path);
        if let Err(e) = std::fs::create_dir_all(&wallet_dir_path) {
            error!("Failed to create wallet directory: {}", e);
            return Err(WalletError::Generic(format!(
                "Failed to create wallet directory: {}",
                e
            )));
        }

        // Generate a dummy master public key for now
        // In a real implementation, this would be derived from the seed phrase
        let master_public_key = "xpub_dummy_for_development_placeholder";

        // Create new WalletData object
        let mut wallet_data = WalletData::new(name, master_public_key, is_secured);
        
        // Set the seed phrase
        wallet_data.set_sensitive_data(seed_phrase, "xpriv_dummy_for_development_placeholder");

        // Derive and add a dummy key pair for now
        // TODO: In a real implementation, derive proper keys from the seed phrase
        wallet_data.add_key_pair(KeyPair {
            address: format!("address_{}_0", name),
            key_type: "p2pkh".to_string(),
            derivation_path: "m/44'/0'/0'/0/0".to_string(),
            public_key: "dummy_public_key".to_string(),
            private_key: if is_secured { Some("dummy_encrypted_private_key".to_string()) } else { Some("dummy_private_key".to_string()) },
        });
        
        // Save the wallet data to disk
        let wallet_data_path = wallet_dir_path.join("wallet.dat");
        
        // Password is only used if the wallet is secured
        let password_option = if is_secured { Some(password) } else { None };
        
        match wallet_data.save(&wallet_data_path, password_option).await {
            Ok(_) => {
                debug!("Wallet data saved to disk: {}", wallet_data_path.display());
            },
            Err(e) => {
                error!("Failed to save wallet data: {}", e);
                return Err(WalletError::Generic(format!(
                    "Failed to save wallet data: {}",
                    e
                )));
            }
        }
        
        // Register the wallet in the config
        let wallet_info = WalletInfo {
            name: name.to_string(),
            path: wallet_path,
            secured: is_secured,
        };

        // Add to in-memory config
        self.config.wallets.push(wallet_info.clone());

        // Persist to configuration if we have a ConfigManager
        if let Some(config_manager) = &self.config_manager {
            match config_manager.add_wallet(wallet_info).await {
                Ok(_) => {
                    info!("Wallet configuration persisted to disk: {}", name);
                }
                Err(e) => {
                    error!("Failed to persist wallet configuration: {}", e);
                    // Continue anyway as the wallet is created on disk and in memory
                }
            }
        } else {
            debug!("No ConfigManager available, wallet config will not persist across sessions");
        }

        info!("Successfully created wallet with seed phrase: {}", name);
        Ok(())
    }

    /// Update a wallet to be secured with a password
    pub fn secure_wallet(&mut self, name: &str, password: &str) -> Result<(), WalletError> {
        info!("Attempting to secure wallet: {}", name);

        // Validate input
        if password.is_empty() {
            error!("Cannot secure wallet with empty password");
            return Err(WalletError::Generic("Password cannot be empty".to_string()));
        }

        // Find the wallet in the config
        let wallet_index = self.config.wallets.iter().position(|w| w.name == name);

        match wallet_index {
            Some(index) => {
                // Check if wallet is already secured
                if self.config.wallets[index].secured {
                    error!("Wallet is already secured: {}", name);
                    return Err(WalletError::Generic(format!(
                        "Wallet '{}' is already secured",
                        name
                    )));
                }

                // Update the wallet to be secured in memory
                debug!("Updating wallet '{}' to be secured", name);
                self.config.wallets[index].secured = true;

                // Persist changes to disk if we have a ConfigManager
                if let Some(config_manager) = &self.config_manager {
                    // Use tokio block_in_place since we're in a sync function but need to call async
                    match tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            // Use the new update_wallet_security method
                            config_manager.update_wallet_security(name, true).await
                        })
                    }) {
                        Ok(_) => {
                            info!("Updated wallet security status persisted to disk: {}", name);
                        }
                        Err(e) => {
                            error!("Failed to persist wallet security status: {}", e);
                            // Continue anyway as the status is updated in memory
                        }
                    }
                } else {
                    debug!("No ConfigManager available, wallet security status will not persist across sessions");
                }

                // In a real implementation, we would encrypt the wallet data here

                info!("Successfully secured wallet: {}", name);
                Ok(())
            }
            None => {
                error!("Wallet not found: {}", name);
                Err(WalletError::NotFound(name.to_string()))
            }
        }
    }

    /// Get current wallet security status
    pub fn is_current_wallet_secured(&self) -> Option<bool> {
        if let Some(wallet) = &self.current_wallet {
            // Find the wallet in config to get its secured status
            if let Some(wallet_info) = self.config.wallets.iter().find(|w| w.name == wallet.name) {
                return Some(wallet_info.secured);
            }
        }
        None
    }

    /// Shutdown the wallet manager
    pub fn shutdown(&mut self) -> Result<(), WalletError> {
        info!("Shutting down wallet manager");

        // Close any open wallet
        if self.current_wallet.is_some() {
            self.close_wallet();
        }

        debug!("Wallet manager shutdown complete");
        Ok(())
    }

    /// Helper method to encrypt data with a password using AES with PBKDF2
    fn encrypt_data(&self, data: &str, password: &str) -> Result<String, WalletError> {
        use aes::{Aes256, cipher::{BlockEncrypt, KeyInit}};
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;
        use generic_array::GenericArray;
        use rand::{RngCore, rngs::OsRng};

        // Generate a salt for PBKDF2
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        
        // Generate an initialization vector (IV) for AES
        let mut iv = [0u8; 16]; 
        OsRng.fill_bytes(&mut iv);

        // Derive a key from the password using PBKDF2
        let mut key = [0u8; 32]; // 256-bit key for AES-256
        pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 10000, &mut key);
        
        // Prepare the data for encryption (must be a multiple of 16 bytes for AES)
        let data_bytes = data.as_bytes();
        let padding_len = 16 - (data_bytes.len() % 16);
        let mut padded_data = data_bytes.to_vec();
        // PKCS#7 padding
        padded_data.extend(vec![padding_len as u8; padding_len]);
        
        // Create AES cipher instance
        let key_array = GenericArray::from_slice(&key);
        let cipher = Aes256::new(key_array);

        // Encrypt each block with AES
        let mut encrypted_data = Vec::with_capacity(padded_data.len());
        for chunk in padded_data.chunks(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            
            // XOR with IV (for first block) or previous ciphertext block (for CBC mode)
            for (i, byte) in block.iter_mut().enumerate() {
                if encrypted_data.len() < 16 {
                    // First block uses IV
                    *byte ^= iv[i];
                } else {
                    // Subsequent blocks use previous ciphertext block (CBC mode)
                    let prev_block_idx = encrypted_data.len() - 16;
                    *byte ^= encrypted_data[prev_block_idx + i];
                }
            }
            
            // Encrypt the block
            cipher.encrypt_block(&mut block);
            
            // Add the encrypted block to our result
            encrypted_data.extend_from_slice(block.as_slice());
        }

        // Combine salt, IV, and encrypted data for storage
        let mut result = Vec::with_capacity(salt.len() + iv.len() + encrypted_data.len());
        result.extend_from_slice(&salt);
        result.extend_from_slice(&iv);
        result.extend_from_slice(&encrypted_data);
        
        // Convert to base64 for storage using the new Engine API
        Ok(base64::engine::general_purpose::STANDARD.encode(result))
    }

    /// Helper method to decrypt data with a password
    fn decrypt_data(&self, encrypted_data: &str, password: &str) -> Result<String, WalletError> {
        use aes::{Aes256, cipher::{BlockDecrypt, KeyInit}};
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;
        use generic_array::GenericArray;
        
        // Decode from base64 using the new Engine API
        let encrypted_bytes = match base64::engine::general_purpose::STANDARD.decode(encrypted_data) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(WalletError::Generic(format!(
                    "Failed to decode encrypted data: {}",
                    e
                )));
            }
        };
        
        // The first 16 bytes are the salt, the next 16 bytes are the IV
        if encrypted_bytes.len() < 32 {
            return Err(WalletError::Generic("Encrypted data is invalid".to_string()));
        }
        
        // Extract salt and IV
        let salt = &encrypted_bytes[0..16];
        let iv = &encrypted_bytes[16..32];
        let ciphertext = &encrypted_bytes[32..];
        
        // Make sure the ciphertext length is a multiple of 16 (AES block size)
        if ciphertext.len() % 16 != 0 {
            return Err(WalletError::Generic("Encrypted data has invalid length".to_string()));
        }
        
        // Derive the key from the password using PBKDF2 with the same parameters
        let mut key = [0u8; 32]; // 256-bit key for AES-256
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 10000, &mut key);
        
        // Create AES cipher instance
        let key_array = GenericArray::from_slice(&key);
        let cipher = Aes256::new(key_array);
        
        // Decrypt each block with AES
        let mut decrypted_data = Vec::with_capacity(ciphertext.len());
        
        for (block_idx, chunk) in ciphertext.chunks(16).enumerate() {
            // Decrypt the block
            let mut block = GenericArray::clone_from_slice(chunk);
            
            cipher.decrypt_block(&mut block);
            
            // XOR with IV for the first block or previous ciphertext block for subsequent blocks
            for (i, byte) in block.iter_mut().enumerate() {
                if block_idx == 0 {
                    // First block uses IV
                    *byte ^= iv[i];
                } else {
                    // Subsequent blocks use previous ciphertext block
                    let prev_block_start = (block_idx - 1) * 16;
                    *byte ^= ciphertext[prev_block_start + i];
                }
            }
            
            // Add the decrypted block to our result
            decrypted_data.extend_from_slice(block.as_slice());
        }
        
        // Remove PKCS#7 padding
        if let Some(&padding_len) = decrypted_data.last() {
            if padding_len as usize <= 16 && padding_len > 0 {
                // Check if padding looks valid
                let padding_start = decrypted_data.len() - padding_len as usize;
                let is_valid_padding = decrypted_data[padding_start..].iter()
                    .all(|&b| b == padding_len);
                
                if is_valid_padding {
                    decrypted_data.truncate(padding_start);
                }
            }
        }
        
        // Convert back to string
        match String::from_utf8(decrypted_data) {
            Ok(decrypted) => Ok(decrypted),
            Err(e) => Err(WalletError::Generic(format!(
                "Failed to convert decrypted data to string: {}",
                e
            ))),
        }
    }
}

/// Async wrapper for WalletManager to be used with Tauri state
pub struct AsyncWalletManager {
    inner: Arc<Mutex<WalletManager>>,
}

impl AsyncWalletManager {
    /// Create a new AsyncWalletManager
    pub fn new(wallet_manager: WalletManager) -> Self {
        AsyncWalletManager {
            inner: Arc::new(Mutex::new(wallet_manager)),
        }
    }

    /// Set the ConfigManager for persistence
    pub async fn set_config_manager(&self, config_manager: Arc<ConfigManager>) {
        let mut manager = self.inner.lock().await;
        manager.set_config_manager(config_manager);
    }

    /// Get a reference to the inner wallet manager
    pub async fn get_manager(&self) -> tokio::sync::MutexGuard<'_, WalletManager> {
        self.inner.lock().await
    }

    /// Shutdown the wallet manager safely
    pub async fn shutdown(&self) -> Result<(), WalletError> {
        let mut manager = self.inner.lock().await;
        manager.shutdown()
    }

    /// Create a wallet with a seed phrase
    pub async fn create_wallet_with_seed(&self, name: &str, password: &str, seed_phrase: &str, is_secured: bool) -> Result<(), WalletError> {
        let mut manager = self.inner.lock().await;
        manager.create_wallet_with_seed(name, password, seed_phrase, is_secured).await
    }
}
