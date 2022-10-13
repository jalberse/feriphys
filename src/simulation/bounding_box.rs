use std::ops::Range;

use cgmath::Vector3;

pub struct BoundingBox {
    pub x_range: Range<f32>,
    pub y_range: Range<f32>,
    pub z_range: Range<f32>,
}

impl BoundingBox {
    /// Gets the acceleration due to the force applied by the bounding box
    pub fn get_repelling_acceleration(&self, position: Vector3<f32>) -> Vector3<f32> {
        let force_x_top = -1.0 / (self.x_range.end - position.x).powi(2);
        let force_x_bottom = 1.0 / (self.x_range.start - position.x).powi(2);
        let force_y_right = -1.0 / (self.y_range.end - position.y).powi(2);
        let force_y_left = 1.0 / (self.y_range.start - position.y).powi(2);
        let force_z_front = -1.0 / (self.z_range.end - position.z).powi(2);
        let force_z_back = 1.0 / (self.z_range.start - position.z).powi(2);

        Vector3::<f32> {
            x: force_x_top + force_x_bottom,
            y: force_y_right + force_y_left,
            z: force_z_back + force_z_front,
        }
    }
}
