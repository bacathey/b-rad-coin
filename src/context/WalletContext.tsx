import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Define the wallet type
interface WalletInfo {
  name: string;
  secured?: boolean;
  // Add other wallet properties as needed in the future
  // e.g., balance, address, etc.
}

// Define wallet details from backend
interface WalletDetails {
  name: string;
  secured: boolean;
}

// Define the shape of our wallet context
interface WalletContextType {
  isWalletOpen: boolean;
  setIsWalletOpen: (isOpen: boolean) => void;
  currentWallet: WalletInfo | null;
  setCurrentWallet: (wallet: WalletInfo | null) => void;
  isWalletSecured: boolean;
  availableWallets: WalletDetails[];
  refreshWalletDetails: () => Promise<void>;
}

// Create the context with default values
const WalletContext = createContext<WalletContextType>({
  isWalletOpen: false,
  setIsWalletOpen: () => {},
  currentWallet: null,
  setCurrentWallet: () => {},
  isWalletSecured: false,
  availableWallets: [],
  refreshWalletDetails: async () => {}
});

// Create a provider component
export function WalletProvider({ children }: { children: ReactNode }) {
  // Default to wallet closed (false)
  const [isWalletOpen, setIsWalletOpen] = useState(false);
  const [currentWallet, setCurrentWallet] = useState<WalletInfo | null>(null);
  const [isWalletSecured, setIsWalletSecured] = useState(false);
  const [availableWallets, setAvailableWallets] = useState<WalletDetails[]>([]);

  // Function to fetch all wallet details (including secured status)
  const refreshWalletDetails = async () => {
    try {
      const wallets = await invoke<WalletDetails[]>('get_wallet_details');
      setAvailableWallets(wallets);
    } catch (error) {
      console.error('Error fetching wallet details:', error);
      setAvailableWallets([]);
    }
  };

  // Effect to fetch the initial wallet state from Rust backend
  useEffect(() => {
    async function checkWalletStatus() {
      try {
        // Call to Rust function to check if wallet is open
        const walletStatus = await invoke<boolean>('check_wallet_status');
        
        if (walletStatus) {
          // If a wallet is open, get its details
          try {
            const walletName = await invoke<string>('get_current_wallet_name');
            if (walletName) {
              // Get security status
              const secured = await invoke<boolean | null>('is_current_wallet_secured');
              
              setCurrentWallet({
                name: walletName,
                secured: secured === true
              });
              setIsWalletSecured(secured === true);
              setIsWalletOpen(true);  // Only set to true if we successfully get the wallet name
            } else {
              // If we can't get the wallet name, treat it as no wallet open
              setIsWalletOpen(false);
              setIsWalletSecured(false);
              setCurrentWallet(null);
            }
          } catch (error) {
            console.error('Error getting current wallet details:', error);
            setIsWalletOpen(false);
            setIsWalletSecured(false);
            setCurrentWallet(null);
          }
        } else {
          setIsWalletOpen(false);
          setIsWalletSecured(false);
          setCurrentWallet(null);
        }
      } catch (error) {
        console.error('Error checking wallet status:', error);
        setIsWalletOpen(false);
        setIsWalletSecured(false);
        setCurrentWallet(null);
      }
      
      // Always refresh available wallets
      await refreshWalletDetails();
    }

    checkWalletStatus();
  }, []);

  const value = {
    isWalletOpen,
    setIsWalletOpen,
    currentWallet,
    setCurrentWallet,
    isWalletSecured,
    availableWallets,
    refreshWalletDetails
  };

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

// Custom hook to use the wallet context
export function useWallet() {
  return useContext(WalletContext);
}