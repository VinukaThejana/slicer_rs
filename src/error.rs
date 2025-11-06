use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use std::fmt::Display;
use validator::ValidationErrors;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Conflict(String),
    UniqueViolation(String),
    Unauthorized(String),
    Validation(#[from] ValidationErrors),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(e) => write!(f, "{}", e),
            Self::BadRequest(e) => write!(f, "{}", e),
            Self::Conflict(e) => write!(f, "{}", e),
            Self::UniqueViolation(e) => write!(f, "{}", e),
            Self::Unauthorized(e) => write!(f, "{}", e),
            Self::Validation(e) => {
                let message = e
                    .field_errors()
                    .values()
                    .flat_map(|e| e.iter())
                    .flat_map(|err| {
                        err.message
                            .as_ref()
                            .map(|msg| msg.to_string())
                            .or(Some(String::from("invalid value")))
                    })
                    .next()
                    .unwrap_or(String::from("invalid value"));

                write!(f, "{}", message)
            }
            Self::Other(e) => write!(f, "{}", e),
        }
    }
}

impl AppError {
    pub fn from_generic_error<E>(e: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Other(e.into())
    }

    pub fn from_not_found<E>(e: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::NotFound(e.to_string())
    }
}

impl AppError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(message.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => {
                log::info!("[not_found] {}", msg);
                (StatusCode::NOT_FOUND, msg)
            }
            AppError::BadRequest(msg) => {
                log::info!("[bad_request] {}", msg);
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::Conflict(msg) => {
                log::info!("[conflict] {}", msg);
                (StatusCode::CONFLICT, msg)
            }
            AppError::UniqueViolation(msg) => {
                log::info!("[unique_violation] {}", msg);
                (StatusCode::CONFLICT, msg)
            }
            AppError::Unauthorized(msg) => {
                log::info!("[unauthorized] {}", msg);
                (StatusCode::UNAUTHORIZED, msg)
            }
            AppError::Validation(err) => {
                let err_msg = AppError::Validation(err).to_string();
                log::info!("[validation] {}", err_msg);
                (StatusCode::BAD_REQUEST, err_msg)
            }
            AppError::Other(err) => {
                log::error!("[internal_error] {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
        };

        (
            status,
            [(header::CONTENT_TYPE, "application/json")],
            Json(serde_json::json!({
                "status": "error",
                "message": message,
            })),
        )
            .into_response()
    }
}
