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
import NetworkStatus from './NetworkStatus';
import { transitions } from '../styles/themeConstants';

interface SidebarProps {
  mode: 'light' | 'dark';
  mobileOpen: boolean;
  handleDrawerToggle: () => void;
}

export default function Sidebar({ mode, mobileOpen, handleDrawerToggle }: SidebarProps) {
  const location = useLocation();
  const navigate = useNavigate();

  return (
    <Box sx={{ 
      display: 'flex', 
      flexDirection: 'column', 
      height: '100%'
    }}>
      <Toolbar sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', padding: '16px' }}>
        <Typography variant="h6" noWrap component="div" sx={{ 
          fontWeight: 600,
          color: mode === 'light' ? '#1a237e' : undefined,
          transition: `${transitions.color}, ${transitions.fontWeight}`
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
              borderLeft: '4px solid #1a237e',
              transition: transitions.all
            } : {
              transition: transitions.all
            }}
          >
            <ListItemIcon sx={mode === 'light' ? {
              color: location.pathname === '/' ? '#1a237e' : 'rgba(0, 0, 0, 0.54)',
              transition: transitions.color
            } : {
              transition: transitions.color
            }}>
              <AccountCircleIcon />
            </ListItemIcon>
            <ListItemText 
              primary="Account" 
              primaryTypographyProps={{
                sx: {
                  color: mode === 'light' && location.pathname === '/' ? '#1a237e' : undefined,
                  fontWeight: location.pathname === '/' ? 500 : 400,
                  transition: `${transitions.color}, ${transitions.fontWeight}`
                }
              }}
            />
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
              borderLeft: '4px solid #1a237e',
              transition: transitions.all
            } : {
              transition: transitions.all
            }}
          >
            <ListItemIcon sx={mode === 'light' ? {
              color: location.pathname === '/transactions' ? '#1a237e' : 'rgba(0, 0, 0, 0.54)',
              transition: transitions.color
            } : {
              transition: transitions.color
            }}>
              <ReceiptLongIcon />
            </ListItemIcon>
            <ListItemText 
              primary="Transactions" 
              primaryTypographyProps={{
                sx: {
                  color: mode === 'light' && location.pathname === '/transactions' ? '#1a237e' : undefined,
                  fontWeight: location.pathname === '/transactions' ? 500 : 400,
                  transition: `${transitions.color}, ${transitions.fontWeight}`
                }
              }}
            />
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
              borderLeft: '4px solid #1a237e',
              transition: transitions.all
            } : {
              transition: transitions.all
            }}
          >
            <ListItemIcon sx={mode === 'light' ? {
              color: location.pathname === '/send-receive' ? '#1a237e' : 'rgba(0, 0, 0, 0.54)',
              transition: transitions.color
            } : {
              transition: transitions.color
            }}>
              <SwapHorizIcon />
            </ListItemIcon>
            <ListItemText 
              primary="Send/Receive" 
              primaryTypographyProps={{
                sx: {
                  color: mode === 'light' && location.pathname === '/send-receive' ? '#1a237e' : undefined,
                  fontWeight: location.pathname === '/send-receive' ? 500 : 400,
                  transition: `${transitions.color}, ${transitions.fontWeight}`
                }
              }}
            />
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
              borderLeft: '4px solid #1a237e',
              transition: transitions.all
            } : {
              transition: transitions.all
            }}
          >
            <ListItemIcon sx={mode === 'light' ? {
              color: location.pathname === '/advanced' ? '#1a237e' : 'rgba(0, 0, 0, 0.54)',
              transition: transitions.color
            } : {
              transition: transitions.color
            }}>
              <MiningIcon />
            </ListItemIcon>
            <ListItemText 
              primary="Advanced" 
              primaryTypographyProps={{
                sx: {
                  color: mode === 'light' && location.pathname === '/advanced' ? '#1a237e' : undefined,
                  fontWeight: location.pathname === '/advanced' ? 500 : 400,
                  transition: `${transitions.color}, ${transitions.fontWeight}`
                }
              }}
            />
          </ListItemButton>
        </ListItem>
      </List>
      
      {/* Push the network status to the bottom with flexbox */}
      <Box sx={{ flexGrow: 1 }} />
      
      {/* Use the NetworkStatus component */}
      <NetworkStatus className="sidebar-network-status" />
    </Box>
  );
}