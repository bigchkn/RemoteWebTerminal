use std::{
    env,
    ffi::OsStr,
    fs, io,
    net::SocketAddr,
    path::{Path, PathBuf},
    process::Command,
};

use thiserror::Error;

const LABEL: &str = "com.remotewebterminal.daemon";
const LOG_DIR: &str = "Library/Logs/RemoteWebTerminal";
const PLIST_DIR: &str = "Library/LaunchAgents";
const PLIST_FILE: &str = "com.remotewebterminal.daemon.plist";

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("HOME is not set; cannot install a user launchd service")]
    MissingHome,

    #[error("failed to determine current user id: {0}")]
    MissingUid(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("{program} failed with exit {code}: {stderr}")]
    CommandFailed {
        program: String,
        code: i32,
        stderr: String,
    },
}

pub struct InstallOptions {
    pub bind: SocketAddr,
    pub tmux_bin: String,
    pub bin_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LaunchdService {
    label: String,
    plist_path: PathBuf,
    log_dir: PathBuf,
}

impl LaunchdService {
    pub fn default_for_user() -> Result<Self, ServiceError> {
        let home = PathBuf::from(env::var_os("HOME").ok_or(ServiceError::MissingHome)?);
        Ok(Self::new(home))
    }

    fn new(home: PathBuf) -> Self {
        Self {
            label: LABEL.to_owned(),
            plist_path: home.join(PLIST_DIR).join(PLIST_FILE),
            log_dir: home.join(LOG_DIR),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn plist_path(&self) -> &Path {
        &self.plist_path
    }

    pub fn install(&self, options: &InstallOptions) -> Result<(), ServiceError> {
        fs::create_dir_all(
            self.plist_path
                .parent()
                .expect("plist path always has a parent directory"),
        )?;
        fs::create_dir_all(&self.log_dir)?;

        fs::write(self.plist_path(), self.render_plist(options))?;
        self.bootout().or_else(ignore_not_loaded)?;
        self.start()?;
        Ok(())
    }

    pub fn uninstall(&self) -> Result<(), ServiceError> {
        let bootout_result = self.bootout();
        if self.plist_path.exists() {
            fs::remove_file(self.plist_path())?;
        }
        bootout_result.or_else(ignore_not_loaded)
    }

    pub fn start(&self) -> Result<(), ServiceError> {
        self.bootstrap()?;
        let target = self.service_target()?;
        self.run_launchctl([
            OsStr::new("kickstart"),
            OsStr::new("-k"),
            OsStr::new(&target),
        ])
        .map(|_| ())
    }

    pub fn stop(&self) -> Result<(), ServiceError> {
        self.bootout().or_else(ignore_not_loaded)
    }

    pub fn status(&self) -> Result<String, ServiceError> {
        let target = self.service_target()?;
        self.run_launchctl([OsStr::new("print"), OsStr::new(&target)])
    }

    fn bootstrap(&self) -> Result<(), ServiceError> {
        let domain = self.user_domain()?;
        self.run_launchctl([
            OsStr::new("bootstrap"),
            OsStr::new(&domain),
            self.plist_path.as_os_str(),
        ])
        .map(|_| ())
        .or_else(ignore_already_loaded)
    }

    fn bootout(&self) -> Result<(), ServiceError> {
        let domain = self.user_domain()?;
        self.run_launchctl([
            OsStr::new("bootout"),
            OsStr::new(&domain),
            self.plist_path.as_os_str(),
        ])
        .map(|_| ())
    }

    fn render_plist(&self, options: &InstallOptions) -> String {
        let stdout_path = self.log_dir.join("stdout.log");
        let stderr_path = self.log_dir.join("stderr.log");
        let path = env::var("PATH").unwrap_or_else(|_| {
            "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin".to_owned()
        });

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{bin}</string>
    <string>--bind</string>
    <string>{bind}</string>
    <string>--tmux-bin</string>
    <string>{tmux_bin}</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>PATH</key>
    <string>{path}</string>
  </dict>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>{stdout}</string>
  <key>StandardErrorPath</key>
  <string>{stderr}</string>
</dict>
</plist>
"#,
            label = xml_escape(&self.label),
            bin = xml_escape(&options.bin_path.display().to_string()),
            bind = xml_escape(&options.bind.to_string()),
            tmux_bin = xml_escape(&options.tmux_bin),
            path = xml_escape(&path),
            stdout = xml_escape(&stdout_path.display().to_string()),
            stderr = xml_escape(&stderr_path.display().to_string()),
        )
    }

    fn user_domain(&self) -> Result<String, ServiceError> {
        Ok(format!("gui/{}", current_uid()?))
    }

    fn service_target(&self) -> Result<String, ServiceError> {
        Ok(format!("{}/{}", self.user_domain()?, self.label))
    }

    fn run_launchctl<I, S>(&self, args: I) -> Result<String, ServiceError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new("launchctl").args(args).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(ServiceError::CommandFailed {
                program: "launchctl".to_owned(),
                code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            })
        }
    }
}

