use common::{OwnedSpline, Vector};

struct Model {
    pos: OwnedSpline,
    vel: OwnedSpline,
}

impl Model {
    pub fn new(initial_pos: OwnedSpline, initial_vel: OwnedSpline) -> Self {
        Self {
            pos: initial_pos,
            vel: initial_vel,
        }
    }
}

impl Model {
    pub fn update(&mut self) {
        todo!()
    }
}

pub fn main() {
    let initial_pos = OwnedSpline::new(
        (0..4).map(|i| Vector::new(i as f32, 0.0)).collect(),
        (0..4)
            .map(|i| Vector::new(1.0 / 3.0, (-1.0_f32).powi(i) / 3.0))
            .collect(),
    );
    let initial_vel = OwnedSpline::new(
        (0..4).map(|_| Vector::zeros()).collect(),
        (0..4).map(|_| Vector::zeros()).collect(),
    );
    let mut model = Model::new(initial_pos, initial_vel);
    for _ in 0..100 {
        model.update();
    }
}
