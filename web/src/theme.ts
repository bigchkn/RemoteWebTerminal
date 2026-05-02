import { createTheme } from '@mui/material/styles'

export const theme = createTheme({
  palette: {
    mode: 'dark',
    background: { default: '#101214', paper: '#181c20' },
    primary: { main: '#4db6ac' },
    error: { main: '#ef5350' },
    text: { primary: '#edf2f7', secondary: '#9aa8b5' },
    divider: '#33404a',
  },
  typography: {
    fontFamily:
      'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  },
  components: {
    MuiButton: {
      styleOverrides: { root: { textTransform: 'none' } },
    },
    MuiAppBar: {
      styleOverrides: { root: { backgroundImage: 'none' } },
    },
  },
})
