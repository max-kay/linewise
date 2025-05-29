use std::f32::consts::TAU;
use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::{fs, io::Write, path::PathBuf};

use anyhow::anyhow;
use random::{MyRng, Rng, gaussian_vector};
use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Deserialize, Serialize};
use svg::{Document, Node, node::element::Group};

use common::{
    BorrowedSegment, CLEAR_LINE, Energy, MOVE_UP, OwnedPolymer, PIXEL_PER_CM, PolymerRef,
    PolymerStorage, QuadTree, Rect, Samples2d, Vector, plt, quad_tree::Bounded,
};

mod builder;

use builder::{ModelBuilder, ParamBuilder};

pub const METHODS: usize = 6;

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

pub fn vary_polymer(
    polymer: &mut OwnedPolymer,
    transition_scale: [f32; METHODS],
    rng: &mut MyRng,
) -> usize {
    let method = rng.random_range(0..METHODS);
    match method {
        0 => polymer.translate(gaussian_vector(rng) * transition_scale[0]),
        1 => polymer.rotate((rng.random::<f32>() - 0.5) * transition_scale[1] * TAU),
        2 => polymer.rotate_segment(
            rng.random_range(0..polymer.count_segments()),
            (rng.random::<f32>() - 0.5) * transition_scale[2] * TAU,
        ),
        3 => polymer.scales_vecs(1.0 - (rng.random::<f32>() - 0.5) * 2.0 * transition_scale[3]),
        4 => polymer.scales_vecs_random(transition_scale[4], rng),
        5 => polymer.stretch(1.0 - (rng.random::<f32>() - 0.5) * 2.0 * transition_scale[5] / 1.0),
        METHODS.. => unreachable!(),
    }
    method
}
pub struct SvgParams {
    format: (f32, f32),
    margins: (f32, f32),
}

// lower accepted rejected
pub struct AcceptanceCounter([[u32; 3]; METHODS]);

impl AcceptanceCounter {
    pub const LOWER: usize = 0;
    pub const ACCEPTED: usize = 1;
    pub const REJECTED: usize = 2;
    pub fn zeros() -> Self {
        Self([[0; 3]; METHODS])
    }

    pub fn increase(&mut self, method: usize, result: usize) {
        self.0[method][result] += 1;
    }

    pub fn update_transitions(&self, transitions: &mut TransitionScales) {
        for (rates, transition) in self.0.iter().zip(transitions.0.iter_mut()) {
            let rate = rates[Self::REJECTED] as f32 / rates.iter().sum::<u32>() as f32;
            if rate < 0.5 {
                *transition = (*transition * 1.4).min(1.0);
            } else if rate > 0.6 {
                *transition = (*transition / 1.4).max(0.0001);
            }
        }
    }

    pub fn to_rates(&self) -> [f32; 3] {
        let mut out = [0.0; 3];
        let mut tot: u32 = 0;
        for method in self.0 {
            tot += method.iter().sum::<u32>();
            for i in 0..3 {
                out[i] += method[i] as f32;
            }
        }
        out.iter_mut().for_each(|val| *val /= tot as f32);
        out
    }

    pub fn clear(&mut self) {
        *self = Self([[0; 3]; METHODS]);
    }
}

#[derive(Debug)]
pub struct TransitionScales([f32; METHODS]);

impl Display for TransitionScales {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        for value in self.0 {
            string = format!("{} {:.4}", string, value)
        }
        write!(f, "[{}]", &string[1..])
    }
}

pub struct Model {
    field: Samples2d<Vector>,
    potential: Samples2d<f32>,
    storage: PolymerStorage,
    polymers: QuadTree<PolymerRef>,
    params: ModelParameters,
    svg_params: SvgParams,
    boundary: Rect,
    energies: Vec<Energy>,
    acceptance_couter: AcceptanceCounter,
    transition_scales: TransitionScales,
    rates: Vec<[f32; 3]>,
    rng: MyRng,
    log_dir: PathBuf,
}

