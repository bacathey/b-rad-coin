import { Grid, Typography, Button, Box, TextField, Switch, List, Divider, useTheme } from '@mui/material';
import { useState, useEffect, useRef } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { PageContainer } from '../components/ui/PageContainer';
import { StyledCard } from '../components/ui/StyledCard';
import { SettingsItem } from '../components/ui/SettingsItem';
import SecurityIcon from '@mui/icons-material/Security';
import { useAppSettings } from '../context/AppSettingsContext';
import { useWallet } from '../context/WalletContext';

export default function Developer() {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  
  const { appSettings, updateSkipSeedPhraseDialogs } = useAppSettings();
  const { setCurrentWallet, setIsWalletOpen, refreshWalletDetails } = useWallet();
  const [logOutput, setLogOutput] = useState<string>('');
  const [customCommand, setCustomCommand] = useState<string>('');
  const [result, setResult] = useState<string>('');
  const [loading, setLoading] = useState<boolean>(false);  const [error, setError] = useState<string | null>(null);
  const [cleanupLoading, setCleanupLoading] = useState<boolean>(false);
  const [deleteAllLoading, setDeleteAllLoading] = useState<boolean>(false);
  const [skipSeedPhraseDialogs, setSkipSeedPhraseDialogs] = useState<boolean>(appSettings?.skip_seed_phrase_dialogs || false);
  
  // Add a ref to track if a toggle operation is in progress
  const toggleInProgressRef = useRef<boolean>(false);

  // Effect to sync with app settings
  useEffect(() => {
    if (appSettings) {
      setSkipSeedPhraseDialogs(appSettings.skip_seed_phrase_dialogs);
    }
  }, [appSettings]);

  const handleRunCommand = async () => {
    if (!customCommand.trim()) return;
    
    setLoading(true);
    setError(null);
    
    try {
      // This is just a placeholder - in a real app, you'd have specific commands 
      // that are safe to run from the frontend
      const response = await invoke('echo_command', { command: customCommand });
      setResult(JSON.stringify(response, null, 2));
    } catch (err) {
      console.error(err);
      setError(`Error executing command: ${err}`);
    } finally {
      setLoading(false);
    }
  };  const handleViewLogs = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const logs = await invoke('get_recent_logs');
      setLogOutput(logs as string);
    } catch (err) {
      console.error(err);
      setError(`Error fetching logs: ${err}`);
    } finally {
      setLoading(false);
    }
  };  const handleCleanupOrphanedWallets = async () => {
    setCleanupLoading(true);
    setError(null);
    
    try {
      const deletedItems = await invoke('cleanup_orphaned_wallets') as string[];
      
      if (deletedItems.length === 0) {
        setResult('No orphaned wallet files or directories found to clean up.');
      } else {
        setResult(`Successfully cleaned up ${deletedItems.length} orphaned wallet items:\n${deletedItems.join('\n')}`);
      }
    } catch (err) {
      console.error('Error cleaning up orphaned wallets:', err);
      setError(`Error cleaning up orphaned wallets: ${err}`);
    } finally {
      setCleanupLoading(false);
    }
  };  const handleDeleteAllWallets = async () => {
    // Show confirmation dialog before proceeding
    const confirmed = window.confirm(
      'WARNING: This will permanently delete ALL wallets from both the configuration and disk!\n\n' +
      'This action cannot be undone. All wallet data will be lost.\n\n' +
      'Are you sure you want to continue?'
    );
    
    if (!confirmed) {
      return;
    }
    
    setDeleteAllLoading(true);
    setError(null);
    
    try {
      const deletedItems = await invoke('delete_all_wallets') as string[];
      
      // Clear wallet state in frontend after successful deletion
      setCurrentWallet(null);
      setIsWalletOpen(false);
      
      // Force refresh the wallet status from backend to ensure consistency
      try {
        const backendWalletStatus = await invoke<boolean>('check_wallet_status');
        if (backendWalletStatus) {
          console.warn('Backend reports wallet is still open after deletion, forcing close');
          await invoke('close_wallet');
        }
      } catch (backendError) {
        console.error('Error checking backend wallet status:', backendError);
      }
      
      // Refresh wallet details to reflect empty state
      await refreshWalletDetails();
      
      if (deletedItems.length === 0) {
        setResult('No wallets found to delete.');
      } else {
        setResult(`Successfully deleted all wallets:\n${deletedItems.join('\n')}\n\nAll wallet data has been permanently removed.\n\nThe wallet selection dialog should now appear since no wallets exist.`);
      }
    } catch (err) {
      console.error('Error deleting all wallets:', err);
      setError(`Error deleting all wallets: ${err}`);
    } finally {
      setDeleteAllLoading(false);
    }  };

