import { useState } from 'react';
import { 
  Dialog, 
  DialogTitle, 
  DialogContent, 
  DialogActions,
  Button, 
  TextField, 
  Typography, 
  CircularProgress,
  useTheme,
  Alert,
  Fade,
} from '@mui/material';
import LockIcon from '@mui/icons-material/Lock';
import { invoke } from '@tauri-apps/api/core';

interface SecureWalletDialogProps {
  open: boolean;
  onClose: () => void;
  walletName: string;
  onSuccess: () => Promise<void>;
}

export default function SecureWalletDialog({ 
  open, 
  onClose,
  walletName,
  onSuccess 
}: SecureWalletDialogProps) {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  
  // State
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState('');

  // Reset state when dialog opens
  const handleEnter = () => {
    setPassword('');
    setConfirmPassword('');
    setErrorMessage('');
  };

  const handleSecureWallet = async () => {
    // Validate inputs
    if (!password) {
      setErrorMessage('Please enter a password');
      return;
    }
    
    if (password !== confirmPassword) {
      setErrorMessage('Passwords do not match');
      return;
    }

    setIsLoading(true);
    try {
      // Call Rust function to secure the wallet
      const result = await invoke<boolean>('secure_wallet', {
        walletName,
        password
      });
      
      if (result) {
        // Success - call onSuccess callback
        await onSuccess();
        onClose();
      } else {
        setErrorMessage('Failed to secure wallet');
      }
    } catch (error) {
      console.error('Error securing wallet:', error);
      setErrorMessage(`Error: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog
      open={open}
      onClose={!isLoading ? onClose : undefined}
      TransitionComponent={Fade}
      TransitionProps={{ 
        timeout: 400,
        onEnter: handleEnter
      }}
      maxWidth="xs"
      fullWidth
      sx={{
        '& .MuiPaper-root': {
          background: isDarkMode 
            ? 'linear-gradient(145deg, #0a1929 0%, #132f4c 100%)' 
            : 'linear-gradient(145deg, #ffffff 0%, #f5f7fa 100%)',
          borderRadius: '12px',
          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
          border: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.08)',
          transition: 'all 400ms cubic-bezier(0.4, 0, 0.2, 1) !important'
        }
      }}
    >
      <DialogTitle sx={{ 
        display: 'flex', 
        alignItems: 'center', 
        gap: 1,
        pb: 1
      }}>
        <LockIcon 
          color="warning" 
          sx={{ mr: 1 }} 
        />
        <Typography variant="h6" component="div">
          Secure Wallet
        </Typography>
      </DialogTitle>
      
      <DialogContent>
        {errorMessage && (
          <Alert severity="error" sx={{ mb: 2 }}>
            {errorMessage}
          </Alert>
        )}
        
        <Typography variant="body1" sx={{ mb: 3 }}>
          Add password protection to "{walletName}"
        </Typography>
        
        <TextField
          fullWidth
          label="Password"
          type="password"
          variant="outlined"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          sx={{ mb: 2 }}
          required
          disabled={isLoading}
        />
        
        <TextField
          fullWidth
          label="Confirm Password"
          type="password"
          variant="outlined"
          value={confirmPassword}
          onChange={(e) => setConfirmPassword(e.target.value)}
          sx={{ mb: 2 }}
          required
          disabled={isLoading}
          error={password !== confirmPassword && confirmPassword !== ''}
          helperText={password !== confirmPassword && confirmPassword !== '' ? 'Passwords do not match' : ''}
        />
      </DialogContent>
      
      <DialogActions sx={{ p: 2 }}>
        <Button 
          onClick={onClose} 
          disabled={isLoading}
          sx={{ textTransform: 'none' }}
        >
          Cancel
        </Button>
        <Button 
          variant="contained" 
          color="primary"
          onClick={handleSecureWallet}
          disabled={isLoading || !password || password !== confirmPassword}
          startIcon={isLoading ? <CircularProgress size={20} /> : null}
          sx={{ textTransform: 'none' }}
        >
          {isLoading ? 'Securing...' : 'Secure Wallet'}
        </Button>
      </DialogActions>
    </Dialog>
  );
}