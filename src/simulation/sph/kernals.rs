use std::f32::consts::PI;

/// s is the maximum distance of influence; r larger than s is returns 0.
pub fn monaghan(r: f32, s: f32) -> f32 {
    let variable_numerator = if r / s >= 0.0 && r / s <= 1.0 {
        return 1.0 - 1.5 * (r / s).powi(2) + 0.75 * (r / s).powi(3);
    } else if r / s >= 1.0 && r / s <= 2.0 {
        return 0.25 * (2.0 - r / s).powi(3);
    } else {
        0.0
    };
    variable_numerator / (PI * s.powi(3))
}

// TODO monaghon_gradiant and monaghon_laplacian
