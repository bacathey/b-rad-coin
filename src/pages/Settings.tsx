import { 
  Typography, 
  Box, 
  Card, 
  CardContent, 
  Switch,
  Divider,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  TextField,
  Button,
  Grid,
  useTheme
} from '@mui/material';
import SecurityIcon from '@mui/icons-material/Security';
import LanguageIcon from '@mui/icons-material/Language';
import NotificationsIcon from '@mui/icons-material/Notifications';
import BackupIcon from '@mui/icons-material/Backup';
import PrivacyTipIcon from '@mui/icons-material/PrivacyTip';
import FolderIcon from '@mui/icons-material/Folder';
import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";

export default function Settings() {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';

  // State for the various settings
  const [notificationsEnabled, setNotificationsEnabled] = useState(true);
  const [autoBackup, setAutoBackup] = useState(true);
  const [anonymousData, setAnonymousData] = useState(false);
  const [nodeAddress, setNodeAddress] = useState('');
  const [language] = useState('English');
  const [configDirectory, setConfigDirectory] = useState<string>('');

  useEffect(() => {
    // Fetch config directory when component mounts
    invoke('get_config_directory')
      .then((dir) => setConfigDirectory(dir as string))
      .catch(console.error);
  }, []);

  // Card style based on theme mode
  const cardStyle = isDarkMode ? {
    background: 'rgba(19, 47, 76, 0.6)',
    backdropFilter: 'blur(10px)',
    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
    border: '1px solid rgba(255, 255, 255, 0.1)'
  } : {
    background: 'linear-gradient(180deg, #ffffff 0%, #f5f7fa 100%)',
    boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
    border: '1px solid rgba(0, 0, 0, 0.08)'
  };

  return (
    <Box 
      sx={{ 
        width: '100%',
        maxWidth: '100%',
        pt: 3,
        px: { xs: 2, sm: 3 },
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center'
      }}
    >
      <Typography 
        variant="h4" 
        component="h1" 
        gutterBottom
        sx={{
          color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#1a237e',
          textShadow: isDarkMode ? '0 2px 10px rgba(0,0,0,0.3)' : 'none',
          fontWeight: 600,
          mb: 3
        }}
      >
        Settings
      </Typography>
      
      <Grid container spacing={3} sx={{ width: '100%', maxWidth: 1200, mx: 'auto' }}>
        {/* General Settings */}
        <Grid item xs={12} md={6}>
          <Card sx={{ ...cardStyle, height: '100%' }}>
            <CardContent>
              <Typography 
                variant="h6" 
                sx={{
                  color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#1a237e',
                  fontWeight: 600,
                  mb: 2
                }}
              >
                General Settings
              </Typography>
              
              <List>
                <ListItem>
                  <ListItemIcon>
                    <NotificationsIcon 
                      color={isDarkMode ? "primary" : "primary"} 
                    />
                  </ListItemIcon>
                  <ListItemText 
                    primary="Notifications" 
                    secondary="Enable or disable notifications" 
                  />
                  <Switch 
                    checked={notificationsEnabled}
                    onChange={(e) => setNotificationsEnabled(e.target.checked)}
                    color="primary"
                  />
                </ListItem>
                <Divider variant="inset" component="li" />
                
                <ListItem>
                  <ListItemIcon>
                    <FolderIcon 
                      color={isDarkMode ? "primary" : "primary"} 
                    />
                  </ListItemIcon>
                  <ListItemText 
                    primary="Config Directory" 
                    secondary={configDirectory || 'Loading...'} 
                  />
                </ListItem>
                <Divider variant="inset" component="li" />
                
                <ListItem>
                  <ListItemIcon>
                    <LanguageIcon 
                      color={isDarkMode ? "primary" : "primary"} 
                    />
                  </ListItemIcon>
                  <ListItemText 
                    primary="Language" 
                    secondary={language} 
                  />
                  <Button color="primary" variant="outlined" size="small">
                    Change
                  </Button>
                </ListItem>
                <Divider variant="inset" component="li" />
                
                <ListItem>
                  <ListItemIcon>
                    <BackupIcon 
                      color={isDarkMode ? "primary" : "primary"} 
                    />
                  </ListItemIcon>
                  <ListItemText 
                    primary="Automatic Backup" 
                    secondary="Back up wallet data automatically" 
                  />
                  <Switch 
                    checked={autoBackup}
                    onChange={(e) => setAutoBackup(e.target.checked)}
                    color="primary"
                  />
                </ListItem>
              </List>
            </CardContent>
          </Card>
        </Grid>
        
        {/* Advanced Settings */}
        <Grid item xs={12} md={6}>
          <Card sx={{ ...cardStyle, height: '100%' }}>
            <CardContent>
              <Typography 
                variant="h6" 
                sx={{
                  color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#1a237e',
                  fontWeight: 600,
                  mb: 2
                }}
              >
                Advanced Settings
              </Typography>
              
              <List>
                <ListItem>
                  <ListItemIcon>
                    <SecurityIcon 
                      color={isDarkMode ? "primary" : "primary"} 
                    />
                  </ListItemIcon>
                  <ListItemText 
                    primary="Security Settings" 
                    secondary="Configure 2FA and security options" 
                  />
                  <Button color="primary" variant="outlined" size="small">
                    Manage
                  </Button>
                </ListItem>
                <Divider variant="inset" component="li" />
                
                <ListItem sx={{ alignItems: 'flex-start' }}>
                  <ListItemIcon sx={{ mt: 1 }}>
                    <PrivacyTipIcon 
                      color={isDarkMode ? "primary" : "primary"} 
                    />
                  </ListItemIcon>
                  <Box sx={{ width: '100%' }}>
                    <ListItemText 
                      primary="Custom Node" 
                      secondary="Connect to your own Bitcoin node" 
                      sx={{ mb: 1 }}
                    />
                    <TextField
                      fullWidth
                      size="small"
                      placeholder="node.example.com:8333"
                      value={nodeAddress}
                      onChange={(e) => setNodeAddress(e.target.value)}
                      sx={{ mb: 1 }}
                    />
                    <Button color="primary" variant="contained" size="small">
                      Connect
                    </Button>
                  </Box>
                </ListItem>
                <Divider variant="inset" component="li" />
                
                <ListItem>
                  <ListItemIcon>
                    <PrivacyTipIcon 
                      color={isDarkMode ? "primary" : "primary"} 
                    />
                  </ListItemIcon>
                  <ListItemText 
                    primary="Usage Data" 
                    secondary="Send anonymous usage data" 
                  />
                  <Switch 
                    checked={anonymousData}
                    onChange={(e) => setAnonymousData(e.target.checked)}
                    color="primary"
                  />
                </ListItem>
              </List>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );
}