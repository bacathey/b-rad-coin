import {
  Box,
  Typography,
  Stack,
  LinearProgress
} from '@mui/material';

interface BlockchainStatusProps {
  mode: 'light' | 'dark';
}

export default function BlockchainStatus({ mode }: BlockchainStatusProps) {
  // Sample blockchain status data - in a real application this would come from your blockchain API
  const blockchainStatus = {
    blocksProgress: 85, // percentage
    blocksText: "682,049 / 800,281",
    agentProgress: 92, // percentage
    agentText: "Active (6 connections)"
  };

  return (
    <Box sx={{ 
      padding: '16px', 
      borderTop: 1,
      borderColor: mode === 'dark' ? 'rgba(255, 255, 255, 0.12)' : 'rgba(0, 0, 0, 0.12)'
    }}>
      <Typography 
        variant="subtitle2" 
        sx={{ 
          color: mode === 'dark' ? 'rgba(255, 255, 255, 0.9)' : '#1a237e',
          fontWeight: 600,
          mb: 2
        }}
      >
        Blockchain Sync Status
      </Typography>
      
      <Stack spacing={2}>
        {/* Blocks Progress */}
        <Box>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 0.5 }}>
            <Typography 
              variant="caption" 
              sx={{ 
                color: mode === 'dark' ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)',
                fontWeight: 500
              }}
            >
              Blocks
            </Typography>
            <Typography 
              variant="caption" 
              sx={{ 
                color: mode === 'dark' ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)',
                fontWeight: 500
              }}
            >
              {blockchainStatus.blocksText}
            </Typography>
          </Box>
          <LinearProgress 
            variant="determinate" 
            value={blockchainStatus.blocksProgress} 
            sx={{
              borderRadius: 1,
              height: 6,
              backgroundColor: mode === 'dark' ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.1)',
              '& .MuiLinearProgress-bar': {
                backgroundColor: mode === 'dark' ? '#64b5f6' : '#1a237e',
              }
            }}
          />
        </Box>
        
        {/* Agent Progress */}
        <Box>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 0.5 }}>
            <Typography 
              variant="caption" 
              sx={{ 
                color: mode === 'dark' ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)',
                fontWeight: 500
              }}
            >
              Agent
            </Typography>
            <Typography 
              variant="caption" 
              sx={{ 
                color: mode === 'dark' ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)',
                fontWeight: 500
              }}
            >
              {blockchainStatus.agentText}
            </Typography>
          </Box>
          <LinearProgress 
            variant="determinate" 
            value={blockchainStatus.agentProgress} 
            sx={{
              borderRadius: 1,
              height: 6,
              backgroundColor: mode === 'dark' ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.1)',
              '& .MuiLinearProgress-bar': {
                backgroundColor: mode === 'dark' ? '#81c784' : '#2e7d32',
              }
            }}
          />
        </Box>
      </Stack>
    </Box>
  );
}