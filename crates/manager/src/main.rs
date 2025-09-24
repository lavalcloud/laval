mod config;
mod error;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{serve, Json, Router};
use clap::Parser;
use config::{ManagerState, NodeRecord};
use error::{AppError, AppResult};
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

type SharedState = Arc<ManagerState>;

#[derive(Parser, Debug)]
#[command(author, version, about = "Laval node management service", long_about = None)]
struct Cli {
    /// Path to the manager configuration file (TOML format)
    #[arg(long, default_value = "manager.toml")]
    config: PathBuf,
    /// Address to bind the HTTP API server
    #[arg(long, default_value = "0.0.0.0:8080")]
    bind: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();
    let state = Arc::new(ManagerState::initialize(cli.config.clone()).await?);

    let app = Router::new()
        .route("/health", get(health))
        .route("/nodes", get(list_nodes).post(create_node))
        .route(
            "/nodes/:name",
            get(get_node).put(update_node).delete(delete_node),
        )
        .with_state(state);

    info!(bind = %cli.bind, "starting manager API");
    let listener = TcpListener::bind(cli.bind).await?;
    serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

async fn list_nodes(State(state): State<SharedState>) -> AppResult<Json<Vec<NodeRecord>>> {
    Ok(Json(state.list()))
}

async fn get_node(
    Path(name): Path<String>,
    State(state): State<SharedState>,
) -> AppResult<Json<NodeRecord>> {
    match state.get(&name) {
        Some(node) => Ok(Json(node)),
        None => Err(AppError::not_found(format!("node '{name}' not found"))),
    }
}

async fn create_node(
    State(state): State<SharedState>,
    Json(mut payload): Json<NodeRecord>,
) -> AppResult<(StatusCode, Json<NodeRecord>)> {
    validate_name(&payload.name)?;
    payload.name = payload.name.trim().to_string();
    state.upsert(payload.clone()).await?;
    Ok((StatusCode::CREATED, Json(payload)))
}

async fn update_node(
    Path(name): Path<String>,
    State(state): State<SharedState>,
    Json(mut payload): Json<NodeRecord>,
) -> AppResult<Json<NodeRecord>> {
    validate_name(&name)?;
    payload.name = name.trim().to_string();
    state.upsert(payload.clone()).await?;
    Ok(Json(payload))
}

async fn delete_node(
    Path(name): Path<String>,
    State(state): State<SharedState>,
) -> AppResult<StatusCode> {
    if state.remove(name.trim()).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found("node not found"))
    }
}

fn validate_name(name: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        Err(AppError::bad_request("node name cannot be empty"))
    } else {
        Ok(())
    }
}
