pub mod obj;
pub mod stl;

use crate::{calculate, error::AppError};
use nalgebra::Vector3;

pub const MAX_TRIANGLES: u32 = 10_000_000;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [[f32; 3]; 3],
}

impl From<calculate::triangulation::Triangle> for Triangle {
    fn from(triangle: calculate::triangulation::Triangle) -> Self {
        Self {
            vertices: [
                triangle.vertices[0].into(),
                triangle.vertices[1].into(),
                triangle.vertices[2].into(),
            ],
        }
    }
}

impl Triangle {
    pub fn signed_volume(&self) -> f64 {
        let a = Vector3::from(self.vertices[0]);
        let b = Vector3::from(self.vertices[1]);
        let c = Vector3::from(self.vertices[2]);

        // signed volume of the tetrahedron formed by joining the triangle to the origin
        (a.dot(&b.cross(&c)) / 6.0).into()
    }
}

pub trait MeshParser {
    fn parse(bytes: &[u8]) -> Result<Vec<Triangle>, AppError>;
}

#[derive(Debug)]
pub enum Format {
    STL,
    OBJ,
}

impl Format {
    pub fn from_content_type(content_type: &str) -> Option<Self> {
        if content_type.contains("application/sla")
            || content_type.contains("application/vnd.ms-pki.stl")
            || content_type.contains("model/stl")
        {
            Some(Format::STL)
        } else if content_type.contains("model/obj") || content_type.contains("application/x-tgif")
        {
            Some(Format::OBJ)
        } else {
            None
        }
    }

    pub fn from_url(url: &str) -> Option<Self> {
        match url.to_lowercase().rsplit('.').next()? {
            "stl" => Some(Format::STL),
            "obj" => Some(Format::OBJ),
            _ => None,
        }
    }

    pub fn from_magic_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        // STL file detection
        // binary STL files detection
        if bytes.len() >= 84 {
            let traingle_count = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]);
            if traingle_count > 0
                && traingle_count <= MAX_TRIANGLES
                && let Some(expected_size) = 84usize.checked_add(traingle_count as usize * 50)
                && bytes.len() >= expected_size
                && bytes.len() <= expected_size + 80
            {
                return Some(Format::STL);
            }
        }

        // ASCII STL files detection
        if bytes.len() >= 5 && &bytes[..5] == b"solid" {
            let preview = &bytes[..bytes.len().min(4096)];
            if let Ok(content) = std::str::from_utf8(preview)
                && content.contains("facet")
                && content.contains("vertex")
            {
                return Some(Format::STL);
            }
        }

        // OBJ file detection
        let preview = &bytes[..bytes.len().min(4096)];
        if let Ok(content) = std::str::from_utf8(preview) {
            let trimmed = content.trim_start();

            // OBJ files typically contain 'v ' (vertex), 'vt ' (texture), 'vn ' (normal), or 'f ' (face) lines
            let markers = trimmed
                .lines()
                .filter(|line| !line.trim().is_empty())
                .take(50)
                .any(|line| {
                    let line = line.trim_start();
                    line.starts_with("v ")
                        || line.starts_with("vt ")
                        || line.starts_with("vn ")
                        || line.starts_with("f ")
                        || line.starts_with("o ")
                        || line.starts_with("g ")
                        || line.starts_with("mtllib ")
                        || line.starts_with("usemtl ")
                });

            if markers {
                return Some(Format::OBJ);
            }
        }

        None
    }

    pub fn validate_bytes(&self, bytes: &[u8]) -> bool {
        match self {
            Self::STL => stl::validate_bytes(bytes),
            Self::OBJ => obj::validate_bytes(bytes),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::STL => "stl",
            Self::OBJ => "obj",
        }
    }
}
