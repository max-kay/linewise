use core::f32;

use crate::quad_tree::Bounded;
use crate::quad_tree::BoundingBox;
use crate::sampler::Samples2d;
use crate::spline::BSpline;
use crate::MyRng;
use crate::Vector;
use crate::BORDER_INTERACTION_RADIUS;
use crate::CALCULATION_PRECISION;
use crate::INTERACTION_RADIUS;

pub mod params {
    pub const S: f32 = 1.0;
    pub const B: f32 = 1.0;
    pub const Q: f32 = 1.0;
    pub const P: f32 = 1.0;
    pub const C: f32 = 1.0;
    pub const A: f32 = 1.0;
}

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

    pub fn strain_energy(&self, steps_per_segment: usize) -> f32 {
        let len = self.shape.length(steps_per_segment);
        params::S * len * (self.original_length - len).powi(2) / self.original_length.powi(2) / 2.0
    }

    pub fn bending_energy(&self, steps_per_segment: usize) -> f32 {
        let mut sum = 0.0;
        for t in (0..self.shape.count_segments() * steps_per_segment)
            .map(|i| (i + 1) as f32 / steps_per_segment as f32)
        {
            let der = self.shape.derivative(t);
            let der2 = self.shape.derivative2(t);
            sum += (der.x * der2.y - der2.x * der.y).powi(2) / der.norm().powi(5);
        }
        params::B * sum / steps_per_segment as f32
    }

    pub fn potential_energy(&self, potential: Samples2d<f32>, steps_per_segment: usize) -> f32 {
        let mut sum = 0.0;
        for t in (0..self.shape.count_segments() * steps_per_segment)
            .map(|i| (i + 1) as f32 / steps_per_segment as f32)
        {
            let position = self.shape.path(t);
            let der = self.shape.derivative(t);
            match potential.get_sample(position) {
                Some(sample) => sum += sample * der.norm(),
                None => return f32::INFINITY,
            }
        }

        params::Q * sum / steps_per_segment as f32
    }

    pub fn field_energy(&self, field: Samples2d<Vector>, steps_per_segment: usize) -> f32 {
        let mut sum = 0.0;
        for t in (0..self.shape.count_segments() * steps_per_segment)
            .map(|i| (i + 1) as f32 / steps_per_segment as f32)
        {
            let position = self.shape.path(t);
            if let Some(vector) = field.get_sample(position) {
                let der = self.shape.derivative(t);
                sum += der.dot(vector);
            }
        }
        params::P * sum / steps_per_segment as f32
    }

    pub fn interaction_energy(&self, other: &Self, steps_per_segment: usize) -> f32 {
        let mut sum = 0.0;
        for t in (0..self.shape.count_segments() * steps_per_segment)
            .map(|i| (i + 1) as f32 / steps_per_segment as f32)
        {
            for s in (0..other.shape.count_segments() * steps_per_segment)
                .map(|i| (i + 1) as f32 / steps_per_segment as f32)
            {
                let norm = (self.shape.path(t) - other.shape.path(s)).norm();
                if norm < INTERACTION_RADIUS {
                    sum += 1.0 / norm;
                }
            }
        }
        params::C * sum / (steps_per_segment * steps_per_segment) as f32
    }

    pub fn boudary_energy(&self, border: BoundingBox, steps_per_segment: usize) -> f32 {
        let mut sum = 0.0;
        for t in (0..self.shape.count_segments() * steps_per_segment)
            .map(|i| (i + 1) as f32 / steps_per_segment as f32)
        {
            let position = self.shape.path(t);
            let signed_dist = border.signed_distance(position);
            if signed_dist < BORDER_INTERACTION_RADIUS {
                if signed_dist > 0.0 {
                    return f32::INFINITY;
                }
                sum += 1.0 / signed_dist
            }
        }
        sum / steps_per_segment as f32
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
