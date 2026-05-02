const state = {
  sessions: [],
  selectedPane: null,
  poll: null,
};

const els = {
  status: document.querySelector("#status"),
  sessions: document.querySelector("#sessions"),
  refresh: document.querySelector("#refresh"),
  newSessionForm: document.querySelector("#new-session-form"),
  sessionName: document.querySelector("#session-name"),
  sessionCommand: document.querySelector("#session-command"),
  paneTitle: document.querySelector("#pane-title"),
  paneMeta: document.querySelector("#pane-meta"),
  terminalOutput: document.querySelector("#terminal-output"),
  sendForm: document.querySelector("#send-form"),
  sendText: document.querySelector("#send-text"),
  sendEnter: document.querySelector("#send-enter"),
  sendButton: document.querySelector("#send-button"),
  interrupt: document.querySelector("#interrupt"),
  killPane: document.querySelector("#kill-pane"),
};

async function api(path, options = {}) {
  const response = await fetch(path, {
    headers: { "content-type": "application/json" },
    ...options,
  });

  if (!response.ok) {
    let message = `${response.status} ${response.statusText}`;
    try {
      const body = await response.json();
      message = body.error ?? message;
    } catch {}
    throw new Error(message);
  }

  if (response.status === 204 || response.status === 201) return null;
  return response.json();
}

function encodePath(value) {
  return encodeURIComponent(value);
}

async function refreshSessions() {
  try {
    state.sessions = await api("/api/sessions");
    els.status.textContent = `${state.sessions.length} tmux session${state.sessions.length === 1 ? "" : "s"}`;
    renderSessions();

    if (!state.selectedPane) {
      const firstPane = state.sessions[0]?.windows[0]?.panes[0];
      if (firstPane) selectPane(firstPane);
    } else {
      await refreshCapture();
    }
  } catch (error) {
    els.status.textContent = error.message;
  }
}

function renderSessions() {
  els.sessions.replaceChildren();

  if (state.sessions.length === 0) {
    const empty = document.createElement("p");
    empty.className = "empty";
    empty.textContent = "No tmux sessions found.";
    els.sessions.append(empty);
    return;
  }

  for (const session of state.sessions) {
    const group = document.createElement("section");
    group.className = "session-group";

    const header = document.createElement("div");
    header.className = "session-header";

    const title = document.createElement("strong");
    title.textContent = session.name;

    const kill = document.createElement("button");
    kill.type = "button";
    kill.textContent = "Kill";
    kill.addEventListener("click", async () => {
      if (!confirm(`Kill tmux session "${session.name}"?`)) return;
      await api(`/api/sessions/${encodePath(session.name)}`, { method: "DELETE" });
      if (state.selectedPane?.session_name === session.name) clearSelection();
      await refreshSessions();
    });

    header.append(title, kill);
    group.append(header);

    for (const windowInfo of session.windows) {
      const windowEl = document.createElement("div");
      windowEl.className = "window-row";
      windowEl.textContent = `${windowInfo.index}: ${windowInfo.name}`;
      group.append(windowEl);

      for (const pane of windowInfo.panes) {
        const button = document.createElement("button");
        button.type = "button";
        button.className = "pane-row";
        if (state.selectedPane?.pane_id === pane.pane_id) button.classList.add("selected");
        button.textContent = `${pane.pane_id} ${pane.current_command}${pane.dead ? " (dead)" : ""}`;
        button.addEventListener("click", () => selectPane(pane));
        group.append(button);
      }
    }

    els.sessions.append(group);
  }
}

async function selectPane(pane) {
  state.selectedPane = pane;
  els.paneTitle.textContent = pane.pane_id;
  els.paneMeta.textContent = `${pane.session_name}:${pane.window_index} · ${pane.current_command} · ${pane.width}x${pane.height}`;
  els.sendText.disabled = false;
  els.sendButton.disabled = false;
  els.interrupt.disabled = false;
  els.killPane.disabled = false;
  renderSessions();
  await refreshCapture();
}

function clearSelection() {
  state.selectedPane = null;
  els.paneTitle.textContent = "No pane selected";
  els.paneMeta.textContent = "";
  els.terminalOutput.textContent = "";
  els.sendText.disabled = true;
  els.sendButton.disabled = true;
  els.interrupt.disabled = true;
  els.killPane.disabled = true;
}

async function refreshCapture() {
  if (!state.selectedPane) return;
  const paneId = state.selectedPane.pane_id;
  try {
    const capture = await api(`/api/panes/${encodePath(paneId)}/capture?lines=240`);
    if (state.selectedPane?.pane_id !== paneId) return;
    els.terminalOutput.textContent = capture.output;
    els.terminalOutput.scrollTop = els.terminalOutput.scrollHeight;
  } catch (error) {
    els.terminalOutput.textContent = error.message;
  }
}

els.refresh.addEventListener("click", refreshSessions);

els.newSessionForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  const name = els.sessionName.value.trim();
  if (!name) return;

  await api("/api/sessions", {
    method: "POST",
    body: JSON.stringify({
      name,
      command: els.sessionCommand.value.trim() || null,
    }),
  });

  els.sessionName.value = "";
  els.sessionCommand.value = "";
  await refreshSessions();
});

els.sendForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!state.selectedPane || !els.sendText.value) return;

  await api(`/api/panes/${encodePath(state.selectedPane.pane_id)}/send-text`, {
    method: "POST",
    body: JSON.stringify({
      text: els.sendText.value,
      enter: els.sendEnter.checked,
    }),
  });

  els.sendText.value = "";
  await refreshCapture();
});

els.interrupt.addEventListener("click", async () => {
  if (!state.selectedPane) return;
  await api(`/api/panes/${encodePath(state.selectedPane.pane_id)}/send-key`, {
    method: "POST",
    body: JSON.stringify({ key: "C-c" }),
  });
  await refreshCapture();
});

els.killPane.addEventListener("click", async () => {
  if (!state.selectedPane) return;
  if (!confirm(`Kill pane ${state.selectedPane.pane_id}?`)) return;
  await api(`/api/panes/${encodePath(state.selectedPane.pane_id)}`, { method: "DELETE" });
  clearSelection();
  await refreshSessions();
});

refreshSessions();
state.poll = window.setInterval(refreshSessions, 3000);
