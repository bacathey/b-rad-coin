// ...existing code...

#[tauri::command]
pub fn open_wallet(name: String, password: String, state: State<'_, WalletManagerState>) -> Result<(), String> {
    let mut wallet_manager = state.0.lock().map_err(|e| e.to_string())?;
    wallet_manager.open_wallet(&name, &password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn close_wallet(state: State<'_, WalletManagerState>) -> Result<(), String> {
    let mut wallet_manager = state.0.lock().map_err(|e| e.to_string())?;
    wallet_manager.close_wallet();
    Ok(())
}

#[tauri::command]
pub fn get_wallet_status(state: State<'_, WalletManagerState>) -> Result<WalletStatus, String> {
    let wallet_manager = state.0.lock().map_err(|e| e.to_string())?;
    let current_wallet = wallet_manager.get_current_wallet();
    
    Ok(WalletStatus {
        is_open: current_wallet.is_some(),
        name: current_wallet.map(|w| w.name().to_string()),
        // ...existing code...
    })
}
// ...existing code...
