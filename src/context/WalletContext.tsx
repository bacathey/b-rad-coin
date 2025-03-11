import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Define the wallet type
interface WalletInfo {
  name: string;
  // Add other wallet properties as needed in the future
  // e.g., balance, address, etc.
}

// Define the shape of our wallet context
interface WalletContextType {
  isWalletOpen: boolean;
  setIsWalletOpen: (isOpen: boolean) => void;
  isWalletLoading: boolean;
  currentWallet: WalletInfo | null;
  setCurrentWallet: (wallet: WalletInfo | null) => void;
}

// Create the context with default values
const WalletContext = createContext<WalletContextType>({
  isWalletOpen: false,
  setIsWalletOpen: () => {},
  isWalletLoading: true,
  currentWallet: null,
  setCurrentWallet: () => {}
});

// Create a provider component
export function WalletProvider({ children }: { children: ReactNode }) {
  const [isWalletOpen, setIsWalletOpen] = useState(false);
  const [isWalletLoading, setIsWalletLoading] = useState(true);
  const [currentWallet, setCurrentWallet] = useState<WalletInfo | null>(null);

  // Effect to fetch the initial wallet state from Rust backend
  useEffect(() => {
    async function checkWalletStatus() {
      try {
        // Call to Rust function to check if wallet is open
        const walletStatus = await invoke('check_wallet_status');
        
        if (walletStatus) {
          // If a wallet is open, get its details
          // In a real application, you would fetch actual wallet details from the backend
          try {
            const walletName = await invoke<string>('get_current_wallet_name');
            if (walletName) {
              setCurrentWallet({
                name: walletName
              });
            }
          } catch (error) {
            console.error('Error getting current wallet details:', error);
          }
        }
        
        setIsWalletOpen(!!walletStatus);
      } catch (error) {
        console.error('Error checking wallet status:', error);
        setIsWalletOpen(false);
        setCurrentWallet(null);
      } finally {
        setIsWalletLoading(false);
      }
    }

    checkWalletStatus();
  }, []);

  // Provide a function to update both the wallet open state and the current wallet
  const closeWallet = () => {
    setIsWalletOpen(false);
    setCurrentWallet(null);
  };

  const value = {
    isWalletOpen,
    setIsWalletOpen,
    isWalletLoading,
    currentWallet,
    setCurrentWallet
  };

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

// Custom hook to use the wallet context
export function useWallet() {
  return useContext(WalletContext);
}