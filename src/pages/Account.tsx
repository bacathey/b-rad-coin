// filepath: c:\Users\bacat\source\repos\b-rad-coin\src\pages\Account.tsx
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
  Typography, 
  Box, 
  Stack, 
  TextField, 
  Button, 
  Card, 
  CardContent,
  useTheme,
  Chip,
  IconButton,
  Tooltip,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  List,
  ListItem,
  ListItemText
} from '@mui/material';
import { 
  ContentCopy, 
  AccountBalanceWallet, 
  Key, 
  ExpandMore,
  Security,
  Visibility,
  VisibilityOff
} from '@mui/icons-material';
import { useWallet } from '../context/WalletContext';
import type { CurrentWalletInfo } from '../types/wallet';

interface AccountProps {
  greetMsg: string;
  name: string;
  setName: (name: string) => void;
  greet: () => void;
}

export default function Account({ greetMsg, name, setName, greet }: AccountProps) {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  const { isWalletOpen, currentWallet } = useWallet();
  
  const [walletInfo, setWalletInfo] = useState<CurrentWalletInfo | null>(null);
  const [showMasterPublicKey, setShowMasterPublicKey] = useState(false);

  // Load wallet info when component mounts or wallet changes
  useEffect(() => {
    if (isWalletOpen && currentWallet) {
      loadWalletInfo();
    } else {
      setWalletInfo(null);
    }
  }, [isWalletOpen, currentWallet]);

  const loadWalletInfo = async () => {
    try {
      const info = await invoke<CurrentWalletInfo | null>('get_current_wallet_info');
      setWalletInfo(info);
    } catch (error) {
      console.error('Failed to load wallet info:', error);
    }
  };

  const copyToClipboard = async (text: string, description: string) => {
    try {
      await navigator.clipboard.writeText(text);
      // You could add a toast notification here
      console.log(`${description} copied to clipboard`);
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
    }
  };

  const formatBalance = (balance: number) => {
    return (balance / 100000000).toFixed(8); // Convert satoshis to BTC
  };

  return (
    <Box 
      sx={{ 
        textAlign: 'center',
        minHeight: 'calc(100vh - 64px - 48px)',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'flex-start',
        width: '100%',
        maxWidth: '100%',
        pt: 3,
        mx: 'auto',
        position: 'static',
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
        Account Dashboard
      </Typography>

      {isWalletOpen && walletInfo ? (
        <Stack spacing={3} sx={{ width: '100%', maxWidth: 800 }}>
          {/* Wallet Overview Card */}
          <Card sx={{ 
            borderRadius: 2,
            ...(isDarkMode ? {
              background: 'rgba(19, 47, 76, 0.6)',
              backdropFilter: 'blur(10px)',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
              border: '1px solid rgba(255, 255, 255, 0.1)'
            } : {
              background: 'linear-gradient(180deg, #ffffff 0%, #f5f7fa 100%)',
              boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
              border: '1px solid rgba(0, 0, 0, 0.08)',
            })
          }}>
            <CardContent sx={{ p: 3 }}>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                <AccountBalanceWallet sx={{ mr: 2, color: 'primary.main' }} />
                <Typography variant="h5" component="h2" sx={{ fontWeight: 600 }}>
                  {walletInfo.name}
                </Typography>
                <Box sx={{ ml: 'auto', display: 'flex', gap: 1 }}>
                  {walletInfo.is_secured && (
                    <Chip 
                      icon={<Security />} 
                      label="Secured" 
                      color="success" 
                      size="small" 
                    />
                  )}
                </Box>
              </Box>
              
              <Typography 
                variant="h6" 
                sx={{ 
                  mb: 2, 
                  color: isDarkMode ? 'rgba(255, 255, 255, 0.8)' : 'rgba(0, 0, 0, 0.7)',
                  fontWeight: 500
                }}
              >
                Balance: {formatBalance(walletInfo.balance)} BTC
              </Typography>
            </CardContent>
          </Card>

          {/* Addresses Card */}
          <Card sx={{ 
            borderRadius: 2,
            ...(isDarkMode ? {
              background: 'rgba(19, 47, 76, 0.6)',
              backdropFilter: 'blur(10px)',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
              border: '1px solid rgba(255, 255, 255, 0.1)'
            } : {
              background: 'linear-gradient(180deg, #ffffff 0%, #f5f7fa 100%)',
              boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
              border: '1px solid rgba(0, 0, 0, 0.08)',
            })
          }}>
            <CardContent sx={{ p: 3 }}>
              <Typography variant="h6" component="h3" sx={{ mb: 2, fontWeight: 600 }}>
                Addresses
              </Typography>
              
              {walletInfo.addresses.map((address, index) => (
                <Accordion key={index} sx={{ mb: 1 }}>
                  <AccordionSummary expandIcon={<ExpandMore />}>
                    <Box sx={{ width: '100%', display: 'flex', alignItems: 'center' }}>
                      <Typography sx={{ fontFamily: 'monospace', fontSize: '0.9rem', flex: 1 }}>
                        {address.address}
                      </Typography>
                      <Chip 
                        label={address.address_type} 
                        size="small" 
                        variant="outlined"
                        sx={{ ml: 1 }}
                      />
                    </Box>
                  </AccordionSummary>
                  <AccordionDetails>
                    <List dense>
                      <ListItem>
                        <ListItemText 
                          primary="Address"
                          secondary={
                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                              <Typography 
                                component="span" 
                                sx={{ fontFamily: 'monospace', fontSize: '0.8rem', wordBreak: 'break-all' }}
                              >
                                {address.address}
                              </Typography>
                              <Tooltip title="Copy address">
                                <IconButton 
                                  size="small" 
                                  onClick={() => copyToClipboard(address.address, 'Address')}
                                >
                                  <ContentCopy fontSize="small" />
                                </IconButton>
                              </Tooltip>
                            </Box>
                          }
                        />
                      </ListItem>
                      <ListItem>
                        <ListItemText 
                          primary="Public Key"
                          secondary={
                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                              <Typography 
                                component="span" 
                                sx={{ fontFamily: 'monospace', fontSize: '0.8rem', wordBreak: 'break-all' }}
                              >
                                {address.public_key}
                              </Typography>
                              <Tooltip title="Copy public key">
                                <IconButton 
                                  size="small" 
                                  onClick={() => copyToClipboard(address.public_key, 'Public key')}
                                >
                                  <ContentCopy fontSize="small" />
                                </IconButton>
                              </Tooltip>
                            </Box>
                          }
                        />
                      </ListItem>
                      <ListItem>
                        <ListItemText 
                          primary="Derivation Path"
                          secondary={address.derivation_path}
                        />
                      </ListItem>
                    </List>
                  </AccordionDetails>
                </Accordion>
              ))}
            </CardContent>
          </Card>

          {/* Master Public Key Card */}
          <Card sx={{ 
            borderRadius: 2,
            ...(isDarkMode ? {
              background: 'rgba(19, 47, 76, 0.6)',
              backdropFilter: 'blur(10px)',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
              border: '1px solid rgba(255, 255, 255, 0.1)'
            } : {
              background: 'linear-gradient(180deg, #ffffff 0%, #f5f7fa 100%)',
              boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
              border: '1px solid rgba(0, 0, 0, 0.08)',
            })
          }}>
            <CardContent sx={{ p: 3 }}>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                <Key sx={{ mr: 2, color: 'primary.main' }} />
                <Typography variant="h6" component="h3" sx={{ fontWeight: 600 }}>
                  Master Public Key
                </Typography>
                <Box sx={{ ml: 'auto' }}>
                  <Tooltip title={showMasterPublicKey ? "Hide" : "Show"}>
                    <IconButton onClick={() => setShowMasterPublicKey(!showMasterPublicKey)}>
                      {showMasterPublicKey ? <VisibilityOff /> : <Visibility />}
                    </IconButton>
                  </Tooltip>
                </Box>
              </Box>
              
              {showMasterPublicKey ? (
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <Typography 
                    sx={{ 
                      fontFamily: 'monospace', 
                      fontSize: '0.8rem', 
                      wordBreak: 'break-all',
                      flex: 1,
                      p: 2,
                      backgroundColor: isDarkMode ? 'rgba(0,0,0,0.2)' : 'rgba(0,0,0,0.05)',
                      borderRadius: 1
                    }}
                  >
                    {walletInfo.master_public_key}
                  </Typography>
                  <Tooltip title="Copy master public key">
                    <IconButton 
                      onClick={() => copyToClipboard(walletInfo.master_public_key, 'Master public key')}
                    >
                      <ContentCopy />
                    </IconButton>
                  </Tooltip>
                </Box>
              ) : (
                <Typography 
                  variant="body2" 
                  sx={{ color: isDarkMode ? 'rgba(255, 255, 255, 0.6)' : 'rgba(0, 0, 0, 0.6)' }}
                >
                  Click the eye icon to reveal the master public key
                </Typography>
              )}
            </CardContent>
          </Card>
        </Stack>
      ) : !isWalletOpen ? (
        <Card sx={{ 
          width: '100%', 
          maxWidth: 500, 
          mt: 4,
          borderRadius: 2,
          ...(isDarkMode ? {
            background: 'rgba(19, 47, 76, 0.6)',
            backdropFilter: 'blur(10px)',
            boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
            border: '1px solid rgba(255, 255, 255, 0.1)'
          } : {
            background: 'linear-gradient(180deg, #ffffff 0%, #f5f7fa 100%)',
            boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
            border: '1px solid rgba(0, 0, 0, 0.08)',
          })
        }}>
          <CardContent sx={{ p: 3, textAlign: 'center' }}>
            <AccountBalanceWallet sx={{ fontSize: 48, color: 'text.secondary', mb: 2 }} />
            <Typography variant="h6" sx={{ mb: 2 }}>
              No Wallet Open
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Please open a wallet to view account information
            </Typography>
          </CardContent>
        </Card>
      ) : (
        <Card sx={{ 
          width: '100%', 
          maxWidth: 500, 
          mt: 4,
          borderRadius: 2,
          ...(isDarkMode ? {
            background: 'rgba(19, 47, 76, 0.6)',
            backdropFilter: 'blur(10px)',
            boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
            border: '1px solid rgba(255, 255, 255, 0.1)'
          } : {
            background: 'linear-gradient(180deg, #ffffff 0%, #f5f7fa 100%)',
            boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
            border: '1px solid rgba(0, 0, 0, 0.08)',
          })
        }}>
          <CardContent sx={{ p: 3, textAlign: 'center' }}>
            <Typography variant="h6" sx={{ mb: 2 }}>
              Loading Wallet Information...
            </Typography>
          </CardContent>
        </Card>
      )}

      {/* Demo Section */}
      <Card sx={{ 
        width: '100%', 
        maxWidth: 500, 
        mt: 4, 
        mb: 'auto',
        mx: 'auto',
        borderRadius: 2,
        ...(isDarkMode ? {
          background: 'rgba(19, 47, 76, 0.6)',
          backdropFilter: 'blur(10px)',
          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
          border: '1px solid rgba(255, 255, 255, 0.1)'
        } : {
          background: 'linear-gradient(180deg, #ffffff 0%, #f5f7fa 100%)',
          boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
          border: '1px solid rgba(0, 0, 0, 0.08)',
          transition: 'all 0.2s ease-in-out',
          '&:hover': {
            transform: 'translateY(-4px)',
            boxShadow: '0 6px 25px rgba(0, 0, 0, 0.2)',
          }
        })
      }}>
        <CardContent sx={{ 
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          padding: '24px'
        }}>
          <Typography variant="h6" sx={{ mb: 2 }}>
            Demo Section
          </Typography>
          
          <Box
            component="form"
            onSubmit={(e) => {
              e.preventDefault();
              greet();
            }}
            sx={{ 
              width: '100%',
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center'
            }}
          >
            <TextField
              fullWidth
              id="greet-input"
              label="Enter a name..."
              variant="outlined"
              value={name}
              onChange={(e) => setName(e.target.value)}
              sx={{ 
                m: 1, 
                width: '100%', 
                maxWidth: '450px',
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
                } : {
                  '& .MuiOutlinedInput-root': {
                    '&.Mui-focused fieldset': {
                      borderColor: '#1a237e',
                    }
                  },
                  '& .MuiInputLabel-root.Mui-focused': {
                    color: '#1a237e',
                  }
                })
              }}
            />
            <Button 
              variant="contained" 
              type="submit"
              sx={{ 
                m: 1, 
                width: '100%', 
                maxWidth: '450px',
                fontWeight: 600,
                padding: '10px',
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
              Greet
            </Button>
          </Box>
          {greetMsg && (
            <Typography 
              variant="h6" 
              sx={{ 
                mt: 2, 
                textAlign: 'center',
                color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#1a237e',
                fontWeight: 600
              }}
            >
              {greetMsg}
            </Typography>
          )}
        </CardContent>
      </Card>
    </Box>
  );
}