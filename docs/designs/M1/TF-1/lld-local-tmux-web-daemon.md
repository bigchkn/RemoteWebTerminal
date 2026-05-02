# Design: Local tmux web daemon

Type: lld

## Goal

Build a small localhost-only daemon that serves a web UI for tmux terminal session management.

## Scope

- Rust binary: `remote-web-daemon`.
- Bind default: `127.0.0.1:8765`.
- UI: embedded static HTML/CSS/JS served by the daemon.
- Backend: Axum JSON APIs that call the local `tmux` binary using structured process arguments.

## API

- `GET /api/health`
- `GET /api/sessions`
- `POST /api/sessions`
- `DELETE /api/sessions/:name`
- `GET /api/panes/:pane_id/capture`
- `POST /api/panes/:pane_id/send-text`
- `POST /api/panes/:pane_id/send-key`
- `DELETE /api/panes/:pane_id`

## Behavior

The daemon lists all tmux panes via `tmux list-panes -a`, groups them by session and window, and lets the browser select a pane for periodic capture. Text input is sent with literal `tmux send-keys -l`; control actions use named tmux keys such as `C-c`.

## Non-Goals

- No remote bind by default.
- No authentication in the first local-only version.
- No pseudo-terminal implementation outside tmux.
