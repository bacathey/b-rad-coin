/**
 * Theme constants for consistent styling across the application
 */

export const colors = {
  // Light mode colors
  light: {
    primary: '#1a237e',
    secondary: '#1565c0',
    success: '#2e7d32',
    warning: '#ed6c02',
    error: '#d32f2f',
    background: '#f5f7fa',
    paper: '#ffffff',
    text: '#1a237e',
    textSecondary: 'rgba(0, 0, 0, 0.7)',
    divider: 'rgba(0, 0, 0, 0.12)'
  },
  
  // Dark mode colors
  dark: {
    primary: '#90caf9',
    secondary: '#64b5f6',
    success: '#81c784',
    warning: '#ffb74d',
    error: '#e57373',
    background: '#0a1929',
    paper: '#132f4c',
    text: 'rgba(255, 255, 255, 0.9)',
    textSecondary: 'rgba(255, 255, 255, 0.7)',
    divider: 'rgba(255, 255, 255, 0.12)'
  }
};

export const gradients = {
  // Light mode gradients
  light: {
    background: '#f5f7fa',
    card: 'linear-gradient(180deg, #ffffff 0%, #f5f7fa 100%)',
    appBar: 'linear-gradient(90deg, #1a237e 0%, rgb(14, 96, 134) 100%)'
  },
  
  // Dark mode gradients
  dark: {
    background: 'linear-gradient(145deg, #0a1929 0%, #0d2b59 50%, rgb(13, 75, 116) 100%)',
    card: 'linear-gradient(145deg, rgba(19, 47, 76, 0.6) 0%, rgba(19, 47, 76, 0.6) 100%)',
    appBar: 'linear-gradient(90deg, #0a1929 0%, rgb(13, 48, 89) 100%)'
  }
};

export const shadows = {
  // Light mode shadows
  light: {
    text: 'none',
    card: '0 4px 20px rgba(0, 0, 0, 0.15)',
    button: '0 2px 10px rgba(57, 73, 171, 0.3)',
    appBar: '0 2px 10px rgba(0, 0, 0, 0.1)'
  },
  
  // Dark mode shadows
  dark: {
    text: '0 2px 10px rgba(0, 0, 0, 0.3)',
    card: '0 8px 32px rgba(0, 0, 0, 0.3)',
    button: '0 4px 20px rgba(41, 121, 255, 0.5)',
    appBar: '0 4px 20px rgba(0, 0, 0, 0.4)'
  }
};

export const borders = {
  // Light mode borders
  light: {
    card: '1px solid rgba(0, 0, 0, 0.08)',
    input: '1px solid rgba(0, 0, 0, 0.23)',
    inputFocused: '2px solid #1a237e'
  },
  
  // Dark mode borders
  dark: {
    card: '1px solid rgba(255, 255, 255, 0.1)',
    input: '1px solid rgba(255, 255, 255, 0.23)',
    inputFocused: '2px solid #90caf9'
  }
};

export const transitions = {
  // Common transitions
  color: 'color 250ms cubic-bezier(0.4, 0, 0.2, 1)',
  backgroundColor: 'background-color 400ms cubic-bezier(0.4, 0, 0.2, 1)',
  borderColor: 'border-color 400ms cubic-bezier(0.4, 0, 0.2, 1)',
  boxShadow: 'box-shadow 400ms cubic-bezier(0.4, 0, 0.2, 1)',
  transform: 'transform 250ms cubic-bezier(0.4, 0, 0.2, 1)',
  fontWeight: 'font-weight 180ms cubic-bezier(0.4, 0, 0.2, 1)',
  fontFamily: 'font-family 180ms cubic-bezier(0.4, 0, 0.2, 1)',
  fontSize: 'font-size 180ms cubic-bezier(0.4, 0, 0.2, 1)',
  all: 'all 250ms cubic-bezier(0.4, 0, 0.2, 1)'
};

// Spacing values (in pixels) for consistent layout
export const spacing = {
  xs: 4,
  sm: 8,
  md: 16,
  lg: 24,
  xl: 32,
  xxl: 48
};

// Common border radius values
export const borderRadius = {
  xs: 4,
  sm: 8,
  md: 12,
  lg: 16,
  xl: 24,
  pill: 500
};