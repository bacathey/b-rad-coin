import { createContext, useContext, useState, useEffect, useCallback } from 'react';
import type { ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useTrayIntegration } from '../hooks/useTrayIntegration';
import { useWalletDialog } from './WalletDialogContext';

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
  getCurrentWalletPath: () => Promise<string | null>;  openWalletFolder: (path: string) => Promise<boolean>;
  openWalletFolderWithShell: (path: string) => Promise<boolean>;
  closeWallet: () => Promise<boolean>;
  deleteWallet: (walletName: string) => Promise<boolean>;
}

// Create the context with default values
const WalletContext = createContext<WalletContextType>({
  isWalletOpen: false,
  setIsWalletOpen: () => {},
  currentWallet: null,
  setCurrentWallet: () => {},
  isWalletSecured: false,
  availableWallets: [],  refreshWalletDetails: async () => {},
  getCurrentWalletPath: async () => null,
  openWalletFolder: async () => false,
  openWalletFolderWithShell: async () => false,
  closeWallet: async () => false,
  deleteWallet: async () => false
});

// Create a provider component
export function WalletProvider({ children }: { children: ReactNode }) {
  // Default to wallet closed (false)
  const [isWalletOpen, setIsWalletOpen] = useState(false);
  const [currentWallet, setCurrentWallet] = useState<WalletInfo | null>(null);
  const [isWalletSecured, setIsWalletSecured] = useState(false);
  const [availableWallets, setAvailableWallets] = useState<WalletDetails[]>([]);
  // Get wallet dialog context if available (it might not be available during initial render)
  const walletDialog = useWalletDialog();

  // Function to close current wallet
  const closeCurrentWallet = async () => {
    if (!isWalletOpen) {
      console.log('No wallet is currently open');
      return;
    }
    
    try {
      console.log('Closing current wallet via tray action');
      const result = await invoke<boolean>('close_wallet');
      if (result) {
        setCurrentWallet(null);
        setIsWalletOpen(false);
        setIsWalletSecured(false);
        console.log('Current wallet closed successfully');
      } else {
        console.error('Failed to close current wallet');
        throw new Error('Failed to close wallet');
      }
    } catch (error) {
      console.error('Error closing current wallet:', error);
      throw error;
    }
  };

  // Initialize tray integration with navigation callbacks
  const { updateTrayWalletStatus, updateTrayNetworkStatus } = useTrayIntegration({
    onOpenWallet: () => {
      console.log('Tray requested open wallet - opening dialog with Open tab');
      if (walletDialog) {
        walletDialog.openDialog(0); // 0 = Open wallet tab
      }
    },    onCreateWallet: () => {
      console.log('Tray requested create wallet - using forceCreateWalletTab');
      console.log('WalletDialog context available:', !!walletDialog);
      if (walletDialog) {
        console.log('Calling walletDialog.forceCreateWalletTab()');
        walletDialog.forceCreateWalletTab();
        console.log('Called walletDialog.forceCreateWalletTab()');
      }
    },
    onCloseCurrentWallet: closeCurrentWallet,
    onWalletClosed: () => {
      console.log('Wallet closed from tray - updating UI state');
      // The wallet state should already be updated by the close command
      // This is just for any additional UI updates if needed
    },
  });

  // Update tray when wallet status changes
  useEffect(() => {
    const walletName = isWalletOpen && currentWallet ? currentWallet.name : null;
    updateTrayWalletStatus(walletName);
  }, [isWalletOpen, currentWallet, updateTrayWalletStatus]);

  // Initialize network status (stub for now)
  useEffect(() => {
    // For now, always show disconnected - this can be updated when network functionality is implemented
    updateTrayNetworkStatus(false);
  }, [updateTrayNetworkStatus]);// Function to fetch all wallet details (including secured status)
  const refreshWalletDetails = useCallback(async () => {
    console.log('WalletContext: refreshWalletDetails called');
    try {
      const wallets = await invoke<WalletDetails[]>('get_wallet_details');
      console.log('WalletContext: Got wallets from backend:', wallets.length);
      setAvailableWallets(wallets);
      
      // Use functional updates to avoid dependencies
      setCurrentWallet(prevWallet => {
        if (prevWallet) {
          const currentWalletDetails = wallets.find(w => w.name === prevWallet.name);
          if (currentWalletDetails) {
            setIsWalletSecured(currentWalletDetails.secured);
            return {
              ...prevWallet,
              secured: currentWalletDetails.secured
            };
          }
        }
        return prevWallet;
      });
      
      console.log('WalletContext: availableWallets state updated');
    } catch (error) {
      console.error('Error fetching wallet details:', error);
      setAvailableWallets([]);
    }
  }, []); // Empty dependencies since we use functional updates

  // Update isWalletSecured whenever currentWallet changes
  useEffect(() => {
    if (currentWallet) {
      setIsWalletSecured(currentWallet.secured === true);
    } else {
      setIsWalletSecured(false);
    }
  }, [currentWallet]);

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
    }    checkWalletStatus();
  }, []);

  // Effect to listen for wallet deletion events from backend
  useEffect(() => {
    const unlisten = listen('wallets-deleted', () => {
      console.log('WalletContext: Received wallets-deleted event, clearing state');
      setCurrentWallet(null);
      setIsWalletOpen(false);
      setIsWalletSecured(false);
      setAvailableWallets([]);
      
      // Refresh wallet details to get the current state
      refreshWalletDetails();
    });

    // Cleanup listener on unmount
    return () => {
      unlisten.then(f => f());
    };
  }, [refreshWalletDetails]);

  // Function to get the path of the current wallet
  const getCurrentWalletPath = async (): Promise<string | null> => {
    try {
      const path = await invoke<string | null>('get_current_wallet_path');
      return path;
    } catch (error) {
      console.error('Failed to get current wallet path:', error);
      // Re-throw the error so the caller can handle it appropriately
      throw new Error(`Failed to get current wallet path: ${error instanceof Error ? error.message : String(error)}`);
    }
  };
  // Function to open a folder in the system's file explorer
  const openWalletFolder = async (path: string): Promise<boolean> => {
    if (!path || path.trim() === '') {
      console.error('Cannot open folder: path is empty');
      return false;
    }
    
    try {
      console.log(`WalletContext: Opening folder at path: "${path}"`);
      const result = await invoke<boolean>('open_folder_in_explorer', { path });
      console.log(`WalletContext: open_folder_in_explorer result: ${result}`);
      return result;
    } catch (error) {
      console.error('WalletContext: Failed to open folder:', error);
      return false;
    }
  };
  // Function to open a folder using shell commands as a fallback
  const openWalletFolderWithShell = async (path: string): Promise<boolean> => {
    if (!path || path.trim() === '') {
      console.error('Cannot open folder with shell: path is empty');
      return false;
    }
    
    try {
      console.log(`WalletContext: Opening folder with shell command at path: "${path}"`);
      const result = await invoke<boolean>('open_folder_with_shell_command', { path });
      console.log(`WalletContext: open_folder_with_shell_command result: ${result}`);
      return result;
    } catch (error) {
      console.error('WalletContext: Failed to open folder with shell:', error);
      return false;
    }
  };
  
  // Function to close the current wallet
  const closeWallet = async (): Promise<boolean> => {
    try {
      const result = await invoke<boolean>('close_wallet');
      if (result) {
        setIsWalletOpen(false);
        setCurrentWallet(null);
        setIsWalletSecured(false);
      }
      return result;
    } catch (error) {
      console.error('Failed to close wallet:', error);
      return false;
    }
  };
  // Function to delete a wallet
  const deleteWallet = async (walletName: string): Promise<boolean> => {
    console.log('WalletContext: deleteWallet called for:', walletName);
    try {
      const result = await invoke<boolean>('delete_wallet', { walletName });
      console.log('WalletContext: delete_wallet backend result:', result);
      if (result) {
        // If the deleted wallet was the current one, reset state
        if (currentWallet?.name === walletName) {
          console.log('WalletContext: Deleted wallet was current, resetting state');
          setIsWalletOpen(false);
          setCurrentWallet(null);
          setIsWalletSecured(false);
        }
        // Refresh the list of available wallets
        console.log('WalletContext: Refreshing wallet details after deletion');
        await refreshWalletDetails();
        console.log('WalletContext: Wallet details refresh completed');
      }
      return result;
    } catch (error) {
      console.error(`Failed to delete wallet ${walletName}:`, error);
      throw new Error(`Failed to delete wallet: ${error instanceof Error ? error.message : String(error)}`);
    }
  };
  return (
    <WalletContext.Provider value={{
      isWalletOpen,
      setIsWalletOpen,
      currentWallet,
      setCurrentWallet,
      isWalletSecured,
      availableWallets,      refreshWalletDetails,
      getCurrentWalletPath,
      openWalletFolder,
      openWalletFolderWithShell,
      closeWallet,
      deleteWallet
    }}>
      {children}
    </WalletContext.Provider>
  );
}

// Custom hook to use the wallet context
export function useWallet() {
  return useContext(WalletContext);
}