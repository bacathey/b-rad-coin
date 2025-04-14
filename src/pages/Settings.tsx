import { 
  Switch,
  Divider,
  List,
  TextField,
  Button,
  Grid
} from '@mui/material';
import SecurityIcon from '@mui/icons-material/Security';
import LanguageIcon from '@mui/icons-material/Language';
import NotificationsIcon from '@mui/icons-material/Notifications';
import BackupIcon from '@mui/icons-material/Backup';
import PrivacyTipIcon from '@mui/icons-material/PrivacyTip';
import FolderIcon from '@mui/icons-material/Folder';
import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { StyledCard } from '../components/ui/StyledCard';
import { SettingsItem } from '../components/ui/SettingsItem';
import { PageContainer } from '../components/ui/PageContainer';
import { FormField } from '../components/ui/FormField';
import { useThemeMode } from '../hooks/useThemeMode';
import { useForm } from '../hooks/useForm';

export default function Settings() {
  const { getTextFieldStyle } = useThemeMode();
  
  // State for the various settings
  const [notificationsEnabled, setNotificationsEnabled] = useState(true);
  const [autoBackup, setAutoBackup] = useState(true);
  const [anonymousData, setAnonymousData] = useState(false);
  const [language] = useState('English');
  const [configDirectory, setConfigDirectory] = useState<string>('');
  const [error, setError] = useState<string | null>(null);

  // Use our custom form hook for the custom node form
  const nodeForm = useForm(
    { nodeAddress: '' },
    {
      nodeAddress: (value) => {
        if (!value) return null;
        // Simple validation for demonstration - could be more robust
        if (!value.includes(':')) return 'Node address should include a port number';
        return null;
      }
    },
    async (values) => {
      console.log('Connecting to node:', values.nodeAddress);
      // In a real app, we would connect to the node here
    }
  );

  useEffect(() => {
    // Fetch config directory when component mounts
    invoke('get_config_directory')
      .then((dir) => setConfigDirectory(dir as string))
      .catch(err => {
        console.error(err);
        setError('Failed to load configuration directory');
      });
  }, []);

  return (
    <PageContainer 
      title="Settings" 
      error={error}
    >
      <Grid container spacing={3} sx={{ width: '100%' }}>
        {/* General Settings */}
        <Grid item xs={12} md={6}>
          <StyledCard title="General Settings" fullHeight>
            <List>
              <SettingsItem
                icon={<NotificationsIcon color="primary" />}
                primary="Notifications"
                secondary="Enable or disable notifications"
                action={
                  <Switch 
                    checked={notificationsEnabled}
                    onChange={(e) => setNotificationsEnabled(e.target.checked)}
                    color="primary"
                  />
                }
              />
              <Divider variant="inset" component="li" />
              
              <SettingsItem
                icon={<FolderIcon color="primary" />}
                primary="Config Directory"
                secondary={configDirectory || 'Loading...'}
              />
              <Divider variant="inset" component="li" />
              
              <SettingsItem
                icon={<LanguageIcon color="primary" />}
                primary="Language"
                secondary={language}
                action={
                  <Button color="primary" variant="outlined" size="small">
                    Change
                  </Button>
                }
              />
              <Divider variant="inset" component="li" />
              
              <SettingsItem
                icon={<BackupIcon color="primary" />}
                primary="Automatic Backup"
                secondary="Back up wallet data automatically"
                action={
                  <Switch 
                    checked={autoBackup}
                    onChange={(e) => setAutoBackup(e.target.checked)}
                    color="primary"
                  />
                }
              />
            </List>
          </StyledCard>
        </Grid>
        
        {/* Advanced Settings */}
        <Grid item xs={12} md={6}>
          <StyledCard title="Advanced Settings" fullHeight>
            <List>
              <SettingsItem
                icon={<SecurityIcon color="primary" />}
                primary="Security Settings"
                secondary="Configure 2FA and security options"
                action={
                  <Button color="primary" variant="outlined" size="small">
                    Manage
                  </Button>
                }
              />
              <Divider variant="inset" component="li" />
              
              <SettingsItem
                icon={<PrivacyTipIcon color="primary" />}
                primary="Custom Node"
                secondary="Connect to your own Bitcoin node"
                extraContent={
                  <form onSubmit={nodeForm.handleSubmit}>
                    <FormField 
                      label="Node Address"
                      helperText={
                        nodeForm.touched.nodeAddress && nodeForm.errors.nodeAddress
                          ? nodeForm.errors.nodeAddress
                          : "Enter your node address with port (e.g. node.example.com:8333)"
                      }
                      error={!!(nodeForm.touched.nodeAddress && nodeForm.errors.nodeAddress)}
                      marginBottom={1}
                    >
                      <TextField
                        fullWidth
                        size="small"
                        name="nodeAddress"
                        placeholder="node.example.com:8333"
                        value={nodeForm.values.nodeAddress}
                        onChange={nodeForm.handleChange}
                        error={!!(nodeForm.touched.nodeAddress && nodeForm.errors.nodeAddress)}
                        sx={getTextFieldStyle()}
                      />
                    </FormField>
                    <Button 
                      color="primary" 
                      variant="contained" 
                      size="small"
                      type="submit"
                      disabled={nodeForm.isSubmitting}
                    >
                      Connect
                    </Button>
                  </form>
                }
              />
              <Divider variant="inset" component="li" />
              
              <SettingsItem
                icon={<PrivacyTipIcon color="primary" />}
                primary="Usage Data"
                secondary="Send anonymous usage data"
                action={
                  <Switch 
                    checked={anonymousData}
                    onChange={(e) => setAnonymousData(e.target.checked)}
                    color="primary"
                  />
                }
              />
            </List>
          </StyledCard>
        </Grid>
      </Grid>
    </PageContainer>
  );
}