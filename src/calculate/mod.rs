pub mod triangulation;

use crate::model;
use rayon::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Vec3(pub f32, pub f32, pub f32);

#[derive(Clone, Copy, Debug)]
pub struct Vec2(pub f32, pub f32);

impl Vec3 {
    pub fn substraction(self, other: Vec3) -> Vec3 {
        Vec3(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }

    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3(
            // basically the determinant of a 3x3 matrix
            self.1 * other.2 - self.2 * other.1,
            self.2 * other.0 - self.0 * other.2,
            self.0 * other.1 - self.1 * other.0,
        )
    }

    pub fn dot(self, other: Vec3) -> f32 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }
}

impl Vec2 {
    pub fn substraction(self, other: Vec2) -> Vec2 {
        Vec2(self.0 - other.0, self.1 - other.1)
    }

    pub fn cross(self, other: Vec2) -> f32 {
        self.0 * other.1 - self.1 * other.0
    }

    pub fn dot(self, other: Vec2) -> f32 {
        self.0 * other.0 + self.1 * other.1
    }
}

pub fn volume(triangles: &[model::Triangle]) -> f64 {
    if triangles.is_empty() {
        return 0.0;
    }

    const PARALLEL_THRESHOLD: usize = 1000;
    const CHUNK_SIZE: usize = 1000;

    let total_volume: f64 = if triangles.len() >= PARALLEL_THRESHOLD {
        triangles.par_chunks(CHUNK_SIZE).map(khan_sum).sum()
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
