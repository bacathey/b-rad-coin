// filepath: c:\Users\bacat\source\repos\b-rad-coin\src\pages\Account.tsx
import reactLogo from "../assets/react.svg";
import bradcoinLogo from "../assets/bradcoin.png";
import { 
  Typography, 
  Box, 
  Stack, 
  TextField, 
  Button, 
  Card, 
  CardContent,
  useTheme 
} from '@mui/material';

interface AccountProps {
  greetMsg: string;
  name: string;
  setName: (name: string) => void;
  greet: () => void;
}

export default function Account({ greetMsg, name, setName, greet }: AccountProps) {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';

  return (
    <Box 
      sx={{ 
        textAlign: 'center',
        minHeight: 'calc(100vh - 64px - 48px)',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'flex-start',
        width: '100%',
        maxWidth: '100%',
        pt: 3,
        mx: 'auto',
        position: 'static',
      }}
    >
      <Typography 
        variant="h4" 
        component="h1" 
        gutterBottom
        sx={{
          color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#1a237e',
          textShadow: isDarkMode ? '0 2px 10px rgba(0,0,0,0.3)' : 'none',
          fontWeight: 600
        }}
      >
        Account Dashboard
      </Typography>

      <Stack direction="row" spacing={2} justifyContent="center" alignItems="center" sx={{ mb: 4 }}>
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src="/Bradcoin-192x192.png" className="logo bradcoin" alt="Bradcoin logo" />
        </a>
      </Stack>
      
      <Typography 
        variant="body1" 
        gutterBottom
        sx={{
          color: isDarkMode ? 'rgba(255, 255, 255, 0.8)' : 'rgba(0, 0, 0, 0.7)',
          fontWeight: 500
        }}
      >
        Click on the Tauri, Vite, React, and Bradcoin logos to learn more.
      </Typography>

      <Card sx={{ 
        width: '100%', 
        maxWidth: 500, 
        mt: 4, 
        mb: 'auto',
        mx: 'auto',
        borderRadius: 2,
        ...(isDarkMode ? {
          background: 'rgba(19, 47, 76, 0.6)',
          backdropFilter: 'blur(10px)',
          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
          border: '1px solid rgba(255, 255, 255, 0.1)'
        } : {
          background: 'linear-gradient(180deg, #ffffff 0%, #f5f7fa 100%)',
          boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
          border: '1px solid rgba(0, 0, 0, 0.08)',
          transition: 'all 0.2s ease-in-out',
          '&:hover': {
            transform: 'translateY(-4px)',
            boxShadow: '0 6px 25px rgba(0, 0, 0, 0.2)',
          }
        })
      }}>
        <CardContent sx={{ 
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          padding: '24px'
        }}>
          <Box
            component="form"
            onSubmit={(e) => {
              e.preventDefault();
              greet();
            }}
            sx={{ 
              width: '100%',
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center'
            }}
          >
            <TextField
              fullWidth
              id="greet-input"
              label="Enter a name..."
              variant="outlined"
              value={name}
              onChange={(e) => setName(e.target.value)}
              sx={{ 
                m: 1, 
                width: '100%', 
                maxWidth: '450px',
                ...(isDarkMode ? {
                  '& .MuiOutlinedInput-root': {
                    '& fieldset': {
                      borderColor: 'rgba(255, 255, 255, 0.15)',
                    },
                    '&:hover fieldset': {
                      borderColor: 'rgba(255, 255, 255, 0.25)',
                    },
                    '&.Mui-focused fieldset': {
                      borderColor: 'rgba(144, 202, 249, 0.6)',
                    }
                  },
                  '& .MuiInputLabel-root': {
                    color: 'rgba(255, 255, 255, 0.7)',
                  },
                  '& .MuiInputBase-input': {
                    color: 'rgba(255, 255, 255, 0.9)',
                  }
                } : {
                  '& .MuiOutlinedInput-root': {
                    '&.Mui-focused fieldset': {
                      borderColor: '#1a237e',
                    }
                  },
                  '& .MuiInputLabel-root.Mui-focused': {
                    color: '#1a237e',
                  }
                })
              }}
            />
            <Button 
              variant="contained" 
              type="submit"
              sx={{ 
                m: 1, 
                width: '100%', 
                maxWidth: '450px',
                fontWeight: 600,
                padding: '10px',
                ...(isDarkMode ? {
                  background: 'linear-gradient(90deg, #0d2b59, #2979ff)',
                  '&:hover': {
                    background: 'linear-gradient(90deg, #0d3074, #448aff)',
                    boxShadow: '0 4px 20px rgba(41, 121, 255, 0.5)',
                  }
                } : {
                  background: 'linear-gradient(90deg, #3949ab, #42a5f5)',
                  boxShadow: '0 2px 10px rgba(57, 73, 171, 0.3)',
                  '&:hover': {
                    background: 'linear-gradient(90deg, #3f51b5, #64b5f6)',
                    boxShadow: '0 4px 15px rgba(57, 73, 171, 0.4)',
                  }
                })
              }}
            >
              Greet
            </Button>
          </Box>
          {greetMsg && (
            <Typography 
              variant="h6" 
              sx={{ 
                mt: 2, 
                textAlign: 'center',
                color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#1a237e',
                fontWeight: 600
              }}
            >
              {greetMsg}
            </Typography>
          )}
        </CardContent>
      </Card>
    </Box>
  );
}