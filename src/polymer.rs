use std::f32::consts::TAU;

use crate::gaussian_vector;
use crate::quad_tree::Bounded;
use crate::quad_tree::Rect;
use crate::sampler::Samples2d;
use crate::spline::BSpline;
use crate::ModelParameters;
use crate::MyRng;
use crate::Vector;

mod energy;
pub use energy::Energy;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize)]
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
    pub fn vary(&self, temp: f32, rng: &mut MyRng) -> Self {
        let mut new = self.clone();
        // TODO: clean up magic number mess
        match rng.gen_range(0..=2) {
            0 => new.shape.translate(gaussian_vector(rng) * 0.2 * temp),
            1 => new.shape.rotate(rng.gen::<f32>() * TAU * temp / 2.0),
            2 => new.shape.bend_at_segment(
                rng.gen_range(0..new.count_segments()),
                rng.gen::<f32>() * TAU,
            ),
            _ => unreachable!(),
        }
        new
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
        boundary: Rect,
        parameters: &ModelParameters,
    ) -> Energy {
        let mut length_sum = 0.0;
        let mut bending_sum = 0.0;
        let mut potential_sum = 0.0;
        let mut field_sum = 0.0;
        let mut interaction_sum = 0.0;
        let mut boundary_sum = 0.0;

        // this loop is the intgral over the current segment
        for (seg, t) in self.shape.index_iter(parameters.precision) {
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
                field_sum -= der.dot(vector).abs();
            }

            for other in (&mut others).into_iter().filter(|o| {
                o.bounding_box()
                    .add_radius(parameters.interaction_radius)
                    .contains_point(position)
            }) {
                let mut inner_sum = 0.0;
                // TODO: should I mark this as unlikely
                if std::ptr::eq(self, other) {
                    continue;
                }

                if !other
                    .bounding_box()
                    .add_radius(parameters.interaction_radius)
                    .contains_point(position)
                {
                    continue;
                }

                // TODO skip segments?
                for (other_seg, other_t) in other.shape.index_iter(parameters.precision) {
                    let norm = (position - unsafe { other.shape.path(other_seg, other_t) }).norm();
                    if norm < parameters.interaction_radius {
                        inner_sum +=
                            // SAFETY is from index_iter
                            unsafe { other.shape.derivative(other_seg, other_t) }.norm().powi(3) / norm;
                    }
                }
                interaction_sum += inner_sum * der_norm;
            }

            let signed_dist = boundary.signed_distance(position);
            if signed_dist > 0.0 {
                boundary_sum = f32::INFINITY;
            } else {
                boundary_sum += 1.0 / signed_dist.powi(2)
            }
        }

        let len = length_sum / parameters.precision as f32;
        Energy {
            strain_energy: parameters.energy_factors.strain_energy
                * len
                * ((self.original_length - len) / self.original_length).powi(2)
                / 2.0,
            bending_energy: parameters.energy_factors.bending_energy * bending_sum
                / parameters.precision as f32,
            potential_energy: parameters.energy_factors.potential_energy * potential_sum
                / parameters.precision as f32,
            field_energy: parameters.energy_factors.field_energy * field_sum
                / parameters.precision as f32,
            interaction_energy: parameters.energy_factors.interaction_energy * interaction_sum
                / (parameters.precision * parameters.precision) as f32,
            boundary_energy: parameters.energy_factors.boundary_energy * boundary_sum
                / parameters.precision as f32,
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
