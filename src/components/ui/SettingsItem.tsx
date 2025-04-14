import React, { ReactNode } from 'react';
import { 
  ListItem, 
  ListItemIcon, 
  ListItemText, 
  ListItemSecondaryAction,
  Box
} from '@mui/material';

interface SettingsItemProps {
  /**
   * Icon for the setting
   */
  icon: ReactNode;
  /**
   * Primary text for the setting
   */
  primary: string;
  /**
   * Optional secondary text/description for the setting
   */
  secondary?: string;
  /**
   * Optional action component (like a Switch, Button, etc.)
   */
  action?: ReactNode;
  /**
   * Optional extra content that requires more space
   */
  extraContent?: ReactNode;
  /**
   * Optional click handler
   */
  onClick?: () => void;
}

/**
 * A reusable component for rendering settings items
 */
export const SettingsItem = React.memo(function SettingsItem({
  icon,
  primary,
  secondary,
  action,
  extraContent,
  onClick
}: SettingsItemProps) {
  const hasAction = Boolean(action);
  const hasExtraContent = Boolean(extraContent);

  return (
    <ListItem 
      onClick={onClick}
      sx={{ 
        alignItems: hasExtraContent ? 'flex-start' : 'center',
        cursor: onClick ? 'pointer' : 'default'
      }}
    >
      {icon && (
        <ListItemIcon sx={{ mt: hasExtraContent ? 1 : 0 }}>
          {icon}
        </ListItemIcon>
      )}
      
      {hasExtraContent ? (
        <Box sx={{ width: '100%' }}>
          <ListItemText 
            primary={primary} 
            secondary={secondary}
            sx={{ mb: 1 }}
          />
          {extraContent}
        </Box>
      ) : (
        <ListItemText 
          primary={primary} 
          secondary={secondary} 
        />
      )}
      
      {hasAction && !hasExtraContent && (
        <ListItemSecondaryAction>
          {action}
        </ListItemSecondaryAction>
      )}
    </ListItem>
  );
});