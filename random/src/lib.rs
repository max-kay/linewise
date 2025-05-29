use nalgebra::Vector2;
pub use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro128StarStar;
use std::f32::consts::TAU;

pub type MyRng = Xoshiro128StarStar;

type Vector = Vector2<f32>;

pub fn gaussian_vector(rng: &mut MyRng) -> Vector {
    // Box Muller transform
    let theta = TAU * rng.random::<f32>();
    let radius = (-2.0 * rng.random::<f32>().ln()).sqrt();
    // TODO: why did I do this?
    match rng.random_range(0..4) {
        0 => Vector::new(radius * theta.cos(), radius * theta.sin()),
        1 => -Vector::new(radius * theta.cos(), radius * theta.sin()),
        2 => Vector::new(radius * theta.sin(), radius * theta.cos()),
        3 => -Vector::new(radius * theta.sin(), radius * theta.cos()),
        _ => unreachable!(),
    }
}

pub fn new_rng() -> MyRng {
    MyRng::try_from_os_rng().expect("failed to get rng from os rng")
}

pub fn rand_unit(rng: &mut MyRng) -> Vector {
    gaussian_vector(rng).normalize()
}
