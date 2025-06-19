import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

interface TrayIntegrationOptions {
  onOpenWallet?: () => void;
  onCreateWallet?: () => void;
  onWalletClosed?: () => void;
  onCloseCurrentWallet?: () => Promise<void>;
}

/**
 * Hook to handle system tray events and integration
 */
export const useTrayIntegration = (options: TrayIntegrationOptions = {}) => {
  const { onOpenWallet, onCreateWallet, onWalletClosed, onCloseCurrentWallet } = options;

  useEffect(() => {
    // Listen for tray events
    const setupTrayListeners = async () => {
      try {
        // Listen for tray open wallet event
        const unlistenOpenWallet = await listen('tray-open-wallet', () => {
          console.log('Tray requested to open wallet dialog');
          if (onOpenWallet) {
            onOpenWallet();
          }
        });        // Listen for tray create wallet event
        const unlistenCreateWallet = await listen('tray-create-wallet', async () => {
          console.log('Tray requested to create wallet dialog');
          
          // If there's a callback to close current wallet, call it first
          if (onCloseCurrentWallet) {
            try {
              await onCloseCurrentWallet();
              console.log('Current wallet closed, now showing create wallet dialog');
            } catch (error) {
              console.error('Failed to close current wallet:', error);
              // Still proceed to show create dialog even if close fails
            }
          }
          
          if (onCreateWallet) {
            onCreateWallet();
          }
        });

        // Listen for wallet closed event
        const unlistenWalletClosed = await listen('wallet-closed', () => {
          console.log('Wallet was closed from tray');
          if (onWalletClosed) {
            onWalletClosed();
          }
        });

        // Return cleanup function
        return () => {
          unlistenOpenWallet();
          unlistenCreateWallet();
          unlistenWalletClosed();
        };
      } catch (error) {
        console.error('Failed to setup tray listeners:', error);
      }
    };    setupTrayListeners();
  }, [onOpenWallet, onCreateWallet, onWalletClosed, onCloseCurrentWallet]);

  // Function to update tray wallet status
  const updateTrayWalletStatus = async (walletName: string | null) => {
    try {
      await invoke('update_tray_wallet_status', { walletName });
    } catch (error) {
      console.error('Failed to update tray wallet status:', error);
    }
  };

  // Function to update tray network status
  const updateTrayNetworkStatus = async (isConnected: boolean, peerCount?: number) => {
    try {
      await invoke('update_tray_network_status', { 
        isConnected, 
        peerCount: peerCount || null 
      });
    } catch (error) {
      console.error('Failed to update tray network status:', error);
    }
  };

  return {
    updateTrayWalletStatus,
    updateTrayNetworkStatus,
  };
};
