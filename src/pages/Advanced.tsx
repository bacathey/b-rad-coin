import { 
  Typography, 
  Box, 
  Card, 
  CardContent, 
  useTheme, 
  Paper, 
  Grid, 
  Button,
  CircularProgress,
  Tooltip,
  Chip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogContentText,
  DialogActions,
  TextField,
  Alert,
  Snackbar,
  Fade, // Add Fade import
  Slider,
  FormControl,
  FormLabel,
  Switch,
  FormControlLabel,
  FormGroup
} from '@mui/material';
import { useState, useEffect } from 'react';
import { useWallet } from '../context/WalletContext';
import { useAppSettings } from '../context/AppSettingsContext';
import LockIcon from '@mui/icons-material/Lock';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import DeleteIcon from '@mui/icons-material/Delete';
import SecurityIcon from '@mui/icons-material/Security';
import VisibilityIcon from '@mui/icons-material/Visibility';
import MinimizeIcon from '@mui/icons-material/Minimize';
import SecureWalletDialog from '../components/SecureWalletDialog';
import { invoke } from '@tauri-apps/api/core';

export default function Advanced() {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';

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
        Advanced
      </Typography>
      
      {/* Container with fixed maximum width and full width */}
      <Box sx={{ width: '100%', maxWidth: 1200, mx: 'auto' }}>
        {/* Wallet File Location Card */}
        <Grid item xs={12} sx={{ mb: 3 }}>
          <Paper sx={{ 
            p: 3, 
            ...(isDarkMode ? {
              background: 'rgba(19, 47, 76, 0.6)',
              backdropFilter: 'blur(10px)',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
              border: '1px solid rgba(255, 255, 255, 0.1)'
            } : {
              background: 'linear-gradient(90deg, #f5f7fa 0%, #ffffff 100%)',
              boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
              border: '1px solid rgba(0, 0, 0, 0.08)'
            }) 
          }}>
            <WalletLocationSection />
          </Paper>
        </Grid>
        
        {/* System Tray Settings Card */}
        <Grid item xs={12} sx={{ mb: 3 }}>
          <Paper sx={{ 
            p: 3, 
            ...(isDarkMode ? {
              background: 'rgba(19, 47, 76, 0.6)',
              backdropFilter: 'blur(10px)',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
              border: '1px solid rgba(255, 255, 255, 0.1)'
            } : {
              background: 'linear-gradient(90deg, #f5f7fa 0%, #ffffff 100%)',
              boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
              border: '1px solid rgba(0, 0, 0, 0.08)'
            }) 
          }}>
            <SystemTraySettingsSection />
          </Paper>
        </Grid>
        
        {/* Mining Card */}
        <Box>
          <Card sx={{ ...cardStyle }}>
            <CardContent>
              <Typography variant="h6" sx={{ fontWeight: 600 }}>
                Mining
              </Typography>
              <Typography variant="body2" sx={{ mt: 1, mb: 3 }}>
                Configure mining thread count and other mining settings.
              </Typography>
              <MiningThreadsSection />
            </CardContent>
          </Card>
        </Box>
      </Box>
    </Box>
  );
}

