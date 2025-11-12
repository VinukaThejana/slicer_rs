use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct CalculateVolumeReq {
    /// 26-character order ID obtained when the object is uploaded to S3
    #[schema(example = "01K9N559GM0BXKW00QX5T5F4FH")]
    #[validate(length(equal = 26, message = "must be 26 characters long"))]
    pub order_id: String,

    /// 26-character item ID obtained when the object is uploaded to S3
    #[schema(example = "01K9N559GM0BXKW00QX9NJ47AR")]
    #[validate(length(equal = 26, message = "must be 26 characters long"))]
    pub item_id: String,

    /// file name with extension obtained when the object is uploaded to S3
    #[schema(example = "model_file.stl")]
    #[validate(regex(
        path = "*FILENAME_REGEX",
        message = "file_name must be alphanumeric characters with hyphens, periods, or underscores only"
    ))]
    pub file_name: String,

    /// unit of measurement: "mm", "cm", or "m"
    #[schema(example = "cm")]
    #[validate(regex(
        path = "*UNIT_REGEX",
        message = "unit must be one of 'mm', 'cm', or 'm'"
    ))]
    pub unit: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CalculateVolumeRes {
    #[schema(example = "success")]
    status: String,

    #[schema(example = 15_000)]
    triangles: usize,

    #[schema(example = 12.345)]
    volume: f32,
}

impl CalculateVolumeRes {
    pub fn new(triangles: usize, volume: f32) -> Self {
        Self {
            status: "success".to_string(),
            triangles,
            volume,
        }
    }
}

// file_name: only alphanumeric, hyphens, periods and underscores
static FILENAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9_.-]+$").unwrap());

// unit: only "mm", "cm", or "m"
static UNIT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(mm|cm|m)$").unwrap());
