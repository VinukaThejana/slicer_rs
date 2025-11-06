// STL file format
// bytes range | description
// ------------|----------------
// 0-79        | 80 byte header
// 80-83       | // 4 byte unsigned int (number of triangles)
// 84-end      | triangle data // INFO: (50 bytes per triangle)

use std::io::Cursor;

use crate::{
    error::AppError,
    model::{MAX_TRIANGLES, MeshParser},
};

pub fn validate_bytes(bytes: &[u8]) -> bool {
    if bytes.len() < 84 {
        return false;
    }

    if &bytes[0..5] == b"solid"
        && let Ok(content) = std::str::from_utf8(bytes)
        && content.contains("facet")
        && content.contains("vertex")
    {
        return true;
    }

    let triangle_count = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]);
    if triangle_count > MAX_TRIANGLES {
        return false;
    }
    if let Some(expected_size) = 84usize.checked_add(triangle_count as usize * 50)
        && bytes.len() >= expected_size
        && bytes.len() < expected_size + 80
    {
        return true;
    }

    false
}

pub struct STlParser;

impl MeshParser for STlParser {
    fn parse(bytes: &[u8]) -> Result<Vec<super::Triangle>, crate::error::AppError> {
        let mut cursor = Cursor::new(bytes);
        let mesh = stl_io::read_stl(&mut cursor)
            .map_err(|e| AppError::bad_request_with_source("failed to parse STL file", e))?;

        mesh.validate()
            .map_err(|e| AppError::bad_request_with_source("invalid STL mesh", e))?;

        let triangles: Vec<super::Triangle> = mesh
            .faces
            .iter()
            .map(|face| {
                let v0 = mesh.vertices[face.vertices[0]].0;
                let v1 = mesh.vertices[face.vertices[1]].0;
                let v2 = mesh.vertices[face.vertices[2]].0;

                super::Triangle {
                    vertices: [v0, v1, v2],
                }
            })
            .collect();

        Ok(triangles)
    }
}
