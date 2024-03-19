use sqlx::{Column, Row, TypeInfo};
use sqlx_mysql::MySqlPool;

use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use hyper::StatusCode;

use serde::{Deserialize, Serialize};

use log::{error, trace};

#[derive(Deserialize)]
pub(super) struct QueryRequest {
    query: String,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
pub(super) struct Record {
    pk: Option<i32>,
    message: Option<String>,
}

pub(super) async fn search(
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
