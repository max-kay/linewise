use nalgebra::Vector2;
use rand::rngs::SmallRng;

pub mod picture;
pub mod polymer;
pub mod quad_tree;
pub mod sampler;
pub mod spline;

use quad_tree::{Bounded, BoundingBox};

pub type MyRng = SmallRng;

const CALCULATION_PRECISION: usize = 10;
const INTERACTION_RADIUS: f32 = 20.0;
const BORDER_INTERACTION_RADIUS: f32 = 20.0;

pub type Vector = Vector2<f32>;

impl Bounded for Vector {
    fn bounding_box(&self) -> quad_tree::BoundingBox {
        BoundingBox::new(self.x, self.x, self.y, self.y)
    }
}
