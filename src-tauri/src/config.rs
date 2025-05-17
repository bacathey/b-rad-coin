use crate::errors::ConfigError;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};


/// Configuration structure for the application
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// List of available wallets
    pub wallets: Vec<WalletInfo>,
    /// Application settings
    pub app_settings: AppSettings,
}

/// Information about a wallet
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletInfo {
    /// Name of the wallet
    pub name: String,
    /// Path to the wallet files
    pub path: String,
    /// Whether the wallet is password protected
    pub secured: bool,
}

/// Application settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    /// User interface theme
    pub theme: String,
    /// Whether automatic backups are enabled
    pub auto_backup: bool,
    /// Whether notifications are enabled
    pub notifications_enabled: bool,
    /// Log level setting
    pub log_level: String,
    /// Developer mode enabled
    #[serde(rename = "developerMode")]
    pub developer_mode: bool,
    /// Whether to show seed phrase dialogs during wallet creation
    #[serde(default = "default_show_seed_phrase_dialogs", rename = "showSeedPhraseDialogs")]
    pub show_seed_phrase_dialogs: bool,
}

/// Default implementation for Config
impl Default for Config {
    fn default() -> Self {
        debug!("Creating default configuration");
        Self {
            wallets: vec![],
            app_settings: AppSettings::default(),
        }
    }
}

/// Default value for show_seed_phrase_dialogs
fn default_show_seed_phrase_dialogs() -> bool {
    true
}