impl Model {
    pub fn new() -> ModelBuilder {
        ModelBuilder::default()
    }

    pub fn initialize(&mut self) -> anyhow::Result<()> {
        let mut storage = PolymerStorage::new();
        let mut polymers: QuadTree<PolymerRef> = QuadTree::new();
        let max_iterations = self.params.polymer_count * 100;
        // TODO: think about the influence of this algorithm for length distr of the polymers
        // and if I even care
        for _ in 0..max_iterations {
            if polymers.len() == self.params.polymer_count {
                break;
            }
            let polymer = OwnedPolymer::new_random(
                self.boundary
                    .add_radius(-(self.params.max_segments as f32) * self.params.segment_len)
                    .from_box_coords((self.rng.random(), self.rng.random())),
                self.params.segment_len,
                self.rng.random_range(1..=self.params.max_segments),
                &mut self.rng,
            );
            let mut intersection = false;
            for other in polymers.query_intersects(polymer.bounding_box()) {
                for o_segment in storage.get_borrowed_segments(other) {
                    intersection |= polymer.intersects(&o_segment, self.params.precision)
                }
            }
            if !intersection {
                let polyref = storage.add_polymer(polymer);
                polymers.insert(polyref);
            }
        }
        if polymers.len() != self.params.polymer_count {
            anyhow::bail!(
                "couldn't place {} nonintersecting polymers in {} iterations",
                self.params.polymer_count,
                max_iterations
            )
        }
        self.storage = storage;
        self.polymers = polymers;
        Ok(())
    }
}

impl Model {
    fn print_sweep_status(&self, sweep: usize) -> anyhow::Result<()> {
        print!(
            "{}running sweep {:>3}/{}    transition_scales: {}",
            CLEAR_LINE, sweep, self.params.sweeps_per_temp, self.transition_scales
        );
        std::io::stdout().flush()?;
        Ok(())
    }

    fn print_temp_status(&self, step: usize, temp: f32) -> anyhow::Result<()> {
        println!(
            "{}{}{}running at temperature {:.3}   step {}/{:>2}",
            CLEAR_LINE, MOVE_UP, CLEAR_LINE, temp, step, self.params.temp_steps
        );
        std::io::stdout().flush()?;
        Ok(())
    }
}

