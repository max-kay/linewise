use crate::sampler::Samples2d;
use crate::spline::BSpline;
use crate::Vector;
use crate::CALCULATION_PRECISION;
use crate::INTERACTION_RADIUS;

pub struct Polymer {
    shape: BSpline,
    original_length: f32,
    youngs_modulus: f32,
    sec_moment_of_width: f32,
    alignement_factor: f32,
    density_factor: f32,
    repulsion_factor: f32,
}

impl Polymer {
    pub fn new(
        shape: BSpline,
        youngs_modulus: f32,
        sec_moment_of_width: f32,
        alignement_factor: f32,
        density_factor: f32,
        repulsion_factor: f32,
    ) -> Self {
        Polymer {
            original_length: shape.length(CALCULATION_PRECISION),
            shape,
            youngs_modulus,
            sec_moment_of_width,
            alignement_factor,
            density_factor,
            repulsion_factor,
        }
    }

    pub fn strain_energy(&self) -> f32 {
        let len = self.shape.length(CALCULATION_PRECISION);
        self.youngs_modulus * len * (self.original_length - len).powi(2)
            / self.original_length.powi(2)
            / 2.0
    }

    pub fn bending_energy(&self) -> f32 {
        self.youngs_modulus
            * self.sec_moment_of_width
            * self.shape.curvature_integral(CALCULATION_PRECISION)
            / 2.0
    }

    pub fn potential_energy(&self, potential: Samples2d<f32>, steps_per_segment: usize) -> f32 {
        let mut sum = 0.0;
        for t in (0..self.shape.count() * steps_per_segment)
            .map(|i| (i + 1) as f32 / steps_per_segment as f32)
        {
            let position = self.shape.path(t);
            match potential.get_sample(position) {
                Some(sample) => sum += sample,
                None => return f32::INFINITY,
            }
        }

        self.density_factor * sum
    }

    pub fn vector_field_energy(&self, field: Samples2d<Vector>, steps_per_segment: usize) -> f32 {
        let mut sum = 0.0;
        for t in (0..self.shape.count() * steps_per_segment)
            .map(|i| (i + 1) as f32 / steps_per_segment as f32)
        {
            let position = self.shape.path(t);
            if let Some(vector) = field.get_sample(position) {
                let der = self.shape.derivative(t);
                sum += der.dot(vector) / der.norm_squared();
            }
        }
        self.alignement_factor * sum
    }

    pub fn interaction_energy(&self, other: &Self, steps_per_segment: usize) -> f32 {
        let mut sum = 0.0;
        for t in (0..self.shape.count() * steps_per_segment)
            .map(|i| (i + 1) as f32 / steps_per_segment as f32)
        {
            for s in (0..other.shape.count() * steps_per_segment)
                .map(|i| (i + 1) as f32 / steps_per_segment as f32)
            {
                let norm = (self.shape.path(t) - other.shape.path(s)).norm();
                if norm < INTERACTION_RADIUS {
                    sum += 1.0 / norm;
                }
            }
        }
        sum * self.repulsion_factor * other.repulsion_factor
    }
}
