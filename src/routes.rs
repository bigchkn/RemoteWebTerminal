use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};

use crate::tmux::{SessionInfo, TmuxClient, TmuxError};

static DIST: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/web/dist");

#[derive(Clone)]
pub struct AppState {
    tmux: Arc<TmuxClient>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    name: String,
    command: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CaptureQuery {
    lines: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SendTextRequest {
    text: String,
    enter: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SendKeyRequest {
    key: String,
}

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
}

#[derive(Debug, Serialize)]
struct CaptureResponse {
    pane_id: String,
    output: String,
}

impl AppState {
    pub fn new(tmux: TmuxClient) -> Self {
        Self {
            tmux: Arc::new(tmux),
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/health", get(health))
        .route("/api/sessions", get(list_sessions).post(create_session))
        .route("/api/sessions/:name", delete(kill_session))
        .route("/api/panes/:pane_id", delete(kill_pane))
        .route("/api/panes/:pane_id/capture", get(capture_pane))
        .route("/api/panes/:pane_id/send-text", post(send_text))
        .route("/api/panes/:pane_id/send-key", post(send_key))
        .route("/{*path}", get(static_asset))
        .with_state(state)
}

async fn index() -> Response {
    serve_dist_file("index.html")
}

async fn static_asset(Path(path): Path<String>) -> Response {
    serve_dist_file(&path)
}

fn serve_dist_file(path: &str) -> Response {
    match DIST.get_file(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(header::CONTENT_TYPE, mime.to_string())],
                Bytes::from_static(file.contents()),
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<SessionInfo>>, ApiFailure> {
    Ok(Json(state.tmux.sessions().await?))
}

async fn create_session(
    State(state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<StatusCode, ApiFailure> {
    let name = request.name.trim();
    if name.is_empty() {
        return Err(ApiFailure::bad_request("session name is required"));
    }

    state
        .tmux
        .create_session(name, request.command.as_deref())
        .await?;
    Ok(StatusCode::CREATED)
}

async fn kill_session(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiFailure> {
    state.tmux.kill_session(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn kill_pane(
    State(state): State<AppState>,
    Path(pane_id): Path<String>,
) -> Result<StatusCode, ApiFailure> {
    state.tmux.kill_pane(&pane_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn capture_pane(
    State(state): State<AppState>,
    Path(pane_id): Path<String>,
    Query(query): Query<CaptureQuery>,
) -> Result<Json<CaptureResponse>, ApiFailure> {
    let output = state
        .tmux
        .capture_pane(&pane_id, query.lines.unwrap_or(200))
        .await?;
    Ok(Json(CaptureResponse { pane_id, output }))
}

async fn send_text(
    State(state): State<AppState>,
    Path(pane_id): Path<String>,
    Json(request): Json<SendTextRequest>,
) -> Result<StatusCode, ApiFailure> {
    state
        .tmux
        .send_text(&pane_id, &request.text, request.enter.unwrap_or(true))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn send_key(
    State(state): State<AppState>,
    Path(pane_id): Path<String>,
    Json(request): Json<SendKeyRequest>,
) -> Result<StatusCode, ApiFailure> {
    state.tmux.send_key(&pane_id, &request.key).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug)]
pub struct ApiFailure {
    status: StatusCode,
    message: String,
}

impl ApiFailure {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }
}

impl From<TmuxError> for ApiFailure {
    fn from(value: TmuxError) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            message: value.to_string(),
        }
    }
}

impl IntoResponse for ApiFailure {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiError {
                error: self.message,
            }),
        )
            .into_response()
    }
}
