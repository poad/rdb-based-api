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

use sqlx::{Column, Row, TypeInfo};
use sqlx_mysql::MySqlPool;

use std::{collections::HashMap, env};

use dotenvy::dotenv;

use serde::{Deserialize, Serialize};

use tracing::Level;
use tracing_log::LogTracer;

use log::{error, trace};

#[derive(Deserialize)]
struct QueryRequest {
    query: String,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
struct Record {
    pk: Option<i32>,
    message: Option<String>,
}

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
        .route("/search", get(search))
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

async fn search(
    Query(request): Query<QueryRequest>,
    State(pool): State<MySqlPool>,
) -> impl IntoResponse {
    let query = request.query;
    let result = sqlx::query(query.as_str()).fetch_all(&pool).await;

    match result {
        Err(err) => {
            error!("{:?}", err);
            Err(StatusCode::FORBIDDEN)
        }
        Ok(rows) => {
            let records = rows
                .iter()
                .map(|row| {
                    let record = row
                        .columns()
                        .iter()
                        .filter(|column| row.try_get_raw(column.name()).is_ok())
                        .map(|column| {
                            trace!("{:?}", column.type_info().name());
                            let type_name = column.type_info().name();
                            let value = match type_name {
                                "INT" => row.get::<i32, &str>(column.name()).to_string(),
                                "TEXT" => row.get::<String, &str>(column.name()),
                                _ => "".to_string(),
                            };
                            (column.name(), value)
                        })
                        .collect::<HashMap<_, _>>();
                    let pk = record.get("pk").map(|value| value.parse::<i32>().unwrap());
                    let message = record.get("message").map(|v| v.to_string());
                    Record { pk, message }
                })
                .collect::<Vec<Record>>();
            trace!("{:?}", records);
            Ok(Json(records))
        }
    }
}
