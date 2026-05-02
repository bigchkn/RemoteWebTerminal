# Design: Daemon installer

Type: lld

## Goal

Provide an installer workflow that runs `remote-web-daemon` as a persistent local service.

## Scope

- Install and uninstall commands or scripts for the service.
- Generate a user-level service definition.
- Prefer macOS `launchd` first because the project is currently developed on macOS.
- Document service lifecycle commands for install, start, stop, status, and uninstall.

## Acceptance Criteria

- A fresh checkout can install the daemon without manually writing service files.
- The installed service binds to localhost by default.
- The installer does not require elevated privileges for the default user-level install.
- The README documents the supported installer commands.
