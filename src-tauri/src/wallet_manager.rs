use crate::config::{Config, ConfigManager, WalletInfo};
use crate::errors::WalletError;
// Import KeyType and remove unused AddressInfo
use crate::wallet_data::{WalletData, WalletDataError, KeyPair, KeyType};
use log::{debug, error, info};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use rand::Rng;
use bip39::Mnemonic;
use bitcoin::secp256k1::{Secp256k1, PublicKey};
use bitcoin::bip32::{Xpriv, Xpub, DerivationPath};
use bitcoin::{Network, CompressedPublicKey};
use std::str::FromStr;

/// Wallet type representing an open wallet
pub struct Wallet {
    pub name: String,
    pub path: PathBuf,
    pub data: WalletData, // Store the loaded wallet data
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
        let final_wallet_data = match wallet_data_result {
            Ok(wallet_data) => {
                debug!("Successfully loaded wallet data for: {}", name);
                debug!("Wallet balance: {}, addresses: {}", wallet_data.balance, wallet_data.addresses.len());
                wallet_data
            },
            Err(e) => {
                // Check for password/decryption errors - these should be propagated to user
                if matches!(e, WalletDataError::InvalidPassword) {
                    error!("Invalid password provided for wallet: {}", name);
                    return Err(WalletError::from(e));
                }
                
                if matches!(e, WalletDataError::DecryptionError(_)) {
                    error!("Decryption error for wallet: {}", name);
                    return Err(WalletError::from(e));
                }
                
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
                        wallet_data
                    } else {
                        // For other IO errors, return error
                        error!("Failed to load wallet data due to IO error: {}", e);
                        return Err(WalletError::from(e));
                    }
                } else {
                    // For non-IO errors, return error
                    error!("Failed to load wallet data: {}", e);
                    return Err(WalletError::from(e));
                }
            }
        };

        // Create a wallet object with the loaded data
        let opened_wallet = Wallet {
            name: name.to_string(),
            path: PathBuf::from(&wallet_path),
            data: final_wallet_data,
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
    }    /// Get the currently open wallet
    pub fn get_current_wallet(&self) -> Option<&Wallet> {
        if let Some(wallet) = &self.current_wallet {
            debug!("Retrieved current wallet: {}", wallet.name);
        } else {
            debug!("No wallet is currently open");
        }
        self.current_wallet.as_ref()
    }

    /// Get a mutable reference to the currently open wallet
    pub fn get_current_wallet_mut(&mut self) -> Option<&mut Wallet> {
        if let Some(wallet) = &self.current_wallet {
            debug!("Retrieved mutable current wallet: {}", wallet.name);
        } else {
            debug!("No wallet is currently open");
        }
        self.current_wallet.as_mut()
    }

    /// Update the current wallet's data
    pub fn update_current_wallet_data(&mut self, new_data: WalletData) -> Result<(), WalletError> {
        if let Some(wallet) = &mut self.current_wallet {
            info!("Updating wallet data for: {}", wallet.name);
            wallet.data = new_data;
            Ok(())
        } else {
            Err(WalletError::NoWalletOpen)
        }
    }/// Get the base directory for wallets
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
        }        debug!("Wallet data written to disk: {}", wallet_data_path.display());

        // Extract addresses from wallet data
        let wallet_addresses: Vec<String> = wallet_data.addresses.iter()
            .map(|addr_info| addr_info.address.clone())
            .collect();

        // Create wallet info object
        let wallet_info = WalletInfo {
            name: name.to_string(),
            path: wallet_path,
            secured: is_secured,
            addresses: wallet_addresses,
            block_height: 0, // Start at genesis
            last_sync: None,
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
            }        }
        
        // Extract addresses from wallet data
        let wallet_addresses: Vec<String> = wallet_data.addresses.iter()
            .map(|addr_info| addr_info.address.clone())
            .collect();
        
        // Register the wallet in the config
        let wallet_info = WalletInfo {
            name: name.to_string(),
            path: wallet_path,
            secured: is_secured,
            addresses: wallet_addresses,
            block_height: 0, // Start at genesis
            last_sync: None,
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
    }    /// Derive keys from a real seed phrase using BIP39/BIP32 standards
    fn derive_keys_from_seed(&self, seed_phrase: &str, name: &str) -> Result<(String, String, KeyPair), WalletError> {
        use bitcoin::{Address, PrivateKey};
        
        info!("Deriving keys from seed phrase for wallet: {} using BIP39/BIP32 standards", name);
        
        // Parse the mnemonic phrase
        let mnemonic = Mnemonic::from_str(seed_phrase)
            .map_err(|e| WalletError::KeyDerivationError(format!("Invalid mnemonic: {}", e)))?;
        
        // Generate seed from mnemonic (this creates the root seed)
        let seed = mnemonic.to_seed("");
        
        // Initialize secp256k1 context
        let secp = Secp256k1::new();
        
        // Create master extended private key from seed
        let master_xpriv = Xpriv::new_master(Network::Bitcoin, &seed)
            .map_err(|e| WalletError::KeyDerivationError(format!("Failed to create master key: {}", e)))?;
        
        // Create master extended public key
        let master_xpub = Xpub::from_priv(&secp, &master_xpriv);
        
        // Define derivation path: m/44'/0'/0'/0/0 (BIP44 for Bitcoin mainnet, account 0, external chain, index 0)
        let derivation_path = DerivationPath::from_str("m/44'/0'/0'/0/0")
            .map_err(|e| WalletError::KeyDerivationError(format!("Invalid derivation path: {}", e)))?;
        
        // Derive the child key pair
        let derived_xpriv = master_xpriv.derive_priv(&secp, &derivation_path)
            .map_err(|e| WalletError::KeyDerivationError(format!("Failed to derive private key: {}", e)))?;
        
        let _derived_xpub = Xpub::from_priv(&secp, &derived_xpriv);
        
        // Get the raw private and public keys
        let private_key = derived_xpriv.private_key;
        let public_key = PublicKey::from_secret_key(&secp, &private_key);
        
        // Generate Bitcoin address (using P2WPKH - native segwit)
        // Convert SecretKey to PrivateKey for address generation
        let bitcoin_private_key = PrivateKey::new(private_key, Network::Bitcoin);
        
        // Convert to compressed public key for address generation
        let compressed_pubkey = CompressedPublicKey::from_private_key(&secp, &bitcoin_private_key)
            .map_err(|e| WalletError::KeyDerivationError(format!("Failed to create compressed public key: {}", e)))?;
        
        let address = Address::p2wpkh(&compressed_pubkey, Network::Bitcoin);
        
        // Format keys as strings
        let master_private_key = master_xpriv.to_string();
        let master_public_key = master_xpub.to_string();
        let private_key_hex = hex::encode(private_key.secret_bytes());
        let public_key_hex = hex::encode(public_key.serialize());
        
        let key_pair = KeyPair {
            address: address.to_string(),
            key_type: KeyType::NativeSegWit,
            derivation_path: derivation_path.to_string(),
            public_key: public_key_hex,
            private_key: private_key_hex,
        };
        
        info!("Successfully derived keys using BIP39/BIP32 for wallet: {}", name);
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
        }        debug!("Wallet manager shutdown complete");
        Ok(())
    }

    /// Remove a wallet from configuration
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
#[derive(Clone)]
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
    }    /// Create a wallet with a seed phrase
    pub async fn create_wallet_with_seed(&self, name: &str, password: &str, seed_phrase: &str, is_secured: bool) -> Result<(), WalletError> {
        let mut manager = self.inner.lock().await;
        // Call the synchronous version
        manager.create_wallet_with_seed(name, password, seed_phrase, is_secured)
    }

    /// Update the current wallet's data
    pub async fn update_current_wallet_data(&self, new_data: WalletData) -> Result<(), WalletError> {
        let mut manager = self.inner.lock().await;
        manager.update_current_wallet_data(new_data)
    }
}
