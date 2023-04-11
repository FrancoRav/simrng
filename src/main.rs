use std::sync::Arc;

use crate::controllers::Generated;
use axum::{http::Method, routing::post, Router};
use simrng::dist::Uniform;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod controllers;

#[tokio::main]
async fn main() {
    let last: Arc<Mutex<Generated>> = Arc::new(Mutex::new(Generated::new(
        vec![],
        Box::new(Uniform {
            lower: 10f64,
            upper: 11f64,
        }),
    )));

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "api=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = CorsLayer::new()
        .allow_origin([
            "http://127.0.0.1:5173".parse().unwrap(),
            "http://localhost:5173".parse().unwrap(),
        ])
        .allow_headers(Any)
        .allow_methods([Method::GET, Method::POST]);

    let app = Router::new()
        .route("/api/generate", post(controllers::get_unified))
        .route("/api/histogram", post(controllers::get_histogram))
        .route("/api/chisquared", post(controllers::get_chisquared))
        .layer(cors)
        .with_state(last);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to start server");
}
