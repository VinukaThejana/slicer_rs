pub mod model;

use axum::{
    Json,
    http::{StatusCode, header},
    response::IntoResponse,
};

pub async fn health() -> impl IntoResponse {
    // NOTE: Add health check logic for all third party services here

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(serde_json::json!
        ({
            "status": "ok"
        })),
    )
}
