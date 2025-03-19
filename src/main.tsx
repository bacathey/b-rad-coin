import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './App.css';
import { invoke } from '@tauri-apps/api/core';

// Ensure wallet is closed when app starts
async function initializeApp() {
  try {
    // Force close any wallet that might have been left open from previous session
    await invoke('close_wallet');
    console.log('App initialized: wallet state reset');
  } catch (error) {
    console.error('Failed to reset wallet state:', error);
  }
}

// Initialize the app before rendering
initializeApp().then(() => {
  ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
});