// System Tray Settings Component
function SystemTraySettingsSection() {
  const { appSettings, updateMinimizeToSystemTray } = useAppSettings();
  const [isUpdating, setIsUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';

  const handleToggleMinimizeToTray = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const enabled = event.target.checked;
    setIsUpdating(true);
    setError(null);

    try {
      await updateMinimizeToSystemTray(enabled);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update setting');
      console.error('Error updating minimize to system tray setting:', err);
    } finally {
      setIsUpdating(false);
    }
  };

  return (
    <>
      <Typography variant="h6" gutterBottom fontWeight={600}>
        System Tray Settings
      </Typography>
      
      <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
        Configure how the application behaves with the system tray. This is especially useful for mining scenarios.
      </Typography>
      
      {error && (
        <Alert 
          severity="error" 
          sx={{ mb: 2 }}
          onClose={() => setError(null)}
        >
          {error}
        </Alert>
      )}
      
      <FormGroup>
        <FormControlLabel
          control={
            <Switch
              checked={appSettings?.minimize_to_system_tray ?? false}
              onChange={handleToggleMinimizeToTray}
              disabled={isUpdating}
              color="primary"
            />
          }
          label={
            <Box sx={{ display: 'flex', alignItems: 'center' }}>
              <MinimizeIcon sx={{ mr: 1, fontSize: '1.2rem' }} />
              <Box>
                <Typography variant="body1" sx={{ fontWeight: 500 }}>
                  Minimize to System Tray
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  When enabled, the application will minimize to the system tray instead of closing when the window is closed. 
                  Enable this for mining scenarios where you want the mining process to continue running in the background.
                </Typography>
              </Box>
              {isUpdating && <CircularProgress size={20} sx={{ ml: 2 }} />}
            </Box>
          }
          sx={{ 
            alignItems: 'flex-start',
            mb: 1,
            '& .MuiFormControlLabel-label': {
              ml: 1
            }
          }}
        />
      </FormGroup>
      
      <Box 
        sx={{ 
          mt: 2,
          p: 2,
          borderRadius: 1,
          backgroundColor: isDarkMode ? 'rgba(33, 150, 243, 0.1)' : 'rgba(25, 118, 210, 0.08)',
          border: '1px solid',
          borderColor: isDarkMode ? 'rgba(33, 150, 243, 0.3)' : 'rgba(25, 118, 210, 0.2)',
        }}
      >
        <Typography 
          variant="body2" 
          sx={{ 
            fontWeight: 600,
            color: isDarkMode ? '#64b5f6' : '#1565c0',
            mb: 0.5
          }}
        >
          💡 Mining Tip
        </Typography>
        <Typography 
          variant="body2" 
          sx={{ 
            color: isDarkMode ? 'rgba(255, 255, 255, 0.8)' : 'rgba(0, 0, 0, 0.7)',
            fontSize: '0.875rem'
          }}
        >
          For mining operations, it's recommended to enable this setting so the mining process can continue running 
          in the background while the UI is minimized to the system tray.
        </Typography>
      </Box>
    </>
  );
}

