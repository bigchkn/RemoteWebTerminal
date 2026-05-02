use std::{env, io::ErrorKind, process::Command as StdCommand};

use serde::Serialize;
use thiserror::Error;
use tokio::process::Command;
use tracing::debug;

const DEFAULT_TMUX_TMPDIR: &str = "/private/tmp";
const FIELD_SEPARATOR: char = '|';
const PANE_FORMAT: &str = "#{session_name}|#{window_index}|#{window_name}|#{pane_id}|#{pane_current_command}|#{pane_dead}|#{pane_width}|#{pane_height}";

#[derive(Debug, Error)]
pub enum TmuxError {
    #[error("tmux binary not found or not executable: {0}")]
    BinaryNotFound(String),

    #[error("tmux command failed with exit {code}: {stderr}")]
    CommandFailed { code: i32, stderr: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct TmuxClient {
    tmux_bin: String,
    socket_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PaneInfo {
    pub session_name: String,
    pub window_index: u32,
    pub window_name: String,
    pub pane_id: String,
    pub current_command: String,
    pub dead: bool,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WindowInfo {
    pub index: u32,
    pub name: String,
    pub panes: Vec<PaneInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SessionInfo {
    pub name: String,
    pub windows: Vec<WindowInfo>,
}

impl TmuxClient {
    pub fn new(tmux_bin: impl Into<String>, socket_path: Option<String>) -> Self {
        Self {
            tmux_bin: tmux_bin.into(),
            socket_path: socket_path.or_else(default_socket_path),
        }
    }

    pub async fn sessions(&self) -> Result<Vec<SessionInfo>, TmuxError> {
        let session_rows = self
            .run_tmux(&["list-sessions", "-F", "#{session_name}"])
            .await?;
        let mut rows = String::new();

        for session_name in parse_session_names(&session_rows) {
            let target = format!("={session_name}");
            rows.push_str(
                &self
                    .run_tmux(&["list-panes", "-t", &target, "-F", PANE_FORMAT])
                    .await?,
            );
        }

        Ok(parse_panes(&rows))
    }

    pub async fn create_session(&self, name: &str, command: Option<&str>) -> Result<(), TmuxError> {
        let mut args = vec!["new-session", "-d", "-s", name];
        if let Some(command) = command.filter(|command| !command.trim().is_empty()) {
            args.push(command);
        }
        self.run_tmux(&args).await?;
        Ok(())
    }

    pub async fn kill_session(&self, name: &str) -> Result<(), TmuxError> {
        self.run_tmux(&["kill-session", "-t", name]).await?;
        Ok(())
    }

    pub async fn kill_pane(&self, pane_id: &str) -> Result<(), TmuxError> {
        self.run_tmux(&["kill-pane", "-t", pane_id]).await?;
        Ok(())
    }

    pub async fn capture_pane(&self, pane_id: &str, lines: usize) -> Result<String, TmuxError> {
        let start = format!("-{}", lines.clamp(1, 5000));
        self.run_tmux(&["capture-pane", "-t", pane_id, "-p", "-S", &start])
            .await
    }

    pub async fn send_text(&self, pane_id: &str, text: &str, enter: bool) -> Result<(), TmuxError> {
        self.run_tmux(&["send-keys", "-t", pane_id, "-l", text])
            .await?;
        if enter {
            self.send_key(pane_id, "Enter").await?;
        }
        Ok(())
    }

    pub async fn send_key(&self, pane_id: &str, key: &str) -> Result<(), TmuxError> {
        self.run_tmux(&["send-keys", "-t", pane_id, key]).await?;
        Ok(())
    }

    async fn run_tmux(&self, args: &[&str]) -> Result<String, TmuxError> {
        debug!(bin = %self.tmux_bin, ?args, "tmux");
        let mut command = Command::new(&self.tmux_bin);
        command.env("TERM", "xterm-256color");
        command.env("TMUX_TMPDIR", tmux_tmpdir());
        if let Some(socket_path) = &self.socket_path {
            command.arg("-S").arg(socket_path);
        }

        let output = command
            .args(args)
            .output()
            .await
            .map_err(|err| match err.kind() {
                ErrorKind::NotFound | ErrorKind::PermissionDenied => {
                    TmuxError::BinaryNotFound(err.to_string())
                }
                _ => TmuxError::Io(err),
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(TmuxError::CommandFailed {
                code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            })
        }
    }
}

fn tmux_tmpdir() -> String {
    tmux_tmpdir_from(env::var("TMUX_TMPDIR").ok())
}

fn tmux_tmpdir_from(value: Option<String>) -> String {
    value.unwrap_or_else(|| DEFAULT_TMUX_TMPDIR.to_owned())
}

fn default_socket_path() -> Option<String> {
    env::var("TMUX_SOCKET")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(socket_path_from_tmux_env)
        .or_else(|| current_uid().map(|uid| format!("{DEFAULT_TMUX_TMPDIR}/tmux-{uid}/default")))
}

fn socket_path_from_tmux_env() -> Option<String> {
    env::var("TMUX")
        .ok()
        .and_then(|value| value.split(',').next().map(str::to_owned))
        .filter(|value| !value.trim().is_empty())
}

fn current_uid() -> Option<String> {
    let output = StdCommand::new("id").arg("-u").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let uid = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if uid.is_empty() {
        None
    } else {
        Some(uid)
    }
}

fn parse_panes(rows: &str) -> Vec<SessionInfo> {
    let mut sessions: Vec<SessionInfo> = Vec::new();

    for row in rows.lines().filter(|row| !row.trim().is_empty()) {
        let fields: Vec<&str> = row.split(FIELD_SEPARATOR).collect();
        if fields.len() != 8 {
            continue;
        }

        let pane = PaneInfo {
            session_name: fields[0].to_owned(),
            window_index: fields[1].parse().unwrap_or(0),
            window_name: fields[2].to_owned(),
            pane_id: fields[3].to_owned(),
            current_command: fields[4].to_owned(),
            dead: fields[5] == "1",
            width: fields[6].parse().unwrap_or(80),
            height: fields[7].parse().unwrap_or(24),
        };

        let session_index = sessions
            .iter()
            .position(|session| session.name == pane.session_name)
            .unwrap_or_else(|| {
                sessions.push(SessionInfo {
                    name: pane.session_name.clone(),
                    windows: Vec::new(),
                });
                sessions.len() - 1
            });

        let windows = &mut sessions[session_index].windows;
        let window_index = windows
            .iter()
            .position(|window| window.index == pane.window_index)
            .unwrap_or_else(|| {
                windows.push(WindowInfo {
                    index: pane.window_index,
                    name: pane.window_name.clone(),
                    panes: Vec::new(),
                });
                windows.len() - 1
            });

        windows[window_index].panes.push(pane);
    }

    sessions
}

fn parse_session_names(rows: &str) -> Vec<String> {
    rows.lines()
        .map(str::trim)
        .filter(|row| !row.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_panes_groups_sessions_windows_and_panes() {
        let rows =
            "dev|0|zsh|%1|zsh|0|100|32\ndev|1|logs|%2|less|0|80|24\nops|0|bash|%3|bash|1|120|40\n";
        let sessions = parse_panes(rows);

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].name, "dev");
        assert_eq!(sessions[0].windows.len(), 2);
        assert_eq!(sessions[0].windows[0].panes[0].pane_id, "%1");
        assert_eq!(sessions[1].windows[0].panes[0].dead, true);
    }

    #[test]
    fn parse_session_names_ignores_empty_lines() {
        assert_eq!(
            parse_session_names("dev\n\nops\n"),
            vec!["dev".to_owned(), "ops".to_owned()]
        );
    }

    #[test]
    fn tmux_tmpdir_defaults_to_interactive_socket_dir() {
        assert_eq!(tmux_tmpdir_from(None), "/private/tmp");
    }

    #[test]
    fn tmux_tmpdir_preserves_explicit_socket_dir() {
        assert_eq!(
            tmux_tmpdir_from(Some("/custom/tmux".to_owned())),
            "/custom/tmux"
        );
    }

    #[test]
    fn socket_path_uses_tmux_socket_env_first() {
        assert_eq!(
            socket_path_from_values(
                Some("/tmp/custom".to_owned()),
                Some("/tmp/from-tmux,1,2".to_owned()),
                Some("501".to_owned()),
            ),
            Some("/tmp/custom".to_owned())
        );
    }

    #[test]
    fn socket_path_can_parse_tmux_env() {
        assert_eq!(
            socket_path_from_values(None, Some("/tmp/from-tmux,1,2".to_owned()), None),
            Some("/tmp/from-tmux".to_owned())
        );
    }

    #[test]
    fn socket_path_defaults_to_interactive_socket() {
        assert_eq!(
            socket_path_from_values(None, None, Some("501".to_owned())),
            Some("/private/tmp/tmux-501/default".to_owned())
        );
    }

    fn socket_path_from_values(
        tmux_socket: Option<String>,
        tmux: Option<String>,
        uid: Option<String>,
    ) -> Option<String> {
        tmux_socket
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                tmux.and_then(|value| value.split(',').next().map(str::to_owned))
                    .filter(|value| !value.trim().is_empty())
            })
            .or_else(|| uid.map(|uid| format!("{DEFAULT_TMUX_TMPDIR}/tmux-{uid}/default")))
    }
}
