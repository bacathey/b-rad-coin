import React, { ReactNode } from 'react';
import { Box, Typography } from '@mui/material';
import { useThemeMode } from '../../hooks/useThemeMode';

interface SectionHeaderProps {
  /**
   * Header content
   */
  children: ReactNode;
  /**
   * Optional bottom margin
   */
  marginBottom?: number | string;
  /**
   * Optional description
   */
  description?: string;
  /**
   * Optional actions to display beside the header
   */
  actions?: ReactNode;
  /**
   * Optional variant for the header
   */
  variant?: 'h5' | 'h6' | 'subtitle1' | 'subtitle2';
}

/**
 * A standardized section header component with consistent styling
 */
export const SectionHeader = React.memo(function SectionHeader({ 
  children, 
  marginBottom = 2,
  description,
  actions,
  variant = 'h6'
}: SectionHeaderProps) {
  const { colors } = useThemeMode();

  return (
    <Box sx={{ 
      display: 'flex', 
      flexDirection: 'column', 
      mb: description ? 1 : marginBottom 
    }}>
      <Box sx={{ 
        display: 'flex', 
        alignItems: 'center', 
        justifyContent: 'space-between' 
      }}>
        <Typography
          variant={variant}
          sx={{
            color: colors.text,
            fontWeight: 600
          }}
        >
          {children}
        </Typography>

        {actions && (
          <Box sx={{ ml: 2 }}>
            {actions}
          </Box>
        )}
      </Box>

      {description && (
        <Typography 
          variant="body2" 
          color="text.secondary"
          sx={{ mt: 0.5, mb: marginBottom }}
        >
          {description}
        </Typography>
      )}
    </Box>
  );
});