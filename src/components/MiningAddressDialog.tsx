import React, { useState, useEffect } from 'react';
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
  Typography,
  CircularProgress,
  Alert,
  Chip,
  Box,
  Divider
} from '@mui/material';
import { AccountBalanceWallet, ContentCopy } from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { WalletAddress } from '../types/mining';

interface MiningAddressDialogProps {
  open: boolean;
  onClose: () => void;
  onAddressSelected: (address: string) => void;
  currentAddress?: string;
}

export default function MiningAddressDialog({
  open,
  onClose,
  onAddressSelected,
  currentAddress
}: MiningAddressDialogProps) {
  const [addresses, setAddresses] = useState<WalletAddress[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedAddress, setSelectedAddress] = useState<string | null>(null);

  // Load all wallet addresses when dialog opens
  useEffect(() => {
    if (open) {
      loadAddresses();
      setSelectedAddress(currentAddress || null);
    }
  }, [open, currentAddress]);

  const loadAddresses = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const walletAddresses = await invoke<WalletAddress[]>('get_all_wallet_addresses');
      setAddresses(walletAddresses);
      
      if (walletAddresses.length === 0) {
        setError('No wallet addresses found. Please create a wallet first.');
      }
    } catch (err) {
      console.error('Failed to load wallet addresses:', err);
      setError(err instanceof Error ? err.message : 'Failed to load wallet addresses');
    } finally {
      setLoading(false);
    }
  };

  const handleAddressSelect = (address: string) => {
    setSelectedAddress(address);
  };

  const handleConfirm = () => {
    if (selectedAddress) {
      onAddressSelected(selectedAddress);
      onClose();
    }
  };

  const handleCopyAddress = async (address: string, event: React.MouseEvent) => {
    event.stopPropagation();
    try {
      await navigator.clipboard.writeText(address);
      // You could add a toast notification here
    } catch (err) {
      console.error('Failed to copy address:', err);
    }
  };

  // Group addresses by wallet for better organization
  const addressesByWallet = addresses.reduce((acc, addr) => {
    if (!acc[addr.wallet_name]) {
      acc[addr.wallet_name] = [];
    }
    acc[addr.wallet_name].push(addr);
    return acc;
  }, {} as Record<string, WalletAddress[]>);

  return (
    <Dialog 
      open={open} 
      onClose={onClose}
      maxWidth="md"
      fullWidth
      PaperProps={{
        sx: { minHeight: '60vh' }
      }}
    >
      <DialogTitle>
        <Box display="flex" alignItems="center" gap={1}>
          <AccountBalanceWallet />
          <Typography variant="h6">Select Mining Reward Address</Typography>
        </Box>
        <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
          Choose which address should receive mining rewards
        </Typography>
      </DialogTitle>

      <DialogContent dividers>
        {loading && (
          <Box display="flex" justifyContent="center" alignItems="center" py={4}>
            <CircularProgress />
            <Typography variant="body2" sx={{ ml: 2 }}>
              Loading wallet addresses...
            </Typography>
          </Box>
        )}

        {error && (
          <Alert severity="error" sx={{ mb: 2 }}>
            {error}
          </Alert>
        )}

        {!loading && !error && (
          <List sx={{ pt: 0 }}>
            {Object.entries(addressesByWallet).map(([walletName, walletAddresses], walletIndex) => (
              <Box key={walletName}>
                {walletIndex > 0 && <Divider sx={{ my: 2 }} />}
                
                {/* Wallet name header */}
                <Box sx={{ px: 2, py: 1, bgcolor: 'action.hover', borderRadius: 1, mb: 1 }}>
                  <Typography variant="subtitle2" fontWeight={600}>
                    {walletName}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    {walletAddresses.length} address{walletAddresses.length !== 1 ? 'es' : ''}
                  </Typography>
                </Box>

                {walletAddresses.map((addr) => (
                  <ListItem key={`${addr.wallet_name}-${addr.address}`} disablePadding>
                    <ListItemButton
                      selected={selectedAddress === addr.address}
                      onClick={() => handleAddressSelect(addr.address)}
                      sx={{
                        borderRadius: 1,
                        mx: 1,
                        mb: 1,
                        border: selectedAddress === addr.address ? 2 : 1,
                        borderColor: selectedAddress === addr.address ? 'primary.main' : 'divider',
                      }}
                    >
                      <ListItemIcon>
                        <AccountBalanceWallet 
                          color={selectedAddress === addr.address ? 'primary' : 'action'} 
                        />
                      </ListItemIcon>
                      
                      <ListItemText
                        primary={
                          <Box display="flex" alignItems="center" gap={1}>
                            <Typography 
                              variant="body2" 
                              fontFamily="monospace"
                              sx={{ 
                                wordBreak: 'break-all',
                                fontSize: '0.875rem'
                              }}
                            >
                              {addr.address}
                            </Typography>
                            <Button
                              size="small"
                              onClick={(e) => handleCopyAddress(addr.address, e)}
                              sx={{ minWidth: 'auto', p: 0.5 }}
                            >
                              <ContentCopy fontSize="small" />
                            </Button>
                          </Box>
                        }
                        secondary={
                          <Box display="flex" alignItems="center" gap={1} mt={0.5}>
                            {addr.label && (
                              <Chip 
                                label={addr.label} 
                                size="small" 
                                variant="outlined"
                                sx={{ height: 20, fontSize: '0.75rem' }}
                              />
                            )}
                            <Typography variant="caption" color="text.secondary">
                              {addr.derivation_path}
                            </Typography>
                            {addr.address === currentAddress && (
                              <Chip 
                                label="Current" 
                                size="small" 
                                color="primary"
                                sx={{ height: 20, fontSize: '0.75rem' }}
                              />
                            )}
                          </Box>
                        }
                      />
                    </ListItemButton>
                  </ListItem>
                ))}
              </Box>
            ))}
          </List>
        )}

        {!loading && !error && addresses.length === 0 && (
          <Box textAlign="center" py={4}>
            <Typography variant="body1" color="text.secondary">
              No wallet addresses found
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
              Create a wallet to add addresses for mining rewards
            </Typography>
          </Box>
        )}
      </DialogContent>

      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button onClick={onClose}>
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
