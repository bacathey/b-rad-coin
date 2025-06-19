import { useState } from 'react';
import { 
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Typography,
  Box,
  TextField,
  Paper,
  Alert,
  Fade,
  useTheme // Import useTheme
} from '@mui/material';
import ContentCopyIcon from '@mui/icons-material/ContentCopy';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';
import WarningIcon from '@mui/icons-material/Warning';

interface SeedPhraseDialogProps {
  open: boolean;
  seedPhrase: string;
  onClose: () => void;
  onContinue: () => void;
}

export default function SeedPhraseDialog({ 
  open, 
  seedPhrase, 
  onClose, 
  onContinue 
}: SeedPhraseDialogProps) {
  const theme = useTheme(); // Add useTheme hook
  const isDarkMode = theme.palette.mode === 'dark'; // Check dark mode
  const [copied, setCopied] = useState(false);
  // Handle copying seed phrase to clipboard
  const handleCopyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(seedPhrase);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500); // Reset after 1.5 seconds (same as private key dialog)
    } catch (error) {
      console.error('Failed to copy:', error);
    }
  };

  // Split seed phrase into words array
  const seedWords = seedPhrase.split(' ');

  return (
    <Dialog 
      open={open} 
      onClose={onClose}
      maxWidth="sm"
      fullWidth
      TransitionComponent={Fade}
      TransitionProps={{ timeout: 500 }}
      // Apply consistent styling
      sx={{
        zIndex: theme.zIndex.drawer + 3, // Match zIndex
        '& .MuiPaper-root': {
          background: isDarkMode 
            ? 'linear-gradient(145deg, #0a1929 0%, #132f4c 100%)' 
            : 'linear-gradient(145deg, #ffffff 0%, #f5f7fa 100%)',
          borderRadius: '12px',
          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
          border: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.08)',
          transition: 'all 500ms cubic-bezier(0.4, 0, 0.2, 1) !important'
        },
        '& .MuiDialog-container': {
          transition: 'all 500ms cubic-bezier(0.4, 0, 0.2, 1)'
        }
      }}
    >      <DialogTitle sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <WarningIcon 
          sx={{ 
            color: isDarkMode ? '#f48fb1' : '#c62828'
          }} 
        />
        <Typography variant="h5">Save Your Seed Phrase</Typography>
      </DialogTitle>
        <DialogContent>        <Box 
          sx={{ 
            mb: 2,
            p: 2,
            borderRadius: 1,
            backgroundColor: isDarkMode ? 'rgba(244, 67, 54, 0.1)' : 'rgba(211, 47, 47, 0.08)',
            border: '1px solid',
            borderColor: isDarkMode ? 'rgba(244, 67, 54, 0.3)' : 'rgba(211, 47, 47, 0.2)',
            display: 'flex',
            flexDirection: 'column',
            gap: 1
          }}
        >
          <Typography 
            variant="body2" 
            sx={{ 
              fontWeight: 600,
              color: isDarkMode ? '#f48fb1' : '#c62828',
              fontSize: '0.95rem'
            }}
          >
            This seed phrase is the only way to recover your wallet if your device is lost, stolen, or damaged.
          </Typography>
        </Box>
        
        <Typography variant="body1" paragraph>
          This 12-word recovery phrase is used to generate the private key for your new wallet. 
          Write down these words in order and keep them in a secure location.
        </Typography>
        
        <Paper 
          elevation={3} 
          sx={{ 
            p: 3, 
            mb: 3, 
            borderRadius: 2,
            background: (theme) => theme.palette.mode === 'dark' 
              ? 'linear-gradient(145deg, #132f4c 0%, #173a5e 100%)' 
              : 'linear-gradient(145deg, #f5f7fa 0%, #ffffff 100%)'
          }}
        >
          <Box sx={{ 
            display: 'grid', 
            gridTemplateColumns: 'repeat(3, 1fr)',
            gap: 2
          }}>
            {seedWords.map((word, index) => (
              <TextField
                key={index}
                variant="outlined"
                size="small"
                fullWidth
                value={`${index + 1}. ${word}`}
                InputProps={{
                  readOnly: true
                }}
                sx={{ 
                  '& .MuiOutlinedInput-root': {
                    fontFamily: 'monospace',
                    fontWeight: 600
                  }
                }}
              />
            ))}
          </Box>
            <Box sx={{ display: 'flex', justifyContent: 'center', mt: 2 }}>
            <Button
              variant="outlined"
              color={copied ? "success" : "primary"}
              startIcon={copied ? <CheckCircleIcon /> : <ContentCopyIcon />}
              onClick={handleCopyToClipboard}
              sx={{ 
                textTransform: 'none',
                minWidth: '160px', // Fixed width to prevent resizing
                borderColor: copied ? 'success.main' : undefined,
                color: copied ? 'success.main' : undefined,
                '&:hover': {
                  borderColor: copied ? 'success.dark' : undefined,
                  // Remove backgroundColor to prevent green fill
                }
              }}
            >
              {copied ? 'Copied!' : 'Copy to Clipboard'}
            </Button>
          </Box>
        </Paper>
        
        <Alert severity="info">
          <Typography fontWeight={500}>
            Never share your seed phrase with anyone. Anyone with this phrase can access and take control of your wallet.
          </Typography>
        </Alert>
      </DialogContent>
      
      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button 
          onClick={onClose} 
          color="inherit"
          sx={{ textTransform: 'none' }}
        >
          Cancel
        </Button>
        <Button 
          onClick={onContinue} 
          variant="contained"
          color="primary"
          sx={{ textTransform: 'none', fontWeight: 500 }}
        >
          I've Saved It
        </Button>
      </DialogActions>
    </Dialog>
  );
}
