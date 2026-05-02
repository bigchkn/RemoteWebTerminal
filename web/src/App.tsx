import { useState, useEffect, useRef, useCallback } from 'react'
import {
  ThemeProvider,
  CssBaseline,
  Box,
  AppBar,
  Toolbar,
  Typography,
  Button,
  TextField,
  Paper,
  IconButton,
  Stack,
  Checkbox,
  FormControlLabel,
  Collapse,
  useMediaQuery,
} from '@mui/material'
import RefreshIcon from '@mui/icons-material/Refresh'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore'
import ExpandLessIcon from '@mui/icons-material/ExpandLess'
import { theme } from './theme'
import { api, type SessionInfo, type PaneInfo } from './api'

export default function App() {
  const [sessions, setSessions] = useState<SessionInfo[]>([])
  const [selectedPane, setSelectedPane] = useState<PaneInfo | null>(null)
  const [output, setOutput] = useState('')
  const [status, setStatus] = useState('Connecting to localhost daemon...')
  const [sessionName, setSessionName] = useState('')
  const [sessionCmd, setSessionCmd] = useState('')
  const [sendText, setSendText] = useState('')
  const [sendEnter, setSendEnter] = useState(true)
  const [sidebarOpen, setSidebarOpen] = useState(true)
  const isMobile = useMediaQuery(theme.breakpoints.down('md'))
  const outputRef = useRef<HTMLPreElement>(null)
  const selectedPaneRef = useRef<PaneInfo | null>(null)
  selectedPaneRef.current = selectedPane

  const doRefresh = useCallback(async () => {
    try {
      const data = await api.sessions()
      setSessions(data)
      setStatus(`${data.length} tmux session${data.length === 1 ? '' : 's'}`)
      const pane = selectedPaneRef.current
      if (!pane) {
        const first = data[0]?.windows[0]?.panes[0]
        if (first) {
          setSelectedPane(first)
          const capture = await api.capturePane(first.pane_id)
          setOutput(capture.output)
        }
      } else {
        const capture = await api.capturePane(pane.pane_id)
        setOutput(capture.output)
      }
    } catch (e) {
      setStatus((e as Error).message)
    }
  }, [])

  useEffect(() => {
    doRefresh()
    const id = setInterval(doRefresh, 3000)
    return () => clearInterval(id)
  }, [doRefresh])

  useEffect(() => {
    if (outputRef.current) {
      outputRef.current.scrollTop = outputRef.current.scrollHeight
    }
  }, [output])

  const handleSelectPane = async (pane: PaneInfo) => {
    setSelectedPane(pane)
    try {
      const capture = await api.capturePane(pane.pane_id)
      setOutput(capture.output)
    } catch (e) {
      setOutput((e as Error).message)
    }
  }

  const handleKillSession = async (name: string) => {
    if (!confirm(`Kill session "${name}"?`)) return
    await api.killSession(name)
    if (selectedPaneRef.current?.session_name === name) {
      setSelectedPane(null)
      setOutput('')
    }
    await doRefresh()
  }

  const handleKillPane = async () => {
    const pane = selectedPaneRef.current
    if (!pane) return
    if (!confirm(`Kill pane ${pane.pane_id}?`)) return
    await api.killPane(pane.pane_id)
    setSelectedPane(null)
    setOutput('')
    await doRefresh()
  }

  const handleNewSession = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!sessionName.trim()) return
    await api.createSession(sessionName.trim(), sessionCmd.trim() || undefined)
    setSessionName('')
    setSessionCmd('')
    await doRefresh()
  }

  const handleSend = async (e: React.FormEvent) => {
    e.preventDefault()
    const pane = selectedPaneRef.current
    if (!pane || !sendText) return
    await api.sendText(pane.pane_id, sendText, sendEnter)
    setSendText('')
    const capture = await api.capturePane(pane.pane_id)
    setOutput(capture.output)
  }

  const handleInterrupt = async () => {
    const pane = selectedPaneRef.current
    if (!pane) return
    await api.sendKey(pane.pane_id, 'C-c')
    const capture = await api.capturePane(pane.pane_id)
    setOutput(capture.output)
  }

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box sx={{ minHeight: '100dvh', display: 'flex', flexDirection: 'column' }}>
        <AppBar
          position="static"
          elevation={0}
          sx={{ bgcolor: 'background.paper', borderBottom: '1px solid', borderColor: 'divider' }}
        >
          <Toolbar sx={{ gap: 2, flexWrap: 'wrap', py: 1, minHeight: 'unset !important' }}>
            <Box>
              <Typography variant="h6" component="h1" sx={{ fontWeight: 700, fontSize: '1.1rem', lineHeight: 1.2 }}>
                Remote Web Terminal
              </Typography>
              <Typography variant="caption" color="text.secondary">
                {status}
              </Typography>
            </Box>
            <Box
              component="form"
              onSubmit={handleNewSession}
              sx={{ display: 'flex', gap: 1, flexWrap: 'wrap', ml: 'auto', alignItems: 'center' }}
            >
              <TextField
                size="small"
                placeholder="session name"
                value={sessionName}
                onChange={e => setSessionName(e.target.value)}
                autoComplete="off"
                sx={{ width: 160 }}
              />
              <TextField
                size="small"
                placeholder="command (optional)"
                value={sessionCmd}
                onChange={e => setSessionCmd(e.target.value)}
                autoComplete="off"
                sx={{ width: 200 }}
              />
              <Button type="submit" variant="outlined" size="small">
                New Session
              </Button>
              <IconButton onClick={doRefresh} size="small" color="inherit" title="Refresh">
                <RefreshIcon fontSize="small" />
              </IconButton>
            </Box>
          </Toolbar>
        </AppBar>

        <Box
          sx={{
            flex: 1,
            display: 'flex',
            minHeight: 0,
            flexDirection: { xs: 'column', md: 'row' },
          }}
        >
          {/* Sidebar */}
          <Box
            sx={{
              width: { md: 340 },
              borderRight: { md: '1px solid' },
              borderBottom: { xs: '1px solid', md: 'none' },
              borderColor: 'divider',
              bgcolor: '#121619',
              flexShrink: 0,
            }}
          >
            {/* Mobile toggle header */}
            {isMobile && (
              <Box
                onClick={() => setSidebarOpen(o => !o)}
                sx={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  px: 2,
                  py: 1,
                  cursor: 'pointer',
                  userSelect: 'none',
                }}
              >
                <Typography variant="body2" fontWeight={600}>
                  Sessions
                  {sessions.length > 0 && (
                    <Typography component="span" variant="caption" color="text.secondary" sx={{ ml: 1 }}>
                      ({sessions.length})
                    </Typography>
                  )}
                </Typography>
                {sidebarOpen ? <ExpandLessIcon fontSize="small" /> : <ExpandMoreIcon fontSize="small" />}
              </Box>
            )}

            <Collapse in={!isMobile || sidebarOpen}>
              <Box
                sx={{
                  maxHeight: { xs: '40vh', md: 'none' },
                  overflow: 'auto',
                  p: 2,
                }}
              >
                <Stack spacing={1.5}>
                  {sessions.length === 0 ? (
                    <Typography color="text.secondary" variant="body2" sx={{ p: 1 }}>
                      No tmux sessions found.
                    </Typography>
                  ) : (
                    sessions.map(session => (
                      <Paper key={session.name} variant="outlined" sx={{ overflow: 'hidden' }}>
                        <Box
                          sx={{
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'space-between',
                            px: 1.5,
                            py: 1,
                            bgcolor: 'background.paper',
                          }}
                        >
                          <Typography variant="body2" fontWeight={600}>
                            {session.name}
                          </Typography>
                          <Button
                            size="small"
                            color="error"
                            onClick={() => handleKillSession(session.name)}
                          >
                            Kill
                          </Button>
                        </Box>
                        {session.windows.map(win => (
                          <Box key={win.index}>
                            <Typography
                              variant="caption"
                              color="text.secondary"
                              sx={{ px: 1.5, py: 0.5, display: 'block' }}
                            >
                              {win.index}: {win.name}
                            </Typography>
                            {win.panes.map(pane => (
                              <Button
                                key={pane.pane_id}
                                fullWidth
                                size="small"
                                variant="outlined"
                                onClick={() => {
                                  handleSelectPane(pane)
                                  if (isMobile) setSidebarOpen(false)
                                }}
                                sx={{
                                  mx: 1,
                                  mb: 1,
                                  width: 'calc(100% - 16px)',
                                  justifyContent: 'flex-start',
                                  borderColor:
                                    selectedPane?.pane_id === pane.pane_id
                                      ? 'primary.main'
                                      : 'divider',
                                  color:
                                    selectedPane?.pane_id === pane.pane_id
                                      ? 'primary.main'
                                      : 'text.primary',
                                }}
                              >
                                {pane.pane_id} {pane.current_command}
                                {pane.dead ? ' (dead)' : ''}
                              </Button>
                            ))}
                          </Box>
                        ))}
                      </Paper>
                    ))
                  )}
                </Stack>
              </Box>
            </Collapse>
          </Box>

          {/* Terminal panel */}
          <Box
            sx={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0, minHeight: 0 }}
          >
            <Box
              sx={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                px: 2,
                py: 1.5,
                borderBottom: '1px solid',
                borderColor: 'divider',
              }}
            >
              <Box>
                <Typography variant="body2" fontWeight={600}>
                  {selectedPane ? selectedPane.pane_id : 'No pane selected'}
                </Typography>
                {selectedPane && (
                  <Typography variant="caption" color="text.secondary">
                    {selectedPane.session_name}:{selectedPane.window_index} ·{' '}
                    {selectedPane.current_command} · {selectedPane.width}×{selectedPane.height}
                  </Typography>
                )}
              </Box>
              <Stack direction="row" spacing={1}>
                <Button
                  size="small"
                  variant="outlined"
                  disabled={!selectedPane}
                  onClick={handleInterrupt}
                >
                  Ctrl-C
                </Button>
                <Button
                  size="small"
                  variant="outlined"
                  color="error"
                  disabled={!selectedPane}
                  onClick={handleKillPane}
                >
                  Kill Pane
                </Button>
              </Stack>
            </Box>

            <Box
              component="pre"
              ref={outputRef}
              sx={{
                flex: 1,
                m: 0,
                p: 2,
                overflow: 'auto',
                bgcolor: '#060708',
                color: '#d7f7df',
                fontFamily:
                  'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace',
                fontSize: { xs: '0.9375rem', md: '0.8125rem' },
                lineHeight: 1.5,
                whiteSpace: 'pre-wrap',
                minHeight: 200,
              }}
            >
              {output}
            </Box>

            <Box
              component="form"
              onSubmit={handleSend}
              sx={{
                display: 'flex',
                alignItems: 'center',
                gap: 1,
                px: 2,
                py: 1.5,
                borderTop: '1px solid',
                borderColor: 'divider',
                bgcolor: 'background.paper',
              }}
            >
              <TextField
                size="small"
                fullWidth
                placeholder="type input for selected pane"
                value={sendText}
                onChange={e => setSendText(e.target.value)}
                disabled={!selectedPane}
                autoComplete="off"
              />
              <FormControlLabel
                control={
                  <Checkbox
                    size="small"
                    checked={sendEnter}
                    onChange={e => setSendEnter(e.target.checked)}
                  />
                }
                label={<Typography variant="caption">Enter</Typography>}
                sx={{ m: 0, whiteSpace: 'nowrap' }}
              />
              <Button
                type="submit"
                variant="contained"
                size="small"
                disabled={!selectedPane}
              >
                Send
              </Button>
            </Box>
          </Box>
        </Box>
      </Box>
    </ThemeProvider>
  )
}
