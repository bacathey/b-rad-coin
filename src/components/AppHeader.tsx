import {
  AppBar,
  Toolbar,
  Typography,
  IconButton,
  Tooltip,
  Box,
  CircularProgress,
  Chip
} from '@mui/material';
import MenuIcon from '@mui/icons-material/Menu';
import SettingsIcon from '@mui/icons-material/Settings';
import ExitToAppIcon from '@mui/icons-material/ExitToApp';
import DarkModeIcon from '@mui/icons-material/DarkMode';
import LightModeIcon from '@mui/icons-material/LightMode';
import bitcoinLogo from '../assets/bitcoin.svg';
import { useNavigate } from 'react-router-dom';
import { useWallet } from '../context/WalletContext';
import { invoke } from '@tauri-apps/api/core';

// Import the version from package.json
import packageJson from '../../package.json';

interface AppHeaderProps {
  mode: 'light' | 'dark';
  toggleColorMode: () => void;
  handleDrawerToggle: () => void;
}

export default function AppHeader({ mode, toggleColorMode, handleDrawerToggle }: AppHeaderProps) {
  const navigate = useNavigate();
  const appVersion = packageJson.version;
  const { isWalletOpen, setIsWalletOpen, isWalletLoading, currentWallet, setCurrentWallet } = useWallet();

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

  return (
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
        
        {/* Bitcoin logo */}
        <Box
          component="img"
          src={bitcoinLogo}
          alt="Bitcoin Logo"
          sx={{
            height: 28,
            width: 28,
            mr: 1.5,
            display: 'flex',
            alignItems: 'center'
          }}
        />
        
        <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
          {currentWallet ? currentWallet.name : 'B-Rad Coin'}
        </Typography>
        
        {/* Version number */}
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
        
        {/* Show loading state or close wallet button based on wallet state */}
        {isWalletLoading ? (
          <CircularProgress color="inherit" size={24} sx={{ mr: 1 }} />
        ) : (
          isWalletOpen && (
            <Tooltip title="Close Wallet">
              <IconButton 
                color="inherit"
                aria-label="close wallet"
                onClick={handleCloseWallet}
              >
                <ExitToAppIcon />
              </IconButton>
            </Tooltip>
          )
        )}
      </Toolbar>
    </AppBar>
  );
}