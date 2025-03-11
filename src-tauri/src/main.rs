// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Global variable to store the name of the currently opened wallet
static mut CURRENT_WALLET: Option<String> = None;

// Command to check if the wallet is open
#[tauri::command]
fn check_wallet_status() -> bool {
    // In a real application, this would check if the wallet file is loaded,
    // if user is authenticated, etc.
    
    unsafe {
        CURRENT_WALLET.is_some()
    }
}

// Command to close the wallet
#[tauri::command]
fn close_wallet() -> Result<bool, String> {
    // In a real application, this would:
    // - Save any pending wallet changes
    // - Close wallet files
    // - Clean up resources
    // - Log the user out

    // Clear the current wallet
    unsafe {
        CURRENT_WALLET = None;
    }

    // For demonstration purposes, we'll just return success
    Ok(true)
}

// Command to get available wallets
#[tauri::command]
fn get_available_wallets() -> Vec<String> {
    // In a real application, this would scan a directory for wallet files
    // or query a database for available wallets
    
    // Returning sample wallets for demonstration
    vec![
        "Main Wallet".to_string(),
        "Trading Wallet".to_string(),
        "Cold Storage".to_string(),
    ]
}

// Command to open a wallet
#[tauri::command]
fn open_wallet(wallet_name: String) -> Result<bool, String> {
    // In a real application, this would:
    // - Load the wallet file
    // - Decrypt the wallet if needed
    // - Initialize wallet data structures
    
    // For demonstration, store the selected wallet name
    unsafe {
        CURRENT_WALLET = Some(wallet_name.clone());
    }
    
    println!("Opening wallet: {}", wallet_name);
    Ok(true)
}

// Command to get the name of the currently opened wallet
#[tauri::command]
fn get_current_wallet_name() -> Option<String> {
    unsafe {
        CURRENT_WALLET.clone()
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            check_wallet_status, 
            close_wallet, 
            get_available_wallets,
            open_wallet,
            get_current_wallet_name
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Existing greet command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
