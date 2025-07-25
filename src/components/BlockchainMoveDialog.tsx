import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Typography,
  Box,
  Alert,
  CircularProgress,
  LinearProgress,
  Stack,
  Chip,
  useTheme,
  Fade
} from '@mui/material';
import FolderOpenIcon from '@mui/icons-material/FolderOpen';
import DriveFileMoveIcon from '@mui/icons-material/DriveFileMove';

interface BlockchainMoveDialogProps {
  isOpen: boolean;
  currentPath: string;
  onMoveComplete: (newPath: string) => void;
  onClose: () => void;
  onError: (error: string) => void;
}

export const BlockchainMoveDialog: React.FC<BlockchainMoveDialogProps> = ({
  isOpen,
  currentPath,
  onMoveComplete,
  onClose,
  onError,
}) => {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  
  const [isMoving, setIsMoving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<string>('');
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [databaseSize, setDatabaseSize] = useState<number | null>(null);
  const [isLoadingSize, setIsLoadingSize] = useState(false);

  // Format file size in human readable format
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  // Load database size when dialog opens
  useEffect(() => {
    if (isOpen) {
      setIsLoadingSize(true);
      invoke<number>('get_blockchain_database_size')
        .then(size => {
          setDatabaseSize(size);
        })
        .catch(err => {
          console.error('Failed to get database size:', err);
          setDatabaseSize(0);
        })
        .finally(() => {
          setIsLoadingSize(false);
        });
    }
  }, [isOpen]);

  const handleSelectTargetLocation = async () => {
    try {
      const selectedPath = await invoke<string | null>('open_folder_picker', {
        title: 'Choose new folder for blockchain database'
      });

      if (selectedPath) {
        if (selectedPath === currentPath) {
          setError('The selected location is the same as the current location.');
          return;
        }
        setSelectedPath(selectedPath);
        setError(null);
      }
    } catch (err: any) {
      console.error('Failed to open folder picker:', err);
      setError('Failed to open folder picker. Please try again.');
    }
  };

  const handleMove = async () => {
    if (!selectedPath) {
      setError('Please select a target location first.');
      return;
    }

    setIsMoving(true);
    setError(null);

    try {
      // Step 1: Stop blockchain services
      setProgress('Stopping blockchain services...');
      const stopSuccess = await invoke<boolean>('stop_blockchain_services');
      if (!stopSuccess) {
        throw new Error('Failed to stop blockchain services');
      }

      // Step 2: Move the database
      setProgress('Moving blockchain database...');
      const moveSuccess = await invoke<boolean>('move_blockchain_database', {
        newLocation: selectedPath
      });
      if (!moveSuccess) {
        throw new Error('Failed to move blockchain database');
      }

      // Step 3: Restart blockchain services
      setProgress('Restarting blockchain services...');
      const startSuccess = await invoke<boolean>('start_blockchain_services');
      if (!startSuccess) {
        throw new Error('Failed to restart blockchain services');
      }

      setProgress('Move completed successfully!');
      setTimeout(() => {
        onMoveComplete(selectedPath);
        setIsMoving(false);
        setProgress('');
        setSelectedPath(null);
      }, 1000);

    } catch (err: any) {
      console.error('Failed to move blockchain database:', err);
      
      // Provide more user-friendly error messages
      let errorMessage = 'Failed to move blockchain database';
      const errorStr = err?.toString() || '';
      
      if (errorStr.includes('already contains blockchain') || errorStr.includes('already exists')) {
        errorMessage = 'The selected location already contains blockchain data. Please choose a different folder.';
      } else if (errorStr.includes('failed to acquire lock') || errorStr.includes('lock')) {
        errorMessage = 'Database is currently in use. Please ensure no other operations are running and try again.';
      } else if (errorStr.includes('permission') || errorStr.includes('access')) {
        errorMessage = 'Permission denied. Please ensure you have write access to both locations or run as administrator.';
      } else if (errorStr.includes('directory') || errorStr.includes('folder')) {
        errorMessage = 'Unable to access the selected directory. Please check that the location exists and is accessible.';
      } else if (errorStr.includes('disk') || errorStr.includes('space')) {
        errorMessage = 'Insufficient disk space. Please ensure you have enough free space at the destination.';
      } else if (errorStr.includes('not found') || errorStr.includes('does not exist')) {
        errorMessage = 'Blockchain database not found at current location.';
      } else if (errorStr.includes('in use') || errorStr.includes('busy')) {
        errorMessage = 'Database files are currently in use. Please close all wallet operations and try again.';
      }
      
      setError(errorMessage);
      onError(errorMessage);
      
      // Try to restart services if they were stopped
      try {
        setProgress('Attempting to restart services...');
        await invoke<boolean>('start_blockchain_services');
      } catch (restartErr) {
        console.error('Failed to restart services after move failure:', restartErr);
      }
      
      setIsMoving(false);
      setProgress('');
    }
  };

  const handleClose = () => {
    if (!isMoving) {
      setError(null);
      setProgress('');
      setSelectedPath(null);
      onClose();
    }
  };

  return (
    <Dialog
      open={isOpen}
      maxWidth="sm"
      fullWidth
      disableEscapeKeyDown={isMoving}
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
        <DriveFileMoveIcon color="primary" sx={{ mr: 1 }} />
        <Box>
          <Typography variant="h6" component="div" fontWeight={600}>
            Move Blockchain Database
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
            This will move your blockchain database files to a new folder location.
          </Typography>
        </Box>
      </DialogTitle>

      <DialogContent sx={{ px: 3 }}>
        {error && (
          <Alert severity="error" sx={{ mb: 2 }}>
            {error}
          </Alert>
        )}

        <Stack spacing={2}>
          <Box>
            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
              Current Location:
            </Typography>
            <Typography 
              variant="body2" 
              sx={{ 
                backgroundColor: isDarkMode ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.04)',
                p: 1.5,
                borderRadius: 2,
                fontFamily: 'monospace',
                fontSize: '0.8rem',
                wordBreak: 'break-all',
                border: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.08)',
              }}
            >
              {currentPath}
            </Typography>
          </Box>

          {/* Database size information */}
          <Box>
            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
              Database Size:
            </Typography>
            <Box display="flex" alignItems="center" gap={1}>
              {isLoadingSize ? (
                <CircularProgress size={16} />
              ) : (
                <Chip 
                  label={databaseSize !== null ? formatFileSize(databaseSize) : 'Unknown'} 
                  size="small" 
                  variant="outlined" 
                />
              )}
              <Typography variant="body2" color="text.secondary">
                {databaseSize !== null && databaseSize > 0 ? 'to be moved' : 'No database found'}
              </Typography>
            </Box>
          </Box>

          {/* Target location selection */}
          <Box>
            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
              Target Location:
            </Typography>
            {selectedPath ? (
              <Box>
                <Typography 
                  variant="body2" 
                  sx={{ 
                    backgroundColor: isDarkMode ? 'rgba(46, 125, 50, 0.2)' : 'rgba(46, 125, 50, 0.1)',
                    color: isDarkMode ? '#81c784' : '#2e7d32',
                    p: 1.5,
                    borderRadius: 2,
                    fontFamily: 'monospace',
                    fontSize: '0.8rem',
                    wordBreak: 'break-all',
                    mb: 1,
                    border: isDarkMode ? '1px solid rgba(129, 199, 132, 0.3)' : '1px solid rgba(46, 125, 50, 0.2)',
                  }}
                >
                  {selectedPath}
                </Typography>
                <Button
                  size="small"
                  onClick={handleSelectTargetLocation}
                  disabled={isMoving}
                  startIcon={<FolderOpenIcon />}
                >
                  Change Location
                </Button>
              </Box>
            ) : (
              <Button
                variant="outlined"
                onClick={handleSelectTargetLocation}
                disabled={isMoving}
                startIcon={<FolderOpenIcon />}
                fullWidth
              >
                Select Target Location
              </Button>
            )}
          </Box>

          {isMoving && (
            <Box sx={{ 
              p: 2, 
              backgroundColor: isDarkMode ? 'rgba(25, 118, 210, 0.1)' : 'rgba(25, 118, 210, 0.05)',
              borderRadius: 2,
              border: isDarkMode ? '1px solid rgba(25, 118, 210, 0.3)' : '1px solid rgba(25, 118, 210, 0.2)',
            }}>
              <Typography variant="body2" color="text.secondary" gutterBottom>
                Progress:
              </Typography>
              <LinearProgress sx={{ mb: 1, borderRadius: 1 }} />
              <Typography variant="body2" color="primary" fontWeight={500}>
                {progress}
              </Typography>
            </Box>
          )}

          {!isMoving && (
            <Alert 
              severity="warning" 
              sx={{ 
                mt: 2,
                backgroundColor: isDarkMode ? 'rgba(255, 152, 0, 0.1)' : 'rgba(255, 152, 0, 0.05)',
                border: isDarkMode ? '1px solid rgba(255, 152, 0, 0.3)' : '1px solid rgba(255, 152, 0, 0.2)',
                borderRadius: 2,
              }}
            >
              <Typography variant="body2">
                <strong>Important:</strong> This operation will temporarily stop blockchain services. 
                Make sure no wallet operations are in progress before proceeding.
              </Typography>
            </Alert>
          )}
        </Stack>
      </DialogContent>

      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button 
          onClick={handleClose} 
          disabled={isMoving}
          color="inherit"
        >
          Cancel
        </Button>
        <Button 
          onClick={handleMove} 
          disabled={isMoving || !selectedPath}
          variant="contained"
          color="primary"
          startIcon={isMoving ? <CircularProgress size={16} /> : <DriveFileMoveIcon />}
        >
          {isMoving ? 'Moving...' : 'Move Database'}
        </Button>
      </DialogActions>
    </Dialog>
  );
};
