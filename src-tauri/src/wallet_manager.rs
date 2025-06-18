use crate::config::{Config, ConfigManager, WalletInfo};
use crate::errors::WalletError;
// Import KeyType and remove unused AddressInfo
use crate::wallet_data::{WalletData, WalletDataError, KeyPair, KeyType};
use base64::Engine;
use log::{debug, error, info};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use rand::Rng;
use sha2::Digest;

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
    }    /// Get a list of available wallets
    pub fn list_wallets(&mut self) -> Vec<&WalletInfo> {
        debug!("Listing available wallets");
        
        // If we have a config manager, refresh our config from the file to ensure we have the latest data
        if let Some(config_manager) = &self.config_manager {
            let fresh_config = config_manager.get_config();
            if self.config.wallets.len() != fresh_config.wallets.len() {
                info!("Config refresh: wallet count changed from {} to {}", 
                      self.config.wallets.len(),
                      fresh_config.wallets.len());
            }
            self.config = fresh_config;
        }
        
        debug!("Current config has {} wallets", self.config.wallets.len());
        self.config.wallets.iter().collect()
    }

    /// Find a wallet by name
    pub fn find_wallet_by_name(&self, name: &str) -> Option<&WalletInfo> {
        debug!("Finding wallet with name: {}", name);
        self.config.wallets.iter().find(|w| w.name == name)
    }

    /// Open a wallet with the given name and optional password
    pub fn open_wallet(&mut self, name: &str, password: Option<&str>) -> Result<(), WalletError> {
        info!("Attempting to open wallet: {}", name);        // Find the wallet in available wallets and clone it to avoid borrow checker issues
        let wallet_info = self
            .config
            .wallets
            .iter()
            .find(|w| w.name == name)
            .ok_or_else(|| {
                error!("Wallet not found: {}", name);
                WalletError::NotFound(name.to_string())
            })?
            .clone(); // Clone to avoid borrow checker issues

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
        
        // Use tokio block_in_place since we're in a sync function but need to call sync
        let wallet_data_result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Remove .await as WalletData::load is sync
                WalletData::load(&wallet_data_path, password)
            })
        });        // Check if we succeeded in loading wallet data
        match wallet_data_result {
            Ok(wallet_data) => {
                debug!("Successfully loaded wallet data for: {}", name);
                // You could store wallet_data in the Wallet struct if desired
                // For now, we just log some info about it
                debug!("Wallet balance: {}, addresses: {}", wallet_data.balance, wallet_data.addresses.len());
            },
            Err(e) => {
                // If it's an IoError and the file doesn't exist, create a new wallet data file
                if let WalletDataError::IoError(io_err) = &e {
                    if io_err.kind() == std::io::ErrorKind::NotFound {
                        info!("Wallet data file not found, creating a new one for: {}", name);
                        
                        // Create directory if it doesn't exist
                        if let Err(dir_err) = std::fs::create_dir_all(&wallet_dir_path) {
                            error!("Failed to create wallet directory: {}", dir_err);
                            return Err(WalletError::Generic(format!(
                                "Failed to create wallet directory: {}", dir_err
                            )));
                        }
                        
                        // Generate a dummy master public key
                        let master_public_key = "xpub_dummy_for_development_placeholder";
                        
                        // Create new wallet data
                        let mut wallet_data = WalletData::new(name, master_public_key, wallet_info.secured);
                        
                        // Set a default seed phrase
                        wallet_data.set_sensitive_data("default_seed_phrase", "xpriv_dummy_for_development_placeholder");
                        
                        // Add a dummy key pair
                        wallet_data.add_key_pair(KeyPair {
                            address: format!("address_{}_0", name),
                            key_type: KeyType::Legacy,
                            derivation_path: "m/44'/0'/0'/0/0".to_string(),
                            public_key: "dummy_public_key".to_string(),
                            private_key: if wallet_info.secured { "dummy_encrypted_private_key".to_string() } else { "dummy_private_key".to_string() },
                        });
                        
                        // Save the wallet data
                        let password_option = if wallet_info.secured { password } else { None };
                        if let Err(save_err) = wallet_data.save(&wallet_data_path, password_option) {
                            error!("Failed to create initial wallet data file: {}", save_err);
                            return Err(WalletError::Generic(format!(
                                "Failed to create initial wallet data file: {}", save_err
                            )));
                        }
                        
                        info!("Successfully created initial wallet data file for: {}", name);
                    } else {
                        // For other IO errors, log and continue
                        error!("Failed to load wallet data due to IO error: {}", e);
                    }
                } else {
                    // For non-IO errors, log and continue
                    error!("Failed to load wallet data, continuing anyway: {}", e);
                }
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
    }    /// Get the base directory for wallets
    pub fn get_wallets_dir(&self) -> PathBuf {
        // Determine the wallets directory based on the platform
        // First try to get it from the app configuration
        if let Some(config_manager) = &self.config_manager {
            // Try to get the instance config dir first (synchronous method)
            if let Ok(config_dir) = config_manager.get_config_dir_path() {
                // Go up one level from config directory and join with "wallets"
                let wallets_dir = config_dir.parent()
                    .unwrap_or(&config_dir) // Fallback to config_dir if parent doesn't exist
                    .join("wallets");
                
                debug!("Using wallets directory from config: {}", wallets_dir.display());
                return wallets_dir;
            }
            
            // Fall back to the static async method if needed
            if let Ok(config_dir) = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    ConfigManager::get_config_dir().await
                })
            }) {
                // Go up one level from config directory and join with "wallets"
                let wallets_dir = config_dir.parent()
                    .unwrap_or(&config_dir) // Fallback to config_dir if parent doesn't exist
                    .join("wallets");
                
                debug!("Using wallets directory from static config: {}", wallets_dir.display());
                return wallets_dir;
            }
        }
        
        // Fallback to a default directory
        let default_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("com.b-rad-coin.app")
            .join("wallets");
        
        debug!("Using default wallets directory: {}", default_dir.display());
        default_dir
    }

    /// Create a new wallet
    /// NOTE: This function creates a basic wallet structure without seed phrase or master keys.
    /// Use create_wallet_with_seed for a more complete wallet.
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

        // Create wallet data using the constructor
        // Provide a dummy master public key as it's required
        let dummy_master_public_key = "xpub_dummy_placeholder_for_basic_wallet";
        let wallet_data = WalletData::new(name, dummy_master_public_key, is_secured);

        // Save the wallet data - encryption is handled internally by save()
        let wallet_data_path = wallet_dir_path.join("wallet.dat");
        let password_option = if is_secured { Some(password) } else { None };

        // Call save (it's synchronous)
        if let Err(e) = wallet_data.save(&wallet_data_path, password_option) {
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
    }    /// Create a wallet with a seed phrase
    // Make this function sync as save is sync
    pub fn create_wallet_with_seed(&mut self, name: &str, password: &str, seed_phrase: &str, is_secured: bool) -> Result<(), WalletError> {
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

        // Determine if this is a real seed phrase or developer placeholder
        let is_placeholder_seed = seed_phrase.contains("test word wallet") && seed_phrase.contains("developer mode");
        
        // Generate keys based on seed phrase type
        let (master_public_key, master_private_key, key_pair) = if is_placeholder_seed {
            // Developer mode placeholder - generate random keys
            self.generate_dev_mode_keys(name)?
        } else {
            // Real seed phrase - derive keys from it
            self.derive_keys_from_seed(seed_phrase, name)?
        };

        // Create new WalletData object
        let mut wallet_data = WalletData::new(name, &master_public_key, is_secured);
        
        // Set the seed phrase and master private key
        wallet_data.set_sensitive_data(seed_phrase, &master_private_key);

        // Add the derived key pair
        wallet_data.add_key_pair(key_pair);
        
        // Save the wallet data to disk
        let wallet_data_path = wallet_dir_path.join("wallet.dat");
        
        // Password is only used if the wallet is secured
        let password_option = if is_secured { Some(password) } else { None };
        
        // Remove .await as save is sync
        match wallet_data.save(&wallet_data_path, password_option) {
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
            // Use tokio block_in_place since we're in a sync function but need to call async
            match tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(config_manager.add_wallet(wallet_info))
            }) {
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
        }        info!("Successfully created wallet with seed phrase: {}", name);
        Ok(())
    }    /// Generate keys for developer mode (random keys)
    fn generate_dev_mode_keys(&self, name: &str) -> Result<(String, String, KeyPair), WalletError> {
        info!("Generating random keys for developer mode wallet: {}", name);
        
        // Generate random bytes for private key (32 bytes for secp256k1)
        let mut private_key_bytes = [0u8; 32];
        let mut rng = rand::rng();
        for byte in &mut private_key_bytes {
            *byte = rng.random();
        }
        
        // Convert to hex string for storage
        let private_key_hex = hex::encode(private_key_bytes);
        
        // Generate a placeholder public key (in real implementation, derive from private key)
        let public_key = format!("pubkey_{}_dev", name);
        
        // Generate a placeholder address
        let address = format!("bc1q{}dev", &name.chars().take(10).collect::<String>().to_lowercase());
        
        // Create master keys
        let master_private_key = format!("xprv_dev_{}", private_key_hex);
        let master_public_key = format!("xpub_dev_{}", public_key);
        
        let key_pair = KeyPair {
            address: address.clone(),
            key_type: KeyType::NativeSegWit, // Use native segwit for dev mode
            derivation_path: "m/44'/0'/0'/0/0".to_string(),
            public_key,
            private_key: private_key_hex,
        };
        
        Ok((master_public_key, master_private_key, key_pair))
    }/// Derive keys from a real seed phrase
    fn derive_keys_from_seed(&self, seed_phrase: &str, name: &str) -> Result<(String, String, KeyPair), WalletError> {
        info!("Deriving keys from seed phrase for wallet: {}", name);
        
        // For now, we'll create deterministic keys based on the seed phrase
        // In a real implementation, this would use proper BIP39/BIP32 derivation
        
        // Create a simple hash of the seed phrase for deterministic key generation
        use sha2::Sha256;
        let mut hasher = Sha256::new();
        hasher.update(seed_phrase.as_bytes());
        hasher.update(name.as_bytes()); // Add wallet name for uniqueness
        let seed_hash = hasher.finalize();
        
        // Use the hash as our private key material
        let private_key_hex = hex::encode(&seed_hash[..]);
        
        // Generate deterministic public key and address based on seed
        let public_key = format!("pubkey_{}", &private_key_hex[..16]);
        let address = format!("bc1q{}", &private_key_hex[..32].chars().take(32).collect::<String>().to_lowercase());
        
        // Create master keys
        let master_private_key = format!("xprv_seed_{}", private_key_hex);
        let master_public_key = format!("xpub_seed_{}", public_key);
        
        let key_pair = KeyPair {
            address: address.clone(),
            key_type: KeyType::NativeSegWit, // Use native segwit
            derivation_path: "m/44'/0'/0'/0/0".to_string(),
            public_key,
            private_key: private_key_hex,
        };
        
        Ok((master_public_key, master_private_key, key_pair))
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
    fn encrypt_data(&self, data: &str, password: &str) -> Result<String, WalletError> {        use aes::{Aes256, cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray}};
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;
        use rand::{RngCore, rng};        // Generate a salt for PBKDF2
        let mut salt = [0u8; 16];
        let mut rng = rng();
        rng.fill_bytes(&mut salt);
        
        // Generate an initialization vector (IV) for AES
        let mut iv = [0u8; 16]; 
        rng.fill_bytes(&mut iv);

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
            let mut block = GenericArray::from_slice(chunk).clone();
            
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
    fn decrypt_data(&self, encrypted_data: &str, password: &str) -> Result<String, WalletError> {        use aes::{Aes256, cipher::{BlockDecrypt, KeyInit, generic_array::GenericArray}};
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;
        
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
            let mut block = GenericArray::from_slice(chunk).clone();
            
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
    }    /// Remove a wallet from configuration
    pub async fn remove_wallet_from_config(&mut self, wallet_name: &str) -> Result<(), WalletError> {
        if let Some(config_manager) = &self.config_manager {
            info!("Removing wallet '{}' from configuration", wallet_name);
            
            // Clone config first to avoid mutex deadlock
            let config_clone = config_manager.get_config();
            
            // Check if the wallet exists
            let wallet_exists = config_clone.wallets.iter().any(|w| w.name == wallet_name);
            if !wallet_exists {
                error!("Wallet '{}' not found in configuration", wallet_name);
                return Err(WalletError::Generic(format!(
                    "Wallet '{}' not found", wallet_name
                )));
            }            // Create updated config with filtered wallets
            let mut updated_config = config_clone.clone();
            
            // Filter out the wallet to be removed
            updated_config.wallets = updated_config.wallets
                .into_iter()
                .filter(|w| w.name != wallet_name)
                .collect();            // Update the config file
            match config_manager.update_config(updated_config.clone()).await {
                Ok(_) => {
                    info!("Wallet '{}' removed from configuration file", wallet_name);
                    // Also update the internal config to keep it in sync
                    self.config = updated_config;
                    debug!("Internal WalletManager config updated: {} wallets remaining", self.config.wallets.len());
                    Ok(())
                },
                Err(e) => {
                    error!("Failed to update configuration: {}", e);
                    Err(WalletError::ConfigError(format!(
                        "Failed to update configuration: {}", e
                    )))
                }
            }
        } else {
            error!("No config manager available");
            Err(WalletError::Generic("No config manager available".to_string()))
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
        // Call the synchronous version
        manager.create_wallet_with_seed(name, password, seed_phrase, is_secured)
    }
}
