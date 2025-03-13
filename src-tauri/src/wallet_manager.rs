// ...existing code...

pub struct WalletManager {
    config: Config,
    current_wallet: Option<Wallet>, // This state is not persisted
    // ...existing code...
}

impl WalletManager {
    // ...existing code...
    
    pub fn open_wallet(&mut self, name: &str, password: &str) -> Result<(), WalletError> {
        // Find the wallet in available wallets
        let wallet_info = self.config.wallets.iter()
            .find(|w| w.name == name)
            .ok_or(WalletError::WalletNotFound)?;
            
        // Open the wallet but don't update config
        // ...existing code...
        
        // Set current wallet in memory only
        self.current_wallet = Some(opened_wallet);
        
        Ok(())
    }
    
    pub fn close_wallet(&mut self) {
        // Simply clear the current wallet from memory
        self.current_wallet = None;
    }
    
    pub fn get_current_wallet(&self) -> Option<&Wallet> {
        self.current_wallet.as_ref()
    }
    
    // ...existing code...
}
// ...existing code...
