import { 
  Typography, 
  Box, 
  Card, 
  CardContent, 
  useTheme,
  Grid,
  Link,
  Divider,
  Stack,
  Avatar
} from '@mui/material';

export default function About() {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  
  // Card styling based on theme
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
        About B-Rad Coin
      </Typography>
      
      <Box sx={{ width: '100%', maxWidth: 1200, mx: 'auto' }}>
        <Grid container spacing={3}>
          {/* Main app info */}
          <Grid item xs={12} md={8}>
            <Card sx={{ ...cardStyle }}>
              <CardContent>
                <Stack direction="row" spacing={2} alignItems="center" mb={3}>
                  <Avatar 
                    src="/bradcoin.png" 
                    alt="B-Rad Coin Logo" 
                    sx={{ width: 64, height: 64 }}
                  />
                  <Box>
                    <Typography variant="h5" gutterBottom fontWeight={600}>
                      B-Rad Coin Wallet
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      A modern, secure wallet for managing your Bradcoin assets
                    </Typography>
                  </Box>
                </Stack>
                
                <Typography variant="body1" paragraph>
                  B-Rad Coin Wallet is a secure, easy-to-use Bradcoin wallet built with modern technologies.
                  Our wallet offers complete control over your Bradcoin assets with advanced security features
                  and an intuitive user interface.
                </Typography>
                
                <Typography variant="body1" paragraph>
                  This application is built using Tauri + React, providing native-level performance with web
                  technologies. The interface is built with Material UI for a clean, responsive design that works
                  across all devices.
                </Typography>

                <Divider sx={{ my: 2 }} />
                
                <Typography variant="h6" gutterBottom>
                  Key Features
                </Typography>
                
                <Typography variant="body1">
                  • Secure, encrypted storage for your private keys<br />
                  • Transaction tracking and management<br />
                  • Support for multiple wallets<br />
                  • Advanced network monitoring<br />
                  • Beautiful light and dark themes<br />
                  • Developer tools for customization
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          
          {/* Technologies used */}
          <Grid item xs={12} md={4}>
            <Card sx={{ ...cardStyle }}>
              <CardContent>
                <Typography variant="h6" gutterBottom fontWeight={600}>
                  Built With
                </Typography>
                
                <Stack spacing={1.5}>
                  <Stack direction="row" spacing={1.5} alignItems="center">
                    <img src="/tauri.svg" alt="Tauri" width={24} height={24} />
                    <Typography variant="body1">
                      <Link href="https://tauri.app" target="_blank" rel="noopener" underline="hover">
                        Tauri
                      </Link>
                      {" - "}Lightweight, secure framework
                    </Typography>
                  </Stack>
                  
                  <Stack direction="row" spacing={1.5} alignItems="center">
                    <img src={"/vite.svg"} alt="Vite" width={24} height={24} />
                    <Typography variant="body1">
                      <Link href="https://vitejs.dev" target="_blank" rel="noopener" underline="hover">
                        Vite
                      </Link>
                      {" - "}Next Generation Frontend Tooling
                    </Typography>
                  </Stack>
                  
                  <Stack direction="row" spacing={1.5} alignItems="center">
                    <img src="/assets/react.svg" alt="React" width={24} height={24} />
                    <Typography variant="body1">
                      <Link href="https://reactjs.org" target="_blank" rel="noopener" underline="hover">
                        React
                      </Link>
                      {" - "}UI Component Library
                    </Typography>
                  </Stack>
                  
                  <Stack direction="row" spacing={1.5} alignItems="center">
                    <Typography variant="body2" color="text.secondary" sx={{ ml: 0.5, mr: 1 }}>MUI</Typography>
                    <Typography variant="body1">
                      <Link href="https://mui.com" target="_blank" rel="noopener" underline="hover">
                        Material UI
                      </Link>
                      {" - "}React Component Library
                    </Typography>
                  </Stack>
                  
                  <Stack direction="row" spacing={1.5} alignItems="center">
                    <Typography variant="body2" color="text.secondary" sx={{ ml: 0.5, mr: 1 }}>TS</Typography>
                    <Typography variant="body1">
                      <Link href="https://www.typescriptlang.org" target="_blank" rel="noopener" underline="hover">
                        TypeScript
                      </Link>
                      {" - "}Type-safe JavaScript
                    </Typography>
                  </Stack>
                </Stack>
                
                <Divider sx={{ my: 2 }} />
                
                <Typography variant="h6" gutterBottom fontWeight={600}>
                  Version
                </Typography>
                <Typography variant="body1">
                  Current Version: 0.1.0
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
                  © 2023-2025 B-Rad Coin
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      </Box>
    </Box>
  );
}
