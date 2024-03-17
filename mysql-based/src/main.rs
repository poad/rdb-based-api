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

use r2d2_mysql::{
    mysql::{prelude::*, Opts, OptsBuilder},
    r2d2, MySqlConnectionManager,
};

use std::{env, sync::Arc};

use dotenvy::dotenv;

use serde::{Serialize, Deserialize};

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
    let opts = Opts::from_url(&url).unwrap();
    let builder = OptsBuilder::from_opts(opts);
    let manager = MySqlConnectionManager::new(builder);
    let pool = Arc::new(r2d2::Pool::builder().max_size(4).build(manager).unwrap());

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
    State(pool): State<Arc<r2d2::Pool<MySqlConnectionManager>>>,
) -> impl IntoResponse {
    let query = request.query;
    let mut conn = pool.get().expect("error getting connection from pool");

    let records = conn
        .query::<Record, String>(query)
        .expect("error executing query");

    Json(records)
}
