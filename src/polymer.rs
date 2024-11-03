use crate::quad_tree::Bounded;
use crate::quad_tree::Rect;
use crate::sampler::Samples2d;
use crate::spline::BSpline;
use crate::MyRng;
use crate::Vector;

mod energy;
pub use energy::Energy;

mod params {
    pub const S: f32 = 100.0;
    pub const B: f32 = 0.00001;
    pub const Q: f32 = 5.0;
    pub const P: f32 = 100000.0;
    pub const C: f32 = 500000.0;
    pub const R: f32 = 0.0001;
}

#[derive(Clone)]
pub struct Polymer {
    shape: BSpline,
    original_length: f32,
}

impl Polymer {
    pub fn new(shape: BSpline, precision: usize) -> Self {
        Polymer {
            original_length: shape.length(precision),
            shape,
        }
    }

    pub fn new_random(
        approx_center: Vector,
        approx_len_per_segment: f32,
        segments: usize,
        precision: usize,
        rng: &mut MyRng,
    ) -> Self {
        let shape = BSpline::new_random(approx_center, approx_len_per_segment, segments, rng);
        Polymer {
            original_length: shape.length(precision),
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

    pub fn bend_at_segment(&mut self, segment: usize, angle: f32) {
        self.shape.bend_at_segment(segment, angle)
    }
}

impl Polymer {
    pub fn length(&self, precision: usize) -> f32 {
        self.shape.length(precision)
    }

    pub fn original_len(&self) -> f32 {
        self.original_length
    }
    pub fn count_segments(&self) -> usize {
        self.shape.count_segments()
    }
}

/// Energy calculation functions
impl Polymer {
    pub fn all_energies<'a>(
        &self,
        potential: &Samples2d<f32>,
        field: &Samples2d<Vector>,
        mut others: impl Iterator<Item = &'a Self>,
        steps_per_segment: usize,
        boundary: Rect,
    ) -> Energy {
        let mut length_sum = 0.0;
        let mut bending_sum = 0.0;
        let mut potential_sum = 0.0;
        let mut field_sum = 0.0;
        let mut interaction_sum = 0.0;
        let mut boundary_sum = 0.0;

        for (seg, t) in self.shape.index_iter(steps_per_segment) {
            let position = unsafe { self.shape.path(seg, t) };
            let der = unsafe { self.shape.derivative(seg, t) };
            let der_norm = der.norm();
            let der2 = unsafe { self.shape.derivative2(seg, t) };

            length_sum += der_norm;

            bending_sum += (der.x * der2.y - der2.x * der.y).powi(2) / der_norm.powi(5);

            if let Some(sample) = potential.get_sample(position) {
                potential_sum += sample * der_norm
            }

            if let Some(vector) = field.get_sample(position) {
                field_sum -= der.dot(vector).powi(2) * der_norm;
            }

            for other in &mut others {
                let mut inner_sum = 0.0;
                if std::ptr::eq(self, other) {
                    continue;
                }

                if !other
                    .bounding_box()
                    .add_radius(super::INTERACTION_RADIUS)
                    .contains_point(position)
                {
                    continue;
                }

                // TODO skip segments?
                for (other_seg, other_t) in other.shape.index_iter(steps_per_segment) {
                    let norm = (position - unsafe { other.shape.path(other_seg, other_t) }).norm();
                    if norm < super::INTERACTION_RADIUS {
                        inner_sum +=
                            // SAFETY is from index_iter
                            unsafe { other.shape.derivative(other_seg, other_t) }.norm().powi(3) / norm;
                    }
                }
                interaction_sum += inner_sum * der_norm;
            }

            let signed_dist = boundary.signed_distance(position);
            if signed_dist > -super::BOUNDARY_INTERACTION_LAYER {
                if signed_dist > 0.0 {
                    boundary_sum = f32::INFINITY;
                } else {
                    boundary_sum += 1.0 / signed_dist.powi(2)
                }
            }
        }

        let len = length_sum / steps_per_segment as f32;
        Energy {
            strain_energy: params::S
                * len
                * ((self.original_length - len) / self.original_length).powi(2)
                / 2.0,
            bending_energy: params::B * bending_sum / steps_per_segment as f32,
            potential_energy: params::Q * potential_sum / steps_per_segment as f32,
            field_energy: params::P * field_sum / steps_per_segment as f32,
            interaction_energy: params::C * interaction_sum
                / (steps_per_segment * steps_per_segment) as f32,
            boundary_energy: params::R * boundary_sum / steps_per_segment as f32,
        }
    }
}

impl Polymer {
    pub fn as_path(&self, stroke_width: f32) -> svg::node::element::Path {
        self.shape.as_path(stroke_width)
    }
}

impl Bounded for Polymer {
    fn bounding_box(&self) -> Rect {
        self.shape.bounding_box()
    }
}
