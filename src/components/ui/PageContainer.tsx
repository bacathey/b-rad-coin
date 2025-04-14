import { Box, Typography } from '@mui/material';
import { ReactNode } from 'react';
import { useThemeMode } from '../../hooks/useThemeMode';

interface PageContainerProps {
  /**
   * Content to display
   */
  children: ReactNode;
  /**
   * Optional page title
   */
  title?: string;
  /**
   * Optional page description
   */
  description?: string;
  /**
   * Maximum width of the content area
   */
  maxContentWidth?: number;
  /**
   * Whether the content is in a loading state
   */
  isLoading?: boolean;
  /**
   * Optional error message to display
   */
  error?: string | null;
}

/**
 * A standardized container for page content with consistent styling
 */
export const PageContainer = ({ 
  children, 
  title, 
  description,
  maxContentWidth = 1200,
  isLoading,
  error
}: PageContainerProps) => {
  const { getPageTitleStyle } = useThemeMode();

  return (
    <Box
      sx={{
        width: '100%',
        maxWidth: '100%',
        pt: 3,
        px: { xs: 2, sm: 3 },
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center'
      }}
    >
      {title && (
        <Typography
          variant="h4"
          component="h1"
          gutterBottom
          sx={getPageTitleStyle()}
        >
          {title}
        </Typography>
      )}

      {description && (
        <Typography
          variant="body1"
          color="text.secondary"
          sx={{ mb: 3, textAlign: 'center', maxWidth: '800px' }}
        >
          {description}
        </Typography>
      )}
      
      {error && (
        <Typography
          variant="body1"
          color="error"
          sx={{ mb: 3, textAlign: 'center', maxWidth: '800px' }}
        >
          {error}
        </Typography>
      )}
      
      <Box sx={{ 
        width: '100%', 
        maxWidth: maxContentWidth, 
        mx: 'auto',
        opacity: isLoading ? 0.7 : 1,
        transition: 'opacity 0.2s',
        pointerEvents: isLoading ? 'none' : 'auto'
      }}>
        {children}
      </Box>
    </Box>
  );
};