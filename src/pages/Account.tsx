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
  Button,
  CircularProgress,
  TextField,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Fade
} from '@mui/material';
import { 
  ContentCopy, 
  AccountBalanceWallet, 
  Key, 
  ExpandMore,
  Security,
  AccountTree,
  Add,
  Edit,
  Check,
  Close,
  Label
} from '@mui/icons-material';
import { useWallet } from '../context/WalletContext';
import type { CurrentWalletInfo } from '../types/wallet';

export default function Account() {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  const { isWalletOpen, currentWallet } = useWallet();
  
  const [walletInfo, setWalletInfo] = useState<CurrentWalletInfo | null>(null);
  const [isAddingKey, setIsAddingKey] = useState(false);
  const [editingLabel, setEditingLabel] = useState<string | null>(null);
  const [labelValue, setLabelValue] = useState('');
  const [showAddKeyDialog, setShowAddKeyDialog] = useState(false);
  const [newKeyLabel, setNewKeyLabel] = useState('');

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
    return (balance / 100000000).toFixed(8); // Convert satoshis to BRAD
  };

  const handleLabelEdit = (address: string, currentLabel?: string) => {
    setEditingLabel(address);
    setLabelValue(currentLabel || '');
  };

  const handleLabelSave = async (address: string) => {
    try {
      await invoke('update_address_label', { 
        address, 
        label: labelValue.trim() || null 
      });
      await loadWalletInfo(); // Reload to get updated data
      setEditingLabel(null);
      setLabelValue('');
    } catch (error) {
      console.error('Failed to update label:', error);
    }
  };

  const handleLabelCancel = () => {
    setEditingLabel(null);
    setLabelValue('');
  };

  const handleAddNewKey = async () => {
    setShowAddKeyDialog(true);
  };

  const handleConfirmAddKey = async () => {
    setIsAddingKey(true);
    try {
      // Call backend to derive new address with custom label
      await invoke('derive_new_address', {
        label: newKeyLabel.trim() || null
      });
      await loadWalletInfo(); // Reload to show new key
      setShowAddKeyDialog(false);
      setNewKeyLabel(''); // Reset the label field
    } catch (error) {
      console.error('Failed to add new key:', error);
    } finally {
      setIsAddingKey(false);
    }
  };

  const handleCancelAddKey = () => {
    setShowAddKeyDialog(false);
    setNewKeyLabel(''); // Reset the label field
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
                Balance: {formatBalance(walletInfo.balance)} BRAD
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
                      <Box sx={{ flex: 1 }}>
                        <Typography sx={{ fontFamily: 'monospace', fontSize: '0.9rem', fontWeight: 500 }}>
                          {address.address}
                        </Typography>
                        {address.label && (
                          <Typography sx={{ fontSize: '0.8rem', color: 'text.secondary', mt: 0.5 }}>
                            {address.label}
                          </Typography>
                        )}
                      </Box>
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
                          primary="Label"
                          secondary={
                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                              {editingLabel === address.address ? (
                                <>
                                  <TextField
                                    size="small"
                                    value={labelValue}
                                    onChange={(e) => setLabelValue(e.target.value)}
                                    placeholder="Enter label"
                                    sx={{ flex: 1, maxWidth: 200 }}
                                    autoFocus
                                  />
                                  <IconButton 
                                    size="small" 
                                    onClick={() => handleLabelSave(address.address)}
                                    color="primary"
                                  >
                                    <Check fontSize="small" />
                                  </IconButton>
                                  <IconButton 
                                    size="small" 
                                    onClick={handleLabelCancel}
                                  >
                                    <Close fontSize="small" />
                                  </IconButton>
                                </>
                              ) : (
                                <>
                                  <Typography 
                                    component="span" 
                                    sx={{ fontSize: '0.8rem' }}
                                  >
                                    {address.label || 'No label'}
                                  </Typography>
                                  <Tooltip title="Edit label">
                                    <IconButton 
                                      size="small" 
                                      onClick={() => handleLabelEdit(address.address, address.label || '')}
                                    >
                                      <Edit fontSize="small" />
                                    </IconButton>
                                  </Tooltip>
                                </>
                              )}
                            </Box>
                          }
                        />
                      </ListItem>
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
              
              {/* Add New Address Button */}
              <Box sx={{ mt: 2, display: 'flex', justifyContent: 'center' }}>
                <Button
                  variant="outlined"
                  startIcon={<Add />}
                  onClick={handleAddNewKey}
                  sx={{ 
                    minWidth: 160,
                    ...(isDarkMode ? {
                      borderColor: 'rgba(255, 255, 255, 0.3)',
                      color: 'rgba(255, 255, 255, 0.9)',
                      '&:hover': {
                        borderColor: 'primary.main',
                        backgroundColor: 'rgba(144, 202, 249, 0.1)'
                      }
                    } : {
                      borderColor: 'rgba(26, 35, 126, 0.3)',
                      color: '#1a237e',
                      '&:hover': {
                        borderColor: 'primary.main',
                        backgroundColor: 'rgba(26, 35, 126, 0.05)'
                      }
                    })
                  }}
                >
                  Add New Address
                </Button>
              </Box>
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
              <Typography variant="h6" component="h3" sx={{ mb: 2, fontWeight: 600 }}>
                Public Keys
              </Typography>
              
              {/* Master Public Key Section */}
              <Accordion sx={{ mb: 1 }}>
                <AccordionSummary expandIcon={<ExpandMore />}>
                  <Box sx={{ width: '100%', display: 'flex', alignItems: 'center' }}>
                    <Key sx={{ mr: 2, color: 'primary.main' }} />
                    <Typography sx={{ fontSize: '0.9rem', flex: 1, fontWeight: 500 }}>
                      Master Public Key
                    </Typography>
                    <Chip 
                      label="xpub" 
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
                        primary="Master Public Key"
                        secondary={
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                            <Typography 
                              component="span" 
                              sx={{ fontFamily: 'monospace', fontSize: '0.8rem', wordBreak: 'break-all' }}
                            >
                              {walletInfo.master_public_key}
                            </Typography>
                            <Tooltip title="Copy master public key">
                              <IconButton 
                                size="small" 
                                onClick={() => copyToClipboard(walletInfo.master_public_key, 'Master public key')}
                              >
                                <ContentCopy fontSize="small" />
                              </IconButton>
                            </Tooltip>
                          </Box>
                        }
                      />
                    </ListItem>
                  </List>
                </AccordionDetails>
              </Accordion>
              
              {/* Child Keys Section */}
              <Accordion sx={{ mb: 1 }}>
                <AccordionSummary expandIcon={<ExpandMore />}>
                  <Box sx={{ width: '100%', display: 'flex', alignItems: 'center' }}>
                    <AccountTree sx={{ mr: 2, color: 'primary.main' }} />
                    <Typography sx={{ fontSize: '0.9rem', flex: 1, fontWeight: 500 }}>
                      Derived Child Keys ({walletInfo.addresses.length})
                    </Typography>
                    <Chip 
                      label="BIP32" 
                      size="small" 
                      variant="outlined"
                      sx={{ ml: 1 }}
                    />
                  </Box>
                </AccordionSummary>
                <AccordionDetails>
                  <List dense>
                    {walletInfo.addresses.map((address, index) => (
                      <ListItem key={index}>
                        <ListItemText 
                          primary={
                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
                              <Typography variant="body2" sx={{ fontWeight: 500 }}>
                                Key #{index + 1}
                              </Typography>
                              {address.label && (
                                <Chip 
                                  label={address.label} 
                                  size="small" 
                                  color="primary"
                                  variant="outlined"
                                />
                              )}
                              <Chip 
                                label={address.address_type} 
                                size="small" 
                                variant="outlined"
                                sx={{ ml: 'auto' }}
                              />
                            </Box>
                          }
                          secondary={
                            <Box>
                              {/* Label editing section */}
                              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
                                {editingLabel === address.address ? (
                                  <>
                                    <TextField
                                      size="small"
                                      value={labelValue}
                                      onChange={(e) => setLabelValue(e.target.value)}
                                      placeholder="Enter label"
                                      sx={{ flex: 1, maxWidth: 200 }}
                                      autoFocus
                                    />
                                    <IconButton 
                                      size="small" 
                                      onClick={() => handleLabelSave(address.address)}
                                      color="primary"
                                    >
                                      <Check fontSize="small" />
                                    </IconButton>
                                    <IconButton 
                                      size="small" 
                                      onClick={handleLabelCancel}
                                    >
                                      <Close fontSize="small" />
                                    </IconButton>
                                  </>
                                ) : (
                                  <>
                                    <Typography variant="body2" sx={{ color: 'text.secondary', fontSize: '0.8rem' }}>
                                      {address.label ? `Label: ${address.label}` : 'No label'}
                                    </Typography>
                                    <IconButton 
                                      size="small" 
                                      onClick={() => handleLabelEdit(address.address, address.label)}
                                    >
                                      <Edit fontSize="small" />
                                    </IconButton>
                                  </>
                                )}
                              </Box>
                              
                              {/* Public key section */}
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
                            </Box>
                          }
                        />
                      </ListItem>
                    ))}
                  </List>
                </AccordionDetails>
              </Accordion>
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

      {/* Add New Address Dialog */}
      <Dialog 
        open={showAddKeyDialog} 
        onClose={handleCancelAddKey}
        maxWidth="xs"
        fullWidth
        TransitionComponent={Fade}
        TransitionProps={{ timeout: 400 }}
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
          <Label 
            color="primary" 
            sx={{ mr: 1 }} 
          />
          <Typography variant="h6" component="div">
            Add New Address
          </Typography>
        </DialogTitle>
        <DialogContent>
          <Box sx={{ pt: 1 }}>
            <TextField
              fullWidth
              label="Label (Optional)"
              value={newKeyLabel}
              onChange={(e) => setNewKeyLabel(e.target.value)}
              placeholder="Enter a label for this address"
              helperText="You can leave this empty to add a label later"
              autoFocus
              sx={{ mb: 1 }}
            />
          </Box>
        </DialogContent>
        <DialogActions sx={{ p: 2 }}>
          <Button 
            onClick={handleCancelAddKey}
            disabled={isAddingKey}
            sx={{ textTransform: 'none' }}
          >
            Cancel
          </Button>
          <Button 
            onClick={handleConfirmAddKey} 
            variant="contained"
            color="primary"
            disabled={isAddingKey}
            startIcon={isAddingKey ? <CircularProgress size={20} /> : <Add />}
            sx={{ textTransform: 'none' }}
          >
            {isAddingKey ? 'Generating...' : 'Add Address'}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}