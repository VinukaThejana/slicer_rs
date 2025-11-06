use crate::model;
use rayon::prelude::*;

pub fn volume(triangles: &[model::Triangle]) -> f32 {
    let total_volume: f32 = triangles
        .par_iter()
        .map(|triangle| triangle.signed_volume())
        .sum();

    total_volume.abs()
}
