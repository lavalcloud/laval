mod config;
mod entity;
mod error;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{http::Method, web, App, HttpResponse, HttpServer};
use anyhow::{Error, Result};
use clap::Parser;
use config::{ManagerState, NodeRecord};
use error::{AppError, AppResult};
use laval_model::{PortMappingMode, PortMappingSpec};
use laval_proto::manager::v1::{
    node_manager_server::{NodeManager, NodeManagerServer},
    GetNodeConfigRequest, GetNodeConfigResponse, PortMappingConfig as ProtoPortMappingConfig,
    PortMappingMode as ProtoPortMappingMode,
};
use tonic::{async_trait, transport::Server, Request, Response, Status};
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

type SharedState = Arc<ManagerState>;

#[derive(Clone)]
struct GrpcService {
    state: SharedState,
}

#[async_trait]
impl NodeManager for GrpcService {
    async fn get_node_config(
        &self,
        request: Request<GetNodeConfigRequest>,
    ) -> Result<Response<GetNodeConfigResponse>, Status> {
        let name = request.into_inner().name;
        let record = self
            .state
            .get(&name)
            .await
            .map_err(|err| Status::internal(format!("failed to fetch node '{name}': {err}")))?
            .ok_or_else(|| Status::not_found(format!("node '{name}' not found")))?;

        let port_mapping = match record.port_mapping.as_ref() {
            Some(spec) => Some(port_mapping_to_proto(spec).map_err(|err| {
                Status::internal(format!("failed to serialize port mapping: {err}"))
            })?),
            None => None,
        };

        Ok(Response::new(GetNodeConfigResponse {
            name: record.name,
            port_mapping,
        }))
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Laval node management service", long_about = None)]
struct Cli {
    /// Path to the manager configuration file (TOML format)
    #[arg(long, default_value = "manager.toml")]
    config: PathBuf,
    /// Address to bind the HTTP API server
    #[arg(long, default_value = "0.0.0.0:8080")]
    bind: SocketAddr,
    /// Address to bind the gRPC server
    #[arg(long, default_value = "0.0.0.0:50051")]
    grpc_bind: SocketAddr,
    /// Database connection string
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();
    let Cli {
        config,
        bind,
        grpc_bind,
        database_url,
    } = cli;

    let state = Arc::new(ManagerState::initialize(config, database_url).await?);

    let http_state = state.clone();
    let http_server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(http_state.clone()))
            .route("/health", web::get().to(health))
            .service(
                web::scope("/nodes")
                    .route("", web::get().to(list_nodes))
                    .route("", web::post().to(create_node))
                    .route("/{name}", web::get().to(get_node))
                    .route("/{name}", web::put().to(update_node))
                    .route("/{name}", web::delete().to(delete_node)),
            )
    })
    .bind(bind)?
    .run();

    let grpc_state = state.clone();

    let grpc_server = async move {
        info!(bind = %grpc_bind, "starting manager gRPC API");
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::POST])
            .allow_headers(Any);

        Server::builder()
            .accept_http1(true)
            .layer(cors)
            .layer(GrpcWebLayer::new())
            .add_service(tonic_web::enable(NodeManagerServer::new(GrpcService {
                state: grpc_state,
            })))
            .serve(grpc_bind)
            .await?;
        Ok::<(), Error>(())
    };

    let http_server = async move {
        info!(bind = %bind, "starting manager HTTP API");
        http_server.await?;
        Ok::<(), Error>(())
    };

    tokio::try_join!(http_server, grpc_server)?;

    Ok(())
}

fn port_mapping_to_proto(
    spec: &PortMappingSpec,
) -> Result<ProtoPortMappingConfig, serde_json::Error> {
    let mode = match spec.mode {
        PortMappingMode::Server => ProtoPortMappingMode::Server,
        PortMappingMode::Client => ProtoPortMappingMode::Client,
    } as i32;

    let config_json = serde_json::to_string(&spec.config)?;

    Ok(ProtoPortMappingConfig { mode, config_json })
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().body("ok")
}

async fn list_nodes(state: web::Data<SharedState>) -> AppResult<web::Json<Vec<NodeRecord>>> {
    let nodes = state.list().await.map_err(AppError::from)?;
    Ok(web::Json(nodes))
}

async fn get_node(
    name: web::Path<String>,
    state: web::Data<SharedState>,
) -> AppResult<web::Json<NodeRecord>> {
    let name = name.into_inner();
    match state.get(&name).await.map_err(AppError::from)? {
        Some(node) => Ok(web::Json(node)),
        None => Err(AppError::not_found(format!("node '{name}' not found"))),
    }
}

async fn create_node(
    state: web::Data<SharedState>,
    payload: web::Json<NodeRecord>,
) -> AppResult<HttpResponse> {
    let mut payload = payload.into_inner();
    validate_name(&payload.name)?;
    payload.name = payload.name.trim().to_string();
    state
        .upsert(payload.clone())
        .await
        .map_err(AppError::from)?;
    Ok(HttpResponse::Created().json(payload))
}

async fn update_node(
    name: web::Path<String>,
    state: web::Data<SharedState>,
    payload: web::Json<NodeRecord>,
) -> AppResult<HttpResponse> {
    let name = name.into_inner();
    let mut payload = payload.into_inner();
    validate_name(&name)?;
    payload.name = name.trim().to_string();
    state
        .upsert(payload.clone())
        .await
        .map_err(AppError::from)?;
    Ok(HttpResponse::Ok().json(payload))
}

async fn delete_node(
    name: web::Path<String>,
    state: web::Data<SharedState>,
) -> AppResult<HttpResponse> {
    let name = name.into_inner();
    if state.remove(name.trim()).await.map_err(AppError::from)? {
        Ok(HttpResponse::NoContent().finish())
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
