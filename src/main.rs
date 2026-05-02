mod routes;
mod tmux;

use std::net::SocketAddr;

use axum::Router;
use clap::Parser;
use routes::AppState;
use tmux::TmuxClient;
use tracing::info;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Address to bind the localhost web daemon to.
    #[arg(long, env = "REMOTE_WEB_BIND", default_value = "127.0.0.1:8765")]
    bind: SocketAddr,

    /// tmux binary to execute.
    #[arg(long, env = "TMUX_BIN", default_value = "tmux")]
    tmux_bin: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "remote_web_daemon=info,tower_http=info".into()),
        )
        .init();

    let args = Args::parse();
    let app = app(TmuxClient::with_binary(args.tmux_bin));
    let listener = tokio::net::TcpListener::bind(args.bind).await?;

    info!("serving tmux web UI at http://{}", args.bind);
    axum::serve(listener, app).await?;
    Ok(())
}

fn app(tmux: TmuxClient) -> Router {
    routes::router(AppState::new(tmux))
}
