use crate::{
    calculate::{Vec2, Vec3},
    error::AppError,
};

pub struct Triangle {
    pub vertices: [Vec3; 3],
}

fn compute_polygon_normal(vertices: &[Vec3], indices: &[usize]) -> Vec3 {
    let mut nx = 0.0;
    let mut ny = 0.0;
    let mut nz = 0.0;

    // Newell's method
    // https://www.khronos.org/opengl/wiki/Calculating_a_Surface_Normal
    for win in indices.windows(2) {
        let v0 = &vertices[win[0]];
        let v1 = &vertices[win[1]];

        nx += (v0.1 - v1.1) * (v0.2 + v1.2);
        ny += (v0.2 - v1.2) * (v0.0 + v1.0);
        nz += (v0.0 - v1.0) * (v0.1 + v1.1);
    }

    // close the polygon
    // last vertex to first vertex
    if indices.len() > 1 {
        // last vertex
        let v0 = &vertices[indices[indices.len() - 1]];
        // first vertex
        let v1 = &vertices[indices[0]];

        // same accumilation formula
        nx += (v0.1 - v1.1) * (v0.2 + v1.2);
        ny += (v0.2 - v1.2) * (v0.0 + v1.0);
        nz += (v0.0 - v1.0) * (v0.1 + v1.1);
    }

    Vec3(nx, ny, nz)
}

fn pick_dominant_axis(normal: Vec3) -> u8 {
    let ax = normal.0.abs();
    let ay = normal.1.abs();
    let az = normal.2.abs();

    if ax > ay && ax > az {
        0
    } else if ay > az {
        1
    } else {
        2
    }
}

fn project_to_2d(vertices: &[Vec3], indices: &[usize]) -> Vec<Vec2> {
    let normal = compute_polygon_normal(vertices, indices);
    let axis = pick_dominant_axis(normal);

    indices
        .iter()
        .map(|i| {
            let v = &vertices[*i];
            match axis {
                0 => Vec2(v.1, v.2), // project to YZ plane
                1 => Vec2(v.0, v.2), // project to XZ plane
                _ => Vec2(v.0, v.1), // project to XY plane
            }
        })
        .collect()
}

fn is_convex(prev: Vec2, curr: Vec2, next: Vec2, winding_positive: bool) -> bool {
    let cross = (curr.substraction(prev)).cross(next.substraction(curr));
    if winding_positive {
        cross > 0.0
    } else {
        cross < 0.0
    }
}

// check if a point is inside a triangle using barycentric coordinates
// https://mathworld.wolfram.com/BarycentricCoordinates.html
fn point_in_triangle(point: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let v0 = b.substraction(a);
    let v1 = c.substraction(a);
    let v2 = point.substraction(a);

    let dot00 = v0.dot(v0); // ||v0||²
    let dot01 = v0.dot(v1); // v0 · v1
    let dot02 = v0.dot(v2); // v0 · v2
    let dot11 = v1.dot(v1); // ||v1||²
    let dot12 = v1.dot(v2); // v1 · v2

    // Barycentric coordinates
    let denom = dot00 * dot11 - dot01 * dot01;
    if denom.abs() < f32::EPSILON {
        // triangle is degenerate (AKA No measurable area)
        return false;
    }

    let inv_denom = 1.0 / denom;
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    (u >= 0.0) && (v >= 0.0) && (u + v <= 1.0)
}

// Polygon triangulation using Ear clipping algorithm
// https://www.geometrictools.com/Documentation/TriangulationByEarClipping.pdf
pub fn triangulate(vertices: &[Vec3], indices: &[usize]) -> Result<Vec<Triangle>, AppError> {
    if indices.len() < 3 {
        return Err(AppError::bad_request(
            "cannot triangulate polygon with less than 3 vertices",
        ));
    }

    let projection = project_to_2d(vertices, indices);

    let winding_positive = {
        // determine polygon winding order by calculating signed area
        // using the shoelace formula
        // https://en.wikipedia.org/wiki/Shoelace_formula
        let mut area = 0.0;
        for i in 0..projection.len() {
            let a = &projection[i];
            let b = &projection[(i + 1) % projection.len()];
            area += a.0 * b.1 - b.0 * a.1;
        }
        area > 0.0
    };

    let mut active_indices: Vec<usize> = (0..indices.len()).collect();
    let mut triangles = Vec::with_capacity(indices.len() - 2);

    let mut i = 0;
    let mut remaining_vertices = active_indices.len();
    let mut loop_count = 0;

    while remaining_vertices > 3 {
        loop_count += 1;
        if loop_count > 2 * (remaining_vertices * remaining_vertices) {
            return Err(AppError::bad_request(
                "failed to triangulate polygon: possible non-simple polygon",
            ));
        }

        let prev_idx = (i + remaining_vertices - 1) % remaining_vertices;
        let next_idx = (i + 1) % remaining_vertices;

        let prev = projection[active_indices[prev_idx]];
        let curr = projection[active_indices[i]];
        let next = projection[active_indices[next_idx]];

        // Not an Ear, try the next vertex
        if !is_convex(prev, curr, next, winding_positive) {
            i = (i + 1) % remaining_vertices;
            continue;
        }

        // Check if any other vertex is inside the triangle
        let mut inside = false;
        for (j, &pj) in active_indices.iter().enumerate() {
            // skip the triangle vertices
            if j == prev_idx || j == i || j == next_idx {
                continue;
            }
            if point_in_triangle(projection[pj], prev, curr, next) {
                inside = true;
                break;
            }
        }
        // Not an Ear, try the next vertex
        // Some other vertex is inside the triangle
        if inside {
            i = (i + 1) % remaining_vertices;
            continue;
        }

        triangles.push(Triangle {
            vertices: [
                vertices[indices[active_indices[prev_idx]]],
                vertices[indices[active_indices[i]]],
                vertices[indices[active_indices[next_idx]]],
            ],
        });
        // remove the ear vertex
        active_indices.remove(i);
        remaining_vertices -= 1;
        i %= remaining_vertices;
        loop_count = 0;
    }

    // add the last triangle
    let a = active_indices[0];
    let b = active_indices[1];
    let c = active_indices[2];

    triangles.push(Triangle {
        vertices: [
            vertices[indices[a]],
            vertices[indices[b]],
            vertices[indices[c]],
        ],
    });

    Ok(triangles)
}
