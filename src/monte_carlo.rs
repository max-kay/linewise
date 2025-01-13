use std::path::Path;
use std::{fs, io::Write, path::PathBuf};

use rand::prelude::*;
use serde::{Deserialize, Serialize};
use svg::{node::element::Group, Document, Node};

use crate::plt;
use crate::Vector;
use crate::{BorrowedSegment, OwnedPolymer, PolymerRef, PolymerStorage};
use crate::{Energy, QuadTree};
use crate::{MyRng, Rect, Samples2d};
use crate::{CLEAR_LINE, MOVE_UP};

mod builders;

use builders::{ModelBuilder, ParamBuilder};

#[derive(Serialize, Deserialize)]
pub struct ModelParameters {
    polymer_count: usize,
    segment_len: f32,
    max_segments: usize,
    interaction_radius: f32,
    energy_factors: Energy,
    precision: usize,
    temp_range: (f32, f32),
    temp_steps: usize,
    sweeps_per_temp: usize,
    make_plots: bool,
    save_parameters: bool,
    save_start_svg: bool,
    save_step_svg: bool,
    save_end_svg: bool,
}

impl ModelParameters {
    pub fn new() -> ParamBuilder {
        Default::default()
    }

    pub fn get_temps(&self) -> Vec<f32> {
        if self.temp_steps == 1 {
            let mut vec = Vec::new();
            vec.push(self.temp_range.0);
            return vec;
        }
        (0..self.temp_steps)
            .map(|i| {
                self.temp_range.0
                    * ((self.temp_range.1 / self.temp_range.0).ln()
                        / ((self.temp_steps - 1) as f32)
                        * i as f32)
                        .exp()
            })
            .collect()
    }
}

pub struct Model {
    field: Samples2d<Vector>,
    potential: Samples2d<f32>,
    storage: PolymerStorage,
    polymers: QuadTree<PolymerRef>,
    params: ModelParameters,
    boundary: Rect,
    energies: Vec<Energy>,
    lower_count: u32,
    accepted_count: u32,
    rejected_count: u32,
    transition_scale: f32,
    rates: Vec<[f32; 3]>,
    rng: MyRng,
    log_dir: PathBuf,
}

impl Model {
    pub fn new() -> ModelBuilder {
        ModelBuilder::default()
    }
}

// the energy terms
impl Model {
    fn potential_term(&self, potential_sum: &mut f32, position: Vector, der_norm: f32) {
        if let Some(sample) = (&self.potential).get_sample(position) {
            *potential_sum += sample * der_norm
        }
    }

    fn length_term(&self, length_sum: &mut f32, der_norm: f32) {
        *length_sum += der_norm;
    }

    fn bending_term(&self, bending_sum: &mut f32, der: Vector, der2: Vector, der_norm: f32) {
        *bending_sum += (der.x * der2.y - der2.x * der.y).powi(2) / der_norm.powi(5);
    }

    fn field_term(&self, field_sum: &mut f32, position: Vector, der: Vector) {
        if let Some(vector) = (&self.field).get_sample(position) {
            *field_sum -= der.dot(vector).abs();
        }
    }

    fn interaction_term(
        &self,
        interaction_sum: &mut f32,
        position: Vector,
        der_norm: f32,
        others_filter_condidion: &impl Fn(&PolymerRef) -> bool,
    ) {
        for other in self
            .polymers
            .querry_iter(|bounds| {
                bounds
                    .add_radius((&self.params).interaction_radius)
                    .contains_point(position)
            })
            .filter(|&o| others_filter_condidion(o))
        {
            let mut inner_sum = 0.0;

            for segment in self.storage.get_borrowed_segments(&other) {
                for (pos, other_der) in segment
                    .position_iter(self.params.precision)
                    .zip(segment.derivative_iter(self.params.precision))
                {
                    let dist = (pos - position).norm();
                    inner_sum += self.interaction_potential(dist) * other_der.norm();
                }
            }
            *interaction_sum += inner_sum * der_norm;
        }
    }

    fn interaction_potential(&self, dist: f32) -> f32 {
        if dist < (&self.params).interaction_radius {
            1.0 / dist.powi(3)
        } else {
            0.0
        }
    }

