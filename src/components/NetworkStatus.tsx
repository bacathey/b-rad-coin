// filepath: c:\Users\bacat\source\repos\b-rad-coin\src\components\NetworkStatus.tsx
import React, { useState, useEffect } from 'react';
import { Box, LinearProgress, Typography, Paper, Stack, Chip } from '@mui/material';
import { useThemeMode } from '../hooks/useThemeMode';
import { useWallet } from '../context/WalletContext';
import CloudDoneIcon from '@mui/icons-material/CloudDone';
import CloudOffIcon from '@mui/icons-material/CloudOff';
import SyncIcon from '@mui/icons-material/Sync';
import PeopleIcon from '@mui/icons-material/People';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface NetworkStatusProps {
  className?: string;
}

interface BlockchainInfo {
  current_height: number;
  network_height: number;
  is_connected: boolean;
  is_syncing: boolean;
  peer_count: number;
}

interface WalletSyncStatus {
  wallet_id: string;
  is_syncing: boolean;
  sync_progress: number;
  last_sync_block: number;
  current_balance: number;
  transaction_count: number;
  utxo_count: number;
}

export const NetworkStatus: React.FC<NetworkStatusProps> = ({ className }) => {
  const { isDarkMode } = useThemeMode();
  const { currentWallet, isWalletOpen } = useWallet();
  const [blockchainInfo, setBlockchainInfo] = useState<BlockchainInfo>({
    current_height: 0,
    network_height: 0,
    is_connected: false,
    is_syncing: false,
    peer_count: 0,
  });
  const [walletSyncStatus, setWalletSyncStatus] = useState<WalletSyncStatus | null>(null);
  const [loading, setLoading] = useState(true);  useEffect(() => {
    const fetchNetworkStatus = async () => {
      try {
        // Get network status from the blockchain sync service
        const networkStatus = await invoke<BlockchainInfo>('get_network_status');
        setBlockchainInfo(networkStatus);
        setLoading(false);
      } catch (error) {
        console.error('Failed to fetch network status:', error);
        setLoading(false);
      }
    };

    const fetchWalletSyncStatus = async () => {
      if (isWalletOpen && currentWallet?.name) {
        try {
          const syncStatus = await invoke<WalletSyncStatus | null>('get_wallet_sync_status', { 
            walletId: currentWallet.name 
          });
          setWalletSyncStatus(syncStatus);
        } catch (error) {
          console.error('Failed to fetch wallet sync status:', error);
          setWalletSyncStatus(null);
        }
      } else {
        setWalletSyncStatus(null);
      }
    };// Listen for blockchain status events
    const setupListener = async () => {
      try {
        const unlistenBlockchain = await listen<BlockchainInfo>('blockchain-status', (event) => {
          setBlockchainInfo(event.payload);
          setLoading(false);
        });

        // Listen for wallet sync status events
        const unlistenWalletSync = await listen<WalletSyncStatus>('wallet-sync-status', (event) => {
          setWalletSyncStatus(event.payload);
        });
        
        // Return cleanup function for both listeners
        return () => {
          unlistenBlockchain();
          unlistenWalletSync();
        };
      } catch (error) {
        console.error('Failed to setup blockchain status listener:', error);
      }
    };    // Initial fetch
    fetchNetworkStatus();
    fetchWalletSyncStatus();
    
    // Setup listener for real-time updates
    let cleanupListeners: (() => void) | undefined;
    setupListener().then(cleanup => {
      cleanupListeners = cleanup;
    });
    
    // Fallback polling every 30 seconds
    const interval = setInterval(() => {
      fetchNetworkStatus();
      fetchWalletSyncStatus();
    }, 30000);

    return () => {
      clearInterval(interval);
      // Cleanup listeners
      cleanupListeners?.();
    };
  }, [isWalletOpen, currentWallet?.name]); // Re-run when wallet changes

  return (
    <Paper 
      className={className}
      elevation={2}
      sx={{ 
        p: 2, 
        borderRadius: 2,
        backgroundColor: isDarkMode 
          ? 'rgba(20, 27, 45, 0.9)' 
          : 'rgba(240, 242, 255, 0.9)', // Much lighter blue in light mode
        backdropFilter: 'blur(10px)',
        color: isDarkMode 
          ? 'rgba(255, 255, 255, 0.9)' 
          : 'rgba(26, 35, 126, 0.9)', // Dark blue text in light mode
        boxShadow: isDarkMode 
          ? '0 4px 20px rgba(0, 0, 0, 0.5)' 
          : '0 4px 20px rgba(0, 0, 0, 0.1)'
      }}
    >
      <Typography variant="h6" sx={{ mb: 2, fontWeight: 'bold' }}>
        Network Status
      </Typography>

      {loading ? (
        <Box sx={{ width: '100%' }}>
          <LinearProgress />
        </Box>
      ) : (
        <>          <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 2 }}>
            {blockchainInfo.is_connected ? 
              <Chip 
                icon={<CloudDoneIcon />} 
                label="Connected" 
                color="success" 
                size="small" 
                variant="outlined"
              /> : 
              <Chip 
                icon={<CloudOffIcon />} 
                label="Disconnected" 
                color={isDarkMode ? "error" : "warning"} // Use warning instead of error in light mode
                size="small" 
                variant="outlined" 
                sx={{
                  borderColor: isDarkMode ? undefined : "rgba(237, 108, 2, 0.7)", // More subdued color for light mode
                  color: isDarkMode ? undefined : "rgba(237, 108, 2, 0.9)"
                }}
              />
            }
            
            <Chip 
              icon={<PeopleIcon />} 
              label={`${blockchainInfo.peer_count} Peers`} 
              color={blockchainInfo.peer_count > 0 ? "primary" : "warning"} 
              size="small" 
              variant="outlined"
              sx={{
                borderColor: isDarkMode ? undefined : "rgba(25, 118, 210, 0.7)", // Customized for light mode
                color: isDarkMode ? undefined : "rgba(25, 118, 210, 0.9)"
              }}
            />
          </Stack>          <Box sx={{ mb: 1 }}>
            <Typography variant="subtitle2" sx={{ mb: 1 }}>
              <SyncIcon fontSize="small" sx={{ verticalAlign: 'middle', mr: 0.5 }} />
              Blockchain Status
            </Typography>
            
            {/* Local and Network Heights */}
            <Stack spacing={0.5} sx={{ mb: 2 }}>
              <Stack direction="row" justifyContent="space-between" alignItems="center">
                <Typography variant="caption" color="text.secondary">
                  Local Height:
                </Typography>
                <Typography variant="caption" fontWeight="medium">
                  {blockchainInfo.current_height.toLocaleString()}
                </Typography>
              </Stack>
              <Stack direction="row" justifyContent="space-between" alignItems="center">
                <Typography variant="caption" color="text.secondary">
                  Network Height:
                </Typography>
                <Typography variant="caption" fontWeight="medium">
                  {blockchainInfo.network_height.toLocaleString()}
                </Typography>
              </Stack>
              <Stack direction="row" justifyContent="space-between" alignItems="center">
                <Typography variant="caption" color="text.secondary">
                  Blocks Behind:
                </Typography>
                <Typography 
                  variant="caption" 
                  fontWeight="medium"
                  color={
                    blockchainInfo.network_height - blockchainInfo.current_height > 0 
                      ? "warning.main" 
                      : "success.main"
                  }
                >
                  {Math.max(0, blockchainInfo.network_height - blockchainInfo.current_height).toLocaleString()}
                </Typography>
              </Stack>
            </Stack>

            {/* Sync Progress */}
            <LinearProgress 
              variant={blockchainInfo.is_syncing ? "indeterminate" : "determinate"}
              value={blockchainInfo.is_syncing ? undefined : 100}
              sx={{ 
                height: 8, 
                borderRadius: 2,
                backgroundColor: isDarkMode ? undefined : 'rgba(200, 208, 255, 0.5)',
                '& .MuiLinearProgress-bar': {
                  backgroundColor: isDarkMode ? undefined : '#4051B5'
                }
              }}
            />
            <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mt: 1 }}>
              <Typography variant="caption">
                {blockchainInfo.is_syncing ? 
                  `Syncing block ${blockchainInfo.current_height}...` : 
                  `Synchronized at block ${blockchainInfo.current_height}`}
              </Typography>
              <Typography variant="caption">
                {blockchainInfo.is_syncing ? 'Syncing' : 'Complete'}
              </Typography>
            </Stack>

            {/* Wallet sync information */}
            {walletSyncStatus && (
              <Stack spacing={0.5} sx={{ mt: 2, pt: 1, borderTop: 1, borderColor: 'divider' }}>
                <Typography variant="caption" color="text.secondary" fontWeight="medium">
                  Wallet Sync Status:
                </Typography>
                <Stack direction="row" justifyContent="space-between" alignItems="center">
                  <Typography variant="caption" color="text.secondary">
                    Wallet Block:
                  </Typography>
                  <Typography variant="caption" fontWeight="medium">
                    {walletSyncStatus.last_sync_block.toLocaleString()}
                  </Typography>
                </Stack>
                <Stack direction="row" justifyContent="space-between" alignItems="center">
                  <Typography variant="caption" color="text.secondary">
                    Wallet Behind:
                  </Typography>
                  <Typography 
                    variant="caption" 
                    fontWeight="medium"
                    color={
                      blockchainInfo.current_height - walletSyncStatus.last_sync_block > 0 
                        ? "warning.main" 
                        : "success.main"
                    }
                  >
                    {Math.max(0, blockchainInfo.current_height - walletSyncStatus.last_sync_block).toLocaleString()}
                  </Typography>
                </Stack>
              </Stack>
            )}
          </Box>
        </>
      )}
    </Paper>
  );
};

export default NetworkStatus;