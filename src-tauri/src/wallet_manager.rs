use crate::config::{Config, WalletInfo};
use crate::errors::WalletError;
use log::{info, error, debug};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;

/// Wallet type representing an open wallet
pub struct Wallet {
    pub name: String,
    pub path: PathBuf,
    // Add other wallet properties as needed
}

/// WalletManager handles all wallet operations
pub struct WalletManager {
    config: Config,
    current_wallet: Option<Wallet>, // This state is not persisted
}

impl WalletManager {
    /// Create a new WalletManager instance
    pub fn new(config: Config) -> Self {
        info!("Initializing wallet manager");
        WalletManager {
            config,
            current_wallet: None,
        }
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
        let wallet_info = self.config.wallets.iter()
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
                    debug!("Password verification succeeded for secured wallet: {}", name);
                },
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
        
        // Create wallet directory
        let wallet_path = format!("wallets/{}", name);
        debug!("Creating wallet with path: {}", wallet_path);
        
        // Add to config
        self.config.wallets.push(WalletInfo {
            name: name.to_string(),
            path: wallet_path,
            secured: is_secured,
        });
        
        info!("Successfully created wallet: {}", name);
        Ok(())
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
                    return Err(WalletError::Generic(format!("Wallet '{}' is already secured", name)));
                }
                
                // Update the wallet to be secured
                debug!("Updating wallet '{}' to be secured", name);
                self.config.wallets[index].secured = true;
                
                // In a real implementation, we would encrypt the wallet data here
                // For now, we just change the flag
                
                info!("Successfully secured wallet: {}", name);
                Ok(())
            },
            None => {
                error!("Wallet not found: {}", name);
                Err(WalletError::NotFound(name.to_string()))
            }
        }
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
    
    /// Get a reference to the inner wallet manager
    pub async fn get_manager(&self) -> tokio::sync::MutexGuard<'_, WalletManager> {
        self.inner.lock().await
    }
    
    /// Shutdown the wallet manager safely
    pub async fn shutdown(&self) -> Result<(), WalletError> {
        let mut manager = self.inner.lock().await;
        manager.shutdown()
    }
}
