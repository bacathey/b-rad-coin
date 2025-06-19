import { useState, useMemo } from "react";
import { BrowserRouter, Routes, Route, useLocation } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

// Material UI imports
import { 
  ThemeProvider, 
  createTheme, 
  CssBaseline,
  Container, 
  Box,
  useMediaQuery,
  Drawer
} from "@mui/material";

// Components
import Sidebar from "./components/Sidebar";
import AppHeader from "./components/AppHeader";
import OpenCreateWalletDialog from "./components/OpenCreateWalletDialog";

// Page components
import Account from "./pages/Account";
import Transactions from "./pages/Transactions";
import SendReceive from "./pages/SendReceive";
import Advanced from "./pages/Advanced";
import Settings from "./pages/Settings";
import Developer from "./pages/Developer";
import About from "./pages/About";

// Context Providers
import { WalletProvider, useWallet } from "./context/WalletContext";
import { AppSettingsProvider } from "./context/AppSettingsContext";
import { WalletDialogProvider } from "./context/WalletDialogContext";

// Sidebar width
const drawerWidth = 240;

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  
  // Add state for theme mode
  const prefersDarkMode = useMediaQuery('(prefers-color-scheme: dark)');
  const [mode, setMode] = useState<'light' | 'dark'>(prefersDarkMode ? 'dark' : 'light');
  
  // Add state for sidebar visibility on mobile
  const [mobileOpen, setMobileOpen] = useState(false);
  
  // Create a theme instance based on the mode
  const theme = useMemo(() => 
    createTheme({
      palette: {
        mode,
        primary: {
          main: mode === 'dark' ? '#90caf9' : '#1a237e', // Darker blue for light theme
        },
        secondary: {
          main: mode === 'dark' ? '#64b5f6' : '#1565c0', // More blue, less turquoise
        },
        background: {
          default: mode === 'dark' ? '#0a1929' : '#f5f7fa', // Light gray background for light theme
          paper: mode === 'dark' ? '#132f4c' : '#ffffff', // Pure white for cards in light mode
        },
        divider: mode === 'dark' ? 'rgba(255, 255, 255, 0.15)' : 'rgba(0, 0, 0, 0.12)',
      },
      components: {
        MuiCssBaseline: {
          styleOverrides: {
            '*': {
              // Split transitions into separate properties for different speeds
              transition: `
                background-color 400ms cubic-bezier(0.4, 0, 0.2, 1),
                color 250ms cubic-bezier(0.4, 0, 0.2, 1),
                border-color 400ms cubic-bezier(0.4, 0, 0.2, 1),
                box-shadow 400ms cubic-bezier(0.4, 0, 0.2, 1)
              `
            },
            body: mode === 'dark' ? {
              background: 'linear-gradient(145deg, #0a1929 0%, #0d2b59 50%,rgb(13, 75, 116) 100%)',
              minHeight: '100vh',
              backgroundAttachment: 'fixed',
              transition: 'background 400ms cubic-bezier(0.4, 0, 0.2, 1)'
            } : {
              background: '#f5f7fa',
              minHeight: '100vh',
              backgroundAttachment: 'fixed',
              transition: 'background 400ms cubic-bezier(0.4, 0, 0.2, 1)'
            },
          }
        },
        MuiListItemText: {
          styleOverrides: {
            primary: {
              transition: 'color 200ms cubic-bezier(0.4, 0, 0.2, 1) !important'
            },
            secondary: {
              transition: 'color 200ms cubic-bezier(0.4, 0, 0.2, 1) !important'
            }
          }
        },
        MuiListItemIcon: {
          styleOverrides: {
            root: {
              transition: 'color 200ms cubic-bezier(0.4, 0, 0.2, 1) !important'
            }
          }
        },
        MuiDrawer: {
          styleOverrides: {
            paper: {
              ...(mode === 'dark' ? {
                backgroundColor: 'rgba(19, 47, 76, 0.9)',
                backdropFilter: 'blur(8px)'
              } : {
                backgroundColor: '#ffffff',
                boxShadow: '0 0 20px rgba(0, 0, 0, 0.05)'
              }),
              transition: 'background-color 400ms cubic-bezier(0.4, 0, 0.2, 1), backdrop-filter 400ms cubic-bezier(0.4, 0, 0.2, 1)'
            }
          }
        },
        MuiAppBar: {
          styleOverrides: {
            root: {
              ...(mode === 'dark' ? {
                background: 'linear-gradient(90deg, #0a1929 0%,rgb(13, 48, 89) 100%)',
                boxShadow: '0 4px 20px rgba(0,0,0,0.4)'
              } : {
                background: 'linear-gradient(90deg, #1a237e 0%,rgb(14, 96, 134) 100%)',
                boxShadow: '0 2px 10px rgba(0,0,0,0.1)'
              }),
              transition: 'background 400ms cubic-bezier(0.4, 0, 0.2, 1), box-shadow 400ms cubic-bezier(0.4, 0, 0.2, 1)'
            }
          }
        },
        MuiCard: {
          styleOverrides: {
            root: {
              transition: 'background-color 400ms cubic-bezier(0.4, 0, 0.2, 1), box-shadow 400ms cubic-bezier(0.4, 0, 0.2, 1), border-color 400ms cubic-bezier(0.4, 0, 0.2, 1)',
              ...(mode === 'light' ? {
                boxShadow: '0 2px 12px rgba(0, 0, 0, 0.1)',
                border: '1px solid rgba(0, 0, 0, 0.05)'
              } : {})
            }
          }
        },
        MuiPaper: {
          styleOverrides: {
            root: {
              transition: 'background-color 400ms cubic-bezier(0.4, 0, 0.2, 1), box-shadow 400ms cubic-bezier(0.4, 0, 0.2, 1), border-color 400ms cubic-bezier(0.4, 0, 0.2, 1)',
              ...(mode === 'light' ? {
                boxShadow: '0 2px 12px rgba(0, 0, 0, 0.1)',
                border: '1px solid rgba(0, 0, 0, 0.05)'
              } : {})
            }
          }
        },
        MuiListItemButton: {
          styleOverrides: {
            root: mode === 'light' ? {
              '&.Mui-selected': {
                backgroundColor: 'rgba(26, 35, 126, 0.1)',
                '&:hover': {
                  backgroundColor: 'rgba(26, 35, 126, 0.15)',
                }
              },
              '&:hover': {
                backgroundColor: 'rgba(0, 0, 0, 0.04)',
              }
            } : {}
          }
        }
      },
    }), 
    [mode]
  );

  // Toggle theme function
  const toggleColorMode = () => {
    setMode((prevMode) => (prevMode === 'light' ? 'dark' : 'light'));
  };

  // Toggle drawer
  const handleDrawerToggle = () => {
    setMobileOpen(!mobileOpen);
  };

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }  return (
    <WalletProvider>
      <AppSettingsProvider>
        <ThemeProvider theme={theme}>
          <CssBaseline />
          <BrowserRouter>
            <WalletDialogProvider>
              <AppContentWrapper 
                mode={mode} 
                toggleColorMode={toggleColorMode} 
                mobileOpen={mobileOpen}
                handleDrawerToggle={handleDrawerToggle}
                greetMsg={greetMsg}
                name={name}
                setName={setName}
                greet={greet}
              />
              <OpenCreateWalletDialog />
            </WalletDialogProvider>
          </BrowserRouter>
        </ThemeProvider>
      </AppSettingsProvider>
    </WalletProvider>
  );
}

