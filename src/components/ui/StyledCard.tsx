import React, { ReactNode } from 'react';
import { Card, CardContent, CardProps, Typography } from '@mui/material';
import { useThemeMode } from '../../hooks/useThemeMode';

interface StyledCardProps extends Omit<CardProps, 'children'> {
  /**
   * Card content
   */
  children: ReactNode;
  /**
   * Optional card title
   */
  title?: string;
  /**
   * Whether the card should take full height of its container
   */
  fullHeight?: boolean;
}

/**
 * A reusable card component with consistent styling based on the theme
 */
export const StyledCard: React.FC<StyledCardProps> = ({ 
  children, 
  title, 
  fullHeight = false,
  sx,
  ...rest 
}) => {
  const { getCardStyle, getSectionHeaderStyle } = useThemeMode();
  
  const cardStyle = getCardStyle();
  
  return (
    <Card
      sx={{
        ...cardStyle,
        ...(fullHeight && { height: '100%' }),
        ...sx
      }}
      {...rest}
    >
      <CardContent>
        {title && (
          <Typography 
            variant="h6" 
            component="h2"
            sx={getSectionHeaderStyle()}
          >
            {title}
          </Typography>
        )}
        {children}
      </CardContent>
    </Card>
  );
};