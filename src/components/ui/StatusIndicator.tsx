import React, { useMemo } from 'react';
import { Box, Typography, Tooltip, CircularProgress, SxProps, Theme, keyframes } from '@mui/material';
import { useThemeMode } from '../../hooks/useThemeMode';

type StatusType = 'online' | 'offline' | 'warning' | 'loading' | 'success' | 'error';

interface StatusIndicatorProps {
  /**
   * The type of status to display
   */
  status: StatusType;
  /**
   * Label text to display next to the indicator
   */
  label?: string;
  /**
   * Optional tooltip text
   */
  tooltip?: string;
  /**
   * Optional size of the indicator (in pixels)
   */
  size?: number;
  /**
   * Optional custom styling
   */
  sx?: SxProps<Theme>;
}

/**
 * A reusable component for displaying various statuses with consistent styling
 */
export const StatusIndicator = React.memo(function StatusIndicator({
  status,
  label,
  tooltip,
  size = 10,
  sx = {}
}: StatusIndicatorProps) {
  const { colors } = useThemeMode();

  // Determine the color based on status
  const getStatusColor = () => {
    switch (status) {
      case 'online':
      case 'success':
        return colors.success;
      case 'offline':
      case 'error':
        return colors.error;
      case 'warning':
        return colors.warning;
      default:
        return colors.primary;
    }
  };

  // Create a pulsing animation with MUI's keyframes
  const statusColor = getStatusColor();
  const pulseAnimation = useMemo(() => 
    keyframes`
      0% {
        box-shadow: 0 0 ${size/2}px ${statusColor};
      }
      50% {
        box-shadow: 0 0 ${size}px ${statusColor};
      }
      100% {
        box-shadow: 0 0 ${size/2}px ${statusColor};
      }
    `,
  [size, statusColor]);

  const indicator = (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        ...sx
      }}
    >
      {status === 'loading' ? (
        <CircularProgress
          size={size}
          thickness={7}
          sx={{ color: colors.primary }}
        />
      ) : (
        <Box
          sx={{
            width: size,
            height: size,
            borderRadius: '50%',
            backgroundColor: getStatusColor(),
            boxShadow: `0 0 ${size/2}px ${getStatusColor()}`,
            animation: status === 'offline' || status === 'error' 
              ? 'none' 
              : `${pulseAnimation} 2s infinite`
          }}
        />
      )}

      {label && (
        <Typography
          variant="caption"
          sx={{
            ml: 1,
            color: colors.textSecondary,
            fontWeight: 500
          }}
        >
          {label}
        </Typography>
      )}
    </Box>
  );

  if (tooltip) {
    return (
      <Tooltip title={tooltip} arrow placement="right">
        {indicator}
      </Tooltip>
    );
  }

  return indicator;
});