use std::f32::INFINITY;
use std::ops::Add;
use std::ops::AddAssign;

use crate::quad_tree::Bounded;
use crate::quad_tree::BoundingBox;
use crate::sampler::Samples2d;
use crate::spline::BSpline;
use crate::MyRng;
use crate::Vector;
use crate::BOUNDARY_INTERACTION_RADIUS;
use crate::CALCULATION_PRECISION;
use crate::INTERACTION_RADIUS;

mod params {
    pub const S: f32 = 1.0;
    pub const B: f32 = 1.0;
    pub const Q: f32 = 1.0;
    pub const P: f32 = 1.0;
    pub const C: f32 = 1.0;
    pub const A: f32 = 1.0;
}

#[derive(Clone)]
pub struct Polymer {
    shape: BSpline,
    original_length: f32,
}

impl From<BSpline> for Polymer {
    fn from(value: BSpline) -> Self {
        Self::new(value)
    }
}

impl Polymer {
    pub fn new(shape: BSpline) -> Self {
        Polymer {
            original_length: shape.length(CALCULATION_PRECISION),
            shape,
        }
    }

    pub fn new_random(
        approx_center: Vector,
        approx_len_per_segment: f32,
        segments: usize,
        rng: &mut MyRng,
    ) -> Self {
        let shape = BSpline::new_random(approx_center, approx_len_per_segment, segments, rng);
        Polymer {
            original_length: shape.length(CALCULATION_PRECISION),
            shape,
        }
    }
}

impl Polymer {
    pub fn translate(&mut self, vector: Vector) {
        self.shape.translate(vector)
    }

    pub fn rotate(&mut self, radians: f32) {
        self.shape.rotate(radians)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Energy {
    pub strain_energy: f32,
    pub bending_energy: f32,
    pub potential_energy: f32,
    pub field_energy: f32,
    pub pair_energy: f32,
    pub boundary_energy: f32,
}

impl Energy {
    pub fn zero() -> Self {
        Self {
            strain_energy: 0.0,
            bending_energy: 0.0,
            potential_energy: 0.0,
            field_energy: 0.0,
            pair_energy: 0.0,
            boundary_energy: 0.0,
        }
    }

    pub fn sum(&self) -> f32 {
        self.strain_energy
            + self.bending_energy
            + self.potential_energy
            + self.field_energy
            + self.pair_energy
            + self.boundary_energy
    }

    pub fn half_interaction(mut self) -> Self {
        self.pair_energy /= 2.0;
        self
    }
}

impl Add for Energy {
    type Output = Energy;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            strain_energy: self.strain_energy + rhs.strain_energy,
            bending_energy: self.bending_energy + rhs.bending_energy,
            potential_energy: self.potential_energy + rhs.potential_energy,
            field_energy: self.field_energy + rhs.field_energy,
            pair_energy: self.pair_energy + rhs.pair_energy,
            boundary_energy: self.boundary_energy + rhs.boundary_energy,
        }
    }
}

impl AddAssign for Energy {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

/// Energy calculation functions
impl Polymer {
    pub fn all_energies(
        &self,
        potential: &Samples2d<f32>,
        field: &Samples2d<Vector>,
        others: Vec<&Self>,
        steps_per_segment: usize,
        boundary: BoundingBox,
    ) -> Energy {
        let mut length_sum = 0.0;
        let mut bending_sum = 0.0;
        let mut potential_sum = 0.0;
        let mut field_sum = 0.0;
        let mut interaction_sum = 0.0;
        let mut boundary_sum = 0.0;

        for t in (0..self.shape.count_segments() * steps_per_segment)
            .map(|i| i as f32 / steps_per_segment as f32)
        {
            let position = self.shape.path(t);
            let der = self.shape.derivative(t);
            let der_norm = der.norm();
            let der2 = self.shape.derivative2(t);

            length_sum += der_norm;

            bending_sum += (der.x * der2.y - der2.x * der.y).powi(2) / der_norm.powi(5);

            match potential.get_sample(position) {
                Some(sample) => potential_sum += sample * der_norm,
                None => (), //TODO ?
            }

            if let Some(vector) = field.get_sample(position) {
                field_sum += der.dot(vector);
            }

            let mut inner_sum = 0.0;
            for other in &others {
                if std::ptr::eq(self, *other) {
                    continue;
                }

                if !other
                    .bounding_box()
                    .add_radius(INTERACTION_RADIUS)
                    .contains_point(position)
                {
                    continue;
                }

                // TODO skip segments?
                for s in (0..other.shape.count_segments() * steps_per_segment)
                    .map(|i| i as f32 / steps_per_segment as f32)
                {
                    let norm = (position - other.shape.path(s)).norm();
                    if norm < INTERACTION_RADIUS {
                        inner_sum += other.shape.derivative(s).norm() / norm;
                    }
                }
                interaction_sum += inner_sum;
            }

            let signed_dist = boundary.signed_distance(position);
            if signed_dist < BOUNDARY_INTERACTION_RADIUS {
                if signed_dist > 0.0 {
                    boundary_sum += INFINITY;
                }
                boundary_sum += 1.0 / signed_dist
            }
        }

        let len = length_sum / steps_per_segment as f32;

        Energy {
            strain_energy: params::S
                * len
                * (self.original_length - len / self.original_length).powi(2)
                / 2.0,
            bending_energy: params::B * bending_sum / steps_per_segment as f32,
            potential_energy: params::Q * potential_sum / steps_per_segment as f32,
            field_energy: params::P * field_sum / steps_per_segment as f32,
            pair_energy: params::C * interaction_sum
                / (steps_per_segment * steps_per_segment) as f32,
            boundary_energy: params::A * boundary_sum / steps_per_segment as f32,
        }
    }

    pub fn as_path(&self, stroke_width: f32) -> svg::node::element::Path {
        self.shape.as_path(stroke_width)
    }
}

impl Bounded for Polymer {
    fn bounding_box(&self) -> BoundingBox {
        self.shape.bounding_box()
    }
}
