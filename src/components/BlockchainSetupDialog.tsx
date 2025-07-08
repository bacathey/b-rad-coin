import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  Button,
  Typography,
  Box,
  Alert,
  CircularProgress,
  Stack,
  useTheme,
  Fade
} from '@mui/material';
import DatabaseIcon from '@mui/icons-material/Storage';

interface BlockchainSetupDialogProps {
  isOpen: boolean;
  onSetupComplete: () => void;
  onError: (error: string) => void;
}

export const BlockchainSetupDialog: React.FC<BlockchainSetupDialogProps> = ({
  isOpen,
  onSetupComplete,
  onError,
}) => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [defaultPath, setDefaultPath] = useState<string>('');
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';

  // Fetch the default path when the dialog opens
  useEffect(() => {
    if (isOpen) {
      const fetchDefaultPath = async () => {
        try {
          // Use the true default system location, not the configured location
          const path = await invoke<string>('get_default_blockchain_database_path');
          setDefaultPath(path);
        } catch (err) {
          console.error('Failed to get default path:', err);
          setDefaultPath('Default location');
        }
      };
      fetchDefaultPath();
    }
  }, [isOpen]);

  // Helper function to complete blockchain setup
  const completeSetup = async () => {
    try {
      // Start the blockchain services now that database is ready
      const success = await invoke<boolean>('start_blockchain_services');
      
      if (success) {
        onSetupComplete();
      } else {
        onError('Failed to start blockchain services after setup');
      }
    } catch (err: any) {
      console.error('Failed to start services after setup:', err);
      onError('Blockchain setup completed but failed to start services: ' + (err?.toString() || 'Unknown error'));
    }
  };

  const handleCreateNew = async () => {
    setIsLoading(true);
    setError(null);

    try {
      // Open folder picker to choose location for new blockchain
      const selectedPath = await invoke<string | null>('open_folder_picker', {
        title: 'Choose location for new blockchain database'
      });

      if (selectedPath) {
        console.log('Creating blockchain database at custom location:', selectedPath);
        const success = await invoke<boolean>('create_blockchain_database_at_location', {
          location: selectedPath
        });

        if (success) {
          await completeSetup();
        } else {
          setError('Failed to create blockchain database at the selected location');
        }
      }
    } catch (err: any) {
      console.error('Failed to create blockchain database:', err);
      
      // Provide more user-friendly error messages
      let errorMessage = 'Failed to create blockchain database';
      const errorStr = err?.toString() || '';
      
      if (errorStr.includes('failed to acquire lock') || errorStr.includes('lock')) {
        errorMessage = 'Database is currently in use by another process. Please ensure no other instances of B-Rad Coin are running and try again.';
      } else if (errorStr.includes('permission') || errorStr.includes('access')) {
        errorMessage = 'Permission denied. Please ensure you have write access to the selected location or run as administrator.';
      } else if (errorStr.includes('directory') || errorStr.includes('folder')) {
        errorMessage = 'Unable to access the selected directory. Please check that the location exists and is accessible.';
      } else if (errorStr.includes('disk') || errorStr.includes('space')) {
        errorMessage = 'Insufficient disk space. Please ensure you have enough free space and try again.';
      }
      
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLocateExisting = async () => {
    setIsLoading(true);
    setError(null);

    try {
      // Open folder picker to locate existing blockchain
      const selectedPath = await invoke<string | null>('open_folder_picker', {
        title: 'Locate existing blockchain database folder'
      });

      if (selectedPath) {
        console.log('Setting blockchain database location to:', selectedPath);
        const success = await invoke<boolean>('set_blockchain_database_location', {
          location: selectedPath
        });

        if (success) {
          await completeSetup();
        } else {
          setError('Failed to load blockchain database from selected location');
        }
      }
    } catch (err: any) {
      console.error('Failed to load blockchain database:', err);
      
      // Provide more user-friendly error messages
      let errorMessage = 'Failed to load blockchain database';
      const errorStr = err?.toString() || '';
      
      if (errorStr.includes('failed to acquire lock') || errorStr.includes('lock')) {
        errorMessage = 'Database is currently in use by another process. Please ensure no other instances of B-Rad Coin are running and try again.';
      } else if (errorStr.includes('permission') || errorStr.includes('access')) {
        errorMessage = 'Permission denied. Please ensure you have read access to the selected location.';
      } else if (errorStr.includes('not found') || errorStr.includes('does not exist')) {
        errorMessage = 'No valid blockchain database found at the selected location. Please choose a different folder.';
      } else if (errorStr.includes('invalid') || errorStr.includes('corrupt')) {
        errorMessage = 'The selected blockchain database appears to be invalid or corrupted.';
      }
      
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const handleCreateDefault = async () => {
    setIsLoading(true);
    setError(null);

    try {
      // Always use the true default system location, not what's in config
      const pathToUse = await invoke<string>('get_default_blockchain_database_path');
      console.log('Creating blockchain database at default location:', pathToUse);
      
      const success = await invoke<boolean>('create_blockchain_database_at_location', {
        location: pathToUse
      });

      if (success) {
        await completeSetup();
      } else {
        setError('Failed to create blockchain database in default location');
      }
    } catch (err: any) {
      console.error('Failed to create default blockchain database:', err);
      
      // Provide more user-friendly error messages
      let errorMessage = 'Failed to create blockchain database';
      const errorStr = err?.toString() || '';
      
      if (errorStr.includes('failed to acquire lock') || errorStr.includes('lock')) {
        errorMessage = 'Database is currently in use by another process. Please ensure no other instances of B-Rad Coin are running and try again.';
      } else if (errorStr.includes('permission') || errorStr.includes('access')) {
        errorMessage = 'Permission denied. Please ensure you have write access to the selected location or run as administrator.';
      } else if (errorStr.includes('directory') || errorStr.includes('folder')) {
        errorMessage = 'Unable to access the directory. Please check that the location exists and is accessible.';
      } else if (errorStr.includes('disk') || errorStr.includes('space')) {
        errorMessage = 'Insufficient disk space. Please ensure you have enough free space and try again.';
      }
      
      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog
      open={isOpen}
      maxWidth="sm"
      fullWidth
      disableEscapeKeyDown
      TransitionComponent={Fade}
      TransitionProps={{ timeout: 500 }}
      PaperProps={{
        sx: {
          background: isDarkMode 
            ? 'linear-gradient(145deg, #0a1929 0%, #132f4c 100%)' 
            : 'linear-gradient(145deg, #ffffff 0%, #f5f7fa 100%)',
          borderRadius: '12px',
          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
          border: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.08)',
        }
      }}
    >
      <DialogTitle sx={{ pb: 1, display: 'flex', alignItems: 'center' }}>
        <DatabaseIcon color="primary" sx={{ mr: 1 }} />
        <Box>
          <Typography variant="h6" component="div" fontWeight={600}>
            B-Rad Coin Blockchain Database Setup
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
            No local blockchain database was found. Please choose an option to continue:
          </Typography>
        </Box>
      </DialogTitle>

      <DialogContent>
        {error && (
          <Alert 
            severity="error" 
            sx={{ 
              mb: 2,
              backgroundColor: isDarkMode ? 'rgba(229, 115, 115, 0.1)' : 'rgba(211, 47, 47, 0.08)',
              border: '1px solid',
              borderColor: isDarkMode ? 'rgba(229, 115, 115, 0.3)' : 'rgba(211, 47, 47, 0.2)',
              color: isDarkMode ? '#ffcdd2' : '#c62828',
              '& .MuiAlert-icon': {
                color: isDarkMode ? '#ef5350' : '#d32f2f'
              }
            }}
          >
            {error}
          </Alert>
        )}

        <Stack spacing={2}>
          <Button
            variant="contained"
            color="primary"
            disabled={isLoading}
            onClick={handleCreateDefault}
            sx={{
              textTransform: 'none',
              fontWeight: 600,
              justifyContent: 'flex-start',
              textAlign: 'left',
              p: 2,
              flexDirection: 'column',
              alignItems: 'flex-start',
              minHeight: '80px',
              background: isDarkMode
                ? 'linear-gradient(135deg, #1976d2 0%, #1565c0 100%)'
                : 'linear-gradient(135deg, #1976d2 0%, #1565c0 100%)',
              '&:hover': {
                background: isDarkMode
                  ? 'linear-gradient(135deg, #1565c0 0%, #0d47a1 100%)'
                  : 'linear-gradient(135deg, #1565c0 0%, #0d47a1 100%)',
              }
            }}
          >
            <Typography variant="subtitle1" fontWeight="bold">
              Create New (Default Location)
            </Typography>
            <Typography variant="body2" sx={{ opacity: 0.8, mt: 0.5, fontSize: '0.875rem' }}>
              {defaultPath || 'Loading default path...'}
            </Typography>
          </Button>

          <Button
            variant="outlined"
            color="primary"
            disabled={isLoading}
            onClick={handleCreateNew}
            sx={{
              textTransform: 'none',
              fontWeight: 600,
              justifyContent: 'flex-start',
              textAlign: 'left',
              p: 2,
              flexDirection: 'column',
              alignItems: 'flex-start',
              minHeight: '80px',
              borderColor: isDarkMode ? 'rgba(255, 255, 255, 0.23)' : 'rgba(0, 0, 0, 0.23)',
              '&:hover': {
                borderColor: isDarkMode ? 'rgba(255, 255, 255, 0.4)' : 'rgba(0, 0, 0, 0.4)',
                backgroundColor: isDarkMode ? 'rgba(25, 118, 210, 0.08)' : 'rgba(25, 118, 210, 0.04)',
              }
            }}
          >
            <Typography variant="subtitle1" fontWeight="bold">
              Create New (Custom Location)
            </Typography>
            <Typography variant="body2" sx={{ opacity: 0.8, mt: 0.5, fontSize: '0.875rem' }}>
              Choose a custom location to create a new blockchain database
            </Typography>
          </Button>

          <Button
            variant="outlined"
            color="secondary"
            disabled={isLoading}
            onClick={handleLocateExisting}
            sx={{
              textTransform: 'none',
              fontWeight: 600,
              justifyContent: 'flex-start',
              textAlign: 'left',
              p: 2,
              flexDirection: 'column',
              alignItems: 'flex-start',
              minHeight: '80px',
              borderColor: isDarkMode ? 'rgba(255, 255, 255, 0.23)' : 'rgba(0, 0, 0, 0.23)',
              '&:hover': {
                borderColor: isDarkMode ? 'rgba(255, 255, 255, 0.4)' : 'rgba(0, 0, 0, 0.4)',
                backgroundColor: isDarkMode ? 'rgba(156, 39, 176, 0.08)' : 'rgba(156, 39, 176, 0.04)',
              }
            }}
          >
            <Typography variant="subtitle1" fontWeight="bold">
              Locate Existing Database
            </Typography>
            <Typography variant="body2" sx={{ opacity: 0.8, mt: 0.5, fontSize: '0.875rem' }}>
              Browse for an existing blockchain database folder
            </Typography>
          </Button>
        </Stack>

        {isLoading && (
          <Box 
            sx={{ 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'center', 
              mt: 3,
              p: 2,
              borderRadius: 1,
              backgroundColor: isDarkMode ? 'rgba(25, 118, 210, 0.08)' : 'rgba(25, 118, 210, 0.04)',
              border: '1px solid',
              borderColor: isDarkMode ? 'rgba(25, 118, 210, 0.3)' : 'rgba(25, 118, 210, 0.2)',
            }}
          >
            <CircularProgress 
              size={24} 
              sx={{ 
                mr: 2,
                color: isDarkMode ? '#90caf9' : '#1976d2'
              }} 
            />
            <Typography 
              variant="body2" 
              sx={{
                color: isDarkMode ? '#90caf9' : '#1976d2',
                fontWeight: 500
              }}
            >
              Setting up blockchain database...
            </Typography>
          </Box>
        )}
      </DialogContent>
    </Dialog>
  );
};
