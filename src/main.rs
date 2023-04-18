use std::sync::Arc;

use crate::controllers::Generated;
use axum::{http::Method, routing::post, Router};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod controllers;

#[tokio::main]
async fn main() {
    // Guarda el último Vec generado y su distribución
    // Necesario para calcular estadísticas
    let last: Arc<RwLock<Generated>> = Arc::new(RwLock::new(Generated::default()));

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "api=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Permitir peticiones desde el puerto del web server, aceptando
    // cualquier header
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods([Method::GET, Method::POST]);

    // Configurar rutas con sus métodos, CORS y estado
    let app = Router::new()
        .route("/api/generate", post(controllers::get_unified))
        .route("/api/statistics", post(controllers::get_statistics))
        .route("/api/page", post(controllers::get_page_numbers))
        .layer(cors)
        .with_state(last);

    let port;
    if let Ok(n) = std::env::var("SIMRNG_PORT") {
        port = n.parse().unwrap_or(3000);
    } else {
        port = 3000;
    }
    // Crear servidor e iniciar en puerto 3000
    tracing::debug!("Listening on {}", addr);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to start server");
}
