import {
  AppBar,
  Toolbar,
  Typography,
  IconButton,
  Tooltip,
  Box
} from '@mui/material';
import MenuIcon from '@mui/icons-material/Menu';
import SettingsIcon from '@mui/icons-material/Settings';
import DarkModeIcon from '@mui/icons-material/DarkMode';
import LightModeIcon from '@mui/icons-material/LightMode';
import bitcoinLogo from '../assets/bitcoin.svg';
import { useNavigate } from 'react-router-dom';

interface AppHeaderProps {
  mode: 'light' | 'dark';
  toggleColorMode: () => void;
  handleDrawerToggle: () => void;
}

export default function AppHeader({ mode, toggleColorMode, handleDrawerToggle }: AppHeaderProps) {
  const navigate = useNavigate();

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
          MyWallet
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
          >
            <SettingsIcon />
          </IconButton>
        </Tooltip>
      </Toolbar>
    </AppBar>
  );
}