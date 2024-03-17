use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
};

use sqlx::FromRow;
use sqlx_mysql::MySqlPool;

use std::env;

use dotenvy::dotenv;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct QueryRequest {
    query: String,
}

#[derive(FromRow, Serialize)]
struct Record {
    pk: i32,
    message: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let url = env::var("DATABASE_URL").unwrap();
    let pool = MySqlPool::connect(&url).await.unwrap();

    let app = Router::new()
        .route("/search", get(search))
        .with_state(pool)
        .layer(CompressionLayer::new())
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
    let records = sqlx::query_as::<_, Record>(query.as_str())
        .fetch_all(&pool)
        .await
        .expect("error executing query");

    Json(records)
}
