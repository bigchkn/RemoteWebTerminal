export interface PaneInfo {
  pane_id: string
  current_command: string
  dead: boolean
  session_name: string
  window_index: number
  width: number
  height: number
}

export interface WindowInfo {
  index: number
  name: string
  panes: PaneInfo[]
}

export interface SessionInfo {
  name: string
  windows: WindowInfo[]
}

async function request(path: string, options: RequestInit = {}) {
  const res = await fetch(path, {
    headers: { 'content-type': 'application/json' },
    ...options,
  })
  if (!res.ok) {
    let message = `${res.status} ${res.statusText}`
    try {
      const body = await res.json()
      message = body.error ?? message
    } catch {
      // use HTTP status message
    }
    throw new Error(message)
  }
  if (res.status === 204 || res.status === 201) return null
  return res.json()
}

const enc = (v: string) => encodeURIComponent(v)

export const api = {
  sessions: (): Promise<SessionInfo[]> => request('/api/sessions'),

  createSession: (name: string, command?: string) =>
    request('/api/sessions', {
      method: 'POST',
      body: JSON.stringify({ name, command: command ?? null }),
    }),

  killSession: (name: string) =>
    request(`/api/sessions/${enc(name)}`, { method: 'DELETE' }),

  capturePane: (paneId: string, lines = 240): Promise<{ pane_id: string; output: string }> =>
    request(`/api/panes/${enc(paneId)}/capture?lines=${lines}`),

  sendText: (paneId: string, text: string, enter: boolean) =>
    request(`/api/panes/${enc(paneId)}/send-text`, {
      method: 'POST',
      body: JSON.stringify({ text, enter }),
    }),

  sendKey: (paneId: string, key: string) =>
    request(`/api/panes/${enc(paneId)}/send-key`, {
      method: 'POST',
      body: JSON.stringify({ key }),
    }),

  killPane: (paneId: string) =>
    request(`/api/panes/${enc(paneId)}`, { method: 'DELETE' }),
}
