// Wallet-related TypeScript interfaces

export interface CurrentWalletInfo {
  name: string;
  addresses: AddressDetails[];
  master_public_key: string;
  balance: number;
  is_secured: boolean;
}

export interface AddressDetails {
  address: string;
  public_key: string;
  derivation_path: string;
  address_type: string;
  label?: string;
}

export interface WalletInfo {
  name: string;
  secured?: boolean;
}

export interface WalletDetails {
  name: string;
  secured: boolean;
}
