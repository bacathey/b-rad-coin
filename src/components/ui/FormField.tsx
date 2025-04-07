import React, { ReactNode } from 'react';
import { Box, Typography, FormHelperText } from '@mui/material';
import { useThemeMode } from '../../hooks/useThemeMode';

interface FormFieldProps {
  /**
   * Field label text
   */
  label: string;
  /**
   * Form field content (input, select, etc.)
   */
  children: ReactNode;
  /**
   * Optional helper text to display below the field
   */
  helperText?: string;
  /**
   * Whether the field has an error
   */
  error?: boolean;
  /**
   * Whether the field is required
   */
  required?: boolean;
  /**
   * Optional additional margin at the bottom
   */
  marginBottom?: number | string;
}

/**
 * A standardized form field component that provides consistent styling and layout
 */
export const FormField = React.memo(function FormField({
  label,
  children,
  helperText,
  error = false,
  required = false,
  marginBottom = 3
}: FormFieldProps) {
  const { colors } = useThemeMode();

  return (
    <Box sx={{ mb: marginBottom }}>
      <Typography
        component="label"
        variant="body2"
        sx={{
          display: 'block',
          mb: 1,
          fontWeight: 500,
          color: error ? colors.error : colors.text
        }}
      >
        {label}
        {required && (
          <Box component="span" sx={{ color: colors.error, ml: 0.5 }}>
            *
          </Box>
        )}
      </Typography>

      {children}

      {helperText && (
        <FormHelperText
          error={error}
          sx={{
            mt: 0.5,
            ml: 0,
            fontSize: '0.75rem'
          }}
        >
          {helperText}
        </FormHelperText>
      )}
    </Box>
  );
});