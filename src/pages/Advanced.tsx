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
  Fade // Add Fade import
} from '@mui/material';
import { useState, useEffect } from 'react';
import { useWallet } from '../context/WalletContext';
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
    deleteWallet,
    refreshWalletDetails  } = useWallet();
  
  const [displayPath, setDisplayPath] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showErrorAlert, setShowErrorAlert] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [confirmWalletName, setConfirmWalletName] = useState("");
  const [deleteError, setDeleteError] = useState<string | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);
  const [secureDialogOpen, setSecureDialogOpen] = useState(false);
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