    pub fn boundary_term(&self, boundary_sum: &mut f32, position: Vector) {
        let signed_dist = self.boundary.signed_distance(position);
        if signed_dist > 0.0 {
            *boundary_sum = f32::INFINITY;
        } else {
            *boundary_sum += 1.0 / signed_dist.powi(2)
        }
    }
}

// energy calculation methods
impl Model {
    pub fn calculate_energy_from_segment<'a>(
        &self,
        segment: BorrowedSegment<'_>,
        others_filter_condidion: impl Fn(&PolymerRef) -> bool,
    ) -> Energy {
        let mut length_sum = 0.0;
        let mut bending_sum = 0.0;
        let mut potential_sum = 0.0;
        let mut field_sum = 0.0;
        let mut interaction_sum = 0.0;
        let mut boundary_sum = 0.0;
        for (position, der, der2) in segment.all_iters(self.params.precision) {
            let der_norm = der.norm();
            self.length_term(&mut length_sum, der_norm);
            // length_sum += der_norm;
            self.bending_term(&mut bending_sum, der, der2, der_norm);
            self.potential_term(&mut potential_sum, position, der_norm);
            self.field_term(&mut field_sum, position, der);
            self.interaction_term(
                &mut interaction_sum,
                position,
                der_norm,
                &others_filter_condidion,
            );
            self.boundary_term(&mut boundary_sum, position);
        }

        let len = length_sum / self.params.precision as f32;
        Energy {
            strain_energy: self.params.energy_factors.strain_energy
                * len
                * ((self.params.segment_len - len) / self.params.segment_len).powi(2)
                / 2.0,
            bending_energy: self.params.energy_factors.bending_energy * bending_sum
                / self.params.precision as f32,
            potential_energy: self.params.energy_factors.potential_energy * potential_sum
                / self.params.precision as f32,
            field_energy: self.params.energy_factors.field_energy * field_sum
                / self.params.precision as f32,
            interaction_energy: self.params.energy_factors.interaction_energy * interaction_sum
                / (self.params.precision * self.params.precision) as f32,
            boundary_energy: self.params.energy_factors.boundary_energy * boundary_sum
                / self.params.precision as f32,
        }
    }

    pub fn calculate_energy_for_this(&self, polymer: &OwnedPolymer) -> Energy {
        let mut energy = Energy::zero();
        for segment in polymer.get_borrowed_segments() {
            energy += self.calculate_energy_from_segment(segment, |p| *polymer == *p)
        }
        energy
    }

    pub fn calculate_energy_for_tot(&self, polymer: &PolymerRef) -> Energy {
        let mut energy = Energy::zero();
        for segment in self.storage.get_borrowed_segments(&polymer) {
            energy += self.calculate_energy_from_segment(segment, |p| *polymer < *p)
        }
        energy
    }

    pub fn log_energies(&mut self) {
        let mut summed_energy = Energy::zero();
        for polymer in self.polymers.iter() {
            summed_energy += self.calculate_energy_for_tot(polymer)
        }
        self.energies.push(summed_energy)
    }
}

impl Model {
    pub fn take_mc_step(&mut self, temp: f32) {
        let old = self.storage.read(self.polymers.pop_random(&mut self.rng));
        let new = old.vary(self.transition_scale, &mut self.rng);

        let e_0 = self.calculate_energy_for_this(&old).sum();
        let e_1 = self.calculate_energy_for_this(&new).sum();
        let d_e = e_1 - e_0;

        if d_e < 0.0 {
            self.lower_count += 1;
            let this_ref = self.storage.overwrite_polymer(new);
            self.polymers.insert(this_ref);
        } else if self.rng.gen::<f32>() < (-d_e / temp).exp() {
            self.accepted_count += 1;
            let this_ref = self.storage.overwrite_polymer(new);
            self.polymers.insert(this_ref)
        } else {
            self.rejected_count += 1;
            let this_ref = self.storage.revalidate_ref(old);
            self.polymers.insert(this_ref)
        }
    }

    pub fn make_mc_sweep(&mut self, temp: f32) {
        for _ in 0..self.polymers.len() {
            self.take_mc_step(temp);
        }
    }

