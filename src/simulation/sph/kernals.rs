use std::f32::consts::PI;

use cgmath::{InnerSpace, Vector3, Zero};

/// s is the maximum distance of influence; r larger than s is returns 0.
pub fn monaghan(r: f32, s: f32) -> f32 {
    let variable_numerator = if r / s >= 0.0 && r / s <= 1.0 {
        1.0 - 1.5 * (r / s).powi(2) + 0.75 * (r / s).powi(3)
    } else if r / s >= 1.0 && r / s <= 2.0 {
        0.25 * (2.0 - r / s).powi(3)
    } else {
        0.0
    };
    variable_numerator / (PI * s.powi(3))
}

pub fn monaghan_gradient(r_vec: Vector3<f32>, s: f32) -> Vector3<f32> {
    if r_vec.is_zero() {
        return Vector3::<f32>::zero();
    }
    let r = r_vec.magnitude();
    let variable_numerator = if r / s >= 0.0 && r / s <= 1.0 {
        3.0 * r / s * (-1.0 + 0.75 * r / s)
    } else if r / s >= 1.0 && r / s <= 2.0 {
        -0.75 * (2.0 - r / s).powi(2)
    } else {
        0.0
    };
    variable_numerator / (PI * s.powi(4)) * r_vec.normalize()
}

pub fn monaghan_laplacian(r: f32, s: f32) -> f32 {
    let variable_numerator = if r / s >= 0.0 && r / s <= 1.0 {
        3.0 * (-1.0 + 1.5 * r / s)
    } else if r / s >= 1.0 && r / s <= 2.0 {
        1.5 * (2.0 - r / s)
    } else {
        0.0
    };
    variable_numerator / (PI * s.powi(5))
}
