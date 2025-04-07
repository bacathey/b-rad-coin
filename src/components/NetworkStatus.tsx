// filepath: c:\Users\bacat\source\repos\b-rad-coin\src\components\NetworkStatus.tsx
import React, { useState, useEffect } from 'react';
import { Box, LinearProgress, Typography, Paper, Stack, Chip } from '@mui/material';
import { useThemeMode } from '../hooks/useThemeMode';
import CloudDoneIcon from '@mui/icons-material/CloudDone';
import CloudOffIcon from '@mui/icons-material/CloudOff';
import SyncIcon from '@mui/icons-material/Sync';
import PeopleIcon from '@mui/icons-material/People';

interface NetworkStatusProps {
  className?: string;
}

interface BlockchainInfo {
  connected: boolean;
  syncProgress: number;
  peers: number;
}

export const NetworkStatus: React.FC<NetworkStatusProps> = ({ className }) => {
  const { isDarkMode } = useThemeMode();
  const [blockchainInfo, setBlockchainInfo] = useState<BlockchainInfo>({
    connected: false,
    syncProgress: 0,
    peers: 0,
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchNetworkStatus = async () => {
      try {
        // In real implementation, this would call a Tauri command to get blockchain info
        // const result = await invoke('get_blockchain_status');
        
        // Mock data for now
        const mockData: BlockchainInfo = {
          connected: true,
          syncProgress: Math.floor(Math.random() * 30) + 70, // 70-100% sync
          peers: Math.floor(Math.random() * 5) + 4, // 4-8 peers
        };
        
        setBlockchainInfo(mockData);
        setLoading(false);
        
      } catch (error) {
        console.error('Failed to fetch network status:', error);
        setLoading(false);
      }
    };

    fetchNetworkStatus();
    const intervalId = setInterval(fetchNetworkStatus, 10000); // Update every 10 seconds
    
    return () => clearInterval(intervalId);
  }, []);

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
        <>
          <Stack direction="row" spacing={1} alignItems="center" sx={{ mb: 2 }}>
            {blockchainInfo.connected ? 
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
              label={`${blockchainInfo.peers} Peers`} 
              color={blockchainInfo.peers > 0 ? "primary" : "warning"} 
              size="small" 
              variant="outlined"
              sx={{
                borderColor: isDarkMode ? undefined : "rgba(25, 118, 210, 0.7)", // Customized for light mode
                color: isDarkMode ? undefined : "rgba(25, 118, 210, 0.9)"
              }}
            />
          </Stack>
          
          <Box sx={{ mb: 1 }}>
            <Typography variant="subtitle2" sx={{ mb: 1 }}>
              <SyncIcon fontSize="small" sx={{ verticalAlign: 'middle', mr: 0.5 }} />
              Synchronization
            </Typography>
            <LinearProgress 
              variant="determinate" 
              value={blockchainInfo.syncProgress} 
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
                {blockchainInfo.syncProgress < 100 ? 
                  'XXX blocks remaining' : 
                  'Fully synchronized'}
              </Typography>
              <Typography variant="caption">
                {blockchainInfo.syncProgress}%
              </Typography>
            </Stack>
          </Box>
        </>
      )}
    </Paper>
  );
};

export default NetworkStatus;