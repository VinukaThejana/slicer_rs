use crate::model;
use rayon::prelude::*;

pub fn volume(triangles: &[model::Triangle]) -> f64 {
    if triangles.is_empty() {
        return 0.0;
    }

    const PARALLEL_THRESHOLD: usize = 1000;
    const CHUNK_SIZE: usize = 1000;

    let total_volume: f64 = if triangles.len() >= PARALLEL_THRESHOLD {
        triangles
            .par_chunks(CHUNK_SIZE)
            .map(|chunk| khan_sum(chunk))
            .sum()
    } else {
        khan_sum(triangles)
    };

    total_volume.abs()
}

#[inline]
fn khan_sum(triangles: &[model::Triangle]) -> f64 {
    let mut sum = 0.0f64;
    let mut compensation = 0.0f64;

    for triangle in triangles {
        let y = triangle.signed_volume() - compensation;
        let t = sum + y;
        compensation = (t - sum) - y;
        sum = t;
    }

    sum
}
