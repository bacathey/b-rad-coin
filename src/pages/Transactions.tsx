// filepath: c:\Users\bacat\source\repos\b-rad-coin\src\pages\Transactions.tsx
import { 
  Typography, 
  Box, 
  Card, 
  CardContent, 
  Paper,
  useTheme,
  Stack,
  Tabs,
  Tab,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow
} from '@mui/material';
// Use the standard Grid component 
import { Grid } from '@mui/material';
import { useState } from 'react';
import {
  ResponsiveContainer,
  ComposedChart,
  Line,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  PieChart,
  Pie,
  Cell
} from 'recharts';

// Add Material UI icons for Transactions cards
import AssessmentIcon from '@mui/icons-material/Assessment';
import NotificationsIcon from '@mui/icons-material/Notifications';
import TimelineIcon from '@mui/icons-material/Timeline';
import BarChartIcon from '@mui/icons-material/BarChart';
import PieChartIcon from '@mui/icons-material/PieChart';

// Sample data for the charts
const monthlyData = [
  { name: 'Jan', incoming: 4000, outgoing: 2400, total: 1600 },
  { name: 'Feb', incoming: 3000, outgoing: 1398, total: 1602 },
  { name: 'Mar', incoming: 2000, outgoing: 9800, total: -7800 },
  { name: 'Apr', incoming: 2780, outgoing: 3908, total: -1128 },
  { name: 'May', incoming: 1890, outgoing: 4800, total: -2910 },
  { name: 'Jun', incoming: 2390, outgoing: 3800, total: -1410 },
  { name: 'Jul', incoming: 3490, outgoing: 4300, total: -810 },
];

const pieData = [
  { name: 'Received', value: 15550 },
  { name: 'Sent', value: 27006 },
  { name: 'Fees', value: 2400 },
  { name: 'Mining Rewards', value: 5000 }
];

// Sample data for the tables
const pendingTransactions = [
  { txid: '3a1b2c...8f9e0d', time: '10:23 AM', amount: '-0.5 BTC', confirmations: 0 },
  { txid: '7e6d5c...1a2b3c', time: '09:15 AM', amount: '+1.2 BTC', confirmations: 1 },
  { txid: '9f8e7d...4c5b6a', time: 'Yesterday', amount: '-0.05 BTC', confirmations: 2 }
];

const transactionSummary = [
  { period: 'Today', sent: '0.55 BTC', received: '1.2 BTC', fees: '0.003 BTC' },
  { period: 'This Week', sent: '2.34 BTC', received: '1.89 BTC', fees: '0.012 BTC' },
  { period: 'This Month', sent: '5.67 BTC', received: '3.45 BTC', fees: '0.028 BTC' }
];

// Custom colors for pie chart
const COLORS = ['#0088FE', '#FF8042', '#FFBB28', '#00C49F'];

