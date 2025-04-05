import {
  AppBar,
  Toolbar,
  Typography,
  IconButton,
  Tooltip,
  Box} from '@mui/material';
import MenuIcon from '@mui/icons-material/Menu';
import SettingsIcon from '@mui/icons-material/Settings';
import ExitToAppIcon from '@mui/icons-material/ExitToApp';
import DarkModeIcon from '@mui/icons-material/DarkMode';
import LightModeIcon from '@mui/icons-material/LightMode';
import LockIcon from '@mui/icons-material/Lock';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import { useNavigate } from 'react-router-dom';
import { useWallet } from '../context/WalletContext';
import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect } from 'react';
import SecureWalletDialog from './SecureWalletDialog';

interface AppHeaderProps {
  mode: 'light' | 'dark';
  toggleColorMode: () => void;
  handleDrawerToggle: () => void;
}

export default function AppHeader({ mode, toggleColorMode, handleDrawerToggle }: AppHeaderProps) {
  const navigate = useNavigate();
  const [appVersion, setAppVersion] = useState('');
  const { isWalletOpen, setIsWalletOpen, currentWallet, setCurrentWallet, isWalletSecured, refreshWalletDetails } = useWallet();
  
  // State for secure wallet dialog
  const [secureDialogOpen, setSecureDialogOpen] = useState(false);

  // Fetch app version from Rust backend on component mount
  useEffect(() => {
    const fetchAppVersion = async () => {
      try {
        const version = await invoke<string>('get_app_version');
        setAppVersion(version);
      } catch (error) {
        console.error('Failed to fetch app version:', error);
        setAppVersion('unknown');
      }
    };
    
    fetchAppVersion();
  }, []);

  // Function to handle closing the wallet
  const handleCloseWallet = async () => {
    try {
      // Call the Rust function to close the wallet
      await invoke('close_wallet');
      // Update React state
      setIsWalletOpen(false);
      setCurrentWallet(null);
      console.log('Wallet closed successfully');
    } catch (error) {
      console.error('Error closing wallet:', error);
    }
  };

  // Function to open secure wallet dialog when clicking on unsecured lock icon
  const handleOpenSecureDialog = () => {
    if (currentWallet && !isWalletSecured) {
      setSecureDialogOpen(true);
    }
  };

  return (
    <>
      <AppBar position="fixed" sx={{ zIndex: (theme) => theme.zIndex.drawer + 1 }}>
        <Toolbar>
          <IconButton
            color="inherit"
            aria-label="open drawer"
            edge="start"
            onClick={handleDrawerToggle}
            sx={{ mr: 2, display: { sm: 'none' } }}
          >
            <MenuIcon />
          </IconButton>
          
          {/* Bradcoin logo */}
          <Box
            component="img"
            src="/bradcoin.png"
            alt="Bradcoin Logo"
            sx={{
              height: 28,
              width: 28,
              mr: 1.5,
              display: 'flex',
              alignItems: 'center'
            }}
          />
          
          <Typography variant="h6" component="div" sx={{ flexGrow: 1, display: 'flex', alignItems: 'center' }}>
            {currentWallet ? (
              <>
                {currentWallet.name}
                {/* Show lock icon based on wallet security status */}
                {isWalletOpen && (
                  isWalletSecured ? (
                    <Tooltip title="This wallet is secured">
                      <Box component="span" sx={{ display: 'inline-flex', ml: 1 }}>
                        <LockIcon color="warning" fontSize="small" />
                      </Box>
                    </Tooltip>
                  ) : (
                    <Tooltip title="Click to add password protection">
                      <IconButton
                        size="small"
                        color="success"
                        onClick={handleOpenSecureDialog}
                        sx={{ ml: 0.5, p: 0.5 }}
                      >
                        <LockOpenIcon fontSize="small" />
                      </IconButton>
                    </Tooltip>
                  )
                )}
              </>
            ) : 'B-Rad Coin'}
          </Typography>
          
          {/* Version number from Tauri backend */}
          <Typography 
            variant="caption" 
            sx={{ 
              mr: 2, 
              opacity: 0.8,
              fontSize: '0.75rem',
              fontWeight: 500,
              display: 'flex',
              alignItems: 'center'
            }}
          >
            v{appVersion}
          </Typography>
          
          <Tooltip title={mode === 'dark' ? "Light mode" : "Dark mode"}>
            <IconButton 
              sx={{ mr: 1 }} 
              onClick={toggleColorMode} 
              color="inherit"
              aria-label="toggle theme"
            >
              {mode === 'dark' ? <LightModeIcon /> : <DarkModeIcon />}
            </IconButton>
          </Tooltip>
          <Tooltip title="Settings">
            <IconButton 
              color="inherit"
              aria-label="settings"
              onClick={() => navigate('/settings')}
              sx={{ mr: 1 }}
            >
              <SettingsIcon />
            </IconButton>
          </Tooltip>
          
          {/* Show close wallet button when a wallet is open */}
          {isWalletOpen && (
            <Tooltip title="Close Wallet">
              <IconButton 
                color="inherit"
                aria-label="close wallet"
                onClick={handleCloseWallet}
              >
                <ExitToAppIcon />
              </IconButton>
            </Tooltip>
          )}
        </Toolbar>
      </AppBar>
      
      {/* Secure Wallet Dialog */}
      {currentWallet && (
        <SecureWalletDialog
          open={secureDialogOpen}
          onClose={() => setSecureDialogOpen(false)}
          walletName={currentWallet.name}
          onSuccess={refreshWalletDetails}
        />
      )}
    </>
  );
}