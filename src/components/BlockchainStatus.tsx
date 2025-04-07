import React, { useState, useEffect } from 'react';
import { Box, LinearProgress, Typography, Paper, Stack, Chip } from '@mui/material';
import { useThemeMode } from '../hooks/useThemeMode';
import CloudDoneIcon from '@mui/icons-material/CloudDone';
import CloudOffIcon from '@mui/icons-material/CloudOff';
import SyncIcon from '@mui/icons-material/Sync';
import PeopleIcon from '@mui/icons-material/People';

interface BlockchainStatusProps {
  className?: string;
}

interface BlockchainInfo {
  connected: boolean;
  syncProgress: number;
  peers: number;
}

export const BlockchainStatus: React.FC<BlockchainStatusProps> = ({ className }) => {
  const { isDarkMode } = useThemeMode();
  const [blockchainInfo, setBlockchainInfo] = useState<BlockchainInfo>({
    connected: false,
    syncProgress: 0,
    peers: 0,
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchBlockchainStatus = async () => {
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
        console.error('Failed to fetch blockchain status:', error);
        setLoading(false);
      }
    };

    fetchBlockchainStatus();
    const intervalId = setInterval(fetchBlockchainStatus, 10000); // Update every 10 seconds
    
    return () => clearInterval(intervalId);
  }, []);

  return (
    <Paper 
      className={className}
      elevation={2}
      sx={{ 
        p: 2, 
        borderRadius: 2,
        backgroundColor: isDarkMode ? 'rgba(30, 30, 30, 0.8)' : 'rgba(255, 255, 255, 0.8)',
        backdropFilter: 'blur(10px)',
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
                color="error" 
                size="small" 
                variant="outlined" 
              />
            }
            
            <Chip 
              icon={<PeopleIcon />} 
              label={`${blockchainInfo.peers} Peers`} 
              color={blockchainInfo.peers > 0 ? "primary" : "warning"} 
              size="small" 
              variant="outlined"
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
              sx={{ height: 8, borderRadius: 2 }}
            />
            <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mt: 1 }}>
              <Typography variant="caption">
                {blockchainInfo.syncProgress < 100 ? 
                  'Synchronizing...' : 
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

export default BlockchainStatus;