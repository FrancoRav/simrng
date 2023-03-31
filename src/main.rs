use axum::{routing::get, Router};
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

    let app = Router::new().route("/api/uniform", get(controllers::get_uniform));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to start server");
}
