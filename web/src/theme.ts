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
      styleOverrides: {
        root: {
          textTransform: 'none',
          // 44px minimum per Apple HIG / Android touch target guidelines
          minHeight: 44,
          minWidth: 44,
          '@media (min-width: 900px)': {
            minHeight: 34,
          },
        },
        sizeSmall: {
          minHeight: 44,
          '@media (min-width: 900px)': {
            minHeight: 34,
          },
        },
      },
    },
    MuiIconButton: {
      styleOverrides: {
        root: {
          minHeight: 44,
          minWidth: 44,
          '@media (min-width: 900px)': {
            minHeight: 36,
            minWidth: 36,
          },
        },
      },
    },
    MuiCheckbox: {
      styleOverrides: {
        root: {
          padding: 10,
          '@media (min-width: 900px)': {
            padding: 4,
          },
        },
      },
    },
    MuiTextField: {
      defaultProps: { size: 'small' },
      styleOverrides: {
        root: {
          '& .MuiInputBase-root': {
            minHeight: 44,
            fontSize: '1rem',
            '@media (min-width: 900px)': {
              minHeight: 34,
              fontSize: '0.875rem',
            },
          },
        },
      },
    },
    MuiAppBar: {
      styleOverrides: { root: { backgroundImage: 'none' } },
    },
    MuiBottomNavigation: {
      styleOverrides: {
        root: {
          // account for home indicator on iOS/Android
          paddingBottom: 'env(safe-area-inset-bottom)',
          height: 'calc(56px + env(safe-area-inset-bottom))',
        },
      },
    },
  },
})
