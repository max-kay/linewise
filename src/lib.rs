// all units ar in PXS as is standart in svgs

use std::f32::{consts::TAU, INFINITY};

use nalgebra::{Rotation2, Vector2};
use rand::prelude::*;
use rand::rngs::SmallRng;
use svg::{node::element::Group, Document, Node};

pub mod picture;
pub mod polymer;
pub mod quad_tree;
pub mod sampler;
pub mod spline;

use polymer::{Energy, Polymer};
use quad_tree::{Bounded, BoundingBox, QuadTree};
use sampler::Samples2d;

pub type MyRng = SmallRng;

const CALCULATION_PRECISION: usize = 10;
const INTERACTION_RADIUS: f32 = 10.0;
const BOUNDARY_INTERACTION_RADIUS: f32 = 20.0;
const WIDTH: usize = 200;
const HEIGHT: usize = 200;

const ROOT_POLYMER_COUNT: usize = 20;

const MAX_SEGMENTS: usize = 5;

pub type Vector = Vector2<f32>;
pub type Rotation = Rotation2<f32>;

impl Bounded for Vector {
    fn bounding_box(&self) -> quad_tree::BoundingBox {
        BoundingBox::new(self.x, self.x, self.y, self.y)
    }
}

pub struct Model {
    field: Samples2d<Vector>,
    potential: Samples2d<f32>,
    polymers: QuadTree<Polymer>,
    boundary: BoundingBox,
    energies: Vec<Energy>,
    rng: MyRng,
}

impl Model {
    pub fn make_example() -> Self {
        let mut rng = MyRng::from_rng(thread_rng()).unwrap();
        let mut field = Vec::new();
        let mut potential = Vec::new();
        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                let vec = Vector::new(i as f32, j as f32)
                    - Vector::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
                field.push(nalgebra::Rotation2::new(std::f32::consts::FRAC_PI_2) * vec);
                potential.push((vec.norm() / 10.0).sin())
            }
        }
        let field = Samples2d::new(field, WIDTH, HEIGHT, 0.0, WIDTH as f32, 0.0, HEIGHT as f32);
        let potential = Samples2d::new(
            potential,
            WIDTH,
            HEIGHT,
            0.0,
            WIDTH as f32,
            0.0,
            HEIGHT as f32,
        );
        let mut polymers = Vec::new();
        for i in 0..ROOT_POLYMER_COUNT {
            for j in 0..ROOT_POLYMER_COUNT {
                let x = (i as f32 + 0.5) * WIDTH as f32 * 0.8 / ROOT_POLYMER_COUNT as f32
                    + WIDTH as f32 * 0.1;
                let y = (j as f32 + 0.5) * HEIGHT as f32 * 0.8 / ROOT_POLYMER_COUNT as f32
                    + HEIGHT as f32 * 0.1;
                let position = Vector::new(x, y);
                polymers.push(Polymer::new_random(
                    position,
                    1.2,
                    rng.gen_range(1..=MAX_SEGMENTS),
                    &mut rng,
                ));
            }
        }

        let polymers = QuadTree::new(polymers);

        Self {
            field,
            potential,
            polymers,
            boundary: BoundingBox::new(0.0, WIDTH as f32, 0.0, HEIGHT as f32),
            energies: Vec::new(),
            rng,
        }
    }

    pub fn log_energies(&mut self, precision: usize) {
        let mut summed_energy = Energy::zero();
        for polymer in self.polymers.iter() {
            summed_energy += polymer.all_energies(
                &self.potential,
                &self.field,
                self.polymers
                    .query_intersects(polymer.bounding_box().add_radius(INTERACTION_RADIUS)),
                precision,
                self.boundary,
            );
        }

        self.energies.push(summed_energy.half_interaction())
    }

    // assumes polymer is not in the collection of polymers
    pub fn calculate_energy_from(&self, polymer: &Polymer, precision: usize) -> f32 {
        polymer
            .all_energies(
                &self.potential,
                &self.field,
                self.polymers
                    .query_intersects(polymer.bounding_box().add_radius(INTERACTION_RADIUS)),
                precision,
                self.boundary,
            )
            .sum()
    }

    pub fn take_mc_step(&mut self, temp: f32, precision: usize) {
        let old = self.polymers.pop_random(&mut self.rng);
        let e_0 = self.calculate_energy_from(&old, precision);
        let mut new = old.clone();
        match self.rng.gen_range(0..=1) {
            0 => new.translate(Vector::new(
                (self.rng.gen::<f32>() * 2.0 - 1.0) * 3.0,
                (self.rng.gen::<f32>() * 2.0 - 1.0) * 3.0,
            )),
            1 => new.rotate(self.rng.gen::<f32>() * TAU),
            _ => unreachable!(),
        }
        let e_1 = self.calculate_energy_from(&new, precision);
        let d_e = e_1 - e_0;
        if d_e < 0.0 {
            self.polymers.insert(new);
            return;
        }
        if d_e == INFINITY {
            self.polymers.insert(old);
            return;
        }
        if self.rng.gen::<f32>() < (-d_e / temp).exp() {
            self.polymers.insert(new)
        } else {
            self.polymers.insert(old)
        }
    }

    pub fn make_mc_sweep(&mut self, temp: f32, precision: usize) {
        for _ in 0..self.polymers.len() {
            self.take_mc_step(temp, precision);
        }
        self.log_energies(precision)
    }
}

impl Model {
    pub fn make_svg(&self) -> svg::Document {
        let mut group = Group::new();
        for polymer in self.polymers.iter() {
            group.append(polymer.as_path(0.1))
        }

        let mut document = Document::new().set("viewBox", (0, 0, WIDTH, HEIGHT));
        document.append(group);
        document.append(self.boundary.as_rect(0.1));
        document
    }
}
