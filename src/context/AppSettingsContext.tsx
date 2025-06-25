import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { AppSettings } from '../types/settings';

// Define the shape of our app settings context
interface AppSettingsContextType {
  appSettings: AppSettings | null;
  updateDeveloperMode: (enabled: boolean) => Promise<void>;
  updateSkipSeedPhraseDialogs: (skip: boolean) => Promise<void>;
  refreshSettings: () => Promise<AppSettings | null>;
}

// Create the context with default values
const AppSettingsContext = createContext<AppSettingsContextType>({
  appSettings: null,
  updateDeveloperMode: async () => {},
  updateSkipSeedPhraseDialogs: async () => {},
  refreshSettings: async () => null
});

// Create a provider component
export function AppSettingsProvider({ children }: { children: ReactNode }) {
  const [appSettings, setAppSettings] = useState<AppSettings | null>(null);

  const refreshSettings = async () => {
    try {
      const settings = await invoke<AppSettings>('get_app_settings');
      setAppSettings(settings);
      return settings;
    } catch (err) {
      console.error('Failed to load app settings:', err);
      return null;
    }
  }; 
  // Update developer mode setting
  const updateDeveloperMode = async (enabled: boolean) => {
    try {
      console.log('Setting developer mode to:', enabled);
      
      // Use snake_case to match the Rust backend expectations
      const result = await invoke<boolean>('update_app_settings', {
        developer_mode: enabled
      });
      
      if (result) {
        console.log('Developer mode updated successfully');
        
        // On success, update local state directly
        setAppSettings(prev => {
          if (!prev) return null;
          return {...prev, developer_mode: enabled};
        });
      } else {
        console.error('Failed to update developer mode in backend');
        throw new Error('Failed to update developer mode');
      }
    } catch (err) {
      console.error('Failed to update developer mode setting:', err);
      await refreshSettings();
      throw err;
    }
  };
    // Update skip seed phrase dialogs setting
  const updateSkipSeedPhraseDialogs = async (skipDialogs: boolean) => {
    try {
      console.log('Updating skip seed phrase dialogs setting to:', skipDialogs);
      
      // Check if developer mode is enabled
      if (!appSettings?.developer_mode) {
        console.error('Cannot update skip seed phrase dialogs: Developer mode is not enabled');
        throw new Error('Developer mode must be enabled to skip seed phrase dialogs');
      }
      
      // Use snake_case to match the Rust backend expectations
      const result = await invoke<boolean>('update_app_settings', {
        skip_seed_phrase_dialogs: skipDialogs
      });
      
      if (result) {
        console.log('Skip seed phrase dialogs setting updated successfully in backend');
          // On success, update local state directly instead of refreshing
        // This ensures UI stays in sync with backend state
        setAppSettings(prev => {
          if (!prev) return null;
          return {...prev, skip_seed_phrase_dialogs: skipDialogs};
        });
      } else {
        console.error('Failed to update skip seed phrase dialogs setting in backend');
        throw new Error('Failed to update skip seed phrase dialogs setting');
      }
    } catch (err) {
      console.error('Failed to update skip seed phrase dialogs setting:', err);
      // On error, refresh to get the actual state from backend
      await refreshSettings();
      throw err;
    }
  };

  // Load app settings on initial render
  useEffect(() => {
    refreshSettings();
  }, []);
  return (
    <AppSettingsContext.Provider value={{
      appSettings,
      updateDeveloperMode,
      updateSkipSeedPhraseDialogs,
      refreshSettings
    }}>
      {children}
    </AppSettingsContext.Provider>
  );
}

// Custom hook to use the app settings context
export const useAppSettings = () => useContext(AppSettingsContext);
