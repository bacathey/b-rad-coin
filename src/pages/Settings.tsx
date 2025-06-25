import React from 'react';
import { 
  Switch,
  Divider,
  List,
  Button,
  Grid
} from '@mui/material';
import SecurityIcon from '@mui/icons-material/Security';
import LanguageIcon from '@mui/icons-material/Language';
import FolderIcon from '@mui/icons-material/Folder';
import StorageIcon from '@mui/icons-material/Storage';
import DriveFileMoveIcon from '@mui/icons-material/DriveFileMove';
import CodeIcon from '@mui/icons-material/Code';
import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { useAppSettings } from '../context/AppSettingsContext';
import { StyledCard } from '../components/ui/StyledCard';
import { SettingsItem } from '../components/ui/SettingsItem';
import { PageContainer } from '../components/ui/PageContainer';
import { BlockchainMoveDialog } from '../components/BlockchainMoveDialog';

export default function Settings() {  
  const { appSettings, updateDeveloperMode } = useAppSettings();
  
  // State for the various settings
  const [language] = useState('English');  const [configDirectory, setConfigDirectory] = useState<string>('');
  const [blockchainPath, setBlockchainPath] = useState<string>('');
  const [error, setError] = useState<string | null>(null);
  const [developerMode, setDeveloperMode] = useState(appSettings?.developer_mode || false);
  const [isMoveDialogOpen, setIsMoveDialogOpen] = useState(false);
  useEffect(() => {
    // Fetch config directory 
    invoke('get_config_directory')
      .then((dir) => setConfigDirectory(dir as string))
      .catch(err => {
        console.error(err);
        setError('Failed to load configuration directory');
      });

    // Fetch blockchain database path
    invoke('get_blockchain_database_path')
      .then((path) => setBlockchainPath(path as string))
      .catch(err => {
        console.error(err);
        setError('Failed to load blockchain database path');
      });    
      // Set local state from app settings context when it's available
    if (appSettings) {
      setDeveloperMode(appSettings.developer_mode);
    }
  }, [appSettings]);    // Use a ref to track toggle operations in progress
  const developerModeToggleInProgress = React.useRef(false);
  
  const handleDeveloperModeToggle = async (enabled: boolean) => {
    // Prevent multiple simultaneous toggle operations
    if (developerModeToggleInProgress.current) {
      console.log('Developer mode toggle already in progress, ignoring');
      return;
    }

    try {
      developerModeToggleInProgress.current = true;
      console.log('Toggling developer mode to:', enabled);
      
      // Set local state immediately for responsive UI feedback
      setDeveloperMode(enabled);
        // Use the context function which will handle the Tauri invocation
      await updateDeveloperMode(enabled);
      
      console.log('Developer mode toggle successful');
    } catch (err) {
      console.error(err);
      setError('Failed to update developer mode setting');
      
      // Get the current state from context to ensure UI is in sync with backend
      if (appSettings) {
        setDeveloperMode(appSettings.developer_mode);
      }
    } finally {
      // Always clear the in-progress flag
      developerModeToggleInProgress.current = false;
    }  };

  const handleOpenMoveDialog = () => {
    setIsMoveDialogOpen(true);
  };

  const handleCloseMoveDialog = () => {
    setIsMoveDialogOpen(false);
  };

  const handleMoveComplete = (newPath: string) => {
    setBlockchainPath(newPath);
    setIsMoveDialogOpen(false);
  };

  const handleMoveError = (error: string) => {
    // Don't propagate move errors to the Settings page
    // They should only show in the move dialog
    console.error('Blockchain move error:', error);
  };

  return (
    <>
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
                icon={<FolderIcon color="primary" />}
                primary="Config Directory"
                secondary={configDirectory || 'Loading...'}
              />
              <Divider variant="inset" component="li" />
              
              <SettingsItem
                icon={<StorageIcon color="primary" />}
                primary="Blockchain Database"
                secondary={blockchainPath || 'Loading...'}
                action={
                  <Button 
                    color="primary" 
                    variant="outlined" 
                    size="small"
                    onClick={handleOpenMoveDialog}
                    startIcon={<DriveFileMoveIcon />}
                    disabled={!blockchainPath}
                  >
                    Move
                  </Button>
                }
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
            </List>
          </StyledCard>
        </Grid>
          {/* Application Settings */}
        <Grid item xs={12} md={6}>          <StyledCard title="Application Settings" fullHeight>
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
                icon={<CodeIcon color="primary" />}
                primary="Developer Mode"
                secondary="Enable advanced debugging tools"
                action={                <Switch 
                    checked={developerMode}
                    onChange={(e) => handleDeveloperModeToggle(e.target.checked)}
                    color="primary"
                    // Disable the switch during update to prevent rapid toggling
                    disabled={appSettings === null || developerModeToggleInProgress.current}
                  />
                }              />
              <Divider variant="inset" component="li" />
            </List>
          </StyledCard>
        </Grid>
      </Grid>
    </PageContainer>
   
    <BlockchainMoveDialog
      isOpen={isMoveDialogOpen}
      currentPath={blockchainPath || ''}
      onMoveComplete={handleMoveComplete}
      onClose={handleCloseMoveDialog}
      onError={handleMoveError}
    />
    </>
  );
}