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
    /// number of triangles in the 3D model
    #[schema(example = "success")]
    status: String,

    /// number of triangles in the 3D model
    #[schema(example = 15_000)]
    triangles: usize,

    /// calculated volume in cubic units, based on the model's native scale
    #[schema(example = -1)]
    #[schema(example = 12.345)]
    volume: f64,

    /// volume after applying the user provided scale factor
    #[schema(example = 1234.567)]
    scaled_volume: f64,

    /// flag indicating whether scaling was applied
    #[schema(example = false)]
    scaled: bool,

    /// unit of measurement: "mm", "cm", or "m"
    #[schema(example = "cm")]
    unit: String,
}

impl CalculateVolumeRes {
    pub fn new(
        triangles: usize,
        volume: f64,
        scaled_volume: f64,
        is_scaled: bool,
        unit: String,
    ) -> Self {
        Self {
            status: "success".to_string(),
            triangles,
            volume,
            scaled_volume,
            scaled: is_scaled,
            unit,
        }
    }
}

// file_name: only alphanumeric, hyphens, periods and underscores
static FILENAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9_.-]+$").unwrap());

// unit: only "mm", "cm", or "m"
static UNIT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(mm|cm|m)$").unwrap());