export default function Transactions() {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';
  const [chartTab, setChartTab] = useState(0);
  
  // Enhanced card styles for light mode
  const cardStyle = isDarkMode ? {
    background: 'rgba(19, 47, 76, 0.6)',
    backdropFilter: 'blur(10px)',
    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
    border: '1px solid rgba(255, 255, 255, 0.1)'
  } : {
    background: 'linear-gradient(180deg, #ffffff 0%, #f5f7fa 100%)',
    boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
    border: '1px solid rgba(0, 0, 0, 0.08)',
    transition: 'transform 0.2s ease-in-out',
    '&:hover': {
      transform: 'translateY(-4px)',
      boxShadow: '0 6px 25px rgba(0, 0, 0, 0.2)',
    }
  };

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setChartTab(newValue);
  };  return (
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
          fontWeight: 600
        }}
      >
        Transactions
      </Typography>
      
      <Grid container spacing={3} sx={{ width: '100%', maxWidth: 1200, mx: 'auto', alignItems: "flex-start", mt: 1 }}>{/* Left column - contains the Pending Transactions and Transaction Summary cards */}
        <Grid item xs={12} md={6}>
          {/* Stack the two cards vertically */}
          <Stack spacing={3}>            {/* Pending Transactions Card */}
            <Card sx={{
              ...cardStyle,
              display: 'flex',
              flexDirection: 'column'
            }}>
              <CardContent>
                <Stack direction="row" spacing={1} alignItems="center" mb={1}>
                  <NotificationsIcon 
                    sx={{ 
                      color: isDarkMode ? '#ffb74d' : '#ed6c02',
                      fontSize: 28
                    }} 
                  />
                  <Typography 
                    variant="h6" 
                    sx={{
                      color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#ed6c02',
                      fontWeight: 600
                    }}
                  >
                    Pending Transactions
                  </Typography>
                </Stack>
                <Typography 
                  variant="body2" 
                  color={isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)'}
                  sx={{ mb: 2 }}
                >
                  Transactions that are currently pending confirmation on the network.
                </Typography>                {/* Pending Transactions Table */}
                <TableContainer sx={{ 
                  height: 200,
                  background: isDarkMode ? 'rgba(10, 25, 41, 0.5)' : 'rgba(0, 0, 0, 0.02)',
                  borderRadius: 1
                }}>
                  <Table size="small" stickyHeader>
                    <TableHead>
                      <TableRow>
                        <TableCell sx={{ 
                          color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)',
                          fontWeight: 600,
                          background: isDarkMode ? 'rgba(19, 47, 76, 0.8)' : '#f5f7fa',
                          borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)'
                        }}>Transaction ID</TableCell>
                        <TableCell sx={{ 
                          color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)',
                          fontWeight: 600,
                          background: isDarkMode ? 'rgba(19, 47, 76, 0.8)' : '#f5f7fa',
                          borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)'
                        }}>Time</TableCell>
                        <TableCell sx={{ 
                          color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)',
                          fontWeight: 600,
                          background: isDarkMode ? 'rgba(19, 47, 76, 0.8)' : '#f5f7fa',
                          borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)'
                        }} align="right">Amount</TableCell>
                        <TableCell sx={{ 
                          color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)',
                          fontWeight: 600,
                          background: isDarkMode ? 'rgba(19, 47, 76, 0.8)' : '#f5f7fa',
                          borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)'
                        }} align="right">Confirmations</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {pendingTransactions.map((tx, index) => (
                        <TableRow key={index} sx={{ 
                          '&:last-child td, &:last-child th': { border: 0 },
                          background: isDarkMode ? 'transparent' : 'white',
                          '&:hover': {
                            background: isDarkMode ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.02)'
                          }
                        }}>
                          <TableCell component="th" scope="row" sx={{ 
                            color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : 'rgba(0, 0, 0, 0.87)',
                            borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)' 
                          }}>
                            {tx.txid}
                          </TableCell>
                          <TableCell sx={{ 
                            color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)',
                            borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)' 
                          }}>
                            {tx.time}
                          </TableCell>
                          <TableCell sx={{ 
                            color: tx.amount.charAt(0) === '+' ? 
                              (isDarkMode ? '#81c784' : '#2e7d32') : 
                              (isDarkMode ? '#ff8a65' : '#c62828'),
                            fontWeight: 600,
                            borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)'
                          }} align="right">
                            {tx.amount}
                          </TableCell>
                          <TableCell sx={{ 
                            color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.6)',
                            borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)' 
                          }} align="right">
                            {tx.confirmations}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
              </CardContent>
            </Card>            {/* Transaction Summary Card */}
            <Card sx={{
              ...cardStyle,
              display: 'flex',
              flexDirection: 'column'
            }}>
              <CardContent>
                <Stack direction="row" spacing={1} alignItems="center" mb={1}>
                  <AssessmentIcon 
                    sx={{ 
                      color: isDarkMode ? 'rgba(144, 202, 249, 0.9)' : '#1a237e',
                      fontSize: 28
                    }} 
                  />
                  <Typography 
                    variant="h6" 
                    sx={{
                      color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#1a237e',
                      fontWeight: 600
                    }}
                  >
                    Transaction Summary
                  </Typography>
                </Stack>
                <Typography 
                  variant="body2" 
                  color={isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)'}
                  sx={{ mb: 2 }}
                >
                  Overview of your Bitcoin transaction history and balances.
                </Typography>                {/* Transaction Summary Table */}
                <TableContainer sx={{ 
                  height: 200,
                  background: isDarkMode ? 'rgba(10, 25, 41, 0.5)' : 'rgba(0, 0, 0, 0.02)',
                  borderRadius: 1
                }}>
                  <Table size="small" stickyHeader>
                    <TableHead>
                      <TableRow>
                        <TableCell sx={{ 
                          color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)',
                          fontWeight: 600,
                          background: isDarkMode ? 'rgba(19, 47, 76, 0.8)' : '#f5f7fa',
                          borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)'
                        }}>Period</TableCell>
                        <TableCell sx={{ 
                          color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)',
                          fontWeight: 600,
                          background: isDarkMode ? 'rgba(19, 47, 76, 0.8)' : '#f5f7fa',
                          borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)'
                        }} align="right">Sent</TableCell>
                        <TableCell sx={{ 
                          color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)',
                          fontWeight: 600,
                          background: isDarkMode ? 'rgba(19, 47, 76, 0.8)' : '#f5f7fa',
                          borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)'
                        }} align="right">Received</TableCell>
                        <TableCell sx={{ 
                          color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)',
                          fontWeight: 600,
                          background: isDarkMode ? 'rgba(19, 47, 76, 0.8)' : '#f5f7fa',
                          borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)'
                        }} align="right">Fees</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {transactionSummary.map((summary, index) => (
                        <TableRow key={index} sx={{ 
                          '&:last-child td, &:last-child th': { border: 0 },
                          background: isDarkMode ? 'transparent' : 'white',
                          '&:hover': {
                            background: isDarkMode ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.02)'
                          }
                        }}>
                          <TableCell component="th" scope="row" sx={{ 
                            color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : 'rgba(0, 0, 0, 0.87)',
                            fontWeight: 500,
                            borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)' 
                          }}>
                            {summary.period}
                          </TableCell>
                          <TableCell sx={{ 
                            color: isDarkMode ? '#ff8a65' : '#c62828',
                            fontWeight: 500,
                            borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)' 
                          }} align="right">
                            {summary.sent}
                          </TableCell>
                          <TableCell sx={{ 
                            color: isDarkMode ? '#81c784' : '#2e7d32',
                            fontWeight: 500,
                            borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)'
                          }} align="right">
                            {summary.received}
                          </TableCell>
                          <TableCell sx={{ 
                            color: isDarkMode ? 'rgba(255, 255, 255, 0.6)' : 'rgba(0, 0, 0, 0.6)',
                            borderBottom: isDarkMode ? '1px solid rgba(255, 255, 255, 0.1)' : '1px solid rgba(0, 0, 0, 0.1)' 
                          }} align="right">
                            {summary.fees}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
              </CardContent>
            </Card>
          </Stack>        </Grid>
          {/* Right column - Transaction Analytics */}
        <Grid item xs={12} md={6}>          <Paper sx={{ 
            p: 2,
            display: 'flex',
            flexDirection: 'column',
            ...(isDarkMode ? {
              background: 'rgba(19, 47, 76, 0.6)',
              backdropFilter: 'blur(10px)',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
              border: '1px solid rgba(255, 255, 255, 0.1)'
            } : {
              background: 'linear-gradient(180deg, #ffffff 0%, #f5f7fa 100%)',
              boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
              border: '1px solid rgba(0, 0, 0, 0.08)',
              borderRadius: '8px',
              transition: 'transform 0.2s ease-in-out',
              '&:hover': {
                transform: 'translateY(-4px)',
                boxShadow: '0 6px 25px rgba(0, 0, 0, 0.2)',
              }
            })
          }}>
            <Stack direction="row" spacing={1} alignItems="center" mb={1}>
              <TimelineIcon 
                sx={{ 
                  color: isDarkMode ? '#4fc3f7' : '#0277bd',
                  fontSize: 28
                }} 
              />
              <Typography 
                variant="h6" 
                sx={{
                  color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#0277bd',
                  fontWeight: 600
                }}
              >
                Transaction Analytics
              </Typography>
            </Stack>
              <Tabs
              value={chartTab}
              onChange={handleTabChange}
              variant="fullWidth"
              sx={{
                mt: 1,
                mb: 2,
                '& .MuiTabs-indicator': {
                  backgroundColor: isDarkMode ? '#90caf9' : '#1a237e',
                },
                '& .MuiTab-root': {
                  color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)',
                  fontWeight: 500,
                  '&.Mui-selected': {
                    color: isDarkMode ? 'rgba(255, 255, 255, 0.9)' : '#1a237e',
                    fontWeight: 600
                  }
                }
              }}
            >
              <Tab icon={<BarChartIcon />} label="Monthly Activity" iconPosition="start" />
              <Tab icon={<PieChartIcon />} label="Transaction Types" iconPosition="start" />
            </Tabs>            <Box sx={{ 
              height: 240,
              background: isDarkMode ? 'rgba(10, 25, 41, 0.5)' : 'rgba(26, 35, 126, 0.03)',
              borderRadius: 1,
              p: 2,
              border: isDarkMode ? 'none' : '1px solid rgba(0, 0, 0, 0.05)'
            }}>
              {chartTab === 0 ? (
                <ResponsiveContainer width="100%" height="100%">
                  <ComposedChart
                    data={monthlyData}
                    margin={{
                      top: 20,
                      right: 30,
                      left: 20,
                      bottom: 20,
                    }}
                  >
                    <CartesianGrid strokeDasharray="3 3" stroke={isDarkMode ? "rgba(255,255,255,0.1)" : "rgba(0,0,0,0.1)"} />
                    <XAxis 
                      dataKey="name" 
                      tick={{ fill: isDarkMode ? 'rgba(255,255,255,0.7)' : 'rgba(0,0,0,0.7)' }}
                    />
                    <YAxis 
                      tick={{ fill: isDarkMode ? 'rgba(255,255,255,0.7)' : 'rgba(0,0,0,0.7)' }}
                    />
                    <Tooltip 
                      contentStyle={{
                        backgroundColor: isDarkMode ? 'rgba(19, 47, 76, 0.9)' : '#fff',
                        borderColor: isDarkMode ? 'rgba(255, 255, 255, 0.2)' : '#ccc',
                        color: isDarkMode ? 'white' : 'black',
                      }}
                    />
                    <Legend 
                      wrapperStyle={{ 
                        color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)'
                      }} 
                    />
                    <Bar dataKey="incoming" name="Incoming (BTC)" barSize={20} fill={isDarkMode ? "#8884d8" : "#3f51b5"} />
                    <Bar dataKey="outgoing" name="Outgoing (BTC)" barSize={20} fill={isDarkMode ? "#ff7300" : "#f44336"} />
                    <Line type="monotone" dataKey="total" name="Net Flow (BTC)" stroke={isDarkMode ? "#64b5f6" : "#1565c0"} strokeWidth={2} />
                  </ComposedChart>
                </ResponsiveContainer>
              ) : (
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={pieData}
                      cx="50%"
                      cy="50%"
                      labelLine={false}
                      outerRadius={100}
                      fill="#8884d8"
                      dataKey="value"
                      label={({ name, percent }) => `${name}: ${(percent * 100).toFixed(0)}%`}
                    >
                      {pieData.map((_entry, index) => (
                        <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip 
                      contentStyle={{
                        backgroundColor: isDarkMode ? 'rgba(19, 47, 76, 0.9)' : '#fff',
                        borderColor: isDarkMode ? 'rgba(255, 255, 255, 0.2)' : '#ccc',
                        color: isDarkMode ? 'white' : 'black',
                      }}
                      formatter={(value: any) => [`${value} BTC`, 'Amount']}
                    />
                    <Legend 
                      wrapperStyle={{ 
                        color: isDarkMode ? 'rgba(255, 255, 255, 0.7)' : 'rgba(0, 0, 0, 0.7)'
                      }} 
                    />
                  </PieChart>
                </ResponsiveContainer>
              )}
            </Box>
          </Paper>
        </Grid>
      </Grid>
    </Box>
  );
}