// Wrapper component that conditionally renders the AppContent
function AppContentWrapper(props: {
  mode: 'light' | 'dark',
  toggleColorMode: () => void,
  mobileOpen: boolean,
  handleDrawerToggle: () => void,
  greetMsg: string,
  name: string,
  setName: (name: string) => void,
  greet: () => void
}) {
  const { isWalletOpen } = useWallet();

  // Only render the AppContent if wallet is open
  return isWalletOpen ? <AppContent {...props} /> : null;
}

// Add interface for AppContent props
interface AppContentProps {
  mode: 'light' | 'dark';
  toggleColorMode: () => void;
  mobileOpen: boolean;
  handleDrawerToggle: () => void;
  greetMsg: string;
  name: string;
  setName: (name: string) => void;
  greet: () => void;
}

// Separate component to use React Router hooks
function AppContent({ mode, toggleColorMode, mobileOpen, handleDrawerToggle, greetMsg, name, setName, greet }: AppContentProps) {
  const location = useLocation();

  return (
    <Box sx={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
      <AppHeader
        mode={mode}
        toggleColorMode={toggleColorMode}
        handleDrawerToggle={handleDrawerToggle}
      />
      
      {/* Mobile drawer */}
      <Drawer
        variant="temporary"
        open={mobileOpen}
        onClose={handleDrawerToggle}
        ModalProps={{
          keepMounted: true, // Better open performance on mobile.
        }}
        sx={{
          display: { xs: 'block', sm: 'none' },
          '& .MuiDrawer-paper': { boxSizing: 'border-box', width: drawerWidth },
        }}
      >
        <Sidebar 
          mode={mode}
          mobileOpen={mobileOpen}
          handleDrawerToggle={handleDrawerToggle}
        />
      </Drawer>
      
      {/* Desktop drawer */}
      <Drawer
        variant="permanent"
        sx={{
          display: { xs: 'none', sm: 'block' },
          width: drawerWidth,
          flexShrink: 0,
          '& .MuiDrawer-paper': { boxSizing: 'border-box', width: drawerWidth },
        }}
        open
      >
        <Sidebar 
          mode={mode}
          mobileOpen={mobileOpen}
          handleDrawerToggle={handleDrawerToggle}
        />
      </Drawer>
      
      {/* Main content */}
      <Box
        component="main"
        sx={{ 
          flexGrow: 1,
          width: { xs: '100%', sm: `calc(100% - ${drawerWidth}px)` },
          mt: '64px',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'auto',
        }}
      >
        <Container 
          disableGutters
          maxWidth={false}
          sx={{ 
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            flexGrow: 1,
            width: '100%',
            '& > *': {
              // Add fade transition for route changes
              animation: 'fadeIn 800ms cubic-bezier(0.4, 0, 0.2, 1)',
              '@keyframes fadeIn': {
                '0%': {
                  opacity: 0,
                  transform: 'translateY(10px)'
                },
                '100%': {
                  opacity: 1,
                  transform: 'translateY(0)'
                }
              }
            }
          }}
        >          <Routes location={location}>            <Route path="/" element={<Account greetMsg={greetMsg} name={name} setName={setName} greet={greet} />} />
            <Route path="/transactions" element={<Transactions />} />
            <Route path="/send-receive" element={<SendReceive />} />
            <Route path="/advanced" element={<Advanced />} />
            <Route path="/developer" element={<Developer />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/about" element={<About />} />
          </Routes>
        </Container>
      </Box>
    </Box>
  );
}

export default App;
