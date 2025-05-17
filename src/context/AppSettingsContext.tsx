import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { AppSettings } from '../types/settings';

// Define the shape of our app settings context
interface AppSettingsContextType {
  appSettings: AppSettings | null;
  updateDeveloperMode: (enabled: boolean) => Promise<void>;
  updateSeedPhraseDialogs: (enabled: boolean) => Promise<void>;
  refreshSettings: () => Promise<AppSettings | null>;
}

// Create the context with default values
const AppSettingsContext = createContext<AppSettingsContextType>({
  appSettings: null,
  updateDeveloperMode: async () => {},
  updateSeedPhraseDialogs: async () => {},
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
      // Log the exact structure being sent to the backend
      console.log('Payload:', { developer_mode: enabled });
      
      // Add a delay to make sure logs appear in sequence
      await new Promise(resolve => setTimeout(resolve, 100));
      
      await invoke('update_app_settings', { 
        developer_mode: enabled 
      });
      
      const updatedSettings = await refreshSettings();
      console.log('Developer mode after refresh:', updatedSettings?.developer_mode);
    } catch (err) {
      console.error('Failed to update developer mode setting:', err);
      throw err;
    }
  };

  // Update seed phrase dialogs setting
  const updateSeedPhraseDialogs = async (enabled: boolean) => {
    try {
      await invoke('update_app_settings', { 
        show_seed_phrase_dialogs: enabled
      });
      await refreshSettings();
    } catch (err) {
      console.error('Failed to update seed phrase dialogs setting:', err);
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
      updateSeedPhraseDialogs,
      refreshSettings
    }}>
      {children}
    </AppSettingsContext.Provider>
  );
}

// Custom hook to use the app settings context
export const useAppSettings = () => useContext(AppSettingsContext);
