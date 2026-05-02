mod routes;
mod service;
mod tmux;

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use clap::{Parser, Subcommand};
use routes::AppState;
use service::{InstallOptions, LaunchdService};
use tmux::TmuxClient;
use tracing::info;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Address to bind the localhost web daemon to.
    #[arg(
        long,
        env = "REMOTE_WEB_BIND",
        default_value = "127.0.0.1:8765",
        global = true
    )]
    bind: SocketAddr,

    /// tmux binary to execute.
    #[arg(long, env = "TMUX_BIN", default_value = "tmux", global = true)]
    tmux_bin: String,

    /// tmux server socket path. Defaults to the standard interactive user socket.
    #[arg(long, env = "TMUX_SOCKET", global = true)]
    tmux_socket: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the localhost web daemon.
    Serve,
    /// Install a user-level launchd service.
    Install {
        /// Binary path launchd should run. Defaults to the current executable.
        #[arg(long)]
        bin_path: Option<PathBuf>,
    },
    /// Uninstall the user-level launchd service and remove its plist.
    Uninstall,
    /// Start or restart the installed launchd service.
    Start,
    /// Stop the installed launchd service.
    Stop,
    /// Print launchd status for the installed service.
    Status,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "remote_web_daemon=info,tower_http=info".into()),
        )
        .init();

    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => serve(cli.bind, cli.tmux_bin, cli.tmux_socket).await?,
        Command::Install { bin_path } => {
            let service = LaunchdService::default_for_user()?;
            let bin_path = bin_path.unwrap_or(std::env::current_exe()?);
            let tmux_socket = cli
                .tmux_socket
                .or_else(|| service.default_tmux_socket_path().ok());
            service.install(&InstallOptions {
                bind: cli.bind,
                tmux_bin: cli.tmux_bin,
                tmux_socket,
                bin_path: bin_path.canonicalize().unwrap_or(bin_path),
            })?;
            println!("installed {}", service.plist_path().display());
        }
        Command::Uninstall => {
            let service = LaunchdService::default_for_user()?;
            service.uninstall()?;
            println!("uninstalled {}", service.label());
        }
        Command::Start => {
            let service = LaunchdService::default_for_user()?;
            service.start()?;
            println!("started {}", service.label());
        }
        Command::Stop => {
            let service = LaunchdService::default_for_user()?;
            service.stop()?;
            println!("stopped {}", service.label());
        }
        Command::Status => {
            let service = LaunchdService::default_for_user()?;
            print!("{}", service.status()?);
        }
    }

    Ok(())
}

async fn serve(
    bind: SocketAddr,
    tmux_bin: String,
    tmux_socket: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = app(TmuxClient::new(tmux_bin, tmux_socket));
    let listener = tokio::net::TcpListener::bind(bind).await?;

    info!("serving tmux web UI at http://{}", bind);
    axum::serve(listener, app).await?;
    Ok(())
}

fn app(tmux: TmuxClient) -> Router {
    routes::router(AppState::new(tmux))
}
