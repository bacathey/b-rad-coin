use serde_json;
use std::error::Error;
use std::fmt;
use std::io;

/// Custom error types for B-Rad Coin application
#[derive(Debug)]
pub enum AppError {
    /// Errors related to wallet operations
    Wallet(WalletError),
    /// Errors related to configuration operations
    Config(ConfigError),
    /// Errors related to security operations
    Security(SecurityError),
    /// Errors related to network operations
    Network(String),
    /// IO errors
    Io(io::Error),
    /// JSON serialization/deserialization errors
    Json(serde_json::Error),
    /// Generic application errors
    Generic(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Wallet(err) => write!(f, "Wallet error: {}", err),
            AppError::Config(err) => write!(f, "Config error: {}", err),
            AppError::Security(err) => write!(f, "Security error: {}", err),
            AppError::Network(err) => write!(f, "Network error: {}", err),
            AppError::Io(err) => write!(f, "IO error: {}", err),
            AppError::Json(err) => write!(f, "JSON error: {}", err),
            AppError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for AppError {}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        AppError::Io(error)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::Json(error)
    }
}

impl From<WalletError> for AppError {
    fn from(error: WalletError) -> Self {
        AppError::Wallet(error)
    }
}

impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self {
        AppError::Config(error)
    }
}

impl From<SecurityError> for AppError {
    fn from(error: SecurityError) -> Self {
        AppError::Security(error)
    }
}

impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError::Generic(error)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError::Generic(error.to_string())
    }
}

/// Wallet-specific error types
#[derive(Debug, Clone)]
pub enum WalletError {
    NotFound(String),
    AccessDenied(String),
    AlreadyExists(String),
    InvalidOperation(String),
    ConfigError(String),
    KeyDerivationError(String),
    NoWalletOpen,
    Generic(String),
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletError::NotFound(name) => write!(f, "Wallet '{}' not found", name),
            WalletError::AccessDenied(name) => write!(f, "Access denied to wallet '{}'", name),
            WalletError::AlreadyExists(name) => write!(f, "Wallet '{}' already exists", name),
            WalletError::InvalidOperation(msg) => write!(f, "Invalid wallet operation: {}", msg),
            WalletError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            WalletError::KeyDerivationError(msg) => write!(f, "Key derivation error: {}", msg),
            WalletError::NoWalletOpen => write!(f, "No wallet is currently open"),
            WalletError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for WalletError {}

// Convert WalletDataError to WalletError
impl From<crate::wallet_data::WalletDataError> for WalletError {
    fn from(error: crate::wallet_data::WalletDataError) -> Self {
        use crate::wallet_data::WalletDataError;
        match error {
            WalletDataError::InvalidPassword => WalletError::AccessDenied("Invalid password".to_string()),
            WalletDataError::DecryptionError(msg) => WalletError::AccessDenied(format!("Decryption failed: {}", msg)),
            WalletDataError::EncryptionError(msg) => WalletError::Generic(format!("Encryption failed: {}", msg)),
            WalletDataError::IoError(err) => WalletError::Generic(format!("IO error: {}", err)),
            WalletDataError::SerializationError(err) => WalletError::Generic(format!("Serialization error: {}", err)),
        }
    }
}

/// Configuration-specific error types
#[derive(Debug, Clone)]
pub enum ConfigError {
    LoadError(String),
    SaveError(String),
    ParseError(String),
    PathError(String),
    Generic(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::LoadError(msg) => write!(f, "Failed to load configuration: {}", msg),
            ConfigError::SaveError(msg) => write!(f, "Failed to save configuration: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Failed to parse configuration: {}", msg),
            ConfigError::PathError(msg) => write!(f, "Configuration path error: {}", msg),
            ConfigError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ConfigError {}

/// Security-specific error types
#[derive(Debug, Clone)]
pub enum SecurityError {
    AuthenticationFailed(String),
    InvalidCredentials(String),
    EncryptionError(String),
    DecryptionError(String),
    Generic(String),
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            SecurityError::InvalidCredentials(msg) => write!(f, "Invalid credentials: {}", msg),
            SecurityError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            SecurityError::DecryptionError(msg) => write!(f, "Decryption error: {}", msg),
            SecurityError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for SecurityError {}

/// Result type alias for Application results
pub type AppResult<T> = Result<T, AppError>;
