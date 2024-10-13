use core::f32;

use linewise::{polymer::Polymer, Vector};

const WIDTH: usize = 200;
const HEIGHT: usize = 200;

const POLYMER_COUNT: usize = 200;

fn main() {
    let mut field = Vec::new();
    let mut potential = Vec::new();
    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            let vec = Vector::new(i as f32, j as f32)
                - Vector::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
            field.push(nalgebra::Rotation2::new(f32::consts::FRAC_PI_2) * vec);
            potential.push((vec.norm() / 10.0).sin())
        }
    }
    let mut polymers = Vec::new();
    for _ in 0..POLYMER_COUNT {
        polymers.push(Polymer::new())
    }
}

