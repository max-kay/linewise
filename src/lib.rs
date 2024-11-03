use std::{error::Error, f32::consts::TAU};

use nalgebra::{Rotation2, Vector2};
use rand::prelude::*;
use rand::rngs::SmallRng;
use rand_distr::Normal;
use svg::{node::element::Group, Document, Node};

pub mod picture;
pub mod plt;
pub mod polymer;
pub mod quad_tree;
pub mod sampler;
pub mod spline;

use polymer::{Energy, Polymer};
use quad_tree::{Bounded, QuadTree, Rect};
use sampler::Samples2d;

pub type MyRng = SmallRng;

pub mod format {
    pub const A4: (f32, f32) = (2480.0, 3508.0);
    pub const A3: (f32, f32) = (3508.0, 4960.0);
}

pub const PX_PER_MM: f32 = 3.7795275591;
pub const MM: f32 = PX_PER_MM;
pub const MM_PER_PX: f32 = 1.0 / PX_PER_MM;

const SAMPLE_WIDTH: usize = 2480;
const SAMPLE_HEIGHT: usize = 3508;

const ROOT_POLYMER_COUNT: usize = 30;
const SEGMENT_LEN: f32 = 0.0113;

const BOUNDARY_INTERACTION_LAYER: f32 = 0.2;

const INTERACTION_RADIUS: f32 = 0.1;

pub type Vector = Vector2<f32>;
pub type Rotation = Rotation2<f32>;

impl Bounded for Vector {
    fn bounding_box(&self) -> quad_tree::Rect {
        Rect::new(self.x, self.x, self.y, self.y)
    }
}

pub fn rand_unit(rng: &mut MyRng) -> Vector {
    let normal = Normal::new(0.0, 1.0).unwrap();
    Vector::new(normal.sample(rng), normal.sample(rng)).normalize()
}

pub struct Model {
    field: Samples2d<Vector>,
    potential: Samples2d<f32>,
    polymers: QuadTree<Polymer>,
    paper_format: (f32, f32),
    scale_factor: f32,
    precision: usize,
    boundary: Rect,
    energies: Vec<Energy>,
    lower_count: u32,
    accepted_count: u32,
    rejected_count: u32,
    rates: Vec<[f32; 3]>,
    rng: MyRng,
}

impl Model {
    pub fn from_fns(
        potential: impl Fn(Vector) -> f32,
        field: impl Fn(Vector) -> Vector,
        paper_format: (f32, f32),
        margin: f32,
        precision: usize,
    ) -> Self {
        let mut rng = MyRng::from_rng(thread_rng()).unwrap();
        let scale_factor =
            ((paper_format.0 - 2.0 * margin) * (paper_format.1 - 2.0 * margin)).sqrt();

        let boundary = Rect::new(
            margin / scale_factor,
            (paper_format.0 - margin) / scale_factor,
            margin / scale_factor,
            (paper_format.1 - margin) / scale_factor,
        );

        let field = Samples2d::from_fn(
            |vec| field(vec * scale_factor),
            SAMPLE_WIDTH,
            SAMPLE_HEIGHT,
            boundary,
        );
        let potential = Samples2d::from_fn(
            |vec| potential(vec * scale_factor),
            SAMPLE_WIDTH,
            SAMPLE_HEIGHT,
            boundary,
        );

        let mut polymers = Vec::new();
        for i in 0..ROOT_POLYMER_COUNT {
            for j in 0..ROOT_POLYMER_COUNT {
                let position = boundary.from_box_coords((
                    0.1 + i as f32 / (ROOT_POLYMER_COUNT - 1) as f32 * 0.8,
                    0.1 + j as f32 / (ROOT_POLYMER_COUNT - 1) as f32 * 0.8,
                ));
                polymers.push(Polymer::new_random(
                    position,
                    SEGMENT_LEN,
                    rng.gen_range(1..=4),
                    precision,
                    &mut rng,
                ));
            }
        }
        let polymers = QuadTree::new(polymers);

        Self {
            field,
            potential,
            polymers,
            boundary,
            paper_format,
            precision,
            scale_factor,
            energies: Vec::new(),
            lower_count: 0,
            accepted_count: 0,
            rejected_count: 0,
            rates: Vec::new(),
            rng,
        }
    }

