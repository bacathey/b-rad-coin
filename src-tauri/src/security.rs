use crate::errors::SecurityError;
use log::{debug, error, info};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Security Manager handles authentication and encryption
pub struct SecurityManager {
    /// Authentication timeout in seconds
    auth_timeout_seconds: u64,
    /// Time of last successful authentication
    last_auth_time: Option<Instant>,
    /// Whether authentication is currently valid
    authenticated: bool,
    /// Storage for wallet passwords (wallet_name -> password_hash)
    wallet_passwords: HashMap<String, String>,
}

impl SecurityManager {
    /// Create a new SecurityManager
    pub fn new(auth_timeout_seconds: u64) -> Self {
        info!(
            "Initializing security manager with timeout of {} seconds",
            auth_timeout_seconds
        );
        SecurityManager {
            auth_timeout_seconds,
            last_auth_time: None,
            authenticated: false,
            wallet_passwords: HashMap::new(),
        }
    }

    /// Attempt to authenticate with the provided credentials
    pub fn authenticate(&mut self, password: &str) -> Result<bool, SecurityError> {
        debug!("Authentication attempt");

        // In a real implementation, this would validate against stored credentials
        // For this example, we'll use a simple check
        if password.is_empty() {
            error!("Authentication failed: Empty password");
            return Err(SecurityError::InvalidCredentials(
                "Password cannot be empty".to_string(),
            ));
        }

        // For demo purposes, accept any non-empty password
        self.authenticated = true;
        self.last_auth_time = Some(Instant::now());

        info!("Authentication successful");
        Ok(true)
    }

    /// Check if the current authentication is still valid
    pub fn is_authenticated(&mut self) -> bool {
        if !self.authenticated {
            return false;
        }

        // Check if the authentication has timed out
        if let Some(last_time) = self.last_auth_time {
            let elapsed = last_time.elapsed();
            let timeout = Duration::from_secs(self.auth_timeout_seconds);

            if elapsed > timeout {
                debug!(
                    "Authentication timed out after {} seconds",
                    elapsed.as_secs()
                );
                self.invalidate_authentication();
                return false;
            }
        } else {
            // No authentication time recorded, invalidate
            self.invalidate_authentication();
            return false;
        }

        true
    }

    /// Invalidate the current authentication
    pub fn invalidate_authentication(&mut self) {
        if self.authenticated {
            debug!("Invalidating authentication");
            self.authenticated = false;
            self.last_auth_time = None;
        }
    }

    /// Encrypt sensitive data (simplified implementation)
    pub fn encrypt_data(&self, data: &str) -> Result<String, SecurityError> {
        // In a real implementation, this would use proper encryption
        // For this example, we'll just simulate encryption
        debug!("Encrypting data");

        // Simple XOR "encryption" for demonstration purposes only
        let key = 42; // Demo key
        let encrypted: String = data.chars().map(|c| ((c as u8) ^ key) as char).collect();

        Ok(encrypted)
    }

    /// Decrypt sensitive data (simplified implementation)
    pub fn decrypt_data(&self, encrypted_data: &str) -> Result<String, SecurityError> {
        // In a real implementation, this would use proper decryption
        // For this example, we'll just simulate decryption
        debug!("Decrypting data");

        // Simple XOR "decryption" for demonstration purposes only
        let key = 42; // Demo key
        let decrypted: String = encrypted_data
            .chars()
            .map(|c| ((c as u8) ^ key) as char)
            .collect();

        Ok(decrypted)
    }

    /// Store a wallet password (using simple hash for demo)
    pub fn store_wallet_password(&mut self, wallet_name: &str, password: &str) {
        let password_hash = self.simple_hash(password);
        self.wallet_passwords.insert(wallet_name.to_string(), password_hash);
        info!("Stored password for wallet: {}", wallet_name);
    }

    /// Authenticate against a specific wallet's password
    pub fn authenticate_wallet(&mut self, wallet_name: &str, password: &str) -> Result<bool, SecurityError> {
        debug!("Authenticating wallet: {}", wallet_name);

        if password.is_empty() {
            error!("Authentication failed: Empty password");
            return Err(SecurityError::InvalidCredentials(
                "Password cannot be empty".to_string(),
            ));
        }

        // Check if we have a stored password for this wallet
        if let Some(stored_hash) = self.wallet_passwords.get(wallet_name) {
            let provided_hash = self.simple_hash(password);
            if &provided_hash == stored_hash {
                self.authenticated = true;
                self.last_auth_time = Some(Instant::now());
                info!("Authentication successful for wallet: {}", wallet_name);
                Ok(true)
            } else {
                error!("Authentication failed: Invalid password for wallet: {}", wallet_name);
                Err(SecurityError::InvalidCredentials(
                    "Invalid password".to_string(),
                ))
            }
        } else {
            // For unsecured wallets, use the old behavior
            debug!("No stored password for wallet: {}, using legacy authentication", wallet_name);
            self.authenticate(password)
        }
    }

    /// Simple hash function for demo purposes
    fn simple_hash(&self, input: &str) -> String {
        // In a real implementation, use a proper password hashing library like argon2
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Async wrapper for SecurityManager to be used with Tauri state
pub struct AsyncSecurityManager {
    inner: Arc<Mutex<SecurityManager>>,
}

impl AsyncSecurityManager {
    /// Create a new AsyncSecurityManager
    pub fn new(security_manager: SecurityManager) -> Self {
        AsyncSecurityManager {
            inner: Arc::new(Mutex::new(security_manager)),
        }
    }

    /// Get a reference to the inner security manager
    pub async fn get_manager(&self) -> tokio::sync::MutexGuard<'_, SecurityManager> {
        self.inner.lock().await
    }
}
