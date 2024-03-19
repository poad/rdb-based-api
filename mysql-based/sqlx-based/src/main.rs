use axum::{
    routing::get,
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};

use sqlx_mysql::MySqlPool;

use std::env;

use dotenvy::dotenv;

use tracing::Level;
use tracing_log::LogTracer;


mod handlers;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .json()
        .with_current_span(false)
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .with_line_number(true)
        .with_file(true)
        .init();

    LogTracer::init().ok();

    let url = env::var("DATABASE_URL").unwrap();
    let pool = MySqlPool::connect(&url).await.unwrap();

    let app = Router::new()
        .route("/search", get(handlers::search))
        .with_state(pool)
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap()
}
