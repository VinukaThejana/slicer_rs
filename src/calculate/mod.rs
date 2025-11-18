pub mod triangulation;

use crate::model;
use rayon::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Vec3(pub f32, pub f32, pub f32);

impl From<Vec3> for [f32; 3] {
    fn from(v: Vec3) -> Self {
        [v.0, v.1, v.2]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vec2(pub f32, pub f32);

impl From<Vec2> for [f32; 2] {
    fn from(v: Vec2) -> Self {
        [v.0, v.1]
    }
}

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
        triangles.par_chunks(CHUNK_SIZE).map(kahan_sum).sum()
    } else {
        kahan_sum(triangles)
    };

    total_volume.abs()
}

#[inline]
fn kahan_sum(triangles: &[model::Triangle]) -> f64 {
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

pub fn autoscale(raw_volume: f64, unit: &str) -> f64 {
    // thresholds for units
    let (min_v, max_v) = match unit {
        "mm" => (1.0, 10_000_000.0),
        "cm" => (0.1, 100_000.0),
        "m" => (1e-6, 100.0),
        _ => return raw_volume,
    };

    if raw_volume >= min_v && raw_volume <= max_v {
        return raw_volume;
    }

    // common scale corrections
    let scales: [(&str, f64); 3] = [
        ("cm", 10.0),   // cm to mm conversion
        ("m", 1000.0),  // m to mm
        ("inch", 25.4), // inch to mm
    ];

    for (_, factor) in scales {
        let fixed_volume = raw_volume * factor.powi(3);
        if fixed_volume >= min_v && fixed_volume <= max_v {
            return fixed_volume;
        }
    }

    raw_volume
}
