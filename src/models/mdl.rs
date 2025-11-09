use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CalculateVolumeReq {
    #[validate(length(equal = 26, message = "must be 26 characters long"))]
    pub order_id: String,

    #[validate(length(equal = 26, message = "must be 26 characters long"))]
    pub item_id: String,

    #[validate(regex(
        path = "*FILENAME_REGEX",
        message = "file_name must be alphanumeric characters with hyphens, periods, or underscores only"
    ))]
    pub file_name: String,

    #[validate(regex(
        path = "*UNIT_REGEX",
        message = "unit must be one of 'mm', 'cm', or 'm'"
    ))]
    pub unit: String,
}

// file_name: only alphanumeric, hyphens, periods and underscores
static FILENAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9_.-]+$").unwrap());

// unit: only "mm", "cm", or "m"
static UNIT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(mm|cm|m)$").unwrap());
