import { useLocation, useNavigate } from "react-router-dom";
import {
  Typography,
  Toolbar,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Divider,
  Box
} from '@mui/material';
import AccountCircleIcon from '@mui/icons-material/AccountCircle';
import SwapHorizIcon from '@mui/icons-material/SwapHoriz';
import ReceiptLongIcon from '@mui/icons-material/ReceiptLong';
import MiningIcon from '@mui/icons-material/Hardware';
import CloseIcon from '@mui/icons-material/Close';

// Import the version from package.json
import packageJson from '../../package.json';

interface SidebarProps {
  mode: 'light' | 'dark';
  mobileOpen: boolean;
  handleDrawerToggle: () => void;
}

export default function Sidebar({ mode, mobileOpen, handleDrawerToggle }: SidebarProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const appVersion = packageJson.version;

  return (
    <Box sx={{ 
      display: 'flex', 
      flexDirection: 'column', 
      height: '100%'
    }}>
      <Toolbar sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', padding: '16px' }}>
        <Typography variant="h6" noWrap component="div" sx={{ 
          fontWeight: 600,
          color: mode === 'light' ? '#1a237e' : undefined 
        }}>
          Navigation
        </Typography>
      </Toolbar>
      <Divider />
      <List>
        <ListItem disablePadding>
          <ListItemButton 
            selected={location.pathname === '/'} 
            onClick={() => {
              navigate('/');
              if (mobileOpen) handleDrawerToggle();
            }}
            sx={mode === 'light' && location.pathname === '/' ? {
              borderLeft: '4px solid #1a237e'
            } : {}}
          >
            <ListItemIcon sx={mode === 'light' ? {
              color: location.pathname === '/' ? '#1a237e' : 'rgba(0, 0, 0, 0.54)'
            } : {}}>
              <AccountCircleIcon />
            </ListItemIcon>
            <ListItemText primary="Account" sx={mode === 'light' && location.pathname === '/' ? {
              color: '#1a237e',
              fontWeight: 500
            } : {}} />
          </ListItemButton>
        </ListItem>
        <ListItem disablePadding>
          <ListItemButton 
            selected={location.pathname === '/transactions'} 
            onClick={() => {
              navigate('/transactions');
              if (mobileOpen) handleDrawerToggle();
            }}
            sx={mode === 'light' && location.pathname === '/transactions' ? {
              borderLeft: '4px solid #1a237e'
            } : {}}
          >
            <ListItemIcon sx={mode === 'light' ? {
              color: location.pathname === '/transactions' ? '#1a237e' : 'rgba(0, 0, 0, 0.54)'
            } : {}}>
              <ReceiptLongIcon />
            </ListItemIcon>
            <ListItemText primary="Transactions" sx={mode === 'light' && location.pathname === '/transactions' ? {
              color: '#1a237e',
              fontWeight: 500
            } : {}} />
          </ListItemButton>
        </ListItem>
        <ListItem disablePadding>
          <ListItemButton 
            selected={location.pathname === '/send-receive'} 
            onClick={() => {
              navigate('/send-receive');
              if (mobileOpen) handleDrawerToggle();
            }}
            sx={mode === 'light' && location.pathname === '/send-receive' ? {
              borderLeft: '4px solid #1a237e'
            } : {}}
          >
            <ListItemIcon sx={mode === 'light' ? {
              color: location.pathname === '/send-receive' ? '#1a237e' : 'rgba(0, 0, 0, 0.54)'
            } : {}}>
              <SwapHorizIcon />
            </ListItemIcon>
            <ListItemText primary="Send/Receive" sx={mode === 'light' && location.pathname === '/send-receive' ? {
              color: '#1a237e',
              fontWeight: 500
            } : {}} />
          </ListItemButton>
        </ListItem>
        <ListItem disablePadding>
          <ListItemButton 
            selected={location.pathname === '/advanced'} 
            onClick={() => {
              navigate('/advanced');
              if (mobileOpen) handleDrawerToggle();
            }}
            sx={mode === 'light' && location.pathname === '/advanced' ? {
              borderLeft: '4px solid #1a237e'
            } : {}}
          >
            <ListItemIcon sx={mode === 'light' ? {
              color: location.pathname === '/advanced' ? '#1a237e' : 'rgba(0, 0, 0, 0.54)'
            } : {}}>
              <MiningIcon />
            </ListItemIcon>
            <ListItemText primary="Advanced" sx={mode === 'light' && location.pathname === '/advanced' ? {
              color: '#1a237e',
              fontWeight: 500
            } : {}} />
          </ListItemButton>
        </ListItem>
        <ListItem disablePadding>
          <ListItemButton>
            <ListItemIcon>

              <CloseIcon />
            </ListItemIcon>
            <ListItemText primary="Close Wallet" />
          </ListItemButton>
        </ListItem>
      </List>
      
      {/* Push the version to the bottom with flexbox */}
      <Box sx={{ flexGrow: 1 }} />
      
      {/* Version display */}
      <Box sx={{ 
        padding: '12px 16px', 
        textAlign: 'center',
        borderTop: 1,
        borderColor: mode === 'dark' ? 'rgba(255, 255, 255, 0.12)' : 'rgba(0, 0, 0, 0.12)'
      }}>
        <Typography 
          variant="caption" 
          sx={{ 
            opacity: 0.7,
            color: mode === 'dark' ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)',
            fontWeight: 500
          }}
        >
          Version {appVersion}
        </Typography>
      </Box>
    </Box>
  );
}