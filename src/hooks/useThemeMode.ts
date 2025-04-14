import { useTheme } from '@mui/material';
import { 
  colors as themeColors, 
  gradients, 
  shadows as themeShadows, 
  borders,
  transitions
} from '../styles/themeConstants';

/**
 * A custom hook that provides theme-related values and utilities.
 * This centralizes theme logic to avoid repetition across components.
 */
export function useThemeMode() {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  const mode = isDarkMode ? 'dark' : 'light';
  
  // Common card style based on theme mode
  const getCardStyle = () => ({
    background: isDarkMode ? 'rgba(19, 47, 76, 0.6)' : gradients.light.card,
    backdropFilter: isDarkMode ? 'blur(10px)' : 'none',
    boxShadow: isDarkMode ? themeShadows.dark.card : themeShadows.light.card,
    border: isDarkMode ? borders.dark.card : borders.light.card,
    transition: `${transitions.backgroundColor}, ${transitions.boxShadow}, ${transitions.borderColor}`
  });

  // Common text colors
  const colors = {
    text: isDarkMode ? themeColors.dark.text : themeColors.light.text,
    textSecondary: isDarkMode ? themeColors.dark.textSecondary : themeColors.light.textSecondary,
    primary: theme.palette.primary.main,
    divider: isDarkMode ? themeColors.dark.divider : themeColors.light.divider,
    background: isDarkMode ? themeColors.dark.background : themeColors.light.background,
    success: isDarkMode ? themeColors.dark.success : themeColors.light.success,
    warning: isDarkMode ? themeColors.dark.warning : themeColors.light.warning,
    error: isDarkMode ? themeColors.dark.error : themeColors.light.error
  };

  // Common shadows
  const shadows = {
    text: isDarkMode ? themeShadows.dark.text : themeShadows.light.text,
    card: isDarkMode ? themeShadows.dark.card : themeShadows.light.card,
    button: isDarkMode ? themeShadows.dark.button : themeShadows.light.button
  };

  // Get page title style
  const getPageTitleStyle = () => ({
    color: colors.text,
    textShadow: shadows.text,
    fontWeight: 600,
    mb: 3
  });
  
  // Get section header style
  const getSectionHeaderStyle = () => ({
    color: colors.text,
    fontWeight: 600,
    mb: 2
  });

  // Get button styles
  const getButtonStyle = (variant: 'primary' | 'secondary' | 'success' | 'warning' | 'error' = 'primary') => {
    // Map the variant to corresponding color
    let variantColor;
    
    switch (variant) {
      case 'primary':
        variantColor = theme.palette.primary.main;
        break;
      case 'secondary':
        variantColor = theme.palette.secondary.main;
        break;
      case 'success':
        variantColor = isDarkMode ? themeColors.dark.success : themeColors.light.success;
        break;
      case 'warning':
        variantColor = isDarkMode ? themeColors.dark.warning : themeColors.light.warning;
        break;
      case 'error':
        variantColor = isDarkMode ? themeColors.dark.error : themeColors.light.error;
        break;
      default:
        variantColor = theme.palette.primary.main;
    }
    
    return {
      boxShadow: shadows.button,
      transition: `${transitions.transform}, ${transitions.boxShadow}`,
      '&:hover': {
        boxShadow: isDarkMode ? '0 6px 16px rgba(0, 0, 0, 0.4)' : '0 4px 12px rgba(0, 0, 0, 0.15)',
        transform: 'translateY(-1px)'
      },
      // Apply variant-specific styles when not using MUI's default variants
      ...(variant !== 'primary' && variant !== 'secondary' && {
        backgroundColor: variantColor,
        color: '#ffffff',
        '&:hover': {
          backgroundColor: variantColor,
          boxShadow: isDarkMode ? '0 6px 16px rgba(0, 0, 0, 0.4)' : '0 4px 12px rgba(0, 0, 0, 0.15)',
          transform: 'translateY(-1px)'
        }
      })
    };
  };

  // Get text field style
  const getTextFieldStyle = () => ({
    '& .MuiOutlinedInput-root': {
      backgroundColor: isDarkMode ? 'rgba(19, 47, 76, 0.4)' : 'rgba(255, 255, 255, 0.8)',
      backdropFilter: 'blur(4px)',
      transition: transitions.backgroundColor,
      '&:hover': {
        backgroundColor: isDarkMode ? 'rgba(19, 47, 76, 0.6)' : 'rgba(255, 255, 255, 1)',
      },
      '&.Mui-focused': {
        backgroundColor: isDarkMode ? 'rgba(19, 47, 76, 0.8)' : 'rgba(255, 255, 255, 1)',
      }
    }
  });

  // Get progress bar style
  const getProgressBarStyle = (color: 'primary' | 'success' | 'warning' | 'error' = 'primary') => {
    let barColor;
    
    if (color === 'primary') barColor = isDarkMode ? themeColors.dark.secondary : themeColors.light.primary;
    else if (color === 'success') barColor = isDarkMode ? themeColors.dark.success : themeColors.light.success;
    else if (color === 'warning') barColor = isDarkMode ? themeColors.dark.warning : themeColors.light.warning;
    else if (color === 'error') barColor = isDarkMode ? themeColors.dark.error : themeColors.light.error;
    
    return {
      borderRadius: 1,
      height: 6,
      backgroundColor: isDarkMode ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.1)',
      '& .MuiLinearProgress-bar': {
        backgroundColor: barColor,
      }
    };
  };

  return {
    theme,
    isDarkMode,
    mode,
    getCardStyle,
    colors,
    shadows,
    getPageTitleStyle,
    getSectionHeaderStyle,
    getButtonStyle,
    getTextFieldStyle,
    getProgressBarStyle
  };
}