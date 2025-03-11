import { useState, useEffect } from 'react';
import { 
  Dialog, 
  DialogTitle, 
  DialogContent, 
  DialogActions, 
  Button, 
  FormControl, 
  InputLabel, 
  Select, 
  MenuItem, 
  Typography, 
  Box, 
  CircularProgress,
  useTheme,
  SelectChangeEvent,
  Backdrop
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { useWallet } from '../context/WalletContext';
import AccountBalanceWalletIcon from '@mui/icons-material/AccountBalanceWallet';

export default function WalletDialog() {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  const { isWalletOpen, setIsWalletOpen, setCurrentWallet } = useWallet();
  
  const [open, setOpen] = useState(!isWalletOpen);
  const [selectedWallet, setSelectedWallet] = useState('');
  const [availableWallets, setAvailableWallets] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isGettingWallets, setIsGettingWallets] = useState(true);

  // Fetch available wallets when the dialog opens
  useEffect(() => {
    async function fetchWallets() {
      try {
        const wallets = await invoke<string[]>('get_available_wallets');
        setAvailableWallets(wallets);
      } catch (error) {
        console.error('Failed to fetch available wallets:', error);
        setAvailableWallets([]);
      } finally {
        setIsGettingWallets(false);
      }
    }

    if (open) {
      fetchWallets();
    }
  }, [open]);

  // Update open state based on wallet status
  useEffect(() => {
    setOpen(!isWalletOpen);
  }, [isWalletOpen]);

  const handleWalletChange = (event: SelectChangeEvent) => {
    setSelectedWallet(event.target.value as string);
  };

  const handleOpenWallet = async () => {
    if (!selectedWallet) return;

    setIsLoading(true);
    try {
      const result = await invoke<boolean>('open_wallet', { walletName: selectedWallet });
      if (result) {
        // Set both the wallet open state and the current wallet info
        setIsWalletOpen(true);
        setCurrentWallet({
          name: selectedWallet
        });
        setOpen(false);
      }
    } catch (error) {
      console.error('Failed to open wallet:', error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <>
      {/* Backdrop that covers the entire app when wallet is not open */}
      <Backdrop 
        open={open} 
        sx={{ 
          // Use a higher z-index to cover the AppBar and Drawer
          zIndex: theme.zIndex.drawer + 2,
          background: isDarkMode 
            ? 'linear-gradient(145deg, #0a1929 0%, #0d2b59 50%, rgb(13, 75, 116) 100%)' 
            : 'linear-gradient(145deg, #f5f7fa 0%, #ffffff 100%)',
          backdropFilter: 'blur(5px)',
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0
        }}
      />
      
      <Dialog 
        open={open} 
        maxWidth="sm" 
        fullWidth 
        disableEscapeKeyDown
        // Ensure dialog appears above the backdrop
        sx={{
          zIndex: theme.zIndex.drawer + 3,
          '& .MuiPaper-root': {
            background: isDarkMode 
              ? 'linear-gradient(145deg, #0a1929 0%, #132f4c 100%)' 
              : 'linear-gradient(145deg, #ffffff 0%, #f5f7fa 100%)',
            borderRadius: '12px',
            boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
            border: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.08)'
          }
        }}
      >
        <DialogTitle sx={{ 
          display: 'flex', 
          alignItems: 'center', 
          gap: 1,
          pb: 1
        }}>
          <AccountBalanceWalletIcon 
            color="primary" 
            fontSize="large" 
            sx={{ mr: 1 }} 
          />
          <Typography variant="h5" component="div" fontWeight={600}>
            B-Rad Coin Wallet
          </Typography>
        </DialogTitle>
        
        <DialogContent>
          <Typography variant="body1" sx={{ mb: 3 }}>
            Please select a wallet to open:
          </Typography>
          
          {isGettingWallets ? (
            <Box sx={{ display: 'flex', justifyContent: 'center', py: 3 }}>
              <CircularProgress />
            </Box>
          ) : (
            <FormControl fullWidth sx={{ mb: 2 }}>
              <InputLabel id="wallet-select-label">Select Wallet</InputLabel>
              <Select
                labelId="wallet-select-label"
                id="wallet-select"
                value={selectedWallet}
                label="Select Wallet"
                onChange={handleWalletChange}
              >
                {availableWallets.map((wallet) => (
                  <MenuItem key={wallet} value={wallet}>
                    {wallet}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          )}
        </DialogContent>
        
        <DialogActions sx={{ px: 3, pb: 3 }}>
          <Button 
            variant="contained"
            color="primary"
            onClick={handleOpenWallet}
            disabled={!selectedWallet || isLoading}
            startIcon={isLoading ? <CircularProgress size={20} /> : null}
            sx={{ 
              minWidth: '120px',
              textTransform: 'none',
              fontWeight: 600
            }}
          >
            {isLoading ? 'Opening...' : 'Open Wallet'}
          </Button>
        </DialogActions>
      </Dialog>
    </>
  );
}