// Function to update skip seed phrase dialogs setting
  const handleSeedPhraseDialogsToggle = async (skipDialogs: boolean) => {
    // Check if developer mode is enabled first
    if (!appSettings?.developer_mode) {
      setError('Developer mode must be enabled in Settings before you can change this setting.');
      return;
    }
    
    // Prevent multiple simultaneous toggle operations
    if (toggleInProgressRef.current) {
      console.log('Toggle already in progress, ignoring this request');
      return;
    }
    
    try {
      toggleInProgressRef.current = true;
      console.log('Setting skip seed phrase dialogs to:', skipDialogs);
      setLoading(true); // Show loading state
      setError(null); // Clear any previous errors
      
      // Update local state for immediate UI feedback
      setSkipSeedPhraseDialogs(skipDialogs);
        // Call the context function to update the backend setting and persist to disk
      await updateSkipSeedPhraseDialogs(skipDialogs);
      
      console.log('UpdateSkipSeedPhraseDialogs completed. Current appSettings:', appSettings);
      
      // Add a small delay to let the context update propagate
      await new Promise(resolve => setTimeout(resolve, 100));
      
      // Verify the setting was updated by checking the appSettings context
      if (appSettings && appSettings.skip_seed_phrase_dialogs !== skipDialogs) {
        console.warn('Settings context does not reflect change, may not have persisted correctly');
        console.warn('Expected:', skipDialogs, 'Actual:', appSettings.skip_seed_phrase_dialogs);
      } else {
        console.log('Skip seed phrase dialogs setting updated successfully and persisted');
      }    } catch (err) {
      console.error('Failed to update skip seed phrase dialogs setting:', err);
      if (err instanceof Error && err.message.includes('Developer mode must be enabled')) {
        setError('Developer mode must be enabled in Settings before you can skip seed phrase dialogs.');
      } else {
        setError('Failed to update skip seed phrase dialogs setting. Changes will not persist across app restarts.');
      }
      
      // Revert UI state if the update failed
      setSkipSeedPhraseDialogs(!skipDialogs);
    } finally {
      setLoading(false); // Hide loading state
      toggleInProgressRef.current = false;
    }
  };

  return (
    <PageContainer title="Developer Tools" error={error}>
      <Typography variant="subtitle1" color="text.secondary" gutterBottom>
        These tools are intended for development and debugging purposes only.
      </Typography>
        <Grid container spacing={3} sx={{ width: '100%', mt: 1 }}>
        {/* Developer Settings First */}
        <Grid item xs={12}>
          <StyledCard title="Developer Settings">
            <List>              <SettingsItem
                icon={<SecurityIcon color="primary" />}
                primary="Skip Seed Phrase Dialogs"
                secondary={
                  appSettings?.developer_mode 
                    ? "Skip seed phrase verification steps during wallet creation" 
                    : "Developer mode must be enabled in Settings to use this feature"
                }
                action={
                  <Switch
                    checked={skipSeedPhraseDialogs}
                    onChange={(e) => handleSeedPhraseDialogsToggle(e.target.checked)}
                    color="primary"
                    disabled={loading || !appSettings?.developer_mode} // Disable if not in developer mode
                  />
                }
              />
              <Divider variant="inset" component="li" />
            </List>
          </StyledCard>
        </Grid>
        {/* Development Tools */}
        <Grid item xs={12}>
          <StyledCard title="Development Tools">            <Typography variant="body2" sx={{ mb: 2 }}>
              Execute developer commands for testing and debugging purposes.
            </Typography>
            
            <Box sx={{ display: 'flex', gap: 2, mb: 2 }}>
              <TextField
                fullWidth
                label="Command"
                variant="outlined"
                value={customCommand}
                onChange={(e) => setCustomCommand(e.target.value)}
                placeholder="Enter command"
              />
              <Button 
                variant="contained" 
                onClick={handleRunCommand}
                disabled={!customCommand.trim() || loading}
                sx={{ minWidth: '140px' }}
              >
                Execute Command
              </Button>
            </Box>            <Button 
              variant="outlined" 
              onClick={handleCleanupOrphanedWallets}
              disabled={cleanupLoading || loading || deleteAllLoading}
              sx={{ 
                mr: 2,
                borderColor: isDarkMode ? '#ffa726' : '#f57c00',
                color: isDarkMode ? '#ffa726' : '#f57c00',
                '&:hover': {
                  borderColor: isDarkMode ? '#ff9800' : '#e65100',
                  backgroundColor: isDarkMode ? 'rgba(255, 167, 38, 0.1)' : 'rgba(245, 124, 0, 0.1)'
                }
              }}
            >
              {cleanupLoading ? 'Cleaning up...' : 'Cleanup Orphaned Wallets'}
            </Button>

            <Button 
              variant="contained"
              color="error"
              onClick={handleDeleteAllWallets}
              disabled={deleteAllLoading || loading || cleanupLoading}
            >
              {deleteAllLoading ? 'Deleting...' : 'Delete ALL Wallets'}
            </Button>
            
            {result && (
              <Box 
                sx={{ 
                  mt: 2,
                  p: 2,
                  bgcolor: 'background.default',
                  borderRadius: 1,
                  fontFamily: 'monospace',
                  fontSize: '0.8rem',
                  whiteSpace: 'pre-wrap'
                }}
              >
                {result}
              </Box>
            )}
          </StyledCard>
        </Grid>
        
        {/* Log Viewer Below */}
        <Grid item xs={12}>
          <StyledCard title="Application Logs">
            <Button 
              variant="contained" 
              onClick={handleViewLogs} 
              disabled={loading}
              sx={{ mb: 2 }}
            >
              View Recent Logs
            </Button>
            {logOutput && (
              <Box 
                sx={{ 
                  height: '500px', /* Increased height for better visibility */
                  overflowY: 'auto', 
                  p: 2, 
                  bgcolor: 'background.default',
                  borderRadius: 1,
                  fontFamily: 'monospace',
                  fontSize: '0.8rem',
                  whiteSpace: 'pre-wrap'
                }}
              >
                {logOutput}
              </Box>
            )}
          </StyledCard>        </Grid>
      </Grid>
    </PageContainer>
  );
}
