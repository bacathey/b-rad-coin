import { 
  Typography, 
  Box, 
  Card, 
  CardContent, 
  useTheme, 
  Paper, 
  Grid, 
  Button 
} from '@mui/material';

export default function Advanced() {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';

  const cardStyle = isDarkMode ? {
    background: 'rgba(19, 47, 76, 0.6)',
    backdropFilter: 'blur(10px)',
    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
    border: '1px solid rgba(255, 255, 255, 0.1)'
  } : {
    background: 'linear-gradient(180deg, #ffffff 0%, #f5f7fa 100%)',
    boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
    border: '1px solid rgba(0, 0, 0, 0.08)'
  };

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
      <Typography 
        variant="h4" 
        component="h1" 
        gutterBottom
        sx={{
          color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#1a237e',
          textShadow: isDarkMode ? '0 2px 10px rgba(0,0,0,0.3)' : 'none',
          fontWeight: 600,
          mb: 3
        }}
      >
        Advanced
      </Typography>
      
      {/* Container with fixed maximum width and full width */}
      <Box sx={{ width: '100%', maxWidth: 1200, mx: 'auto' }}>
        {/* Wallet Recovery - Moved from Settings */}
        <Grid item xs={12} sx={{ mb: 3 }}>
          <Paper sx={{ 
            p: 3, 
            ...(isDarkMode ? {
              background: 'rgba(19, 47, 76, 0.6)',
              backdropFilter: 'blur(10px)',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
              border: '1px solid rgba(255, 255, 255, 0.1)'
            } : {
              background: 'linear-gradient(90deg, #f5f7fa 0%, #ffffff 100%)',
              boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
              border: '1px solid rgba(0, 0, 0, 0.08)'
            }) 
          }}>
            <Grid container spacing={2} alignItems="center">
              <Grid item xs={12} sm={8}>
                <Typography variant="h6" gutterBottom fontWeight={600}>
                  Wallet Recovery
                </Typography>
                <Typography variant="body2" color="text.secondary" paragraph>
                  Make sure to back up your wallet recovery phrase. Your recovery phrase is the only way to restore your wallet if you lose access to this device.
                </Typography>
              </Grid>
              <Grid item xs={12} sm={4} sx={{ display: 'flex', justifyContent: { xs: 'flex-start', sm: 'flex-end' } }}>
                <Button variant="contained" color="primary">
                  Backup Wallet
                </Button>
                <Button variant="outlined" color="primary" sx={{ ml: 1 }}>
                  View Phrase
                </Button>
              </Grid>
            </Grid>
          </Paper>
        </Grid>
        
        {/* Mining Card */}
        <Box>
          <Card sx={{ ...cardStyle }}>
            <CardContent>
              <Typography variant="h6" sx={{ fontWeight: 600 }}>
                Mining
              </Typography>
              <Typography variant="body2" sx={{ mt: 1 }}>
                Manage your mining operations and settings.
              </Typography>
            </CardContent>
          </Card>
        </Box>
      </Box>
    </Box>
  );
}