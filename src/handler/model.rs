use crate::error::AppError;
use crate::model::MeshParser;
use crate::{calculate, model};
use axum::{
    Json,
    http::{StatusCode, header},
    response::IntoResponse,
};

pub async fn calculate_volume() -> Result<impl IntoResponse, AppError> {
    let unit = "cm";
    // let url = "https://polyvoxel-objects.s3.ap-southeast-1.amazonaws.com/01JX7NKP8V694ESRBEMGG7WPSJ/orders/01K0904Q8RHHMA522WZ1CDFEZ4/01K0904Q8RHHMA522WZ1YDC55P/peugeot-keychain.stl";
    // let url = "https://polyvoxel-objects.s3.ap-southeast-1.amazonaws.com/testing/Ethereal_Glow_0502184232_texture.stl";
    // let url =
    //     "https://polyvoxel-objects.s3.ap-southeast-1.amazonaws.com/testing/Gear+Knob+%5B6cm%5D.stl";
    let url = "https://polyvoxel-objects.s3.ap-southeast-1.amazonaws.com/testing/Part+Studio+1+-+Part+1-2.stl";

    let response = reqwest::get(url)
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

    let format = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .and_then(model::Format::from_content_type)
        .or_else(|| model::Format::from_url(url));
    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::bad_request_with_source("failed to read model bytes", e))?;

    const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MB
    if bytes.len() > MAX_FILE_SIZE {
        return Err(AppError::bad_request("model file size exceeds limit"));
    }

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
    let volume = match unit {
        "mm" => volume,
        "cm" => volume / 1000.0,
        "m" => volume / 1_000_000.0,
        _ => volume,
    };

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(serde_json::json!({
            "status": "success",
            "triangles": triangles.len(),
            "volume": volume,
        })),
    ))
}
