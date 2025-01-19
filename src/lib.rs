use nalgebra::{Rotation2, Vector2};

pub mod energy;
mod monte_carlo;
pub mod plt;
pub mod polymer;
pub mod quad_tree;
pub mod sampler;
mod utils;

use energy::Energy;
use polymer::{BorrowedSegment, OwnedPolymer, PolymerRef, PolymerStorage};
use quad_tree::{Bounded, QuadTree, Rect};
use rand_xoshiro::Xoshiro512StarStar;
use sampler::Samples2d;

pub use monte_carlo::{Model, ModelParameters};
pub use utils::{clear_or_create_dir, gaussian_vector, rand_unit};

pub type MyRng = Xoshiro512StarStar;

const CLEAR_LINE: &'static str = "\x1B[2K\r";
const MOVE_UP: &'static str = "\x1B[A\r";
const PIXEL_PER_CM: f32 = 37.795275591;

pub type Vector = Vector2<f32>;
pub type Rotation = Rotation2<f32>;

impl Bounded for Vector {
    fn bounding_box(&self) -> quad_tree::Rect {
        Rect::new(self.x, self.x, self.y, self.y)
    }
}
