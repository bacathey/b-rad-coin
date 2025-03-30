import { invoke } from '@tauri-apps/api/core';

export interface WalletStatus {
  isOpen: boolean;
  name?: string;
  secured?: boolean;
}

export interface WalletDetails {
  name: string;
  secured: boolean;
}

export async function openWallet(name: string, password?: string): Promise<boolean> {
  return invoke('open_wallet', { walletName: name, password });
}

export async function closeWallet(): Promise<boolean> {
  return invoke('close_wallet');
}

export async function getWalletStatus(): Promise<WalletStatus> {
  const isOpen = await invoke<boolean>('check_wallet_status');
  let name: string | undefined;
  let secured: boolean | undefined;
  
  if (isOpen) {
    name = await invoke<string | null>('get_current_wallet_name') || undefined;
    secured = await invoke<boolean | null>('is_current_wallet_secured') || undefined;
  }
  
  return { isOpen, name, secured };
}

export async function createWallet(name: string, password: string, usePassword: boolean): Promise<boolean> {
  return invoke('create_wallet', { walletName: name, password, usePassword });
}

export async function getWalletDetails(): Promise<WalletDetails[]> {
  return invoke('get_wallet_details');
}

export async function isCurrentWalletSecured(): Promise<boolean | null> {
  return invoke('is_current_wallet_secured');
}