// Wallet file component
function WalletLocationSection() {  const { 
    currentWallet, 
    isWalletOpen, 
    isWalletSecured, 
    getCurrentWalletPath, 
    deleteWallet,
    refreshWalletDetails  } = useWallet();
  
  const [displayPath, setDisplayPath] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showErrorAlert, setShowErrorAlert] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [confirmWalletName, setConfirmWalletName] = useState("");
  const [deleteError, setDeleteError] = useState<string | null>(null);  const [isDeleting, setIsDeleting] = useState(false);  const [secureDialogOpen, setSecureDialogOpen] = useState(false);
  const [privateKey, setPrivateKey] = useState('');
  const [privateKeyLoading, setPrivateKeyLoading] = useState(false);
  const [showCopySuccess, setShowCopySuccess] = useState(false);
  const theme = useTheme(); // Add this line to get the theme object
  const isDarkMode = theme.palette.mode === 'dark'; // Add this line
    useEffect(() => {
    if (isWalletOpen && currentWallet) {
      fetchWalletPath();
    } else {
      setDisplayPath(null);
    }
  }, [isWalletOpen, currentWallet]);
  const fetchWalletPath = async () => {
    setIsLoading(true);
    setError(null);
    try {
      // Get the path from the backend
      const path = await getCurrentWalletPath();
      
      // Log the received path with detailed info
      console.log(`Retrieved wallet path: "${path}"`);
      if (path) {
        console.log(`Path type: ${typeof path}`);
        console.log(`Path length: ${path.length}`);
        console.log('Path characters:');
        for (let i = 0; i < path.length; i++) {
          const char = path.charAt(i);
          const code = path.charCodeAt(i);
          console.log(`  Position ${i}: "${char}" (char code: ${code})`);
        }
      }
        if (path) {
        // Create a normalized path for display, ensuring Windows-style backslashes
        const normalizedPath = path.replace(/\//g, '\\');
        setDisplayPath(normalizedPath);
        
        console.log(`Original path: ${path}`);
        console.log(`Display path: ${normalizedPath}`);
      } else {
        setDisplayPath(null);
      }
    } catch (error) {
      console.error('Failed to get wallet path:', error);
      setError('Failed to get wallet path. Please check if the wallet still exists.');
      setShowErrorAlert(true);
    } finally {
      setIsLoading(false);
    }
  };    // Compare paths and handle folder opening more directly
  
  const handleOpenDeleteDialog = () => {
    setDeleteDialogOpen(true);
    setConfirmWalletName("");
    setDeleteError(null);
  };

  const handleCloseDeleteDialog = () => {
    setDeleteDialogOpen(false);
    setDeleteError(null);
  };

  const handleDeleteWallet = async () => {
    if (!currentWallet) return;
    
    // Check if the wallet name matches
    if (confirmWalletName !== currentWallet.name) {
      setDeleteError("The wallet name doesn't match. Please type the exact name.");
      return;
    }

    setIsDeleting(true);
    try {
      await deleteWallet(currentWallet.name);
      setDeleteDialogOpen(false);
    } catch (error) {
      console.error('Error deleting wallet:', error);
      setDeleteError(`Failed to delete wallet: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsDeleting(false);
    }
  };
  
  const handleCloseErrorAlert = () => {
    setShowErrorAlert(false);
  };

  const handleOpenSecureDialog = () => {
    setSecureDialogOpen(true);
  };
  
  const handleCloseSecureDialog = () => {
    setSecureDialogOpen(false);
  };
    const handleSuccessfulSecurity = async () => {
    // Refresh wallet details to update the security status
    await refreshWalletDetails();
  };  const handleShowPrivateKey = async () => {
    if (!currentWallet) return;
    
    // Reset previous state
    setPrivateKey('');
    
    // Since the wallet data is already decrypted in memory when opened, 
    // we can directly get the private key without asking for password again
    await fetchPrivateKey();
  };  const fetchPrivateKey = async () => {
    if (!currentWallet) return;
    
    setPrivateKeyLoading(true);
    
    try {
      const key = await invoke<string>('get_wallet_private_key');
      setPrivateKey(key);
    } catch (error) {
      console.error('Failed to get private key:', error);
    } finally {
      setPrivateKeyLoading(false);
    }
  };  const handleClosePrivateKeyDisplay = () => {
    setPrivateKey('');
  };
  
  if (!isWalletOpen || !currentWallet) {
    return (
      <Grid container spacing={2} alignItems="center">
        <Grid item xs={12}>
          <Typography variant="h6" gutterBottom fontWeight={600}>
            Wallet File
          </Typography>
          <Typography variant="body2" color="text.secondary">
            No wallet is currently open. Open a wallet to view its location.
          </Typography>
        </Grid>
      </Grid>
    );
  }
  
  return (
    <>
      <Grid container spacing={2} alignItems="center">
        <Grid item xs={12} md={8}>
          <Typography variant="h6" gutterBottom fontWeight={600}>
            Wallet File
          </Typography>
          
          {isLoading ? (
            <CircularProgress size={20} sx={{ mt: 1, mb: 1 }} />
          ) : (
            <>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <Typography variant="body2" fontWeight={500}>
                  Current Wallet:
                </Typography>
                <Typography variant="body2" sx={{ ml: 1 }}>
                  {currentWallet.name}
                </Typography>                <Chip
                  icon={isWalletSecured ? <LockIcon /> : <LockOpenIcon />}
                  label={isWalletSecured ? "Password Protected" : "No Password"}
                  size="small"
                  color={isWalletSecured ? "success" : "warning"}
                  variant="outlined"
                  sx={{ ml: 2 }}
                />
              </Box>                <Typography variant="body2" color="text.secondary" sx={{ wordBreak: 'break-all' }}>
                <strong>Path:</strong> {displayPath || "Path not available"}
              </Typography>
            </>
          )}
          
          {error && (
            <Alert 
              severity="error" 
              sx={{ mt: 2, maxWidth: "100%" }}
              onClose={() => setError(null)}
            >
              {error}
            </Alert>
          )}
        </Grid>        <Grid item xs={12} md={4} sx={{ 
          display: 'flex', 
          gap: 2,
          flexDirection: { xs: 'column', sm: 'row' },
          justifyContent: { xs: 'flex-start', md: 'flex-end' }
        }}>          <Tooltip title="Show the private key for this wallet">
            <Button 
              variant="outlined" 
              color="primary" 
              startIcon={privateKeyLoading ? <CircularProgress size={20} /> : <VisibilityIcon />} 
              onClick={handleShowPrivateKey}
              disabled={isLoading || privateKeyLoading}
            >
              {privateKeyLoading ? 'Loading...' : 'Show Private Key'}
            </Button>
          </Tooltip>
          
          <Tooltip title="Delete this wallet">
            <Button 
              variant="outlined" 
              color="error" 
              startIcon={<DeleteIcon />} 
              onClick={handleOpenDeleteDialog}
              disabled={isLoading}            >
              Delete Wallet
            </Button>
          </Tooltip>
          {!isWalletSecured && (
            <Tooltip title="Secure this wallet with a password">
              <Button
                variant="outlined" 
                startIcon={<SecurityIcon />}
                onClick={handleOpenSecureDialog}
                disabled={isLoading}
                sx={{
                  borderColor: isDarkMode ? '#81c784' : '#2e7d32',
                  color: isDarkMode ? '#81c784' : '#2e7d32',
                  '&:hover': {
                    borderColor: isDarkMode ? '#66bb6a' : '#1b5e20',
                    backgroundColor: isDarkMode ? 'rgba(129, 199, 132, 0.1)' : 'rgba(46, 125, 50, 0.1)'
                  }
                }}
              >
                Secure Wallet
              </Button>
            </Tooltip>
          )}
        </Grid>
      </Grid>
      
      {/* Delete Wallet Dialog */}
      <Dialog
        open={deleteDialogOpen}
        onClose={handleCloseDeleteDialog}
        aria-labelledby="delete-wallet-dialog-title"
        TransitionComponent={Fade} // Added TransitionComponent
        TransitionProps={{ timeout: 500 }} // Added TransitionProps
        PaperProps={{
          sx: {
            background: isDarkMode 
              ? 'linear-gradient(145deg, #0a1929 0%, #132f4c 100%)' 
              : 'linear-gradient(145deg, #ffffff 0%, #f5f7fa 100%)',
            borderRadius: '12px',
            boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
            border: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.08)',
            minWidth: { xs: '90%', sm: '500px' }, // Ensure a decent minimum width
          }
        }}
      >
        <DialogTitle id="delete-wallet-dialog-title" sx={{ pb: 1, display: 'flex', alignItems: 'center' }}>
          <DeleteIcon color="error" sx={{ mr: 1 }} />
          <Typography variant="h6" component="div" fontWeight={600}>
            Delete Wallet
          </Typography>
        </DialogTitle>
        <DialogContent>
          <DialogContentText sx={{ mb: 2 }}>
            Are you sure you want to permanently delete the wallet "<strong>{currentWallet?.name}</strong>"? 
            This action cannot be undone and will remove all associated data.
          </DialogContentText>
          <DialogContentText sx={{ mb: 2, fontWeight: 'bold', color: 'error.main' }}>
            To confirm, please type the full wallet name below:
          </DialogContentText>
          {deleteError && (
            <Alert severity="error" sx={{ mb: 2 }} variant="filled">
              {deleteError}
            </Alert>
          )}
          <TextField
            autoFocus
            margin="dense"
            id="confirm-wallet-name-delete"
            label={`Type "${currentWallet?.name || ''}" to confirm`}
            fullWidth
            variant="outlined"
            value={confirmWalletName}
            onChange={(e) => setConfirmWalletName(e.target.value)}
            error={!!deleteError && confirmWalletName !== currentWallet?.name} // Highlight if error and name doesn't match
            sx={{
              mb: 2,
              '& .MuiOutlinedInput-root': {
                '&.Mui-focused fieldset': {
                  borderColor: 'error.main',
                },
              },
            }}
          />
        </DialogContent>
        <DialogActions sx={{ px: 3, pb: 2 }}>
          <Button onClick={handleCloseDeleteDialog} variant="outlined" sx={{ mr: 1 }}>Cancel</Button>
          <Button 
            onClick={handleDeleteWallet} 
            color="error"
            variant="contained"
            disabled={confirmWalletName !== currentWallet?.name || isDeleting}
            startIcon={isDeleting ? <CircularProgress size={20} color="inherit" /> : null}
            sx={{ minWidth: '180px' }}
          >
            {isDeleting ? 'Deleting...' : 'Delete Permanently'}
          </Button>
        </DialogActions>      </Dialog>
      
      {/* Private Key Display Dialog */}
      <Dialog
        open={!!privateKey}
        onClose={handleClosePrivateKeyDisplay}
        aria-labelledby="private-key-display-dialog-title"
        maxWidth="md"
        fullWidth
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
        <DialogTitle id="private-key-display-dialog-title" sx={{ pb: 1, display: 'flex', alignItems: 'center' }}>
          <VisibilityIcon color="primary" sx={{ mr: 1 }} />
          <Typography variant="h6" component="div" fontWeight={600}>
            Private Key for "{currentWallet?.name}"
          </Typography>
        </DialogTitle>        <DialogContent>
          <Box 
            sx={{ 
              mb: 3,
              p: 2,
              borderRadius: 1,
              backgroundColor: isDarkMode ? 'rgba(229, 115, 115, 0.1)' : 'rgba(211, 47, 47, 0.08)',
              border: '1px solid',
              borderColor: isDarkMode ? 'rgba(229, 115, 115, 0.3)' : 'rgba(211, 47, 47, 0.2)',
              display: 'flex',
              flexDirection: 'column',
              gap: 1
            }}
          >
            <Typography 
              variant="body2" 
              sx={{ 
                fontWeight: 700,
                color: isDarkMode ? '#ffcdd2' : '#c62828',
                display: 'flex',
                alignItems: 'center',
                gap: 0.5
              }}
            >
              ⚠️ KEEP THIS PRIVATE KEY SECURE
            </Typography>
            <Typography 
              variant="body2" 
              sx={{ 
                color: isDarkMode ? 'rgba(255, 255, 255, 0.8)' : 'rgba(0, 0, 0, 0.7)',
                fontSize: '0.9rem'
              }}
            >
              Never share this private key with anyone. Anyone with access to this key can control your wallet and funds.
            </Typography>
          </Box>
          
          <Typography variant="body2" sx={{ mb: 2 }}>
            Master Private Key:
          </Typography>
          
          <Box 
            sx={{ 
              p: 2,
              bgcolor: isDarkMode ? 'rgba(0, 0, 0, 0.3)' : 'rgba(0, 0, 0, 0.05)',
              borderRadius: 1,
              fontFamily: 'monospace',
              fontSize: '0.9rem',
              wordBreak: 'break-all',
              border: '1px solid',
              borderColor: isDarkMode ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.1)',
              maxHeight: '200px',
              overflowY: 'auto'
            }}
          >
            {privateKey}
          </Box>
        </DialogContent>        <DialogActions sx={{ px: 3, pb: 2 }}>          <Box sx={{ display: 'flex', alignItems: 'center', width: '100%' }}>
            <Box sx={{ ml: 'auto', display: 'flex', gap: 1 }}>              <Button 
                onClick={() => {
                  navigator.clipboard.writeText(privateKey);
                  setShowCopySuccess(true);
                  // Auto-hide after 1.5 seconds (faster)
                  setTimeout(() => setShowCopySuccess(false), 1500);
                }}
                variant="outlined"
                color={showCopySuccess ? "success" : "primary"}
                sx={{ 
                  minWidth: '160px', // Fixed width to prevent resizing
                  borderColor: showCopySuccess ? 'success.main' : undefined,
                  color: showCopySuccess ? 'success.main' : undefined,
                  '&:hover': {
                    borderColor: showCopySuccess ? 'success.dark' : undefined,
                    // Remove backgroundColor to prevent green fill
                  }
                }}
              >
                {showCopySuccess ? "Copied!" : "Copy to Clipboard"}
              </Button>
              <Button onClick={handleClosePrivateKeyDisplay} variant="contained" color="primary">
                Close
              </Button>
            </Box>
          </Box>        </DialogActions>
      </Dialog>
      
      <Snackbar 
        open={showErrorAlert} 
        autoHideDuration={6000} 
        onClose={handleCloseErrorAlert}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      >
        <Alert 
          onClose={handleCloseErrorAlert} 
          severity="error"
          variant="filled"
          sx={{ width: '100%' }}
        >
          {error}        </Alert>
      </Snackbar>
      
      {/* Secure Wallet Dialog */}
      <SecureWalletDialog 
        open={secureDialogOpen}
        onClose={handleCloseSecureDialog}
        walletName={currentWallet?.name || ""}
        onSuccess={handleSuccessfulSecurity}
      />
    </>
  );
}

// Mining Threads Settings Component
function MiningThreadsSection() {
  const { appSettings, updateMiningThreads } = useAppSettings();
  const [isUpdating, setIsUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [maxCores, setMaxCores] = useState<number>(1);
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';

  // Get available CPU cores on component mount
  useEffect(() => {
    const getCpuCores = async () => {
      try {
        const cores = await invoke<number>('get_cpu_cores');
        setMaxCores(cores);
      } catch (err) {
        console.error('Failed to get CPU cores:', err);
        setMaxCores(1); // fallback to 1 core
      }
    };
    getCpuCores();
  }, []);

  const handleThreadsChange = async (_event: Event, newValue: number | number[]) => {
    const threads = typeof newValue === 'number' ? newValue : newValue[0];
    setIsUpdating(true);
    setError(null);

    try {
      await updateMiningThreads(threads);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update mining threads');
    } finally {
      setIsUpdating(false);
    }
  };

  const currentThreads = appSettings?.mining_threads || 1;

  return (
    <Box>
      <FormControl component="fieldset" sx={{ width: '100%' }}>
        <FormLabel component="legend" sx={{ 
          mb: 2, 
          color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : 'rgba(0, 0, 0, 0.87)',
          fontWeight: 600
        }}>
          Mining Threads
        </FormLabel>
        
        <Typography variant="body2" sx={{ 
          mb: 2, 
          color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)' 
        }}>
          Number of CPU threads to use for mining operations. Higher values may improve mining performance but increase CPU usage.
        </Typography>

        <Box sx={{ px: 2, mb: 2 }}>
          <Slider
            value={currentThreads}
            onChange={handleThreadsChange}
            disabled={isUpdating}
            min={1}
            max={maxCores}
            step={1}
            marks={Array.from({length: maxCores}, (_, i) => ({
              value: i + 1,
              label: i + 1 === 1 ? '1' : i + 1 === maxCores ? `${maxCores}` : ''
            }))}
            valueLabelDisplay="on"
            sx={{
              color: isDarkMode ? '#90caf9' : '#1976d2',
              '& .MuiSlider-thumb': {
                backgroundColor: isDarkMode ? '#90caf9' : '#1976d2',
              },
              '& .MuiSlider-track': {
                backgroundColor: isDarkMode ? '#90caf9' : '#1976d2',
              },
              '& .MuiSlider-rail': {
                backgroundColor: isDarkMode ? 'rgba(255, 255, 255, 0.3)' : 'rgba(0, 0, 0, 0.26)',
              }
            }}
          />
        </Box>

        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mt: 1 }}>
          <Typography variant="body2" sx={{ 
            color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)' 
          }}>
            Current: {currentThreads} thread{currentThreads !== 1 ? 's' : ''} / {maxCores} available
          </Typography>
          
          {isUpdating && (
            <CircularProgress size={16} sx={{ color: isDarkMode ? '#90caf9' : '#1976d2' }} />
          )}
        </Box>

        {error && (
          <Alert severity="error" sx={{ mt: 2 }}>
            {error}
          </Alert>
        )}
      </FormControl>
    </Box>
  );
}