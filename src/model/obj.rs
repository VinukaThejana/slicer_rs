// file format
// OBJ files are ASCII text files with the following line types:
// v x y z          | vertex position
// vt u v           | texture coordinate
// vn x y z         | vertex normal
// f v1 v2 v3       | face (triangle) - can reference v/vt/vn indices
// f v1/vt1 v2/vt2 v3/vt3           | face with texture coords
// f v1/vt1/vn1 v2/vt2/vn2 v3/vt3/vn3 | face with texture and normals
// f v1//vn1 v2//vn2 v3//vn3        | face with normals only
// # comment        | comment line
// o name           | object name
// g name           | group name
// mtllib file      | material library
// usemtl name      | use material
use crate::{
    calculate::{Vec3, triangulation::triangulate},
    error::AppError,
    model::{MeshParser, Triangle},
};
use rayon::prelude::*;
use std::io::{BufRead, Cursor};

pub fn validate_bytes(bytes: &[u8]) -> bool {
    let Ok(content) = std::str::from_utf8(bytes) else {
        return false;
    };

    let mut has_vertices = false;
    let mut has_faces = false;

    for line in content.lines().take(1000) {
        let line = line.trim();
        if line.is_empty() || line.starts_with("#") {
            continue;
        }

        if line.starts_with("v ") {
            has_vertices = true;
        } else if line.starts_with("f ") {
            has_faces = true;
        }

        if has_vertices && has_faces {
            return true;
        }
    }

    has_vertices
}

pub struct OBJParser;

impl MeshParser for OBJParser {
    fn parse(bytes: &[u8]) -> Result<Vec<super::Triangle>, crate::error::AppError> {
        let mut vertices = Vec::<Vec3>::new();
        let mut faces = Vec::<Vec<usize>>::new();

        let mut cursor = Cursor::new(bytes);
        let mut line_buf = String::new();

        while cursor
            .read_line(&mut line_buf)
            .map_err(AppError::from_generic_error)?
            > 0
        {
            let line = line_buf.trim();

            // vertex parsing
            // v x y z
            if line.starts_with("v ") {
                let mut split = line.split_whitespace();
                split.next(); // skip the "v"

                let x: f32 = split
                    .next()
                    .ok_or_else(|| AppError::bad_request("missing x coordinate on vertex"))?
                    .parse()
                    .map_err(|e| {
                        AppError::bad_request_with_source("invalid x coordinate on vertex", e)
                    })?;
                let y: f32 = split
                    .next()
                    .ok_or_else(|| AppError::bad_request("missing y coordinate on vertex"))?
                    .parse()
                    .map_err(|e| {
                        AppError::bad_request_with_source("invalid y coordinate on vertex", e)
                    })?;
                let z: f32 = split
                    .next()
                    .ok_or_else(|| AppError::bad_request("missing z coordinate on vertex"))?
                    .parse()
                    .map_err(|e| {
                        AppError::bad_request_with_source("invalid z coordinate on vertex", e)
                    })?;

                vertices.push(Vec3(x, y, z));
            }
            // face parsing
            else if line.starts_with("f ") {
                let mut split = line.split_whitespace();
                split.next(); // skip the "f"

                let mut face = Vec::new();

                for part in split {
                    let mut nums = part.split('/');

                    let part_str = nums
                        .next()
                        .ok_or_else(|| AppError::bad_request("missing vertex index on face"))?;
                    let idx: usize = part_str.parse().map_err(|e| {
                        AppError::bad_request_with_source("invalid vertex index", e)
                    })?;

                    face.push(idx - 1); // OBJ indices are 1-based
                }

                faces.push(face);
            }

            line_buf.clear();
        }

        let result_vecs: Result<Vec<Vec<Triangle>>, AppError> = faces
            .par_iter()
            .map(|face_indices| {
                if face_indices.len() == 3 {
                    Ok(vec![Triangle {
                        vertices: [
                            vertices[face_indices[0]].into(),
                            vertices[face_indices[1]].into(),
                            vertices[face_indices[2]].into(),
                        ],
                    }])
                } else if face_indices.len() > 3 {
                    let triangles = triangulate(&vertices, face_indices)?
                        .into_iter()
                        .map(Triangle::from)
                        .collect::<Vec<Triangle>>();
                    Ok(triangles)
                } else {
                    Ok(Vec::new())
                }
            })
            .collect();

        let result: Vec<Triangle> = result_vecs?.into_iter().flatten().collect();
        Ok(result)
    }
}
