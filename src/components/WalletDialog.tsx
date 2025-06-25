import { useState, useEffect } from 'react';
import { 
  Dialog, 
  DialogTitle, 
  DialogContent, 
  Button, 
  FormControl, 
  InputLabel, 
  Select, 
  MenuItem, 
  Typography, 
  Box, 
  CircularProgress,
  useTheme,
  SelectChangeEvent,
  Backdrop,
  Tabs,
  Tab,
  TextField,
  Alert,
  Fade,
  Grow,
  ListItemIcon,
  FormControlLabel,
  Checkbox,
} from '@mui/material';
import { useWallet } from '../context/WalletContext';
import { getWalletDetails } from '../lib/wallet';
import { invoke } from '@tauri-apps/api/core';
import AccountBalanceWalletIcon from '@mui/icons-material/AccountBalanceWallet';
import AddIcon from '@mui/icons-material/Add';
import LockIcon from '@mui/icons-material/Lock';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import SecureWalletDialog from './SecureWalletDialog';
import SeedPhraseDialog from './SeedPhraseDialog';
import VerifySeedPhraseDialog from './VerifySeedPhraseDialog';

// Interface for tab panel props
interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

// Interface for wallet details
interface WalletDetails {
  name: string;
  secured: boolean;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;
  const isActive = value === index;

  return (
    <div
      role="tabpanel"
      hidden={!isActive}
      id={`wallet-tabpanel-${index}`}
      aria-labelledby={`wallet-tab-${index}`}
      {...other}
      style={{ 
        width: '100%',
        display: isActive ? 'block' : 'none'
      }}
    >
      <Grow 
        in={isActive} 
        timeout={500}  // Increased from 300ms to 500ms
        easing={{
          enter: 'cubic-bezier(0.4, 0, 0.2, 1)'
        }}
      >
        <Box sx={{ pt: 3 }}>
          {children}
        </Box>
      </Grow>
    </div>
  );
}

