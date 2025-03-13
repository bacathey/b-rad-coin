// ...existing code...

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub wallets: Vec<WalletInfo>,
    // Remove any fields related to open wallet state
    // ...existing code...
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    pub name: String,
    pub path: String,
    // Remove any open/active/selected fields
    // ...existing code...
}

impl Config {
    // ...existing code...
    
    pub fn add_wallet(&mut self, name: &str, path: &str) -> Result<(), ConfigError> {
        // ...existing code...
        self.wallets.push(WalletInfo {
            name: name.to_string(),
            path: path.to_string(),
            // Remove any initialization of open/active/selected fields
        });
        // ...existing code...
    }
    
    // Remove any methods that set or get current open wallet
    // ...existing code...
}
// ...existing code...
