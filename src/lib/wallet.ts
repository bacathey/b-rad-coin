import { invoke } from '@tauri-apps/api/core';

export interface WalletStatus {
  isOpen: boolean;
  name?: string;
}

export async function openWallet(name: string, password: string): Promise<void> {
  return invoke('open_wallet', { name, password });
}

export async function closeWallet(): Promise<void> {
  return invoke('close_wallet');
}

export async function getWalletStatus(): Promise<WalletStatus> {
  return invoke('get_wallet_status');
}

// When app starts, we don't attempt to reopen any previously "open" wallet
// since we no longer persist that state
