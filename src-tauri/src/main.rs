// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Just use the run function from lib.rs
fn main() {
    b_rad_coin_lib::run();
}
