# Project Roadmap: New Project



## Active Milestones

### Local tmux web daemon (M1)
**Status:** Todo

- [x] **TF-1**: Build localhost daemon for tmux session management(`Done`)
  - [Lld] docs/designs/M1/TF-1/lld-local-tmux-web-daemon.md (`Approved`)

### Daemon installer (M2)
**Status:** Todo

- [x] **TF-2**: Create installer for running remote-web-daemon as a local service(`Done`)
  - [Lld] docs/designs/M2/TF-2/lld-daemon-installer.md (`Approved`)
  - [x] **TF-3**: Generate and manage user launchd plist (Parent: TF-2)(`Done`)
  - [x] **TF-4**: Add installer lifecycle tests (Parent: TF-2)(`Done`)
  - [x] **TF-5**: Document installer usage in README (Parent: TF-2)(`Done`)
  - [x] **TF-6**: Add service installer CLI commands (Parent: TF-2)(`Done`)
- [x] **TF-7**: Make launchd install idempotent when service is not loaded(`Done`)
- [x] **TF-8**: Show existing tmux sessions from installed daemon(`Done`)

### Mobile usability (M3)
**Status:** Todo

- [x] **TF-9**: Add collapsible session sidebar toggle for mobile(`Done`)
- [x] **TF-10**: Increase touch target sizes and font scaling on mobile(`Done`)
- [x] **TF-11**: Add tab-bar navigation between sessions and terminal on mobile(`Done`)
- [x] **TF-12**: Handle safe-area insets and virtual keyboard layout on mobile(`Done`)
- [x] **TF-13**: Improve send-input UX for mobile keyboard(`Done`)
- [x] **TF-14**: Integrate Material UI component library for consistent mobile-first UI(`Done`)

## Backlog

_Backlog is empty._

