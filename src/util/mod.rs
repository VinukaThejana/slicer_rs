use axum::Json;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use governor::middleware::NoOpMiddleware;
use serde::{Deserialize, Deserializer};
use std::sync::Arc;
use tokio::signal;
use tower_governor::GovernorError;
use tower_governor::governor::{GovernorConfig, GovernorConfigBuilder};
use tower_governor::key_extractor::SmartIpKeyExtractor;

pub fn governor_conf() -> Arc<GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware>> {
    Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    )
}

pub fn governor_err(err: GovernorError) -> Response {
    match err {
        GovernorError::TooManyRequests {
            wait_time,
            headers: _,
        } => (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::CONTENT_TYPE, "application/json")],
            Json(serde_json::json!({
                "error": "Too many requests",
                "wait_time": wait_time
            })),
        )
            .into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "application/json")],
            Json(serde_json::json!({
                "error": "Internal server error"
            })),
        )
            .into_response(),
    }
}

pub async fn shutdown(// NOTE: Add state if needed
) {
    let ctrl_c = async {
        signal::ctrl_c().await.unwrap_or_else(|_| {
            log::error!("failed to listen for the ctrl+c signal");
            std::process::exit(1);
        })
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .unwrap_or_else(|_| {
                log::error!("failed to listen for the SIGTERM signal");
                std::process::exit(1);
            })
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            log::info!("recieved ctrl+c signal");
        }
        _ = terminate => {
            log::info!("received SIGTERM signal");
        }
    };

    log::info!("shutting down ... ");
}

pub fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    Ok(s.into())
}
