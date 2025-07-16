import { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  ListItemIcon,
  Radio,
  Typography,
  Box,
  Chip,
  CircularProgress,
  Alert,
  useTheme
} from '@mui/material';
import { AccountBalanceWallet } from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { WalletAddress } from '../types/mining';

interface AddressSelectionDialogProps {
  open: boolean;
  onClose: () => void;
  onSelectAddress: (address: string) => void;
  currentAddress?: string;
}

export default function AddressSelectionDialog({
  open,
  onClose,
  onSelectAddress,
  currentAddress
}: AddressSelectionDialogProps) {
  const [addresses, setAddresses] = useState<WalletAddress[]>([]);
  const [selectedAddress, setSelectedAddress] = useState<string>(currentAddress || '');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';

  useEffect(() => {
    if (open) {
      fetchAddresses();
    }
  }, [open]);

  useEffect(() => {
    setSelectedAddress(currentAddress || '');
  }, [currentAddress]);

  const fetchAddresses = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<WalletAddress[]>('get_all_wallet_addresses');
      setAddresses(result);
      
      // If no current address is selected and we have addresses, select the first one
      if (!currentAddress && result.length > 0) {
        setSelectedAddress(result[0].address);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load wallet addresses');
    } finally {
      setLoading(false);
    }
  };

  const handleSelectAddress = (address: string) => {
    setSelectedAddress(address);
  };

  const handleConfirm = () => {
    if (selectedAddress) {
      onSelectAddress(selectedAddress);
      onClose();
    }
  };

  const handleCancel = () => {
    setSelectedAddress(currentAddress || '');
    onClose();
  };

  return (
    <Dialog
      open={open}
      onClose={handleCancel}
      maxWidth="sm"
      fullWidth
      PaperProps={{
        sx: {
          ...(isDarkMode ? {
            background: 'rgba(19, 47, 76, 0.95)',
            backdropFilter: 'blur(10px)',
            border: '1px solid rgba(255, 255, 255, 0.1)'
          } : {
            background: 'linear-gradient(135deg, #f5f7fa 0%, #ffffff 100%)',
            border: '1px solid rgba(0, 0, 0, 0.08)'
          })
        }
      }}
    >
      <DialogTitle>
        <Typography variant="h6" sx={{ fontWeight: 600 }}>
          Select Mining Reward Address
        </Typography>
        <Typography variant="body2" sx={{ 
          mt: 1, 
          color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)' 
        }}>
          Choose which address should receive mining rewards
        </Typography>
      </DialogTitle>
      
      <DialogContent sx={{ px: 3 }}>
        {loading && (
          <Box sx={{ display: 'flex', justifyContent: 'center', py: 3 }}>
            <CircularProgress size={24} />
          </Box>
        )}
        
        {error && (
          <Alert severity="error" sx={{ mb: 2 }}>
            {error}
          </Alert>
        )}
        
        {!loading && !error && addresses.length === 0 && (
          <Alert severity="info">
            No wallet addresses found. Please create a wallet first.
          </Alert>
        )}
        
        {!loading && addresses.length > 0 && (
          <List sx={{ py: 0 }}>
            {addresses.map((walletAddr) => (
              <ListItem key={`${walletAddr.wallet_name}-${walletAddr.address}`} disablePadding>
                <ListItemButton
                  onClick={() => handleSelectAddress(walletAddr.address)}
                  selected={selectedAddress === walletAddr.address}
                  sx={{
                    borderRadius: 1,
                    mb: 1,
                    border: selectedAddress === walletAddr.address ? 
                      `2px solid ${isDarkMode ? '#90caf9' : '#1976d2'}` : 
                      `1px solid ${isDarkMode ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.12)'}`,
                    '&:hover': {
                      backgroundColor: isDarkMode ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.04)'
                    }
                  }}
                >
                  <ListItemIcon>
                    <Radio
                      checked={selectedAddress === walletAddr.address}
                      onChange={() => handleSelectAddress(walletAddr.address)}
                      value={walletAddr.address}
                    />
                  </ListItemIcon>
                  
                  <ListItemIcon>
                    <AccountBalanceWallet 
                      sx={{ color: isDarkMode ? '#90caf9' : '#1976d2' }} 
                    />
                  </ListItemIcon>
                  
                  <ListItemText
                    primary={
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                        <Typography variant="subtitle2" sx={{ fontWeight: 600 }}>
                          {walletAddr.address}
                        </Typography>
                        {walletAddr.label && (
                          <Chip 
                            label={walletAddr.label} 
                            size="small" 
                            variant="outlined"
                            sx={{ fontSize: '0.75rem' }}
                          />
                        )}
                      </Box>
                    }
                    secondary={
                      <Box>
                        <Typography variant="body2" sx={{ 
                          color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)' 
                        }}>
                          Wallet: {walletAddr.wallet_name}
                        </Typography>
                        <Typography variant="caption" sx={{ 
                          color: isDarkMode ? 'rgba(255, 255, 255, 0.5)' : 'rgba(0, 0, 0, 0.4)' 
                        }}>
                          Path: {walletAddr.derivation_path}
                        </Typography>
                      </Box>
                    }
                  />
                </ListItemButton>
              </ListItem>
            ))}
          </List>
        )}
      </DialogContent>
      
      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button onClick={handleCancel} variant="outlined">
          Cancel
        </Button>
        <Button 
          onClick={handleConfirm} 
          variant="contained" 
          disabled={!selectedAddress || loading}
        >
          Select Address
        </Button>
      </DialogActions>
    </Dialog>
  );
}
