pub mod stl;

use nalgebra::Vector3;

use crate::error::AppError;

pub const MAX_TRIANGLES: u32 = 10_000_000;

#[derive(Debug, Clone)]
pub struct Triangle {
    vertices: [[f32; 3]; 3],
}

impl Triangle {
    pub fn signed_volume(&self) -> f32 {
        let a = Vector3::from(self.vertices[0]);
        let b = Vector3::from(self.vertices[1]);
        let c = Vector3::from(self.vertices[2]);

        // signed volume of the tetrahedron formed by joining the triangle to the origin
        a.dot(&b.cross(&c)) / 6.0
    }
}

pub trait MeshParser {
    fn parse(bytes: &[u8]) -> Result<Vec<Triangle>, AppError>;
}

pub enum Format {
    STL,
}

impl Format {
    pub fn from_content_type(content_type: &str) -> Option<Self> {
        if content_type.contains("application/sla")
            || content_type.contains("application/vnd.ms-pki.stl")
            || content_type.contains("model/stl")
        {
            Some(Format::STL)
        } else {
            None
        }
    }

    pub fn from_url(url: &str) -> Option<Self> {
        let url = url.to_lowercase();
        if url.ends_with(".stl") {
            Some(Format::STL)
        } else {
            None
        }
    }

    pub fn from_magic_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        // STL file detection
        // binary STL files detection
        let traingle_count = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]);
        if traingle_count > 0
            && traingle_count <= MAX_TRIANGLES
            && let Some(expected_size) = 84usize.checked_add(traingle_count as usize * 50)
            && bytes.len() >= expected_size
            && bytes.len() <= expected_size + 80
        {
            return Some(Format::STL);
        }

        // ASCII STL files detection
        if &bytes[..5] == b"solid" {
            let preview = &bytes[..bytes.len().min(4096)];
            if let Ok(content) = std::str::from_utf8(preview)
                && content.contains("facet")
                && content.contains("vertex")
            {
                return Some(Format::STL);
            }
        }

        None
    }

    pub fn validate_bytes(&self, bytes: &[u8]) -> bool {
        match self {
            Self::STL => stl::validate_bytes(bytes),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::STL => "stl",
        }
    }
}
