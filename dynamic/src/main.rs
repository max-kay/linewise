use common::{Spline, Vector};

struct Model {
    _pos: Spline,
    _vel: Spline,
}

impl Model {
    pub fn new(initial_pos: Spline, initial_vel: Spline) -> Self {
        Self {
            _pos: initial_pos,
            _vel: initial_vel,
        }
    }
}

impl Model {
    pub fn update(&mut self) {
        todo!()
    }
}

pub fn main() {
    let initial_pos = Spline::new(
        (0..4).map(|i| Vector::new(i as f32, 0.0)).collect(),
        (0..4)
            .map(|i| Vector::new(1.0 / 3.0, (-1.0_f32).powi(i) / 3.0))
            .collect(),
    );
    let initial_vel = Spline::new(
        (0..4).map(|_| Vector::zeros()).collect(),
        (0..4).map(|_| Vector::zeros()).collect(),
    );
    let mut model = Model::new(initial_pos, initial_vel);
    for _ in 0..100 {
        model.update();
    }
}
