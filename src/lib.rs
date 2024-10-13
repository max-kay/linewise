use nalgebra::Vector2;

pub type Vector = Vector2<f32>;

const CALCULATION_PRECISION: usize = 10;
const INTERACTION_RADIUS: f32 = 20.0;

pub mod picture;
pub mod polymer;
pub mod quad_tree;
pub mod sampler;
pub mod spline;
