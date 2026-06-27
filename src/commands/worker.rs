use crate::errors::McpcError;
use crate::schema::Module;
use crate::generator;
use axum::{
    routing::post,
    Json, Router,
};
use std::net::SocketAddr;
use tokio::runtime::Runtime;
use clap::Parser;

#[derive(Parser)]
pub struct WorkerArgs {
    #[arg(short, long, default_value = "50051")]
    pub port: u16,
}

pub fn run_worker(args: WorkerArgs) -> Result<(), McpcError> {
    let port = args.port;
    let rt = Runtime::new().map_err(McpcError::Io)?;
    
    rt.block_on(async {
        tracing::info!("[mcpc] Starting remote builder on port {}", port);
        
        let app = Router::new().route("/build", post(handle_build));
        
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    Ok(())
}

async fn handle_build(Json(module): Json<Module>) -> Json<generator::ModuleOutput> {
    tracing::info!("[mcpc-worker] received build request for module '{}'", module.name);
    let output = generator::generate_module(&module).unwrap_or_default();
    Json(output)
}
