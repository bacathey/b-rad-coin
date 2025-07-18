// filepath: c:\Users\bacat\source\repos\b-rad-coin\src\pages\Account.tsx
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
  Typography, 
  Box, 
  Stack, 
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
  ListItemText,
  Divider
} from '@mui/material';
import { 
  ContentCopy, 
  AccountBalanceWallet, 
  Key, 
  ExpandMore,
  Security,
  Visibility,
  VisibilityOff,
  AccountTree
} from '@mui/icons-material';
import { useWallet } from '../context/WalletContext';
import type { CurrentWalletInfo } from '../types/wallet';

export default function Account() {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  const { isWalletOpen, currentWallet } = useWallet();
  
  const [walletInfo, setWalletInfo] = useState<CurrentWalletInfo | null>(null);
  const [showMasterPublicKey, setShowMasterPublicKey] = useState(false);

  // Load wallet info when component mounts or wallet changes
  // Note: Public key data is securely stored in the wallet data file (wallet.dat),
  // not in the configuration file, ensuring proper key separation
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

          {/* Public Keys Card */}
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
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 3 }}>
                <Key sx={{ mr: 2, color: 'primary.main' }} />
                <Typography variant="h6" component="h3" sx={{ fontWeight: 600 }}>
                  Public Keys
                </Typography>
              </Box>
              
              {/* Master Public Key Section */}
              <Box sx={{ mb: 4 }}>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                  <Typography variant="h6" component="h4" sx={{ fontWeight: 600, flex: 1 }}>
                    Master Public Key
                  </Typography>
                  <Tooltip title={showMasterPublicKey ? "Hide" : "Show"}>
                    <IconButton onClick={() => setShowMasterPublicKey(!showMasterPublicKey)}>
                      {showMasterPublicKey ? <VisibilityOff /> : <Visibility />}
                    </IconButton>
                  </Tooltip>
                </Box>
                
                <Typography 
                  variant="body2" 
                  sx={{ 
                    mb: 2, 
                    color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)' 
                  }}
                >
                  The root public key for this wallet, used to derive all child keys according to BIP32 standards
                </Typography>
                
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
              </Box>

              <Divider sx={{ mb: 3, opacity: 0.3 }} />

              {/* Child Keys Section */}
              <Box>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                  <AccountTree sx={{ mr: 2, color: 'primary.main' }} />
                  <Typography variant="h6" component="h4" sx={{ fontWeight: 600 }}>
                    Derived Child Keys
                  </Typography>
                </Box>
                
                <Typography 
                  variant="body2" 
                  sx={{ 
                    mb: 3, 
                    color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)' 
                  }}
                >
                  Individual keys derived from the master public key using hierarchical deterministic key derivation (BIP32)
                </Typography>

                {walletInfo.addresses.map((address, index) => (
                  <Box 
                    key={index} 
                    sx={{ 
                      mb: 2, 
                      p: 2, 
                      borderRadius: 1,
                      backgroundColor: isDarkMode ? 'rgba(0,0,0,0.2)' : 'rgba(0,0,0,0.03)',
                      border: `1px solid ${isDarkMode ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.1)'}`
                    }}
                  >
                    <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                      <Typography variant="body2" sx={{ fontWeight: 600, mr: 1 }}>
                        Path:
                      </Typography>
                      <Typography 
                        variant="body2" 
                        sx={{ 
                          fontFamily: 'monospace', 
                          color: 'primary.main',
                          fontWeight: 500
                        }}
                      >
                        {address.derivation_path}
                      </Typography>
                      <Chip 
                        label={address.address_type} 
                        size="small" 
                        variant="outlined"
                        sx={{ ml: 'auto' }}
                      />
                    </Box>
                    
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
                      <Typography variant="body2" sx={{ fontWeight: 600, minWidth: 80 }}>
                        Address:
                      </Typography>
                      <Typography 
                        variant="body2" 
                        sx={{ 
                          fontFamily: 'monospace', 
                          fontSize: '0.8rem', 
                          wordBreak: 'break-all',
                          flex: 1
                        }}
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
                    
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                      <Typography variant="body2" sx={{ fontWeight: 600, minWidth: 80 }}>
                        Public Key:
                      </Typography>
                      <Typography 
                        variant="body2" 
                        sx={{ 
                          fontFamily: 'monospace', 
                          fontSize: '0.8rem', 
                          wordBreak: 'break-all',
                          flex: 1
                        }}
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
                  </Box>
                ))}
              </Box>
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
    </Box>
  );
}