export interface WalletAddress {
  wallet_name: string;
  address: string;
  label?: string;
  derivation_path: string;
}

export interface MiningConfiguration {
  wallet_id: string;
  is_mining: boolean;
  mining_address: string;
  hash_rate: number;
  blocks_mined: number;
  current_difficulty: number;
}
