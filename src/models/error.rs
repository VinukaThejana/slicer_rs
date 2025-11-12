use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ResponseError {
    // "error"
    #[schema(example = "error")]
    pub status: String,

    // this describes the reason for the error, can be directly shown to the user
    #[schema(example = "the reason for the error")]
    pub message: String,
}
