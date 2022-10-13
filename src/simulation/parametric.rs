use cgmath::Vector3;

/// Represents a curve in R3 defined by a parametric equation on time.
pub struct Parametric {
    path: fn(t: f32) -> Vector3<f32>,
    curr_time: f32,
}

impl Parametric {
    pub fn new(path: fn(t: f32) -> Vector3<f32>) -> Parametric {
        Parametric {
            path,
            curr_time: 0.0,
        }
    }

    pub fn step(&mut self, dt: f32) -> Vector3<f32> {
        let position = (self.path)(self.curr_time);
        self.curr_time = self.curr_time + dt;
        position
    }
}
