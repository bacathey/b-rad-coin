import { createContext, useContext, useState, ReactNode } from 'react';

interface WalletDialogContextType {
  isDialogOpen: boolean;
  selectedTab: number;
  isExplicitlyOpened: boolean;
  openDialog: (tab?: number) => void;
  closeDialog: () => void;
  setTab: (tab: number) => void;
  forceCreateWalletTab: () => void;
}

const WalletDialogContext = createContext<WalletDialogContextType>({
  isDialogOpen: false,
  selectedTab: 0,
  isExplicitlyOpened: false,
  openDialog: () => {},
  closeDialog: () => {},
  setTab: () => {},
  forceCreateWalletTab: () => {},
});

export function WalletDialogProvider({ children }: { children: ReactNode }) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [selectedTab, setSelectedTab] = useState(0);
  const [isExplicitlyOpened, setIsExplicitlyOpened] = useState(false);
  const openDialog = (tab: number = 0) => {
    console.log('WalletDialogContext: openDialog called with tab:', tab);
    
    // Set all states together
    setSelectedTab(tab);
    setIsDialogOpen(true);
    setIsExplicitlyOpened(true);
    
    // Clear the explicitly opened flag after a short delay to allow normal operation
    setTimeout(() => {
      console.log('WalletDialogContext: Clearing isExplicitlyOpened flag to allow normal operation');
      setIsExplicitlyOpened(false);
    }, 100);
    
    // Use setTimeout to ensure state is fully updated before logging
    setTimeout(() => {
      console.log('WalletDialogContext: Dialog opened with selectedTab:', tab, 'isDialogOpen: true');
    }, 0);
  };

  const closeDialog = () => {
    setIsDialogOpen(false);
    setIsExplicitlyOpened(false);
  };
  const setTab = (tab: number) => {
    setSelectedTab(tab);
  };

  const forceCreateWalletTab = () => {
    console.log('WalletDialogContext: forceCreateWalletTab called - forcing tab to Create Wallet (1)');
    setSelectedTab(1);
    setIsDialogOpen(true);
    setIsExplicitlyOpened(true);
    
    // Use multiple techniques to ensure tab is set
    setTimeout(() => {
      console.log('WalletDialogContext: Setting tab to 1 after timeout');
      setSelectedTab(1);
    }, 0);
    
    setTimeout(() => {
      console.log('WalletDialogContext: Setting tab to 1 after longer timeout');
      setSelectedTab(1);
    }, 50);
    
    setTimeout(() => {
      console.log('WalletDialogContext: Clearing isExplicitlyOpened flag');
      setIsExplicitlyOpened(false);
    }, 200);
  };

  return (
    <WalletDialogContext.Provider value={{
      isDialogOpen,
      selectedTab,
      isExplicitlyOpened,
      openDialog,
      closeDialog,
      setTab,
      forceCreateWalletTab,
    }}>
      {children}
    </WalletDialogContext.Provider>
  );
}

export const useWalletDialog = () => {
  return useContext(WalletDialogContext);
};
