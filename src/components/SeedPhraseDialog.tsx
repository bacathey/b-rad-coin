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
      setTimeout(() => setCopied(false), 3000); // Reset after 3 seconds
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
    >
      <DialogTitle sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <WarningIcon color="warning" />
        <Typography variant="h5">Save Your Seed Phrase</Typography>
      </DialogTitle>
      
      <DialogContent>
        <Alert severity="warning" sx={{ mb: 2 }}>
          <Typography fontWeight={500}>
            This seed phrase is the only way to recover your wallet if your device is lost, stolen, or damaged.
          </Typography>
        </Alert>
        
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
              color="primary"
              startIcon={copied ? <CheckCircleIcon /> : <ContentCopyIcon />}
              onClick={handleCopyToClipboard}
              sx={{ textTransform: 'none' }}
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