fn current_uid() -> Result<String, ServiceError> {
    let output = Command::new("id").arg("-u").output()?;
    if output.status.success() {
        let uid = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if uid.is_empty() {
            Err(ServiceError::MissingUid(
                "id -u returned no output".to_owned(),
            ))
        } else {
            Ok(uid)
        }
    } else {
        Err(ServiceError::MissingUid(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ))
    }
}

fn ignore_not_loaded(error: ServiceError) -> Result<(), ServiceError> {
    match error {
        ServiceError::CommandFailed { stderr, .. }
            if stderr.contains("No such process") || stderr.contains("not bootstrapped") =>
        {
            Ok(())
        }
        other => Err(other),
    }
}

fn ignore_already_loaded(error: ServiceError) -> Result<(), ServiceError> {
    match error {
        ServiceError::CommandFailed { stderr, .. }
            if stderr.contains("Bootstrap failed") && stderr.contains("Input/output error") =>
        {
            Ok(())
        }
        other => Err(other),
    }
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_contains_launchd_service_definition() {
        let service = LaunchdService::new(PathBuf::from("/Users/example"));
        let options = InstallOptions {
            bind: "127.0.0.1:9999".parse().unwrap(),
            tmux_bin: "/opt/homebrew/bin/tmux".into(),
            bin_path: PathBuf::from("/Applications/Remote Web/remote-web-daemon"),
        };

        let plist = service.render_plist(&options);

        assert!(plist.contains("<string>com.remotewebterminal.daemon</string>"));
        assert!(plist.contains("<string>/Applications/Remote Web/remote-web-daemon</string>"));
        assert!(plist.contains("<string>127.0.0.1:9999</string>"));
        assert!(plist.contains("<string>/opt/homebrew/bin/tmux</string>"));
        assert!(plist.contains("<key>RunAtLoad</key>"));
        assert!(plist.contains("<key>KeepAlive</key>"));
        assert!(plist
            .contains("<string>/Users/example/Library/Logs/RemoteWebTerminal/stdout.log</string>"));
    }

    #[test]
    fn plist_escapes_xml_values() {
        let service = LaunchdService::new(PathBuf::from("/Users/example"));
        let options = InstallOptions {
            bind: "127.0.0.1:8765".parse().unwrap(),
            tmux_bin: "tmux&helper".into(),
            bin_path: PathBuf::from("/tmp/remote<web>\"daemon\""),
        };

        let plist = service.render_plist(&options);

        assert!(plist.contains("/tmp/remote&lt;web&gt;&quot;daemon&quot;"));
        assert!(plist.contains("tmux&amp;helper"));
    }

    #[test]
    fn user_paths_are_under_home() {
        let service = LaunchdService::new(PathBuf::from("/Users/example"));

        assert_eq!(
            service.plist_path(),
            Path::new("/Users/example/Library/LaunchAgents/com.remotewebterminal.daemon.plist")
        );
    }
}
