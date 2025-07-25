import { 
  Typography, 
  Box, 
  TextField,
  Button,
  Stack,
  Tabs,
  Tab,
  InputAdornment,
  Divider,
  Paper,
  useTheme,
  Grid
} from '@mui/material';
import { useState } from 'react';

// Icons
import SendIcon from '@mui/icons-material/Send';
import QrCodeIcon from '@mui/icons-material/QrCode';
import ContentCopyIcon from '@mui/icons-material/ContentCopy';
import DownloadIcon from '@mui/icons-material/Download';
import FileUploadIcon from '@mui/icons-material/FileUpload';
import AttachMoneyIcon from '@mui/icons-material/AttachMoney';
import PersonIcon from '@mui/icons-material/Person';
import NoteAltIcon from '@mui/icons-material/NoteAlt';

export default function SendReceive() {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  const [tabValue, setTabValue] = useState(0);
  
  // State for form fields
  const [recipientAddress, setRecipientAddress] = useState('');
  const [sendAmount, setSendAmount] = useState('');
  const [sendNote, setSendNote] = useState('');
  
  // Sample receive address for the wallet
  const walletAddress = '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa';

  // Paper style - used for the main container
  const paperStyle = isDarkMode ? {
    background: 'rgba(19, 47, 76, 0.4)',
    backdropFilter: 'blur(10px)',
    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
    border: '1px solid rgba(255, 255, 255, 0.1)'
  } : {
    background: 'white',
    boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
    border: '1px solid rgba(0, 0, 0, 0.08)'
  };

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setTabValue(newValue);
  };

  // Function to copy address to clipboard
  const copyToClipboard = () => {
    navigator.clipboard.writeText(walletAddress);
    // In a real app, you would show a notification that the address was copied
  };

  const handleSendSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    // In a real app, this would process the transaction
    console.log('Send transaction', { recipientAddress, sendAmount, sendNote });
    // Reset form after submission
    setRecipientAddress('');
    setSendAmount('');
    setSendNote('');
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
          fontWeight: 600
        }}
      >
        Send & Receive
      </Typography>

      <Paper 
        sx={{ 
          width: '100%', 
          maxWidth: 900,
          mx: 'auto',
          mb: 4,
          overflow: 'hidden',
          borderRadius: 2,
          ...paperStyle
        }}
      >
        <Tabs
          value={tabValue}
          onChange={handleTabChange}
          variant="fullWidth"
          sx={{
            '& .MuiTabs-indicator': {
              backgroundColor: isDarkMode ? '#90caf9' : '#1a237e',
              height: 3
            },
            '& .MuiTab-root': {
              color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)',
              fontWeight: 500,
              py: 2,
              '&.Mui-selected': {
                color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#1a237e',
                fontWeight: 600
              }
            }
          }}
        >
          <Tab 
            icon={<SendIcon />} 
            label="SEND" 
            iconPosition="start"
            sx={{ fontSize: '0.95rem' }}
          />
          <Tab 
            icon={<QrCodeIcon />} 
            label="RECEIVE" 
            iconPosition="start" 
            sx={{ fontSize: '0.95rem' }}
          />
        </Tabs>

        <Box sx={{ p: 3 }}>
          {/* SEND Tab */}
          {tabValue === 0 && (
            <Box component="form" onSubmit={handleSendSubmit}>
              <Grid container spacing={3}>
                <Grid item xs={12}>
                  <TextField
                    fullWidth
                    required
                    id="recipient"
                    label="Recipient Address"
                    placeholder="Enter Bradcoin address"
                    variant="outlined"
                    value={recipientAddress}
                    onChange={(e) => setRecipientAddress(e.target.value)}
                    InputProps={{
                      startAdornment: (
                        <InputAdornment position="start">
                          <PersonIcon color={isDarkMode ? "primary" : "primary"} />
                        </InputAdornment>
                      ),
                    }}
                    sx={{
                      ...(isDarkMode ? {
                        '& .MuiOutlinedInput-root': {
                          '& fieldset': {
                            borderColor: 'rgba(255, 255, 255, 0.15)',
                          },
                          '&:hover fieldset': {
                            borderColor: 'rgba(255, 255, 255, 0.25)',
                          },
                          '&.Mui-focused fieldset': {
                            borderColor: 'rgba(144, 202, 249, 0.6)',
                          }
                        },
                        '& .MuiInputLabel-root': {
                          color: 'rgba(255, 255, 255, 0.7)',
                        },
                        '& .MuiInputBase-input': {
                          color: 'rgba(255, 255, 255, 0.9)',
                        }
                      } : {})
                    }}
                  />
                </Grid>
                <Grid item xs={12}>
                  <TextField
                    fullWidth
                    required
                    id="amount"
                    label="Amount"
                    placeholder="0.00"
                    type="number"
                    variant="outlined"
                    value={sendAmount}
                    onChange={(e) => setSendAmount(e.target.value)}
                    InputProps={{
                      startAdornment: (
                        <InputAdornment position="start">
                          <AttachMoneyIcon color={isDarkMode ? "primary" : "primary"} />
                        </InputAdornment>
                      ),
                      endAdornment: (
                        <InputAdornment position="end">
                          <Typography color={isDarkMode ? "primary" : "primary"}>BRAD</Typography>
                        </InputAdornment>
                      ),
                    }}
                    inputProps={{
                      step: 0.00001,
                      min: 0
                    }}
                    sx={{
                      ...(isDarkMode ? {
                        '& .MuiOutlinedInput-root': {
                          '& fieldset': {
                            borderColor: 'rgba(255, 255, 255, 0.15)',
                          },
                          '&:hover fieldset': {
                            borderColor: 'rgba(255, 255, 255, 0.25)',
                          },
                          '&.Mui-focused fieldset': {
                            borderColor: 'rgba(144, 202, 249, 0.6)',
                          }
                        },
                        '& .MuiInputLabel-root': {
                          color: 'rgba(255, 255, 255, 0.7)',
                        },
                        '& .MuiInputBase-input': {
                          color: 'rgba(255, 255, 255, 0.9)',
                        }
                      } : {})
                    }}
                  />
                </Grid>
                <Grid item xs={12}>
                  <TextField
                    fullWidth
                    id="note"
                    label="Note (Optional)"
                    placeholder="Add a note to this transaction"
                    variant="outlined"
                    value={sendNote}
                    onChange={(e) => setSendNote(e.target.value)}
                    multiline
                    rows={3}
                    InputProps={{
                      startAdornment: (
                        <InputAdornment position="start" sx={{ alignSelf: 'flex-start', marginTop: '16px' }}>
                          <NoteAltIcon color={isDarkMode ? "primary" : "primary"} />
                        </InputAdornment>
                      ),
                    }}
                    sx={{
                      ...(isDarkMode ? {
                        '& .MuiOutlinedInput-root': {
                          '& fieldset': {
                            borderColor: 'rgba(255, 255, 255, 0.15)',
                          },
                          '&:hover fieldset': {
                            borderColor: 'rgba(255, 255, 255, 0.25)',
                          },
                          '&.Mui-focused fieldset': {
                            borderColor: 'rgba(144, 202, 249, 0.6)',
                          }
                        },
                        '& .MuiInputLabel-root': {
                          color: 'rgba(255, 255, 255, 0.7)',
                        },
                        '& .MuiInputBase-input': {
                          color: 'rgba(255, 255, 255, 0.9)',
                        }
                      } : {})
                    }}
                  />
                </Grid>
                <Grid item xs={12} sx={{ mt: 2 }}>
                  <Button
                    type="submit"
                    variant="contained"
                    size="large"
                    fullWidth
                    startIcon={<SendIcon />}
                    sx={{ 
                      py: 1.5,
                      fontWeight: 600,
                      ...(isDarkMode ? {
                        background: 'linear-gradient(90deg, #0d2b59, #2979ff)',
                        '&:hover': {
                          background: 'linear-gradient(90deg, #0d3074, #448aff)',
                          boxShadow: '0 4px 20px rgba(41, 121, 255, 0.5)',
                        }
                      } : {
                        background: 'linear-gradient(90deg, #3949ab, #42a5f5)',
                        boxShadow: '0 2px 10px rgba(57, 73, 171, 0.3)',
                        '&:hover': {
                          background: 'linear-gradient(90deg, #3f51b5, #64b5f6)',
                          boxShadow: '0 4px 15px rgba(57, 73, 171, 0.4)',
                        }
                      })
                    }}
                  >
                    Send Bradcoin
                  </Button>
                </Grid>
              </Grid>
            </Box>
          )}

          {/* RECEIVE Tab */}
          {tabValue === 1 && (
            <Grid container spacing={3}>
              <Grid item xs={12} md={6} sx={{ 
                display: 'flex', 
                flexDirection: 'column',
                justifyContent: 'center',
                alignItems: 'center'
              }}>
                {/* QR Code placeholder - in a real app, this would be an actual QR code */}
                <Box sx={{ 
                  width: 200, 
                  height: 200, 
                  border: '1px solid',
                  borderColor: isDarkMode ? 'rgba(255, 255, 255, 0.2)' : 'rgba(0, 0, 0, 0.1)',
                  bgcolor: 'white',
                  borderRadius: 1,
                  display: 'flex', 
                  alignItems: 'center',
                  justifyContent: 'center',
                  mb: 2
                }}>
                  <QrCodeIcon sx={{ fontSize: 120, color: '#000' }} />
                </Box>
                <Button
                  variant="outlined"
                  startIcon={<DownloadIcon />}
                  sx={{ mb: 1 }}
                >
                  Save QR Code
                </Button>
              </Grid>
              <Grid item xs={12} md={6}>
                <Typography 
                  variant="h6" 
                  sx={{
                    color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#1a237e',
                    fontWeight: 600,
                    mb: 2
                  }}
                >
                  Your Bitcoin Address
                </Typography>
                
                <TextField
                  fullWidth
                  value={walletAddress}
                  variant="outlined"
                  InputProps={{
                    readOnly: true,
                    endAdornment: (
                      <InputAdornment position="end">
                        <Button 
                          onClick={copyToClipboard}
                          startIcon={<ContentCopyIcon />}
                        >
                          Copy
                        </Button>
                      </InputAdornment>
                    ),
                  }}
                  sx={{
                    mb: 3,
                    ...(isDarkMode ? {
                      '& .MuiOutlinedInput-root': {
                        '& fieldset': {
                          borderColor: 'rgba(255, 255, 255, 0.15)',
                        }
                      },
                      '& .MuiInputBase-input': {
                        color: 'rgba(255, 255, 255, 0.9)',
                      }
                    } : {})
                  }}
                />

                <Divider sx={{ my: 2 }}>
                  <Typography 
                    variant="body2" 
                    sx={{ 
                      color: isDarkMode ? 'rgba(255, 255, 255, 0.6)' : 'rgba(0, 0, 0, 0.6)',
                      px: 1
                    }}
                  >
                    OR
                  </Typography>
                </Divider>

                <Stack spacing={2}>
                  <Button
                    variant="contained"
                    startIcon={<FileUploadIcon />}
                    sx={{ 
                      py: 1.5,
                      fontWeight: 500,
                      ...(isDarkMode ? {
                        background: 'rgba(41, 121, 255, 0.2)',
                        color: '#90caf9',
                        '&:hover': {
                          background: 'rgba(41, 121, 255, 0.3)',
                        }
                      } : {
                        background: 'rgba(57, 73, 171, 0.1)',
                        color: '#1a237e',
                        '&:hover': {
                          background: 'rgba(57, 73, 171, 0.15)',
                        }
                      })
                    }}
                  >
                    Create Payment Request
                  </Button>
                  
                  <Button
                    variant="outlined"
                    startIcon={<FileUploadIcon />}
                    sx={{ 
                      py: 1.5,
                      fontWeight: 500
                    }}
                  >
                    Generate New Address
                  </Button>
                </Stack>
              </Grid>
            </Grid>
          )}
        </Box>
      </Paper>
    </Box>
  );
}