    pub fn log_energies(&mut self) {
        let mut summed_energy = Energy::zero();
        for polymer in self.polymers.iter() {
            summed_energy += polymer.all_energies(
                &self.potential,
                &self.field,
                self.polymers
                    .query_intersects(polymer.bounding_box().add_radius(INTERACTION_RADIUS)),
                self.precision,
                self.boundary,
            );
        }
        // TODO this can be done better!
        self.energies.push(summed_energy.half_interaction())
    }

    pub fn calculate_energy_from(&self, polymer: &Polymer) -> f32 {
        polymer
            .all_energies(
                &self.potential,
                &self.field,
                self.polymers
                    .query_intersects(polymer.bounding_box().add_radius(INTERACTION_RADIUS)),
                self.precision,
                self.boundary,
            )
            .sum()
    }

    pub fn count_polymers(&self) -> usize {
        self.polymers.len()
    }

    pub fn take_mc_step(&mut self, temp: f32) {
        let old = self.polymers.pop_random(&mut self.rng);
        let e_0 = self.calculate_energy_from(&old);
        let mut new = old.clone();
        match self.rng.gen_range(0..=2) {
            0 => new.translate(rand_unit(&mut self.rng) * self.rng.gen::<f32>()),
            1 => new.rotate(self.rng.gen::<f32>() * TAU),
            2 => new.bend_at_segment(
                self.rng.gen_range(0..new.count_segments()),
                self.rng.gen::<f32>() * TAU,
            ),
            _ => unreachable!(),
        }
        let e_1 = self.calculate_energy_from(&new);
        let d_e = e_1 - e_0;
        if d_e < 0.0 {
            self.lower_count += 1;
            self.polymers.insert(new);
            return;
        }
        if d_e == f32::INFINITY {
            self.rejected_count += 1;
            self.polymers.insert(old);
            return;
        }
        if self.rng.gen::<f32>() < (-d_e / temp).exp() {
            self.accepted_count += 1;
            self.polymers.insert(new)
        } else {
            self.rejected_count += 1;
            self.polymers.insert(old)
        }
    }

    pub fn make_mc_sweep(&mut self, temp: f32) {
        for _ in 0..self.polymers.len() {
            self.take_mc_step(temp);
        }
        self.rates.push([
            self.lower_count as f32 / self.polymers.len() as f32,
            self.accepted_count as f32 / self.polymers.len() as f32,
            self.rejected_count as f32 / self.polymers.len() as f32,
        ]);
        self.lower_count = 0;
        self.accepted_count = 0;
        self.rejected_count = 0;
        self.log_energies()
    }

    pub fn clear_logs(&mut self) {
        self.energies = Vec::new();
        self.rates = Vec::new();
    }
}

impl Model {
    pub fn make_svg(&self, line_width: f32) -> svg::Document {
        let line_width = line_width / self.scale_factor;
        let mut group = Group::new().set("transform", format!("scale({})", self.scale_factor));
        for polymer in &self.polymers {
            group.append(polymer.as_path(line_width))
        }

        let mut document = Document::new().set(
            "viewBox",
            (0.0, 0.0, self.paper_format.0, self.paper_format.1),
        );
        document.append(group);
        document.append(
            self.boundary
                .as_rect(line_width)
                .set("transform", format!("scale({})", self.scale_factor)),
        );
        document.append(
            Rect::new(0.0, self.paper_format.0, 0.0, self.paper_format.1).as_rect(line_width),
        );
        document
    }

    pub fn make_all_plots(&self, caption: &str, name: &str) -> Result<(), Box<dyn Error>> {
        plt::simple_line(
            &self
                .energies
                .iter()
                .map(|val| val.sum())
                .collect::<Vec<_>>(),
            caption,
            format!("out/{}_tot.png", name),
        )?;

        plt::divergent_chart(&self.energies, caption, format!("out/{}_all.png", name))?;

        plt::rate_plot(&self.rates, caption, format!("out/{}_acceptance.png", name))?;
        Ok(())
    }
}
