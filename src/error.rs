use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use validator::ValidationErrors;

use crate::models;

pub trait StdErrorExt: std::error::Error + Send + Sync + 'static {}
impl<T> StdErrorExt for T where T: std::error::Error + Send + Sync + 'static {}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{user_message}")]
    BadRequest {
        user_message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{user_message}")]
    NotFound {
        user_message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{user_message}")]
    Conflict {
        user_message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{user_message}")]
    UniqueViolation {
        user_message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{user_message}")]
    Unauthorized {
        user_message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("{0}")]
    Validation(#[from] ValidationErrors),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest {
            user_message: msg.into(),
            source: None,
        }
    }

    pub fn bad_request_with_source<E>(msg: impl Into<String>, err: E) -> Self
    where
        E: StdErrorExt,
    {
        Self::BadRequest {
            user_message: msg.into(),
            source: Some(anyhow::Error::new(err)),
        }
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound {
            user_message: msg.into(),
            source: None,
        }
    }

    pub fn not_found_with_source<E>(msg: impl Into<String>, err: E) -> Self
    where
        E: StdErrorExt,
    {
        Self::NotFound {
            user_message: msg.into(),
            source: Some(anyhow::Error::new(err)),
        }
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict {
            user_message: msg.into(),
            source: None,
        }
    }

    pub fn conflict_with_source<E>(msg: impl Into<String>, err: E) -> Self
    where
        E: StdErrorExt,
    {
        Self::Conflict {
            user_message: msg.into(),
            source: Some(anyhow::Error::new(err)),
        }
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized {
            user_message: msg.into(),
            source: None,
        }
    }

    pub fn unauthorized_with_source<E>(msg: impl Into<String>, err: E) -> Self
    where
        E: StdErrorExt,
    {
        Self::Unauthorized {
            user_message: msg.into(),
            source: Some(anyhow::Error::new(err)),
        }
    }
}

impl AppError {
    pub fn from_generic_error<E>(err: E) -> Self
    where
        E: StdErrorExt,
    {
        Self::Other(anyhow::Error::new(err))
    }

    fn source_error(&self) -> Option<&anyhow::Error> {
        match self {
            AppError::BadRequest { source, .. }
            | AppError::NotFound { source, .. }
            | AppError::Conflict { source, .. }
            | AppError::UniqueViolation { source, .. }
            | AppError::Unauthorized { source, .. } => source.as_ref(),
            _ => None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, tag, user_message) = match &self {
            AppError::BadRequest { user_message, .. } => {
                (StatusCode::BAD_REQUEST, "bad_request", user_message)
            }
            AppError::NotFound { user_message, .. } => {
                (StatusCode::NOT_FOUND, "not_found", user_message)
            }
            AppError::Conflict { user_message, .. } => {
                (StatusCode::CONFLICT, "conflict", user_message)
            }
            AppError::UniqueViolation { user_message, .. } => {
                (StatusCode::CONFLICT, "unique_violation", user_message)
            }
            AppError::Unauthorized { user_message, .. } => {
                (StatusCode::UNAUTHORIZED, "unauthorized", user_message)
            }
            AppError::Validation(errs) => {
                let msg = errs
                    .field_errors()
                    .values()
                    .flat_map(|v| v.iter())
                    .flat_map(|e| e.message.as_ref().map(|m| m.to_string()))
                    .next()
                    .unwrap_or_else(|| "invalid value".to_string());

                (StatusCode::BAD_REQUEST, "validation", &msg.to_string())
            }
            AppError::Other(error) => {
                log::error!("[other] unexpected error: {:?}", error);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "other",
                    &String::from("something went wrong"),
                )
            }
        };

        match (tag == "other", self.source_error()) {
            (true, Some(source)) => log::error!("[{}] {}: {:?}", tag, user_message, source),
            (true, None) => log::error!("[{}] {}", tag, user_message),
            (false, Some(source)) => log::info!("[{}] {}: {:?}", tag, user_message, source),
            (false, None) => log::info!("[{}] {}", tag, user_message),
        }

        (
            status_code,
            [(header::CONTENT_TYPE, "application/json")],
            Json(models::error::ResponseError {
                status: String::from("error"),
                message: user_message.to_string(),
            }),
        )
            .into_response()
    }
}

macro_rules! impl_from_error {
    ($($t:ty),+ $(,)?) => {
        $(
            impl From<$t> for AppError {
                fn from(err: $t) -> Self {
                    Self::Other(anyhow::Error::new(err))
                }
            }
        )+
    };
}

impl_from_error!(
    std::io::Error,
    base64::DecodeError,
    std::string::FromUtf8Error,
    yup_oauth2::Error,
    reqwest::Error,
);
