import { useState, useEffect, useMemo } from 'react';
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
  Link,
  Collapse, // Add Collapse import
} from '@mui/material';
import { useWallet } from '../context/WalletContext';
import { getWalletDetails } from '../lib/wallet'; 
import { invoke } from '@tauri-apps/api/core';
import { AppSettings } from '../types/settings';
import AccountBalanceWalletIcon from '@mui/icons-material/AccountBalanceWallet';
import AddIcon from '@mui/icons-material/Add';
import LockIcon from '@mui/icons-material/Lock';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import RestoreIcon from '@mui/icons-material/Restore';
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

export default function OpenCreateWalletDialog() {
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
  const [walletToSecure] = useState(''); // Keep if needed, otherwise remove

  // State for wallet recovery
  const [isRecoveryMode, setIsRecoveryMode] = useState(false);
  // Rename seedPhrase state used for recovery input
  const [recoverySeedPhrase, setRecoverySeedPhrase] = useState(''); 
  
  // Validation state
  const [isWalletNameDuplicate, setIsWalletNameDuplicate] = useState(false);

  // Add state for seed phrase flow
  const [seedPhraseDialogOpen, setSeedPhraseDialogOpen] = useState(false);
  const [verifySeedDialogOpen, setVerifySeedDialogOpen] = useState(false);
  const [generatedSeedPhrase, setGeneratedSeedPhrase] = useState('');
  const [tempWalletData, setTempWalletData] = useState<{name: string; password: string; usePassword: boolean} | null>(null);
  const [showSeedPhraseDialogs, setShowSeedPhraseDialogs] = useState(true);

  // Load showSeedPhraseDialogs setting
  useEffect(() => {
    async function loadSettings() {
      try {
        const settings = await invoke<AppSettings>('get_app_settings');
        setShowSeedPhraseDialogs(settings.show_seed_phrase_dialogs);
      } catch (error) {
        console.error('Failed to load app settings:', error);
        // Default to true if settings can't be loaded
        setShowSeedPhraseDialogs(true);
      }
    }
    
    loadSettings();
  }, []);

  // Check for duplicate wallet names when wallet name changes
  useEffect(() => {
    if (newWalletName.trim()) {
      const isDuplicate = walletsList.some(wallet => 
        wallet.name.toLowerCase() === newWalletName.trim().toLowerCase()
      );
      setIsWalletNameDuplicate(isDuplicate);
    } else {
      setIsWalletNameDuplicate(false);
    }
  }, [newWalletName, walletsList]);
  
  // Generate error message for duplicate wallet name
  const walletNameErrorMessage = useMemo(() => {
    if (isWalletNameDuplicate) {
      return `A wallet with name "${newWalletName}" already exists`;
    }
    return "";
  }, [isWalletNameDuplicate, newWalletName]);

  // Fetch available wallets when the component mounts or when isWalletOpen changes
  useEffect(() => {
    async function fetchWallets() {
      try {
        setIsGettingWallets(true);
        const wallets = await getWalletDetails();
        setWalletsList(wallets);
        
        // If we have wallets, select the first one by default for better UX
        if (wallets.length > 0) {
          setSelectedWallet(wallets[0].name);
          setIsSelectedWalletSecured(wallets[0].secured);
          setTabValue(0); // Set to Open Wallet tab when wallets exist
        } else if (wallets.length === 0) {
          // If no wallets exist, set to Create Wallet tab
          setTabValue(1);
        }
      } catch (error) {
        console.error('Failed to fetch available wallets:', error);
        setWalletsList([]);
        setTabValue(1); // Set to Create Wallet tab on error
      } finally {
        setIsGettingWallets(false);
      }
    }

    // Always fetch wallets when the dialog should be open
    if (!isWalletOpen) {
      fetchWallets();
    }
  }, [isWalletOpen]); // Remove selectedWallet dependency to avoid loop

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
    // Reset recovery mode when changing tabs
    setIsRecoveryMode(false);
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

    if (isWalletNameDuplicate) {
      setErrorMessage(`A wallet with name "${newWalletName}" already exists`);
      return;
    }
    
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

    setIsLoading(true);    try {
      const phrase = await invoke<string>('generate_seed_phrase');
      
      if (phrase) {
        // Store wallet data
        setTempWalletData({
          name: newWalletName,
          password: walletPassword,
          usePassword: usePasswordProtection
        });
        setGeneratedSeedPhrase(phrase);
        
        // Check if we should show seed phrase dialogs based on settings
        if (showSeedPhraseDialogs) {
          // Show the seed phrase dialog
          setSeedPhraseDialogOpen(true);
        } else {
          // Skip seed phrase dialogs and create wallet directly
          finalizeWalletCreation();
        }
      } else {
        setErrorMessage('Failed to generate seed phrase');
        setIsLoading(false); // Set loading false if phrase generation fails
      }
    } catch (error) {
      console.error('Failed to start wallet creation:', error);
      setErrorMessage(`Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
      setIsLoading(false); // Set loading false on error
    }
    // Loading state for dialog flow is handled within finalizeWalletCreation
  };
  
  // Function to create the wallet after seed phrase verification (Dialog Flow)
  const finalizeWalletCreation = async () => {
    if (!tempWalletData) return;
    
    setIsLoading(true); // Set loading true for the final step
    setErrorMessage('');
    try {
      const result = await invoke('create_wallet', { 
        walletName: tempWalletData.name, 
        password: tempWalletData.password,
        usePassword: tempWalletData.usePassword,
        seedPhrase: generatedSeedPhrase // Use the phrase stored in state
      });
      
      if (result) {
        setCurrentWallet({
          name: tempWalletData.name,
          secured: tempWalletData.usePassword
        });
        setIsWalletOpen(true);
        await refreshWalletDetails();
        
        setTempWalletData(null);
        setGeneratedSeedPhrase('');
        setNewWalletName('');
        setWalletPassword('');
        setConfirmPassword('');
        setUsePasswordProtection(false);
      } else {
        setErrorMessage('Failed to create wallet after verification');
      }
    } catch (error) {
      console.error('Failed to finalize wallet creation:', error);
      setErrorMessage(`Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
      setTempWalletData(null); 
      setGeneratedSeedPhrase('');
    } finally {
      setIsLoading(false); // Ensure loading is false after finalization
      setSeedPhraseDialogOpen(false); 
      setVerifySeedDialogOpen(false);
    }
  };

  // Renamed handleRecoverWallet, uses recoverySeedPhrase state
  const handleRecoverWallet = async () => {
    setErrorMessage('');
    
    if (!newWalletName) {
      setErrorMessage('Please enter a wallet name');
      return;
    }

    if (isWalletNameDuplicate) {
      setErrorMessage(`A wallet with name "${newWalletName}" already exists`);
      return;
    }
    
    // Use recoverySeedPhrase state here
    if (!recoverySeedPhrase || recoverySeedPhrase.trim().split(/\s+/).length < 12) { 
      setErrorMessage('Please enter a valid seed phrase (at least 12 words)');
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
      // Call recover_wallet with recoverySeedPhrase
      const result = await invoke('recover_wallet', { 
        walletName: newWalletName, 
        seedPhrase: recoverySeedPhrase, // Pass the recovery seed phrase
        password: walletPassword,
        usePassword: usePasswordProtection
      });
      
      if (result) {
        setCurrentWallet({
          name: newWalletName,
          secured: usePasswordProtection
        });
        setIsWalletOpen(true);
        await refreshWalletDetails(); // Refresh wallet details in context
        // Reset form fields
        setNewWalletName('');
        setWalletPassword('');
        setConfirmPassword('');
        setUsePasswordProtection(false);
        setRecoverySeedPhrase('');
        setIsRecoveryMode(false); 
      } else {
        setErrorMessage('Failed to recover wallet');
      }
    } catch (error) {
      console.error('Failed to recover wallet:', error);
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

  const toggleRecoveryMode = () => {
    setIsRecoveryMode(!isRecoveryMode);
    setErrorMessage('');
    // Clear recovery seed phrase when exiting recovery mode
    if (isRecoveryMode) { 
      setRecoverySeedPhrase('');
    }
    // Also clear create wallet fields when toggling
    setNewWalletName('');
    setWalletPassword('');
    setConfirmPassword('');
    setUsePasswordProtection(false);
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
      
      {/* Secure Wallet Dialog (keep if needed) */}
      <SecureWalletDialog
        open={secureDialogOpen}
        onClose={() => setSecureDialogOpen(false)}
        walletName={walletToSecure} // Ensure walletToSecure state is managed if used
        onSuccess={refreshWalletDetails}
      />

      {/* Add Seed Phrase Dialog */}
      <SeedPhraseDialog
        open={seedPhraseDialogOpen}
        seedPhrase={generatedSeedPhrase}
        onClose={() => {
          setSeedPhraseDialogOpen(false);
          setTempWalletData(null); // Clear temp data if user cancels
          setGeneratedSeedPhrase('');
        }}
        onContinue={() => {
          setSeedPhraseDialogOpen(false);
          setVerifySeedDialogOpen(true); // Open verification dialog
        }}
      />
      
      {/* Add Verify Seed Phrase Dialog */}
      <VerifySeedPhraseDialog
        open={verifySeedDialogOpen}
        seedPhrase={generatedSeedPhrase}
        onClose={() => {
          setVerifySeedDialogOpen(false);
          setTempWalletData(null); // Clear temp data if user cancels verification
          setGeneratedSeedPhrase('');
        }}
        onVerified={() => {
          setVerifySeedDialogOpen(false);
          finalizeWalletCreation(); // Call the final creation step
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
          overflow: 'hidden' // Consider 'auto' or 'visible' if content gets cut off
        }}>
          {/* Error message for both tabs */}
          {errorMessage && (
            <Alert severity="error" sx={{ mb: 2 }}>
              {errorMessage}
            </Alert>
          )}
          
          {/* Open Wallet Tab */}
          <TabPanel value={tabValue} index={0}>
            {walletsList.length > 0 ? (
              <>
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
                          <Box sx={{ display: 'flex', alignItems: 'center', width: '100%' }}>
                            <ListItemIcon sx={{ minWidth: 36 }}>
                              {wallet.secured ? 
                                <LockIcon color="warning" fontSize="small" /> : 
                                <LockOpenIcon color="success" fontSize="small" />
                              }
                            </ListItemIcon>
                            {wallet.name}
                          </Box>
                        </MenuItem>
                      ))}
                    </Select>
                  </FormControl>
                )}
                
                {/* Only show password field for secured wallets, wrapped in Collapse */}
                <Collapse in={isSelectedWalletSecured} timeout="auto" unmountOnExit>
                  <Box sx={{ pt: 2 }}> {/* Add padding top for spacing */}
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
                  </Box>
                </Collapse>
                
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
              </>
            ) : (
              <Box sx={{ textAlign: 'center', py: 4 }}>
                <Typography variant="body1" sx={{ mb: 2 }}>
                  No wallets found.
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  Please switch to the "Create New" tab to create your first wallet.
                </Typography>
                <Button 
                  sx={{ mt: 3 }}
                  onClick={() => setTabValue(1)}
                  variant="outlined"
                >
                  Create New Wallet
                </Button>
              </Box>
            )}
          </TabPanel>
          
          {/* Create New Wallet Tab */}
          <TabPanel value={tabValue} index={1}>
            {walletsList.length === 0 && (
              <Alert severity="info" sx={{ mb: 3 }}>
                No wallets found. Create your first wallet below.
              </Alert>
            )}
            
            <Typography variant="body1" sx={{ mb: 3 }}>
              {isRecoveryMode ? 'Recover an existing wallet:' : 'Create a new wallet:'}
            </Typography>
            
            <TextField
              fullWidth
              label="Wallet Name"
              variant="outlined"
              value={newWalletName}
              onChange={(e) => setNewWalletName(e.target.value)}
              sx={{ mb: 2 }}
              required
              error={isWalletNameDuplicate}
              helperText={walletNameErrorMessage}
              autoComplete="off" // Disable autocomplete to prevent saved information tooltip
              inputProps={{ 
                autoComplete: "new-password", // Force browsers to not show saved passwords
                "data-form-type": "other" // Additional hint to browsers that this isn't for credentials
              }}
            />
            
            {/* Wrap recovery seed phrase field in Collapse */}
            <Collapse in={isRecoveryMode} timeout="auto" unmountOnExit>
              <Box sx={{ pt: 2 }}> {/* Add padding top for spacing */}
                <TextField
                  fullWidth
                  label="Seed Phrase"
                  variant="outlined"
                  value={recoverySeedPhrase} // Use recoverySeedPhrase state
                  onChange={(e) => setRecoverySeedPhrase(e.target.value)} // Update recoverySeedPhrase state
                  multiline
                  rows={3}
                  placeholder="Enter your 12 or 24-word seed phrase separated by spaces"
                  sx={{ mb: 2 }}
                  required
                  helperText="Your seed phrase will allow you to recover your wallet"
                />
              </Box>
            </Collapse>
            
            <FormControlLabel
              control={(
                <Checkbox
                  checked={usePasswordProtection}
                  onChange={handlePasswordProtectionToggle}
                  color="primary"
                />
              )}
              label={(
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  {usePasswordProtection ? 
                    <LockIcon color="warning" fontSize="small" sx={{ mr: 1 }} /> : 
                    <LockOpenIcon color="success" fontSize="small" sx={{ mr: 1 }} />
                  }
                  <Typography>
                    Password protect this wallet
                  </Typography>
                </Box>
              )}
              sx={{ mb: usePasswordProtection ? 0 : 2 }} // Adjust margin based on visibility
            />
            
            {/* Wrap password fields in Collapse */}
            <Collapse in={usePasswordProtection} timeout="auto" unmountOnExit>
              <Box sx={{ pt: 2 }}> {/* Add padding top for spacing */}
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
              </Box>
            </Collapse>
            
            <Box sx={{ display: 'flex', flexDirection: 'column', mt: 2, gap: 2 }}>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <Link
                  component="button"
                  underline="hover"
                  onClick={toggleRecoveryMode}
                  sx={{ 
                    display: 'flex',
                    alignItems: 'center',
                    gap: 0.5,
                    color: theme.palette.primary.main,
                    fontSize: '0.875rem'
                  }}
                >
                  <RestoreIcon fontSize="small" />
                  {isRecoveryMode ? 'Create a new wallet instead' : 'Recover existing wallet'}
                </Link>
                
                <Button 
                  variant="contained"
                  color="primary"
                  // Use correct handler based on mode
                  onClick={isRecoveryMode ? handleRecoverWallet : handleCreateWallet} 
                  disabled={isLoading || !newWalletName || isWalletNameDuplicate || (usePasswordProtection && (!walletPassword || walletPassword !== confirmPassword)) || (isRecoveryMode && (!recoverySeedPhrase || recoverySeedPhrase.trim() === ''))}
                  startIcon={isLoading && tabValue === 1 ? <CircularProgress size={20} /> : isRecoveryMode ? <RestoreIcon /> : <AddIcon />}
                  sx={{ 
                    minWidth: '160px',
                    textTransform: 'none',
                    fontWeight: 600
                  }}
                >
                  {isLoading && tabValue === 1 
                    ? (isRecoveryMode ? 'Recovering...' : 'Creating...') 
                    : (isRecoveryMode ? 'Recover Wallet' : 'Create Wallet')}
                </Button>
              </Box>
            </Box>
          </TabPanel>
        </DialogContent>
      </Dialog>
    </>
  );
}