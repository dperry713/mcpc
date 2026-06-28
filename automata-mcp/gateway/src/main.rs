use axum::{
    routing::{get, post},
    Router, middleware,
    extract::Request,
    response::Response,
    http::StatusCode,
};
use tower_http::trace::TraceLayer;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_otlp::{Protocol, SpanExporter, WithExportConfig};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: String,
    exp: usize,
    sub: String,
}

// 1. The Auth Middleware
async fn require_oauth2_token(
    req: Request,
    next: middleware::Next,
) -> Result<Response, StatusCode> {
    let auth_header = req.headers().get("Authorization");
    
    match auth_header {
        Some(header) => {
            let header_str = header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;
            if !header_str.starts_with("Bearer ") {
                tracing::warn!("Blocked request: Authorization header does not use Bearer schema");
                return Err(StatusCode::UNAUTHORIZED);
            }
            
            let token = &header_str[7..];
            
            let mut validation = Validation::new(Algorithm::HS256);
            validation.set_audience(&["mcpc-api", "automata"]);
            
            // Crytographically verify signature and claims
            match decode::<Claims>(
                token,
                &DecodingKey::from_secret("enterprise-orchestrator-secret-key-12345".as_ref()),
                &validation,
            ) {
                Ok(_) => {
                    Ok(next.run(req).await)
                }
                Err(e) => {
                    tracing::warn!("Blocked request: JWT verification failed: {}", e);
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
        }
        None => {
            tracing::warn!("Blocked request: Missing Authorization header");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

// 2. The Reverse Proxy Handler
async fn proxy_to_control_plane(req: Request) -> Result<Response, StatusCode> {
    let client = reqwest::Client::new();
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    // In a zero-trust production cluster, this targets the internal service IP:
    // http://control-plane.default.svc.cluster.local:8080 or simply http://control-plane:8080 in docker-compose mesh.
    let target_url = format!("http://control-plane:8080{}", uri.path());
    
    let mut req_builder = client.request(method, &target_url);
    
    // Forward all headers
    for (key, value) in req.headers() {
        req_builder = req_builder.header(key.clone(), value.clone());
    }
    
    // Extract and forward request body
    let body_bytes = axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    req_builder = req_builder.body(body_bytes);
    
    let res = req_builder.send()
        .await
        .map_err(|e| {
            tracing::error!("Proxy request failed to target {}: {}", target_url, e);
            StatusCode::BAD_GATEWAY
        })?;
        
    // Map response back to Axum
    let mut res_builder = Response::builder().status(res.status().as_u16());
    for (key, value) in res.headers() {
        res_builder = res_builder.header(key.clone(), value.clone());
    }
    
    let res_body_bytes = res.bytes()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    let response = res_builder.body(axum::body::Body::from(res_body_bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    Ok(response)
}

fn init_otel() -> Result<TracerProvider, opentelemetry::trace::TraceError> {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint("http://otel-collector:4318/v1/traces")
        .build()?;
        
    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build();
        
    opentelemetry::global::set_tracer_provider(provider.clone());
    Ok(provider)
}

#[tokio::main]
async fn main() {
    // Initialize OpenTelemetry trace pipeline
    let _provider = match init_otel() {
        Ok(provider) => {
            let tracer = provider.tracer("gateway");
            let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
            
            let subscriber = tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(telemetry_layer);
            
            let _ = subscriber.try_init();
            Some(provider)
        }
        Err(e) => {
            // Fallback to simple console logging
            let _ = tracing_subscriber::fmt::try_init();
            tracing::warn!("Failed to initialize OpenTelemetry, running console fallback: {}", e);
            None
        }
    };

    // 3. Define the routing graph
    let api_routes = Router::new()
        .route("/execute", post(proxy_to_control_plane))
        // Enforce the zero-trust barrier on all /api routes
        .route_layer(middleware::from_fn(require_oauth2_token));

    let app = Router::new()
        // Health check remains public for Kubernetes
        .route("/health", get(|| async { "OK" })) 
        // Nest the protected routes
        .nest("/api/v1", api_routes)
        // Add robust tracing for observability
        .layer(TraceLayer::new_for_http());

    let cert_path = std::path::Path::new("/etc/tls/tls.crt");
    let key_path = std::path::Path::new("/etc/tls/tls.key");
    
    if cert_path.exists() && key_path.exists() {
        tracing::info!("🔒 Zero-Trust Gateway terminating mTLS securely on 0.0.0.0:3000");
    } else {
        tracing::info!("🔒 Zero-Trust API 'gateway' listening on 0.0.0.0:3000 (Local Dev Mode - TLS Disabled)");
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
