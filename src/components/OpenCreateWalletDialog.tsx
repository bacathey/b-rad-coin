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
import { useAppSettings } from '../context/AppSettingsContext';
import { invoke } from '@tauri-apps/api/core';
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

export default function OpenCreateWalletDialog({ blockchainReady = true }: { blockchainReady?: boolean }) {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  const { isWalletOpen, setIsWalletOpen, setCurrentWallet, refreshWalletDetails, availableWallets } = useWallet();
  const { appSettings } = useAppSettings();
  
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
  const [showSeedPhraseDialogs, setShowSeedPhraseDialogs] = useState(true);  const [isDeveloperMode, setIsDeveloperMode] = useState(false);

  // Helper function to reset Create Wallet dialog fields and submit button
  const resetCreateWalletForm = () => {
    setNewWalletName('');
    setWalletPassword('');
    setConfirmPassword('');
    setUsePasswordProtection(false);
    setErrorMessage('');
    setIsLoading(false);
    setIsRecoveryMode(false);
    setRecoverySeedPhrase('');
    setTempWalletData(null);
    setGeneratedSeedPhrase('');
    console.log('Create Wallet dialog form has been reset');
  };
  // Load app settings  // Load and sync settings from context
  useEffect(() => {
    if (appSettings) {
      setIsDeveloperMode(appSettings.developer_mode);
      
      // Only allow skipping seed phrase dialogs if developer mode is enabled
      if (appSettings.developer_mode && appSettings.skip_seed_phrase_dialogs) {
        setShowSeedPhraseDialogs(false); // Skip dialogs only if both conditions are true
      } else {
        setShowSeedPhraseDialogs(true); // Always show dialogs in non-dev mode
      }
        console.log('App settings synced from context:', { 
        developerMode: appSettings.developer_mode, 
        skipSeedDialogs: appSettings.skip_seed_phrase_dialogs,
        willShowDialogs: !appSettings.developer_mode || !appSettings.skip_seed_phrase_dialogs,
        finalShowSeedPhraseDialogs: appSettings.developer_mode && appSettings.skip_seed_phrase_dialogs ? false : true
      });
    } else {
      // Default to showing seed phrase dialogs if settings aren't loaded yet
      setShowSeedPhraseDialogs(true);
      setIsDeveloperMode(false);
    }  }, [appSettings]); // Re-run whenever appSettings changes

  // Sync local walletsList with context's availableWallets
  useEffect(() => {
    console.log('OpenCreateWalletDialog: Syncing wallet list from context, available wallets:', availableWallets.length);
    console.log('OpenCreateWalletDialog: Available wallet names:', availableWallets.map(w => w.name));
    console.log('OpenCreateWalletDialog: Currently selected wallet:', selectedWallet);
    
    setWalletsList(availableWallets);
    
    // If we have wallets, handle selection
    if (availableWallets.length > 0) {
      // Check if currently selected wallet still exists
      const selectedWalletExists = selectedWallet && availableWallets.find(w => w.name === selectedWallet);
      
      if (!selectedWalletExists) {
        // Either no wallet selected or selected wallet doesn't exist - select first one
        console.log('OpenCreateWalletDialog: Selecting first wallet:', availableWallets[0].name);
        setSelectedWallet(availableWallets[0].name);
        setIsSelectedWalletSecured(availableWallets[0].secured);
      } else {
        // Selected wallet still exists, update its security status
        const currentWalletDetails = availableWallets.find(w => w.name === selectedWallet);
        if (currentWalletDetails) {
          setIsSelectedWalletSecured(currentWalletDetails.secured);
        }
      }
      setTabValue(0); // Set to Open Wallet tab when wallets exist
    } else {
      // If no wallets exist, clear selection and set to Create Wallet tab
      console.log('OpenCreateWalletDialog: No wallets available, clearing selection');
      setSelectedWallet('');
      setIsSelectedWalletSecured(false);
      setTabValue(1);
    }
    setIsGettingWallets(false);
  }, [availableWallets, selectedWallet]); // Re-adding selectedWallet but with better logic to prevent loops  // Check for duplicate wallet names when wallet name changes
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
  }, [isWalletNameDuplicate, newWalletName]);  // Trigger wallet details refresh when the dialog opens
  useEffect(() => {
    // Always refresh wallets when the dialog should be open to ensure fresh data
    if (!isWalletOpen) {
      console.log('OpenCreateWalletDialog: Dialog opening, refreshing wallet details');
      setIsGettingWallets(true);
      refreshWalletDetails().finally(() => setIsGettingWallets(false));
    }
  }, [isWalletOpen]); // Removed refreshWalletDetails from dependencies
  // Force refresh when dialog becomes visible to ensure we have fresh data
  useEffect(() => {
    if (!isWalletOpen) {
      console.log('OpenCreateWalletDialog: Dialog became visible, ensuring fresh wallet list');
      // Force a fresh load from the context
      refreshWalletDetails().catch(error => {
        console.error('Failed to refresh wallet details on dialog open:', error);
      });
    }
  }, [isWalletOpen, refreshWalletDetails]);

  // Clear all password fields whenever the dialog becomes visible
  useEffect(() => {
    if (!isWalletOpen) {
      console.log('OpenCreateWalletDialog: Dialog opened, clearing all password fields');
      setWalletPassword('');
      setOpenWalletPassword('');
      setConfirmPassword('');
    }
  }, [isWalletOpen]);
  const handleWalletChange = (event: SelectChangeEvent) => {
    const walletName = event.target.value as string;
    setSelectedWallet(walletName);
    
    // Find if the selected wallet is secured
    const walletInfo = walletsList.find(w => w.name === walletName);
    setIsSelectedWalletSecured(walletInfo?.secured || false);
    
    // Always clear password when switching wallets for security
    setOpenWalletPassword('');
  };

  const handleTabChange = (_: React.SyntheticEvent, newValue: number) => {
    setTabValue(newValue);
    setErrorMessage('');
    // Reset recovery mode when changing tabs
    setIsRecoveryMode(false);
    // Clear all password fields when switching tabs for security
    setWalletPassword('');
    setOpenWalletPassword('');
    setConfirmPassword('');
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
  // New function to create wallet directly without relying on state updates
  const createWalletDirectly = async (walletData: {name: string; password: string; usePassword: boolean}) => {
    setErrorMessage('');
    
    try {
      console.log('Creating wallet directly:', walletData.name);
      
      // Generate a seed phrase for wallet creation
      // Only in developer mode with skip dialogs should we pass undefined to let backend generate placeholder
      let seedPhraseToUse: string | undefined;
      
      if (isDeveloperMode && !showSeedPhraseDialogs) {
        // Developer mode with skip enabled - let backend handle with placeholder
        console.log('Developer mode with skip dialogs: using backend placeholder');
        seedPhraseToUse = undefined;
      } else {
        // Normal mode or developer mode without skip - generate real seed phrase
        console.log('Generating seed phrase for direct wallet creation');
        seedPhraseToUse = await invoke<string>('generate_seed_phrase');
        if (!seedPhraseToUse) {
          throw new Error('Failed to generate seed phrase');
        }
      }
      
      const result = await invoke('create_wallet', { 
        walletName: walletData.name, 
        password: walletData.password,
        usePassword: walletData.usePassword,
        seedPhrase: seedPhraseToUse
      });
      
      if (result) {
        console.log('Wallet created successfully:', walletData.name);
        setCurrentWallet({
          name: walletData.name,
          secured: walletData.usePassword
        });
        setIsWalletOpen(true);
        await refreshWalletDetails();
        
        // Clear form data
        setNewWalletName('');
        setWalletPassword('');
        setConfirmPassword('');
        setUsePasswordProtection(false);
      } else {
        console.error('Wallet creation returned false');
        setErrorMessage('Failed to create wallet');
      }
    } catch (error) {
      console.error('Failed to create wallet directly:', error);
      if (error instanceof Error) {
        setErrorMessage(`Error: ${error.message}`);
      } else {
        setErrorMessage('Unknown error occurred during wallet creation');
      }
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

    setIsLoading(true);
    
    try {
      // Create the wallet data object that we'll use
      const walletData = {
        name: newWalletName,
        password: walletPassword,
        usePassword: usePasswordProtection
      };
        // Check if we should show seed phrase dialogs
      console.log('Before wallet creation decision:', { 
        showSeedPhraseDialogs, 
        isDeveloperMode, 
        appSettingsDeveloperMode: appSettings?.developer_mode,
        appSettingsSkipSeed: appSettings?.skip_seed_phrase_dialogs
      });
      
      if (showSeedPhraseDialogs) {
        // Normal flow - show seed phrase generation and verification dialogs
        console.log('Normal flow: showing seed phrase dialogs');
        setTempWalletData(walletData);
        
        const phrase = await invoke<string>('generate_seed_phrase');
        if (phrase) {
          setGeneratedSeedPhrase(phrase);
          setSeedPhraseDialogOpen(true);
        } else {
          setErrorMessage('Failed to generate seed phrase');
          setIsLoading(false);
        }
      } else if (isDeveloperMode) {
        // Developer mode with skip dialogs enabled - create wallet directly
        console.log('Developer mode with skip dialogs enabled, creating wallet directly');
        console.log(`Developer mode: ${isDeveloperMode}, Skip dialogs: ${!showSeedPhraseDialogs}`);
        
        // Use the direct creation function which will handle the seed phrase appropriately
        await createWalletDirectly(walletData);
      } else {
        // This should not happen - if not in developer mode, dialogs should always be shown
        console.error('Invalid state: not in developer mode but trying to skip dialogs');
        setErrorMessage('Seed phrase verification is required for wallet creation');
        setIsLoading(false);
      }
    } catch (error) {
      console.error('Failed to start wallet creation:', error);
      if (error instanceof Error) {
        setErrorMessage(`Error: ${error.message}`);
      } else {
        setErrorMessage('Unknown error occurred during wallet creation');
      }
      setIsLoading(false);
    }
  };
  // Function to create the wallet after seed phrase verification (Dialog Flow)
  const finalizeWalletCreation = async () => {
    if (!tempWalletData) {
      console.error('No temporary wallet data available for regular flow');
      setErrorMessage('Wallet creation failed: No wallet data available');
      setIsLoading(false);
      return;
    }
    
    setIsLoading(true);
    setErrorMessage('');
    
    try {
      console.log('Creating wallet after seed phrase verification:', tempWalletData.name);
      
      // For regular flow, we always use the seed phrase from the verification process
      const result = await invoke('create_wallet', { 
        walletName: tempWalletData.name, 
        password: tempWalletData.password,
        usePassword: tempWalletData.usePassword,
        seedPhrase: generatedSeedPhrase
      });
      
      if (result) {
        console.log('Wallet created successfully:', tempWalletData.name);
        setCurrentWallet({
          name: tempWalletData.name,
          secured: tempWalletData.usePassword
        });
        setIsWalletOpen(true);
        await refreshWalletDetails();
        
        // Clear all wallet creation data
        setTempWalletData(null);
        setGeneratedSeedPhrase('');
        setNewWalletName('');
        setWalletPassword('');
        setConfirmPassword('');
        setUsePasswordProtection(false);
      } else {
        console.error('Wallet creation returned false');
        setErrorMessage('Failed to create wallet after verification');
      }
    } catch (error) {
      console.error('Failed to finalize wallet creation:', error);
      if (error instanceof Error) {
        setErrorMessage(`Error: ${error.message}`);
      } else {
        setErrorMessage('Unknown error occurred during wallet creation');
      }
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

  // Debug log to track wallet list changes
  console.log('OpenCreateWalletDialog render: walletsList =', walletsList.map(w => w.name), 'selectedWallet =', selectedWallet);

  return (
    <>
      {/* Backdrop that covers the entire app when wallet is not open and blockchain is ready */}      
      <Backdrop 
        open={!isWalletOpen && blockchainReady} 
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
        seedPhrase={generatedSeedPhrase}        onClose={() => {
          setSeedPhraseDialogOpen(false);
          resetCreateWalletForm(); // Reset all Create Wallet dialog fields and submit button
        }}
        onContinue={() => {
          setSeedPhraseDialogOpen(false);
          setVerifySeedDialogOpen(true); // Open verification dialog
        }}
      />
      
      {/* Add Verify Seed Phrase Dialog */}
      <VerifySeedPhraseDialog
        open={verifySeedDialogOpen}
        seedPhrase={generatedSeedPhrase}        onClose={() => {
          setVerifySeedDialogOpen(false);
          resetCreateWalletForm(); // Reset all Create Wallet dialog fields and submit button
        }}
        onVerified={() => {
          setVerifySeedDialogOpen(false);
          finalizeWalletCreation(); // Call the final creation step
        }}
      />
      
      <Dialog 
        open={!isWalletOpen && blockchainReady}
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
                ) : (                  <FormControl fullWidth sx={{ mb: 2 }}>
                    <InputLabel id="wallet-select-label">Select Wallet</InputLabel>
                    <Select
                      labelId="wallet-select-label"
                      id="wallet-select"
                      value={selectedWallet}
                      label="Select Wallet"
                      onChange={handleWalletChange}
                      key={walletsList.length}
                    >                      {walletsList.map((wallet) => {
                        console.log('OpenCreateWalletDialog: Rendering MenuItem for wallet:', wallet.name);
                        return (
                          <MenuItem key={wallet.name} value={wallet.name}>
                            <Box sx={{ display: 'flex', alignItems: 'center', width: '100%' }}>
                              <ListItemIcon sx={{ minWidth: 36 }}>                                {wallet.secured ? 
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
                        );
                      })}
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
                  Please switch to the "Create New" tab to create or recover a wallet.
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
                <Box sx={{ display: 'flex', alignItems: 'center' }}>                  {usePasswordProtection ? 
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