use std::io::ErrorKind;

use serde::Serialize;
use thiserror::Error;
use tokio::process::Command;
use tracing::debug;

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
    pub fn with_binary(tmux_bin: impl Into<String>) -> Self {
        Self {
            tmux_bin: tmux_bin.into(),
        }
    }

    pub async fn sessions(&self) -> Result<Vec<SessionInfo>, TmuxError> {
        let rows = self
            .run_tmux(&[
                "list-panes",
                "-a",
                "-F",
                "#{session_name}\t#{window_index}\t#{window_name}\t#{pane_id}\t#{pane_current_command}\t#{pane_dead}\t#{pane_width}\t#{pane_height}",
            ])
            .await?;
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
        let output = Command::new(&self.tmux_bin)
            .env("TERM", "xterm-256color")
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

fn parse_panes(rows: &str) -> Vec<SessionInfo> {
    let mut sessions: Vec<SessionInfo> = Vec::new();

    for row in rows.lines().filter(|row| !row.trim().is_empty()) {
        let fields: Vec<&str> = row.split('\t').collect();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_panes_groups_sessions_windows_and_panes() {
        let rows = "dev\t0\tzsh\t%1\tzsh\t0\t100\t32\ndev\t1\tlogs\t%2\tless\t0\t80\t24\nops\t0\tbash\t%3\tbash\t1\t120\t40\n";
        let sessions = parse_panes(rows);

        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].name, "dev");
        assert_eq!(sessions[0].windows.len(), 2);
        assert_eq!(sessions[0].windows[0].panes[0].pane_id, "%1");
        assert_eq!(sessions[1].windows[0].panes[0].dead, true);
    }
}
