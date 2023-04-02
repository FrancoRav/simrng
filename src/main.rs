use axum::{
    http::{HeaderValue, Method},
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod controllers;

#[tokio::main]
async fn main() {
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
        .route("/api/uniform", post(controllers::get_uniform))
        .route("/api/normal-bm", post(controllers::get_normal_bm))
        .route("/api/normal-conv", post(controllers::get_normal_conv))
        .route("/api/exponential", post(controllers::get_exponential))
        .route("/api/poisson", post(controllers::get_poisson))
        .layer(cors);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to start server");
}
