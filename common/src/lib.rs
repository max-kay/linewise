use nalgebra::{Rotation2, Vector2};

pub mod energy;
pub mod plt;
pub mod polymer;
pub mod quad_tree;
pub mod sampler;

pub use energy::Energy;
pub use polymer::{BorrowedSegment, OwnedPolymer, PolymerRef, PolymerStorage};
pub use quad_tree::{Bounded, QuadTree, Rect};
pub use sampler::Samples2d;

pub const CLEAR_LINE: &'static str = "\x1B[2K\r";
pub const MOVE_UP: &'static str = "\x1B[A\r";
pub const PIXEL_PER_CM: f32 = 37.795275591;

pub type Vector = Vector2<f32>;
pub type Rotation = Rotation2<f32>;

impl Bounded for Vector {
    fn bounding_box(&self) -> quad_tree::Rect {
        Rect::new(self.x, self.x, self.y, self.y)
    }
}
