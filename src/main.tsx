import React, { useEffect, useState } from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './App.css';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Application state
const AppInitializer = ({ children }: { children: React.ReactNode }) => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [initError, setInitError] = useState<string | null>(null);

  useEffect(() => {
    // Setup application shutdown listener
    const setupShutdownListener = async () => {
      console.log('Setting up shutdown event listener');
      return await listen('app-shutdown-complete', () => {
        console.log('App shutdown sequence completed, window will be closed by Rust backend...');
        // Let the Rust backend handle window closing
      });
    };

    // Initialize the application
    const initializeApp = async () => {
      console.log('Initializing application...');
      
      try {
        // Ensure any open wallets are closed when app starts
        await invoke('close_wallet');
        console.log('App initialized: wallet state reset');
        
        // Setup the shutdown listener
        const unlisten = await setupShutdownListener();

        // Register a cleanup function
        window.addEventListener('beforeunload', () => {
          console.log('Window unloading, performing cleanup...');
          unlisten();
        });

        setIsInitialized(true);
      } catch (error) {
        console.error('Failed to initialize application:', error);
        setInitError(`Failed to initialize: ${error instanceof Error ? error.message : String(error)}`);
      }
    };

    initializeApp();
  }, []);

  // Show loading state while initializing
  if (!isInitialized) {
    return (
      <div className="app-initializing">
        {initError ? (
          <div className="error-container">
            <h2>Initialization Error</h2>
            <p>{initError}</p>
            <button onClick={() => window.location.reload()}>Retry</button>
          </div>
        ) : (
          <div className="loading-container">
            <h2>Starting B-Rad Coin...</h2>
            <div className="loading-spinner" />
          </div>
        )}
      </div>
    );
  }

  // Render the application once initialized
  return <>{children}</>;
};

// Initialize the app with proper lifecycle management
ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <AppInitializer>
      <App />
    </AppInitializer>
  </React.StrictMode>
);