    pub fn run(&mut self, format: (f32, f32), margin: f32) -> anyhow::Result<()> {
        if self.params.save_start_svg {
            self.save_svg_doc("img_start.svg", format, margin)?
        }

        for (i, temp) in self.params.get_temps().into_iter().enumerate() {
            println!(
                "Model running at temp = {} | {}/{}",
                temp,
                i + 1,
                self.params.temp_steps
            );
            std::io::stdout().flush()?;
            self.clear_logs();
            for j in 1..=self.params.sweeps_per_temp {
                // TODO: dynamically adjust movement of polymers when total_acceptance_rate !~= 0.5
                if j % 50 == 0 {
                    print!(
                        "{}running {}/{}",
                        CLEAR_LINE, j, self.params.sweeps_per_temp
                    );
                    std::io::stdout().flush()?;
                }
                self.make_mc_sweep(temp);
                // TODO: should this be part of parameters?
                if (self.rejected_count as f32 / self.polymers.len() as f32) < 0.5 {
                    self.transition_scale *= 1.4;
                }
                if (self.rejected_count as f32 / self.polymers.len() as f32) > 0.6 {
                    self.transition_scale /= 1.4;
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
            print!("{}", CLEAR_LINE);
            std::io::stdout().flush()?;
            if self.params.make_plots {
                self.make_all_plots(&format!("Temp {}", temp), &format!("{}", i))?;
            }
            if self.params.save_step_svg {
                self.save_svg_doc(&format!("img_{}_{}.svg", i, temp), format, margin)?;
            }
            print!("{}{}", MOVE_UP, CLEAR_LINE);
            std::io::stdout().flush()?;
        }
        if self.params.save_end_svg && !self.params.save_step_svg {
            self.save_svg_doc("img_end.svg", format, margin)?;
        }
        if self.params.save_parameters {
            let path = self.log_dir.join("parameters.ron");
            fs::write(
                path,
                ron::ser::to_string_pretty(&self.params, ron::ser::PrettyConfig::default())?,
            )?
        }
        println!("Finished Running");
        Ok(())
    }
}

impl Model {
    pub fn count_polymers(&self) -> usize {
        self.polymers.len()
    }

    pub fn clear_logs(&mut self) {
        self.energies = Vec::new();
        self.rates = Vec::new();
    }

    const LINE_WIDTH_FACTOR: f32 = 0.1;
    pub fn make_svg_group(&self) -> (Group, Rect) {
        let mut group = Group::new();
        let mut polymers = Group::new();
        for polymer in &self.polymers {
            polymers.append(
                self.storage
                    .as_path(polymer, Self::LINE_WIDTH_FACTOR * self.params.segment_len),
            )
        }
        group.append(polymers);
        group.append(
            self.boundary
                .as_svg(Self::LINE_WIDTH_FACTOR * self.params.segment_len),
        );
        (group, self.boundary)
    }

    pub fn make_svg_doc(&self, format: (f32, f32), margin: f32) -> Document {
        let mut doc = Document::new().set("viewBox", (0, 0, format.0, format.1));
        // // for debuging
        // doc.append(
        //     Rect::new(0.0, format.0, 0.0, format.1)
        //         .as_svg(0.0)
        //         .set("fill", "gray")
        //         .set("stroke", "none"),
        // );

        let (group, rect) = self.make_svg_group();
        let scale = ((format.0 - 2.0 * margin) / rect.width())
            .min((format.1 - 2.0 * margin) / rect.height());
        doc.append(group.set(
            "transform",
            format!(
                "translate({} {}) scale({})",
                (format.0 - rect.width() * scale) / 2.0,
                (format.1 - rect.height() * scale) / 2.0,
                scale
            ),
        ));
        doc
    }

    pub fn save_svg_doc(
        &self,
        path: impl AsRef<Path>,
        format: (f32, f32),
        margin: f32,
    ) -> std::io::Result<()> {
        svg::save(self.log_dir.join(path), &self.make_svg_doc(format, margin))
    }

    pub fn make_all_plots(&self, caption: &str, name: &str) -> anyhow::Result<()> {
        plt::simple_line(
            &self
                .energies
                .iter()
                .map(|val| val.sum())
                .collect::<Vec<_>>(),
            caption,
            self.log_dir.join(&format!("{}_tot.png", name)),
        )?;

        plt::divergent_chart(
            &self.energies,
            caption,
            self.log_dir.join(&format!("{}_all.png", name)),
        )?;

        plt::rate_plot(
            &self.rates,
            caption,
            self.log_dir.join(&format!("{}_rates.png", name)),
        )?;
        Ok(())
    }
}
