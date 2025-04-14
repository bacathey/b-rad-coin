import { useState, useEffect } from 'react';
import { 
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Typography,
  Box,
  Alert,
  Fade,
  FormHelperText,
  useTheme // Import useTheme
} from '@mui/material';
import LockIcon from '@mui/icons-material/Lock';
import ShuffleIcon from '@mui/icons-material/Shuffle';

interface VerifySeedPhraseDialogProps {
  open: boolean;
  seedPhrase: string;
  onClose: () => void;
  onVerified: () => void;
}

export default function VerifySeedPhraseDialog({ 
  open, 
  seedPhrase, 
  onClose, 
  onVerified 
}: VerifySeedPhraseDialogProps) {
  const theme = useTheme(); // Add useTheme hook
  const isDarkMode = theme.palette.mode === 'dark'; // Check dark mode
  const [selectedWords, setSelectedWords] = useState<string[]>([]);
  const [availableWords, setAvailableWords] = useState<string[]>([]);
  const [isCorrect, setIsCorrect] = useState(true);
  const [hasAttemptedVerification, setHasAttemptedVerification] = useState(false);
  
  // Split seed phrase into individual words
  const seedWords = seedPhrase.split(' ');
  
  // Initialize with shuffled seed words
  useEffect(() => {
    if (open) {
      // Reset state when dialog opens
      setSelectedWords([]);
      setIsCorrect(true);
      setHasAttemptedVerification(false);
      
      // Shuffle the seed words
      const shuffled = [...seedWords].sort(() => Math.random() - 0.5);
      setAvailableWords(shuffled);
    }
  }, [open, seedPhrase]);
  
  // Handle selecting a word
  const handleSelectWord = (word: string, index: number) => {
    // Add word to selected array
    setSelectedWords([...selectedWords, word]);
    
    // Remove word from available array
    const newAvailable = [...availableWords];
    newAvailable.splice(index, 1);
    setAvailableWords(newAvailable);
  };
  
  // Handle removing a selected word
  const handleRemoveWord = (word: string, index: number) => {
    // Remove word from selected array
    const newSelected = [...selectedWords];
    newSelected.splice(index, 1);
    setSelectedWords(newSelected);
    
    // Add word back to available array
    setAvailableWords([...availableWords, word]);
  };
  
  // Handle verification
  const handleVerify = () => {
    setHasAttemptedVerification(true);
    
    // Check if selected words match original seed phrase
    const selectedPhrase = selectedWords.join(' ');
    const isMatching = selectedPhrase === seedPhrase;
    
    setIsCorrect(isMatching);
    
    if (isMatching) {
      onVerified();
    }
  };
  
  // Handle reshuffling available words
  const handleReshuffle = () => {
    setAvailableWords(prevWords => [...prevWords].sort(() => Math.random() - 0.5));
  };

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
        <LockIcon color="primary" />
        <Typography variant="h5">Verify Your Seed Phrase</Typography>
      </DialogTitle>
      
      <DialogContent>
        <Typography variant="body1" paragraph>
          To confirm you've saved your seed phrase, please recreate it by selecting the words in the correct order.
        </Typography>
        
        {/* Selected Words Section */}
        <Box sx={{ 
          p: 3, 
          mb: 3, 
          border: '1px dashed',
          borderColor: 'primary.main',
          borderRadius: 2,
          minHeight: '100px',
          background: (theme) => theme.palette.mode === 'dark' 
            ? 'rgba(0, 30, 60, 0.3)' 
            : 'rgba(240, 247, 255, 0.8)'
        }}>
          <Box sx={{ 
            display: 'flex', 
            flexWrap: 'wrap',
            gap: 1
          }}>
            {selectedWords.map((word, index) => (
              <Button
                key={`selected-${word}-${index}`}
                variant="contained"
                size="small"
                onClick={() => handleRemoveWord(word, index)}
                sx={{ 
                  textTransform: 'none',
                  fontWeight: 600
                }}
              >
                {word}
              </Button>
            ))}
            
            {/* Placeholder when no words are selected */}
            {selectedWords.length === 0 && (
              <Typography variant="body2" color="text.secondary" sx={{ fontStyle: 'italic' }}>
                Click the words below to recreate your seed phrase...
              </Typography>
            )}
          </Box>
        </Box>
        
        {/* Error message when verification fails */}
        {!isCorrect && hasAttemptedVerification && (
          <Alert severity="error" sx={{ mb: 3 }}>
            The seed phrase doesn't match. Please try again.
          </Alert>
        )}
        
        {/* Available Words Section */}
        <Box sx={{ mb: 2 }}>
          <Box sx={{ 
            display: 'flex', 
            alignItems: 'center',
            justifyContent: 'space-between',
            mb: 1
          }}>
            <Typography variant="body2" fontWeight={500}>
              Available Words
            </Typography>
            
            <Button
              startIcon={<ShuffleIcon />}
              size="small"
              onClick={handleReshuffle}
              sx={{ textTransform: 'none' }}
            >
              Shuffle
            </Button>
          </Box>
          
          <Box sx={{ 
            display: 'flex', 
            flexWrap: 'wrap',
            gap: 1
          }}>
            {availableWords.map((word, index) => (
              <Button
                key={`available-${word}-${index}`}
                variant="outlined"
                size="small"
                onClick={() => handleSelectWord(word, index)}
                sx={{ 
                  textTransform: 'none',
                  fontWeight: 600
                }}
              >
                {word}
              </Button>
            ))}
          </Box>
        </Box>
        
        <FormHelperText sx={{ mt: 1 }}>
          Click on words to add them to your phrase, and click again to remove them.
        </FormHelperText>
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
          onClick={handleVerify} 
          variant="contained"
          color="primary"
          disabled={selectedWords.length !== seedWords.length}
          sx={{ textTransform: 'none', fontWeight: 500 }}
        >
          Verify
        </Button>
      </DialogActions>
    </Dialog>
  );
}
