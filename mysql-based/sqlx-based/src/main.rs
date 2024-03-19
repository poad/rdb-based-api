use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use hyper::StatusCode;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};

use sqlx::FromRow;
use sqlx_mysql::MySqlPool;

use std::env;

use dotenvy::dotenv;

use serde::{Deserialize, Serialize};

use tracing_subscriber;
use tracing_log::LogTracer;
use tracing::Level;

use log::{ trace, error };

#[derive(Deserialize)]
struct QueryRequest {
    query: String,
}

#[derive(Debug, FromRow, Serialize)]
struct Record {
    pk: i32,
    message: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt().json()
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
        .route("/search", get(search))
        .with_state(pool)
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new()
                    .level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new()
                    .level(Level::INFO)),
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

async fn search(
    Query(request): Query<QueryRequest>,
    State(pool): State<MySqlPool>,
) -> impl IntoResponse {
    let query = request.query;
    let result = sqlx::query_as::<_, Record>(query.as_str())
        .fetch_all(&pool)
        .await;

    match result {
        Err(err) => {
            error!("{:?}", err);
            Err(StatusCode::FORBIDDEN)
        },
        Ok(records) => {
            trace!("{:?}", records);
            Ok(Json(records))
        }
    }

    
}