export default function WalletDialog() {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  const { isWalletOpen, setIsWalletOpen, setCurrentWallet, refreshWalletDetails } = useWallet();
  
  // Dialog state
  const [selectedWallet, setSelectedWallet] = useState('');
  const [walletsList, setWalletsList] = useState<WalletDetails[]>([]);
  const [isSelectedWalletSecured, setIsSelectedWalletSecured] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [isGettingWallets, setIsGettingWallets] = useState(true);
  
  // New state for wallet creation
  const [tabValue, setTabValue] = useState(0);
  const [newWalletName, setNewWalletName] = useState('');
  const [walletPassword, setWalletPassword] = useState('');
  const [openWalletPassword, setOpenWalletPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [errorMessage, setErrorMessage] = useState('');
  const [usePasswordProtection, setUsePasswordProtection] = useState(false);
  // State for secure wallet dialog
  const [secureDialogOpen, setSecureDialogOpen] = useState(false);
  const [walletToSecure] = useState('');
  
  // State for seed phrase dialogs
  const [seedPhraseDialogOpen, setSeedPhraseDialogOpen] = useState(false);
  const [verifySeedDialogOpen, setVerifySeedDialogOpen] = useState(false);
  const [seedPhrase, setSeedPhrase] = useState('');
  const [tempWalletData, setTempWalletData] = useState<{name: string; password: string; usePassword: boolean} | null>(null);

  // Fetch available wallets when the component mounts or when isWalletOpen changes
  useEffect(() => {
    async function fetchWallets() {
      try {
        setIsGettingWallets(true);
        const wallets = await getWalletDetails();
        setWalletsList(wallets);
        
        // If we have wallets, select the first one by default for better UX
        if (wallets.length > 0 && !selectedWallet) {
          setSelectedWallet(wallets[0].name);
          setIsSelectedWalletSecured(wallets[0].secured);
        }
      } catch (error) {
        console.error('Failed to fetch available wallets:', error);
        setWalletsList([]);
      } finally {
        setIsGettingWallets(false);
      }
    }

    // Always fetch wallets when the dialog should be open
    if (!isWalletOpen) {
      fetchWallets();
    }
  }, [isWalletOpen, selectedWallet]);

  const handleWalletChange = (event: SelectChangeEvent) => {
    const walletName = event.target.value as string;
    setSelectedWallet(walletName);
    
    // Find if the selected wallet is secured
    const walletInfo = walletsList.find(w => w.name === walletName);
    setIsSelectedWalletSecured(walletInfo?.secured || false);
    
    // Clear password if switching to an unsecured wallet
    if (walletInfo && !walletInfo.secured) {
      setOpenWalletPassword('');
    }
  };

  const handleTabChange = (_: React.SyntheticEvent, newValue: number) => {
    setTabValue(newValue);
    setErrorMessage('');
  };

  const handleOpenWallet = async () => {
    if (!selectedWallet) return;
    
    // Clear any previous error messages
    setErrorMessage('');
    
    // Check if the wallet is secured and validate password if needed
    if (isSelectedWalletSecured && !openWalletPassword) {
      setErrorMessage('Please enter your password for this secured wallet');
      return;
    }

    setIsLoading(true);
    try {
      // Pass wallet name and password (only if it's a secured wallet)
      const result = await invoke('open_wallet', { 
        walletName: selectedWallet,
        password: isSelectedWalletSecured ? openWalletPassword : undefined
      });
      
      if (result) {
        setCurrentWallet({
          name: selectedWallet,
          secured: isSelectedWalletSecured
        });
        setIsWalletOpen(true);
        await refreshWalletDetails(); // Refresh wallet details in context
      } else {
        setErrorMessage('Failed to open wallet');
      }
    } catch (error) {
      console.error('Failed to open wallet:', error);
      setErrorMessage(`Error: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setIsLoading(false);
    }
  };
  const handleCreateWallet = async () => {
    setErrorMessage('');
    
    if (!newWalletName) {
      setErrorMessage('Please enter a wallet name');
      return;
    }
    
    // Only validate passwords if using password protection
    if (usePasswordProtection) {
      if (!walletPassword) {
        setErrorMessage('Please enter a password');
        return;
      }
      
      if (walletPassword !== confirmPassword) {
        setErrorMessage('Passwords do not match');
        return;
      }
    }
    
    setIsLoading(true);
    try {
      // Generate a seed phrase for the new wallet
      const generatedSeedPhrase = await invoke<string>('generate_seed_phrase');
      
      if (generatedSeedPhrase) {
        // Store wallet data temporarily until seed phrase is verified
        setTempWalletData({
          name: newWalletName,
          password: walletPassword,
          usePassword: usePasswordProtection
        });
        
        // Set the seed phrase and show the dialog
        setSeedPhrase(generatedSeedPhrase);
        setSeedPhraseDialogOpen(true);
      } else {
        setErrorMessage('Failed to generate seed phrase');
      }
    } catch (error) {
      console.error('Failed to start wallet creation:', error);
      setErrorMessage(`Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsLoading(false);
    }
  };
  
  // Function to create the wallet after seed phrase verification
  const finalizeWalletCreation = async () => {
    if (!tempWalletData) return;
    
    setIsLoading(true);
    try {
      const result = await invoke('create_wallet', { 
        walletName: tempWalletData.name, 
        password: tempWalletData.password,
        usePassword: tempWalletData.usePassword,
        seedPhrase: seedPhrase // Pass the seed phrase to create wallet with this specific phrase
      });
      
      if (result) {
        setCurrentWallet({
          name: tempWalletData.name,
          secured: tempWalletData.usePassword
        });
        setIsWalletOpen(true);
        await refreshWalletDetails(); // Refresh wallet details in context
        
        // Reset temporary data and close dialogs
        setTempWalletData(null);
        setSeedPhrase('');
      } else {
        setErrorMessage('Failed to create wallet');
      }
    } catch (error) {
      console.error('Failed to create wallet:', error);
      setErrorMessage(`Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handlePasswordProtectionToggle = (event: React.ChangeEvent<HTMLInputElement>) => {
    setUsePasswordProtection(event.target.checked);
    
    // Reset password fields when toggling off password protection
    if (!event.target.checked) {
      setWalletPassword('');
      setConfirmPassword('');
    }
  };

  return (
    <>
      {/* Backdrop that covers the entire app when wallet is not open */}      
      <Backdrop 
        open={!isWalletOpen} 
        sx={{ 
          zIndex: theme.zIndex.drawer + 2,
          background: isDarkMode 
            ? 'linear-gradient(145deg, #0a1929 0%, #0d2b59 50%, rgb(13, 75, 116) 100%)' 
            : 'linear-gradient(145deg, #f5f7fa 0%, #ffffff 100%)',
          backdropFilter: 'blur(5px)',
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0
        }}
      />
        {/* Secure Wallet Dialog */}
      <SecureWalletDialog
        open={secureDialogOpen}
        onClose={() => setSecureDialogOpen(false)}
        walletName={walletToSecure}
        onSuccess={refreshWalletDetails}
      />
      
      {/* Seed Phrase Dialog */}
      <SeedPhraseDialog
        open={seedPhraseDialogOpen}
        seedPhrase={seedPhrase}
        onClose={() => {
          setSeedPhraseDialogOpen(false);
          setTempWalletData(null);
          setSeedPhrase('');
        }}
        onContinue={() => {
          setSeedPhraseDialogOpen(false);
          setVerifySeedDialogOpen(true);
        }}
      />
      
      {/* Verify Seed Phrase Dialog */}
      <VerifySeedPhraseDialog
        open={verifySeedDialogOpen}
        seedPhrase={seedPhrase}
        onClose={() => {
          setVerifySeedDialogOpen(false);
          setTempWalletData(null);
          setSeedPhrase('');
        }}
        onVerified={() => {
          setVerifySeedDialogOpen(false);
          finalizeWalletCreation();
        }}
      />
      
      <Dialog 
        open={!isWalletOpen}
        maxWidth="sm" 
        fullWidth 
        disableEscapeKeyDown
        hideBackdrop // Hide default backdrop since we're using a custom one
        onClose={(_event, _reason) => {
          // Prevent closing the dialog by clicking outside or pressing Escape
          // We want to force the user to open/create a wallet
          return false;
        }}
        TransitionComponent={Fade}
        TransitionProps={{ 
          timeout: 500
        }}
        sx={{
          zIndex: theme.zIndex.drawer + 3,
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
        <DialogTitle sx={{ 
          display: 'flex', 
          alignItems: 'center', 
          gap: 1,
          pb: 1
        }}>
          <AccountBalanceWalletIcon 
            color="primary" 
            fontSize="large" 
            sx={{ mr: 1 }} 
          />
          <Typography variant="h5" component="div" fontWeight={600}>
            B-Rad Coin Wallet
          </Typography>
        </DialogTitle>
        
        <Box sx={{ borderBottom: 1, borderColor: 'divider', px: 3 }}>
          <Tabs 
            value={tabValue} 
            onChange={handleTabChange} 
            aria-label="wallet options"
            variant="fullWidth"
            sx={{
              '& .MuiTabs-indicator': {
                transition: 'all 500ms cubic-bezier(0.4, 0, 0.2, 1)'  // Increased from 300ms and added easing
              }
            }}
          >
            <Tab label="Open Wallet" id="wallet-tab-0" aria-controls="wallet-tabpanel-0" />
            <Tab label="Create New" id="wallet-tab-1" aria-controls="wallet-tabpanel-1" />
          </Tabs>
        </Box>
        
        <DialogContent sx={{ 
          position: 'relative',
          overflow: 'hidden'
        }}>
          {/* Error message for both tabs */}
          {errorMessage && (
            <Alert severity="error" sx={{ mb: 2 }}>
              {errorMessage}
            </Alert>
          )}
          
          {/* Open Wallet Tab */}
          <TabPanel value={tabValue} index={0}>
            <Typography variant="body1" sx={{ mb: 3 }}>
              Please select a wallet to open:
            </Typography>
            
            {isGettingWallets ? (
              <Box sx={{ display: 'flex', justifyContent: 'center', py: 3 }}>
                <CircularProgress />
              </Box>
            ) : (
              <FormControl fullWidth sx={{ mb: 2 }}>
                <InputLabel id="wallet-select-label">Select Wallet</InputLabel>
                <Select
                  labelId="wallet-select-label"
                  id="wallet-select"
                  value={selectedWallet}
                  label="Select Wallet"
                  onChange={handleWalletChange}
                >
                  {walletsList.map((wallet) => (
                    <MenuItem key={wallet.name} value={wallet.name}>
                      <Box sx={{ display: 'flex', alignItems: 'center', width: '100%' }}>                        <ListItemIcon sx={{ minWidth: 36 }}>
                          {wallet.secured ? 
                            <LockIcon color="success" fontSize="small" /> : 
                            <LockOpenIcon 
                              sx={{ color: isDarkMode ? '#ffeb3b' : '#f57c00' }} 
                              fontSize="small" 
                            />
                          }
                        </ListItemIcon>
                        {wallet.name}
                      </Box>
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            )}
            
            {/* Only show password field for secured wallets */}
            {isSelectedWalletSecured && (
              <TextField
                fullWidth
                label="Password"
                type="password"
                variant="outlined"
                value={openWalletPassword}
                onChange={(e) => setOpenWalletPassword(e.target.value)}
                sx={{ mb: 2 }}
                required
              />
            )}
            
            <Box sx={{ display: 'flex', justifyContent: 'flex-end', mt: 2 }}>
              <Button 
                variant="contained"
                color="primary"
                onClick={handleOpenWallet}
                disabled={!selectedWallet || isLoading || (isSelectedWalletSecured && !openWalletPassword)}
                startIcon={isLoading && tabValue === 0 ? <CircularProgress size={20} /> : null}
                sx={{ 
                  minWidth: '120px',
                  textTransform: 'none',
                  fontWeight: 600
                }}
              >
                {isLoading && tabValue === 0 ? 'Opening...' : 'Open Wallet'}
              </Button>
            </Box>
          </TabPanel>
          
          {/* Create New Wallet Tab */}
          <TabPanel value={tabValue} index={1}>
            <Typography variant="body1" sx={{ mb: 3 }}>
              Create a new wallet:
            </Typography>
            
            <TextField
              fullWidth
              label="Wallet Name"
              variant="outlined"
              value={newWalletName}
              onChange={(e) => setNewWalletName(e.target.value)}
              sx={{ mb: 2 }}
              required
            />
            
            <FormControlLabel
              control={(
                <Checkbox
                  checked={usePasswordProtection}
                  onChange={handlePasswordProtectionToggle}
                  color="primary"
                />
              )}
              label={(                <Box sx={{ display: 'flex', alignItems: 'center' }}>                  {usePasswordProtection ? 
                    <LockIcon color="success" fontSize="small" sx={{ mr: 1 }} /> : 
                    <LockOpenIcon 
                      sx={{ color: isDarkMode ? '#ffeb3b' : '#f57c00', mr: 1 }} 
                      fontSize="small" 
                    />
                  }
                  <Typography>
                    Password protect this wallet
                  </Typography>
                </Box>
              )}
              sx={{ mb: 2 }}
            />
            
            {usePasswordProtection && (
              <>
                <TextField
                  fullWidth
                  label="Password"
                  type="password"
                  variant="outlined"
                  value={walletPassword}
                  onChange={(e) => setWalletPassword(e.target.value)}
                  sx={{ mb: 2 }}
                  required={usePasswordProtection}
                />
                
                <TextField
                  fullWidth
                  label="Confirm Password"
                  type="password"
                  variant="outlined"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  sx={{ mb: 2 }}
                  required={usePasswordProtection}
                  error={walletPassword !== confirmPassword && confirmPassword !== ''}

                  helperText={walletPassword !== confirmPassword && confirmPassword !== '' ? 'Passwords do not match' : ''}

                />
              </>
            )}
            
            <Box sx={{ display: 'flex', justifyContent: 'flex-end', mt: 2 }}>
              <Button 
                variant="contained"
                color="primary"
                onClick={handleCreateWallet}
                disabled={isLoading || !newWalletName || (usePasswordProtection && (!walletPassword || walletPassword !== confirmPassword))}
                startIcon={isLoading && tabValue === 1 ? <CircularProgress size={20} /> : <AddIcon />}
                sx={{ 
                  minWidth: '140px',
                  textTransform: 'none',
                  fontWeight: 600
                }}
              >
                {isLoading && tabValue === 1 ? 'Creating...' : 'Create Wallet'}
              </Button>
            </Box>
          </TabPanel>
        </DialogContent>      </Dialog>

      {/* Secure Wallet Dialog */}
      <SecureWalletDialog
        open={secureDialogOpen}
        onClose={() => setSecureDialogOpen(false)}
        walletName={walletToSecure}
        onSuccess={refreshWalletDetails}
      />
      
      {/* Seed Phrase Dialog */}
      <SeedPhraseDialog
        open={seedPhraseDialogOpen}
        seedPhrase={seedPhrase}
        onClose={() => {
          setSeedPhraseDialogOpen(false);
          setTempWalletData(null);
          setSeedPhrase('');
        }}
        onContinue={() => {
          setSeedPhraseDialogOpen(false);
          setVerifySeedDialogOpen(true);
        }}
      />
      
      {/* Verify Seed Phrase Dialog */}
      <VerifySeedPhraseDialog
        open={verifySeedDialogOpen}
        seedPhrase={seedPhrase}
        onClose={() => {
          setVerifySeedDialogOpen(false);
          setTempWalletData(null);
          setSeedPhrase('');
        }}
        onVerified={() => {
          setVerifySeedDialogOpen(false);
          finalizeWalletCreation();
        }}
      />
    </>
  );
}