/// Default implementation for AppSettings
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            auto_backup: true,
            notifications_enabled: true,
            log_level: "info".to_string(),
            developer_mode: false,
            show_seed_phrase_dialogs: true,
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    config: std::sync::Mutex<Config>,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new ConfigManager instance
    pub async fn new() -> Result<Self, ConfigError> {
        debug!("Initializing configuration manager");
        let (config, config_path) = Self::load_config().await?;

        Ok(ConfigManager {
            config: std::sync::Mutex::new(config),
            config_path,
        })
    }

    /// Get a reference to the current configuration
    pub fn get_config(&self) -> Config {
        self.config.lock().unwrap().clone()
    }

    /// Update application settings
    pub async fn update_app_settings(&self, settings: AppSettings) -> Result<(), ConfigError> {
        info!("Updating application settings");

        // Clone the config first to avoid holding the mutex guard across an await point
        let config_clone;
        {
            let mut config = self.config.lock().unwrap();
            config.app_settings = settings;
            config_clone = config.clone();
        } // Mutex guard is dropped here

        // Now we can await without holding the mutex guard
        self.save_config_to_path(&config_clone, &self.config_path)
            .await?;

        // Update the stored config
        let mut config = self.config.lock().unwrap();
        *config = config_clone;

        Ok(())
    }

    /// Save the configuration to a specific path
    async fn save_config_to_path(
        &self,
        config: &Config,
        path: &PathBuf,
    ) -> Result<(), ConfigError> {
        debug!("Serializing configuration to JSON");
        info!("About to save developer_mode value: {}", config.app_settings.developer_mode);
        let config_json = match serde_json::to_string_pretty(config) {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to serialize config to JSON: {}", e);
                return Err(ConfigError::SaveError(format!(
                    "Failed to serialize config to JSON: {}",
                    e
                )));
            }
        };

        debug!("Writing configuration to file: {}", path.display());
        let mut file = match fs::File::create(path).await {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to create config file: {}", e);
                return Err(ConfigError::SaveError(format!(
                    "Failed to create config file: {}",
                    e
                )));
            }
        };

        if let Err(e) = file.write_all(config_json.as_bytes()).await {
            error!("Failed to write configuration to file: {}", e);
            return Err(ConfigError::SaveError(format!(
                "Failed to write configuration to file: {}",
                e
            )));
        }

        info!("Configuration saved successfully");
        Ok(())
    }

    /// Add a new wallet to configuration
    pub async fn add_wallet(&self, wallet_info: WalletInfo) -> Result<(), ConfigError> {
        // Check if wallet already exists
        let mut config = self.config.lock().unwrap();
        if config.wallets.iter().any(|w| w.name == wallet_info.name) {
            error!(
                "Wallet '{}' already exists in configuration",
                wallet_info.name
            );
            return Err(ConfigError::Generic(format!(
                "Wallet '{}' already exists",
                wallet_info.name
            )));
        }

        info!("Adding new wallet '{}' to configuration", wallet_info.name);
        config.wallets.push(wallet_info);
        self.save_config_to_path(&config, &self.config_path).await
    }

    /// Remove all wallets from the configuration
    pub async fn remove_all_wallets(&self) -> Result<(), ConfigError> {
        info!("Removing all wallets from configuration");

        // Clone the config first to avoid holding the mutex guard across an await point
        let config_clone;
        {
            let mut config = self.config.lock().unwrap();
            // Clear the wallets vector
            config.wallets.clear();
            config_clone = config.clone();
        } // Mutex guard is dropped here

        // Now we can await without holding the mutex guard
        self.save_config_to_path(&config_clone, &self.config_path)
            .await?;

        // Update the stored config
        let mut config = self.config.lock().unwrap();
        *config = config_clone;

        info!("All wallets removed from configuration successfully");
        Ok(())
    }

    /// Load the configuration from file
    async fn load_config() -> Result<(Config, PathBuf), ConfigError> {
        let config_path = Self::get_config_path().await?;

        // Check if the config file exists
        match fs::try_exists(&config_path).await {
            Ok(exists) => {
                if exists {
                    debug!(
                        "Loading existing configuration from {}",
                        config_path.display()
                    );
                    // Read and parse the config file
                    let mut file = match fs::File::open(&config_path).await {
                        Ok(file) => file,
                        Err(e) => {
                            error!("Failed to open config file: {}", e);
                            return Err(ConfigError::LoadError(format!(
                                "Failed to open config file: {}",
                                e
                            )));
                        }
                    };

                    let mut config_content = String::new();
                    if let Err(e) = file.read_to_string(&mut config_content).await {
                        error!("Failed to read config file: {}", e);
                        return Err(ConfigError::LoadError(format!(
                            "Failed to read config file: {}",
                            e
                        )));
                    }

                    let config: Config = match serde_json::from_str(&config_content) {
                        Ok(config) => config,
                        Err(e) => {
                            error!("Failed to parse config file: {}", e);
                            return Err(ConfigError::ParseError(format!(
                                "Failed to parse config file: {}",
                                e
                            )));
                        }
                    };

                    info!("Configuration loaded successfully");
                    Ok((config, config_path))
                } else {
                    // Create a default config if it doesn't exist
                    info!("No configuration found. Creating default configuration");
                    let default_config = Config::default();
                    Self::save_config_to_path_static(&default_config, &config_path).await?;
                    Ok((default_config, config_path))
                }
            }
            Err(e) => {
                error!("Failed to check if config file exists: {}", e);
                Err(ConfigError::LoadError(format!(
                    "Failed to check if config file exists: {}",
                    e
                )))
            }
        }
    }

    /// Save configuration to path (static version)
    async fn save_config_to_path_static(
        config: &Config,
        path: &PathBuf,
    ) -> Result<(), ConfigError> {
        debug!("Serializing configuration to JSON");
        let config_json = match serde_json::to_string_pretty(config) {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to serialize config to JSON: {}", e);
                return Err(ConfigError::SaveError(format!(
                    "Failed to serialize config to JSON: {}",
                    e
                )));
            }
        };

        debug!("Writing configuration to file: {}", path.display());
        let mut file = match fs::File::create(path).await {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to create config file: {}", e);
                return Err(ConfigError::SaveError(format!(
                    "Failed to create config file: {}",
                    e
                )));
            }
        };

        if let Err(e) = file.write_all(config_json.as_bytes()).await {
            error!("Failed to write configuration to file: {}", e);
            return Err(ConfigError::SaveError(format!(
                "Failed to write configuration to file: {}",
                e
            )));
        }

        info!("Configuration saved successfully");
        Ok(())
    }

    /// Get the configuration directory path
    pub async fn get_config_dir() -> Result<PathBuf, ConfigError> {
        // In Tauri 2.0, we need to fall back to standard platform-specific paths
        // since we can't access the Tauri API directly during initialization

        // Get the app-specific data directory based on the platform
        let app_data_dir = match dirs::data_dir() {
            Some(dir) => dir.join("com.b-rad-coin.app"), // Match the identifier in tauri.conf.json
            None => {
                error!("Failed to get app data directory");
                return Err(ConfigError::PathError(
                    "Failed to get app data directory".to_string(),
                ));
            }
        };

        // Join with our config directory name
        let config_dir = app_data_dir.join("config");
        debug!("Configuration directory: {}", config_dir.display());

        // Create directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&config_dir).await {
            error!("Failed to create config directory: {}", e);
            return Err(ConfigError::PathError(format!(
                "Failed to create config directory: {}",
                e
            )));
        }

        Ok(config_dir)
    }

    /// Get the configuration file path
    pub async fn get_config_path() -> Result<PathBuf, ConfigError> {
        let config_dir = Self::get_config_dir().await?;
        let config_path = config_dir.join("app_config.json"); // Changed filename
        debug!("Configuration file path: {}", config_path.display());
        Ok(config_path)
    }

    /// Update wallet security status
    pub async fn update_wallet_security(
        &self,
        wallet_name: &str,
        secured: bool,
    ) -> Result<(), ConfigError> {
        info!(
            "Updating wallet security status for wallet: {}",
            wallet_name
        );

        // Clone the config first to avoid holding the mutex guard across an await point
        let config_clone;
        {
            let mut config = self.config.lock().unwrap();

            // Find the wallet to update
            if let Some(wallet) = config.wallets.iter_mut().find(|w| w.name == wallet_name) {
                // Update the secured flag
                wallet.secured = secured;
                config_clone = config.clone();
            } else {
                error!("Wallet '{}' not found in configuration", wallet_name);
                return Err(ConfigError::Generic(format!(
                    "Wallet '{}' not found",
                    wallet_name
                )));
            }
        } // Mutex guard is dropped here

        // Now we can await without holding the mutex guard
        self.save_config_to_path(&config_clone, &self.config_path)
            .await?;

        // Update the stored config
        let mut config = self.config.lock().unwrap();
        *config = config_clone;

        info!("Wallet security status updated successfully");
        Ok(())
    }
}
