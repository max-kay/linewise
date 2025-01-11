use std::ops::{Add, AddAssign};

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Energy {
    pub strain_energy: f32,
    pub bending_energy: f32,
    pub potential_energy: f32,
    pub field_energy: f32,
    pub interaction_energy: f32,
    pub boundary_energy: f32,
}

impl Energy {
    pub const NAMES: [&'static str; 6] = [
        "strain",
        "bending",
        "potential",
        "field",
        "interaction",
        "boundary",
    ];

    pub fn zero() -> Self {
        Self {
            strain_energy: 0.0,
            bending_energy: 0.0,
            potential_energy: 0.0,
            field_energy: 0.0,
            interaction_energy: 0.0,
            boundary_energy: 0.0,
        }
    }

    pub fn as_array(self) -> [f32; 6] {
        [
            self.strain_energy,
            self.bending_energy,
            self.potential_energy,
            self.field_energy,
            self.interaction_energy,
            self.boundary_energy,
        ]
    }

    pub fn sum(&self) -> f32 {
        self.strain_energy
            + self.bending_energy
            + self.potential_energy
            + self.field_energy
            + self.interaction_energy
            + self.boundary_energy
    }

    pub fn half_interaction(mut self) -> Self {
        self.interaction_energy /= 2.0;
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
            interaction_energy: self.interaction_energy + rhs.interaction_energy,
            boundary_energy: self.boundary_energy + rhs.boundary_energy,
        }
    }
}

impl AddAssign for Energy {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}
