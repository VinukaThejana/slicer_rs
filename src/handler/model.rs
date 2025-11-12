use crate::config::ENV;
use crate::error::AppError;
use crate::model::MeshParser;
use crate::models::mdl::{CalculateVolumeReq, CalculateVolumeRes};
use crate::{calculate, model, models};
use axum::Extension;
use axum::{
    Json,
    http::{StatusCode, header},
    response::IntoResponse,
};
use bytes::BytesMut;
use futures_util::StreamExt;
use validator::Validate;

const MAX_MODEL_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MB

/// Calculate the volume of a 3D model file stored in S3.
///
/// The model file must be in STL format and not exceed 100MB in size.
/// The volume is calculated based on the provided unit (mm, cm, m).
///
/// This endpoint requires authentication. The user's access token must be
/// provided in the Authorization header as a Bearer token.
#[utoipa::path(
    post,
    path = "/api/volume",
    tag = "Model Calculations",
    request_body = CalculateVolumeReq,
    responses(
        (status = 200, description = "Volume calculated successfully", body = CalculateVolumeRes),
        (status = 400, description = "Bad Request (file too large, invalid format, validation error)", body = models::error::ResponseError),
        (status = 404, description = "Model not found, or related error", body = models::error::ResponseError),
        (status = 500, description = "Internal Server Error", body = models::error::ResponseError),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn calculate_volume(
    Extension(user_id): Extension<models::user::UserId>,
    Json(payload): Json<CalculateVolumeReq>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    let url = format!(
        "https://{}.s3.{}.amazonaws.com/{}/orders/{}/{}/{}",
        ENV.s3_bucket_name,
        ENV.s3_region,
        user_id,
        payload.order_id,
        payload.item_id,
        payload.file_name,
    );
    let client = reqwest::Client::new();

    let head_response = client
        .head(&url)
        .send()
        .await
        .map_err(|e| AppError::bad_request_with_source("failed to fetch model metadata", e))?;

    if !head_response.status().is_success() {
        let error = match head_response.status() {
            reqwest::StatusCode::NOT_FOUND => AppError::not_found("model not found"),
            _ => AppError::bad_request("failed to fetch model metadata"),
        };
        return Err(error);
    }
    if let Some(content_length) = head_response.headers().get(reqwest::header::CONTENT_LENGTH)
        && let Ok(length_str) = content_length.to_str()
        && let Ok(length) = length_str.parse::<usize>()
        && length > MAX_MODEL_FILE_SIZE
    {
        return Err(AppError::bad_request(format!(
            "model file too large: {} bytes (max: {} bytes)",
            length, MAX_MODEL_FILE_SIZE
        )));
    }

    let format = head_response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .and_then(model::Format::from_content_type)
        .or_else(|| model::Format::from_url(&url));

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::bad_request_with_source("failed to fetch model", e))?;
    let status = response.status();
    if !status.is_success() {
        let error = match status {
            reqwest::StatusCode::NOT_FOUND => AppError::not_found("model not found"),
            _ => AppError::bad_request("failed to fetch model"),
        };
        return Err(error);
    }

    let mut stream = response.bytes_stream();
    let mut buffer = BytesMut::with_capacity(8192); // 8KB initial capacity
    let mut total_size = 0usize;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk
            .map_err(|e| AppError::bad_request_with_source("error reading model stream", e))?;
        total_size += chunk.len();
        if total_size > MAX_MODEL_FILE_SIZE {
            return Err(AppError::bad_request(format!(
                "model file size exceeds limit during download (max: {} bytes)",
                MAX_MODEL_FILE_SIZE
            )));
        }
        buffer.extend_from_slice(&chunk);
    }

    let bytes = buffer.freeze();

    let format = format
        .or_else(|| model::Format::from_magic_bytes(&bytes))
        .ok_or_else(|| AppError::bad_request("unsupported model format"))?;
    if !format.validate_bytes(&bytes) {
        return Err(AppError::bad_request("invalid model file"));
    }

    let triangles = match format {
        model::Format::STL => model::stl::STlParser::parse(&bytes),
    }?;

    let volume = calculate::volume(&triangles);
    let volume = match payload.unit.as_str() {
        "mm" => volume,
        "cm" => volume / 1000.0,
        "m" => volume / 1_000_000.0,
        _ => volume,
    };

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(CalculateVolumeRes::new(triangles.len(), volume)),
    ))
}