// the energy terms
impl Model {
    fn potential_term(&self, potential_sum: &mut f32, position: Vector, der_norm: f32) {
        if let Some(sample) = self.potential.get_sample(position) {
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
        if let Some(vector) = self.field.get_sample(position) {
            *field_sum -= der.dot(vector).abs();
        }
    }

    fn interaction_potential(&self, dist: f32) -> f32 {
        if dist < self.params.interaction_radius {
            let ratio = self.params.interaction_radius / 2.0_f32.powf(1.0 / 6.0) / dist;
            ratio.powi(12) - ratio.powi(6) - 1.0 / 4.0
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
    pub fn calculate_energy_from_segment<'a>(&self, segment: BorrowedSegment<'_>) -> Energy {
        let mut length_sum = 0.0;
        let mut bending_sum = 0.0;
        let mut potential_sum = 0.0;
        let mut field_sum = 0.0;
        let mut boundary_sum = 0.0;
        for (position, der, der2) in segment.all_iters(self.params.precision) {
            let der_norm = der.norm();
            self.length_term(&mut length_sum, der_norm);
            self.bending_term(&mut bending_sum, der, der2, der_norm);
            self.potential_term(&mut potential_sum, position, der_norm);
            self.field_term(&mut field_sum, position, der);
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
            interaction_energy: 0.0,
            boundary_energy: self.params.energy_factors.boundary_energy * boundary_sum
                / self.params.precision as f32,
        }
    }

    pub fn calculate_energy_for_this(&self, polymer: &OwnedPolymer) -> Energy {
        let mut energy = Energy::zero();
        for segment in polymer.get_borrowed_segments() {
            energy += self.calculate_energy_from_segment(segment)
        }
        let mut interaction_sum = 0.0;
        for other in self
            .polymers
            .query_intersects(
                polymer
                    .bounding_box()
                    .add_radius(self.params.interaction_radius),
            )
            .filter(|&p| *polymer == *p)
            .collect::<Vec<&PolymerRef>>()
        {
            for other_segment in self.storage.get_borrowed_segments(other) {
                for (o_pos, o_der) in other_segment.pos_and_der_iter(self.params.precision) {
                    let mut inner_sum = 0.0;
                    for my_segment in polymer.get_borrowed_segments() {
                        for (m_pos, m_der) in my_segment.pos_and_der_iter(self.params.precision) {
                            inner_sum +=
                                self.interaction_potential((m_pos - o_pos).norm()) * m_der.norm();
                        }
                    }
                    interaction_sum += inner_sum * o_der.norm()
                }
            }
        }
        energy.interaction_energy = self.params.energy_factors.interaction_energy * interaction_sum
            / self.params.precision.pow(2) as f32;
        energy
    }

    pub fn calculate_energy_for_tot(&self, polymer: &PolymerRef) -> Energy {
        let mut energy = Energy::zero();

        for segment in self.storage.get_borrowed_segments(&polymer) {
            energy += self.calculate_energy_from_segment(segment)
        }
        let mut interaction_sum = 0.0;
        for other in self
            .polymers
            .query_intersects(
                polymer
                    .bounding_box()
                    .add_radius(self.params.interaction_radius),
            )
            .filter(|&p| *polymer < *p)
            .collect::<Vec<&PolymerRef>>()
        {
            for other_segment in self.storage.get_borrowed_segments(other) {
                for (o_pos, o_der) in other_segment.pos_and_der_iter(self.params.precision) {
                    let mut inner_sum = 0.0;
                    for my_segment in self.storage.get_borrowed_segments(polymer) {
                        for (m_pos, m_der) in my_segment.pos_and_der_iter(self.params.precision) {
                            inner_sum +=
                                self.interaction_potential((m_pos - o_pos).norm()) * m_der.norm();
                        }
                    }
                    interaction_sum += inner_sum * o_der.norm()
                }
            }
        }
        energy.interaction_energy = self.params.energy_factors.interaction_energy * interaction_sum
            / self.params.precision.pow(2) as f32;
        energy
    }

    pub fn calc_tot_energy(&self) -> Energy {
        let mut summed_energy = Energy::zero();
        for polymer in self.polymers.iter() {
            let res = self.calculate_energy_for_tot(polymer);
            // assert!(res.is_finite(), "Energy was infinite at {:?}", res);
            if !res.is_finite() {
                println!("Energy was not finite at {:?}", res);
            }

            summed_energy += res;
        }
        summed_energy
    }

    pub fn log_energies(&mut self) {
        self.energies.push(self.calc_tot_energy())
    }
}

impl Model {
    pub fn take_mc_step(&mut self, temp: f32) {
        let mut polymer = self.storage.read(self.polymers.pop_random(&mut self.rng));
        let e_0 = self.calculate_energy_for_this(&polymer).sum();

        let method = vary_polymer(&mut polymer, self.transition_scales.0, &mut self.rng);

        let e_1 = self.calculate_energy_for_this(&polymer).sum();

        let d_e = e_1 - e_0;

        if d_e < 0.0 {
            self.acceptance_couter
                .increase(method, AcceptanceCounter::LOWER);
            self.polymers
                .insert(self.storage.overwrite_polymer(polymer));
        } else if self.rng.random::<f32>() < (-d_e / temp).exp() {
            self.acceptance_couter
                .increase(method, AcceptanceCounter::ACCEPTED);
            self.polymers
                .insert(self.storage.overwrite_polymer(polymer))
        } else {
            self.acceptance_couter
                .increase(method, AcceptanceCounter::REJECTED);
            self.polymers.insert(self.storage.revalidate_ref(polymer))
        }
    }

    pub fn run_at_temp(
        &mut self,
        temp: f32,
        tx: Option<&Sender<PolymerStorage>>,
    ) -> anyhow::Result<()> {
        self.clear_logs();

        for j in 1..=self.params.sweeps_per_temp {
            self.print_sweep_status(j)?;
            for _ in 0..self.polymers.len() {
                self.take_mc_step(temp);
            }

            if let Some(tx) = tx {
                tx.send(self.storage.clone())?
            }

            self.acceptance_couter
                .update_transitions(&mut self.transition_scales);

            self.rates.push(self.acceptance_couter.to_rates());
            self.acceptance_couter.clear();

            if self.params.make_plots {
                self.log_energies()
            }
        }
        Ok(())
    }

    pub fn run(
        mut self,
        display_opts: Option<(Sender<PolymerStorage>, Arc<AtomicBool>)>,
    ) -> anyhow::Result<()> {
        if self.params.save_parameters {
            let path = self.log_dir.join("parameters.ron");
            fs::write(
                path,
                to_string_pretty(&self.params, PrettyConfig::default())?,
            )?
        }

        self.initialize()?;

        if let Some((tx, _)) = &display_opts {
            tx.send(self.storage.clone())?
        }

        if self.params.save_start_svg {
            self.save_svg_doc("img_start.svg")?
        }
        anyhow::ensure!(
            self.calc_tot_energy().is_finite(),
            "initial energy needs to be finite"
        );

        for (i, temp) in self.params.get_temps().into_iter().enumerate() {
            self.print_temp_status(i + 1, temp)?;
            self.run_at_temp(temp, display_opts.as_ref().map(|(tx, _)| tx))?;

            if self.params.make_plots {
                self.make_all_plots(&format!("Temp {}", temp), &format!("{}", i))?;
            }

            if self.params.save_step_svg {
                self.save_svg_doc(&format!("img_{}_{}.svg", i, temp))?;
            }
            if let Some((_, stop_flag)) = display_opts.as_ref() {
                if stop_flag.load(Ordering::Relaxed) {
                    if self.params.save_end_svg && !self.params.save_step_svg {
                        self.save_svg_doc("img_end.svg")?;
                    }
                    return Err(anyhow!("Stopped running"));
                }
            }
        }
        if self.params.save_end_svg && !self.params.save_step_svg {
            self.save_svg_doc("img_end.svg")?;
        }

        println!("\nFinished Running");
        Ok(())
    }
}

impl Model {
    pub fn count_polymers(&self) -> usize {
        self.polymers.len()
    }

    pub fn get_bounds(&self) -> Rect {
        self.boundary
    }

    pub fn clear_logs(&mut self) {
        self.energies = Vec::new();
        self.rates = Vec::new();
    }

    const LINE_WIDTH_FACTOR: f32 = 0.1;
    pub fn calc_linewidth(&self) -> f32 {
        Self::LINE_WIDTH_FACTOR * self.params.segment_len
    }
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

    pub fn make_svg_doc(&self) -> Document {
        let mut doc = Document::new()
            .set("width", format!("{}cm", self.svg_params.format.0))
            .set("height", format!("{}cm", self.svg_params.format.1));
        let (group, rect) = self.make_svg_group();
        let scale = ((self.svg_params.format.0 - 2.0 * self.svg_params.margins.0) / rect.width())
            .min((self.svg_params.format.1 - 2.0 * self.svg_params.margins.1) / rect.height());
        doc.append(group.set(
            "transform",
            format!(
                "translate({} {}) scale({})",
                (self.svg_params.format.0 - rect.width() * scale) / 2.0 * PIXEL_PER_CM,
                (self.svg_params.format.1 - rect.height() * scale) / 2.0 * PIXEL_PER_CM,
                scale * PIXEL_PER_CM
            ),
        ));
        doc
    }

    pub fn save_svg_doc(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        svg::save(self.log_dir.join(path), &self.make_svg_doc())
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
