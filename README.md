# RemoteWebTerminal

Small Rust daemon that serves a localhost web page for managing terminal sessions
run by tmux.

## Run

```bash
cargo run
```

Then open:

```text
http://127.0.0.1:8765
```

The daemon binds to `127.0.0.1:8765` by default. Override it with:

```bash
cargo run -- --bind 127.0.0.1:9000
```

## Features

- List tmux sessions, windows, and panes.
- Create and kill tmux sessions.
- Capture pane output.
- Send literal text or named keys such as `C-c`.
- Kill individual panes.

## API

- `GET /api/health`
- `GET /api/sessions`
- `POST /api/sessions`
- `DELETE /api/sessions/:name`
- `GET /api/panes/:pane_id/capture?lines=200`
- `POST /api/panes/:pane_id/send-text`
- `POST /api/panes/:pane_id/send-key`
- `DELETE /api/panes/:pane_id`
