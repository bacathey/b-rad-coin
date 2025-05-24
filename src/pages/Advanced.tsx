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
  Snackbar
} from '@mui/material';
import { useState, useEffect } from 'react';
import { useWallet } from '../context/WalletContext';
import FolderIcon from '@mui/icons-material/Folder';
import LockIcon from '@mui/icons-material/Lock';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import DeleteIcon from '@mui/icons-material/Delete';
import SecurityIcon from '@mui/icons-material/Security';
import SecureWalletDialog from '../components/SecureWalletDialog';

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
      <Box sx={{ width: '100%', maxWidth: 1200, mx: 'auto' }}>        {/* Wallet File Location Card */}
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
        
        {/* Mining Card */}
        <Box>
          <Card sx={{ ...cardStyle }}>
            <CardContent>
              <Typography variant="h6" sx={{ fontWeight: 600 }}>
                Mining
              </Typography>
              <Typography variant="body2" sx={{ mt: 1 }}>
                Manage your mining operations and settings.
              </Typography>
            </CardContent>
          </Card>
        </Box>
      </Box>
    </Box>
  );
}

// Wallet file component
function WalletLocationSection() {  const { 
    currentWallet, 
    isWalletOpen, 
    isWalletSecured, 
    getCurrentWalletPath, 
    openWalletFolder,
    openWalletFolderWithShell,
    deleteWallet,
    refreshWalletDetails 
  } = useWallet();
  
  const [walletPath, setWalletPath] = useState<string | null>(null);
  const [displayPath, setDisplayPath] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showErrorAlert, setShowErrorAlert] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [confirmWalletName, setConfirmWalletName] = useState("");
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [secureDialogOpen, setSecureDialogOpen] = useState(false);
  
  useEffect(() => {
    if (isWalletOpen && currentWallet) {
      fetchWalletPath();
    } else {
      setWalletPath(null);
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
        // Store the original path for operations (opening folder)
        setWalletPath(path);
        
        // Create a normalized path for display, ensuring Windows-style backslashes
        const normalizedPath = path.replace(/\//g, '\\');
        setDisplayPath(normalizedPath);
        
        console.log(`Original path: ${path}`);
        console.log(`Display path: ${normalizedPath}`);
      } else {
        setWalletPath(null);
        setDisplayPath(null);
      }
    } catch (error) {
      console.error('Failed to get wallet path:', error);
      setError('Failed to get wallet path. Please check if the wallet still exists.');
      setShowErrorAlert(true);
    } finally {
      setIsLoading(false);
    }
  };
    // Compare paths and handle folder opening more directly
  const handleOpenFolder = async () => {
    if (!walletPath) {
      console.error('Cannot open folder: walletPath is null or undefined');
      return;
    }
    
    try {
      // Log the path being used
      console.log(`Opening wallet folder with path: "${walletPath}"`);
      console.log(`Path type: ${typeof walletPath}`);
      console.log(`Path length: ${walletPath.length}`);
      
      // If the path has unusual characters, log them
      if (walletPath.includes('\u0000') || 
          walletPath.includes('\r') || 
          walletPath.includes('\n') ||
          walletPath.includes('\t')) {
        console.warn('Path contains special characters that may cause issues');
        console.log('Path with visible special chars:', 
                    walletPath.replace(/\u0000/g, '\\0')
                             .replace(/\r/g, '\\r')
                             .replace(/\n/g, '\\n')
                             .replace(/\t/g, '\\t'));
      }
      
      // Try extracting the wallet name from the path to use as a fallback
      const pathParts = walletPath.split(/[\/\\]/);
      let walletName = currentWallet?.name;
      
      // Try to find the wallet name in the path parts if not available from currentWallet
      if (!walletName) {
        for (const part of pathParts) {
          if (part && part.length > 0 && !part.includes('.')) {
            walletName = part;
            break;
          }
        }
      }
      
      console.log(`Extracted wallet name: ${walletName}`);
      
      // Try opening with the opener library first
      console.log('Attempting to open folder with opener library...');
      let result = await openWalletFolder(walletPath);
      console.log(`openWalletFolder result: ${result}`);
      
      // If that fails, try with the shell command fallback
      if (!result) {
        console.log('Opener library failed, trying with shell command fallback...');
        result = await openWalletFolderWithShell(walletPath);
        console.log(`openWalletFolderWithShell result: ${result}`);
      }
      
      // If that still fails and we have a wallet name, try a different path construction
      if (!result && walletName) {
        const fallbackPath = `C:\\Users\\bacat\\source\\repos\\b-rad-coin\\src-tauri\\wallets\\${walletName}`;
        console.log(`Both methods failed. Trying again with fallback path: ${fallbackPath}`);
        
        // Try both methods with the fallback path
        result = await openWalletFolder(fallbackPath);
        if (!result) {
          result = await openWalletFolderWithShell(fallbackPath);
        }
      }
      
      if (!result) {
        const errorMessage = 'Failed to open wallet folder. The folder may no longer exist at the specified location.';
        console.error(errorMessage);
        setError(errorMessage);
        setShowErrorAlert(true);
      } else {
        console.log('Successfully opened folder');
      }
    } catch (error) {
      console.error('Error opening wallet folder:', error);
      setError(`Error opening wallet folder: ${error instanceof Error ? error.message : 'Unknown error'}`);
      setShowErrorAlert(true);
    }
  };
    // Debug function to test folder opening with a fixed path
  const testOpenFolder = async () => {
    try {
      // Use a known-good path to test
      const testPath = "C:\\Users\\bacat\\source\\repos\\b-rad-coin\\src-tauri\\wallets";
      console.log(`Testing folder opening with fixed path: "${testPath}"`);
      
      // Try opener library first
      console.log("Testing opener library method...");
      let result = await openWalletFolder(testPath);
      console.log(`Test openWalletFolder result: ${result}`);
      
      if (!result) {
        // Try shell command fallback
        console.log("Opening with opener library failed, testing shell command fallback...");
        result = await openWalletFolderWithShell(testPath);
        console.log(`Test openWalletFolderWithShell result: ${result}`);
      }
      
      if (!result) {
        console.error('Test failed: Could not open test folder with either method');
        setError('Failed to open test folder. This may indicate a system-level issue.');
        setShowErrorAlert(true);
      } else {
        console.log('Test succeeded: Opened test folder');
      }
    } catch (error) {
      console.error('Error in test folder open:', error);
    }
  };
  
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
                </Typography>
                <Chip
                  icon={isWalletSecured ? <LockIcon /> : <LockOpenIcon />}
                  label={isWalletSecured ? "Password Protected" : "No Password"}
                  size="small"
                  color={isWalletSecured ? "warning" : "success"}
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
        </Grid>
        <Grid item xs={12} md={4} sx={{ 
          display: 'flex', 
          gap: 2,
          flexDirection: { xs: 'column', sm: 'row' },
          justifyContent: { xs: 'flex-start', md: 'flex-end' }
        }}>
          <Tooltip title="Delete this wallet">
            <Button 
              variant="outlined" 
              color="error" 
              startIcon={<DeleteIcon />} 
              onClick={handleOpenDeleteDialog}
              disabled={isLoading}
            >
              Delete Wallet
            </Button>
          </Tooltip>          {!isWalletSecured && (
            <Tooltip title="Secure this wallet with a password">
              <Button 
                variant="outlined" 
                color="warning" 
                startIcon={<SecurityIcon />}
                onClick={handleOpenSecureDialog}
                disabled={isLoading}
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
      >
        <DialogTitle id="delete-wallet-dialog-title">
          Delete Wallet
        </DialogTitle>
        <DialogContent>
          <DialogContentText sx={{ mb: 2 }}>
            Are you sure you want to delete the wallet "{currentWallet?.name}"? This action cannot be undone.
          </DialogContentText>
          <DialogContentText sx={{ mb: 2, fontWeight: 'bold', color: 'error.main' }}>
            To confirm, please type the wallet name.
          </DialogContentText>
          {deleteError && (
            <Alert severity="error" sx={{ mb: 2 }}>
              {deleteError}
            </Alert>
          )}
          <TextField
            autoFocus
            margin="dense"
            label="Wallet Name"
            fullWidth
            variant="outlined"
            value={confirmWalletName}
            onChange={(e) => setConfirmWalletName(e.target.value)}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={handleCloseDeleteDialog}>Cancel</Button>
          <Button 
            onClick={handleDeleteWallet} 
            color="error"
            disabled={!confirmWalletName || isDeleting}
          >
            {isDeleting ? 'Deleting...' : 'Delete Permanently'}
          </Button>
        </DialogActions>
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
          {error}
        </Alert>
      </Snackbar>      {/* Secure Wallet Dialog */}
      <SecureWalletDialog 
        open={secureDialogOpen}
        onClose={handleCloseSecureDialog}
        walletName={currentWallet?.name || ""}
        onSuccess={handleSuccessfulSecurity}
      />
    </>
  );
}