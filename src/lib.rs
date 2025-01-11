use std::{
    error::Error,
    f32::consts::TAU,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::Utc;
use convolve2d::{convolve2d, kernel};
use image::DynamicImage;
use nalgebra::{Rotation2, Vector2};
use rand::prelude::*;
use rand::rngs::SmallRng;
use serde::{Deserialize, Serialize};
use svg::{node::element::Group, Document, Node};

pub mod plt;
pub mod polymer;
pub mod quad_tree;
pub mod sampler;
pub mod spline;
pub mod utils;

use polymer::{Energy, Polymer};
use quad_tree::{Bounded, QuadTree, Rect};
use sampler::Samples2d;

pub type MyRng = SmallRng;

const CLEAR_LINE: &'static str = "\x1B[2K\r";
const MOVE_UP: &'static str = "\x1B[A\r";

// TODO: change to points
pub mod format {
    pub const A5: (f32, f32) = (1754.0, 2480.0);
    pub const A4: (f32, f32) = (2480.0, 3508.0);
    pub const A3: (f32, f32) = (3508.0, 4960.0);
}

pub type Vector = Vector2<f32>;
pub type Rotation = Rotation2<f32>;

impl Bounded for Vector {
    fn bounding_box(&self) -> quad_tree::Rect {
        Rect::new(self.x, self.x, self.y, self.y)
    }
}

pub fn gaussian_vector(rng: &mut MyRng) -> Vector {
    // Box Muller transform
    let theta = TAU * rng.gen::<f32>();
    let radius = (-2.0 * rng.gen::<f32>().ln()).sqrt();
    Vector::new(radius * theta.cos(), radius * theta.sin())
}

pub fn rand_unit(rng: &mut MyRng) -> Vector {
    gaussian_vector(rng).normalize()
}

pub const PX_PER_MM: f32 = 3.7795275591;
pub const MM: f32 = PX_PER_MM;
pub const MM_PER_PX: f32 = 1.0 / PX_PER_MM;

pub struct ParamBuilder {
    polymer_count: Option<usize>,
    segment_len: Option<f32>,
    max_segments: Option<usize>,
    interaction_radius: Option<f32>,
    energy_factors: Option<Energy>,
    precision: Option<usize>,
    temp_range: Option<(f32, f32)>,
    temp_steps: Option<usize>,
    sweeps_per_temp: Option<usize>,
    make_plots: bool,
    save_full: bool,
    save_start_svg: bool,
    save_step_svg: bool,
    save_end_svg: bool,
}

impl ParamBuilder {
    pub fn build(self) -> ModelParameters {
        let default_energy = Energy {
            strain_energy: 1000000.0,
            bending_energy: 0.01,
            potential_energy: 100.0,
            field_energy: 1000.0,
            interaction_energy: 500000.0,
            boundary_energy: 0.0001,
        };
        ModelParameters {
            polymer_count: self.polymer_count.unwrap_or(900),
            segment_len: self.segment_len.unwrap_or(0.03),
            max_segments: self.max_segments.unwrap_or(4),
            interaction_radius: self.interaction_radius.unwrap_or(0.01),
            energy_factors: self.energy_factors.unwrap_or(default_energy),
            precision: self.precision.unwrap_or(12),
            temp_range: self.temp_range.unwrap_or((1.0, 0.005)),
            temp_steps: self.temp_steps.unwrap_or(10),
            sweeps_per_temp: self.sweeps_per_temp.unwrap_or(150),
            make_plots: self.make_plots,
            save_full: self.save_full,
            save_start_svg: self.save_start_svg,
            save_step_svg: self.save_step_svg,
            save_end_svg: self.save_end_svg,
        }
    }
    pub fn polymer_count(mut self, polymer_count: usize) -> Self {
        self.polymer_count = Some(polymer_count);
        self
    }
    pub fn segment_len(mut self, segment_len: f32) -> Self {
        self.segment_len = Some(segment_len);
        self
    }
    pub fn max_segments(mut self, max_segments: usize) -> Self {
        self.max_segments = Some(max_segments);
        self
    }
    pub fn interaction_radius(mut self, interaction_radius: f32) -> Self {
        self.interaction_radius = Some(interaction_radius);
        self
    }
    pub fn energy_factors(mut self, energy_factors: Energy) -> Self {
        self.energy_factors = Some(energy_factors);
        self
    }
    pub fn precision(mut self, precision: usize) -> Self {
        self.precision = Some(precision);
        self
    }
    pub fn temp_range(mut self, temp_range: (f32, f32)) -> Self {
        self.temp_range = Some(temp_range);
        self
    }
    pub fn temp_steps(mut self, temp_steps: usize) -> Self {
        self.temp_steps = Some(temp_steps);
        self
    }
    pub fn sweeps_per_temp(mut self, sweeps_per_temp: usize) -> Self {
        self.sweeps_per_temp = Some(sweeps_per_temp);
        self
    }
    pub fn set_make_plots(mut self) -> Self {
        self.make_plots = true;
        self
    }
    pub fn unset_make_plots(mut self) -> Self {
        self.make_plots = false;
        self
    }
    pub fn set_save_full(mut self) -> Self {
        self.save_full = true;
        self
    }
    pub fn unset_save_full(mut self) -> Self {
        self.save_full = false;
        self
    }
    pub fn set_save_start_svg(mut self) -> Self {
        self.save_start_svg = true;
        self
    }
    pub fn unset_save_start_svg(mut self) -> Self {
        self.save_start_svg = false;
        self
    }
    pub fn set_save_step_svg(mut self) -> Self {
        self.save_step_svg = true;
        self
    }
    pub fn unset_save_step_svg(mut self) -> Self {
        self.save_step_svg = false;
        self
    }
    pub fn set_save_end_svg(mut self) -> Self {
        self.save_end_svg = true;
        self
    }
    pub fn unset_save_end_svg(mut self) -> Self {
        self.save_end_svg = false;
        self
    }
}
impl Default for ParamBuilder {
    fn default() -> Self {
        Self {
            make_plots: true,
            save_full: false,
            save_start_svg: false,
            save_step_svg: false,
            save_end_svg: true,
            polymer_count: None,
            segment_len: None,
            max_segments: None,
            interaction_radius: None,
            energy_factors: None,
            precision: None,
            temp_range: None,
            temp_steps: None,
            sweeps_per_temp: None,
        }
    }
}

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
    save_full: bool,
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
            return vec![self.temp_range.0];
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

pub struct ModelBuilder {
    field: Option<Samples2d<Vector>>,
    potential: Option<Samples2d<f32>>,
    model_parameters: Option<ModelParameters>,
    log_dir: Option<PathBuf>,
    aspect_ratio: Option<f32>,
}

impl ModelBuilder {
    pub fn add_samples_from_img(mut self, img: DynamicImage) -> Self {
        let gray = img.to_luma32f();
        let (w, h, x) = convolve2d(&gray, &kernel::sobel::x::<f32>()).into_parts();
        let (_, _, y) = convolve2d(&gray, &kernel::sobel::y::<f32>()).into_parts();
        let width = gray.width() as usize;
        let height = gray.height() as usize;
        let aspect = width as f32 / height as f32;
        let boundary = Rect::new(0.0, aspect.sqrt(), 0.0, 1.0 / aspect.sqrt());

        let potential = Samples2d::new(
            gray.pixels().map(|val| val.0[0]).collect(),
            width,
            height,
            boundary,
        );

        let field = Samples2d::new(
            x.iter()
                .zip(y.iter())
                // the division by 8 is for the max value a pixel could be
                .map(|(&x, &y)| Vector::new(x / 8.0, y / 8.0))
                .collect(),
            w,
            h,
            boundary,
        );
        // potential.as_img("pot.png");
        // field.map(|vec| vec.x).as_img("x.png");
        // field.map(|vec| vec.y).as_img("y.png");
        self.aspect_ratio = Some(aspect);
        self.field = Some(field);
        self.potential = Some(potential);
        self
    }

    pub fn potential_from_fn(
        mut self,
        field: impl Fn(Vector) -> f32,
        sample_region: Rect,
        sample_dim: (usize, usize),
    ) -> Self {
        let aspect_ratio = sample_region.aspect_ratio();
        if let Some(aspect) = self.aspect_ratio {
            assert_eq!(
                aspect, aspect_ratio,
                "tried to add field with different aspect ratio"
            );
        }
        let mut potential =
            Samples2d::from_fn(|vec| field(vec), sample_dim.0, sample_dim.1, sample_region);
        potential.set_bounds(Rect::new(
            0.0,
            aspect_ratio.sqrt(),
            0.0,
            1.0 / aspect_ratio.sqrt(),
        ));
        self.potential = Some(potential);
        self
    }

    pub fn field_from_fn(
        mut self,
        field: impl Fn(Vector) -> Vector,
        sample_region: Rect,
        sample_dim: (usize, usize),
    ) -> Self {
        let aspect_ratio = sample_region.aspect_ratio();
        if let Some(aspect) = self.aspect_ratio {
            assert!(
                (aspect - aspect_ratio).abs() < 0.001,
                "tried to add field with different aspect ratio"
            );
        } else {
            self.aspect_ratio = Some(aspect_ratio)
        }
        let mut field =
            Samples2d::from_fn(|vec| field(vec), sample_dim.0, sample_dim.1, sample_region);
        field.set_bounds(Rect::new(
            0.0,
            aspect_ratio.sqrt(),
            0.0,
            1.0 / aspect_ratio.sqrt(),
        ));
        self.field = Some(field);
        self
    }

    pub fn add_parameters(mut self, parameters: ModelParameters) -> Self {
        self.model_parameters = Some(parameters);
        self
    }

    pub fn build(self) -> Model {
        let parameters = self
            .model_parameters
            .unwrap_or(ModelParameters::new().build());
        let aspect = self.aspect_ratio.unwrap_or(1.0);
        let boundary = Rect::new(0.0, aspect.sqrt(), 0.0, 1.0 / aspect.sqrt());
        let mut rng = MyRng::from_rng(thread_rng()).unwrap();
        let mut polymers = Vec::new();
        for _ in 0..parameters.polymer_count {
            polymers.push(Polymer::new_random(
                boundary
                    .add_radius(-0.15)
                    .from_box_coords((rng.gen(), rng.gen())),
                parameters.segment_len,
                rng.gen_range(1..=parameters.max_segments),
                parameters.precision,
                &mut rng,
            ));
        }
        let polymers = QuadTree::new(polymers);
        let log_dir = self.log_dir.unwrap_or_else(|| {
            Path::new("out").join(Utc::now().format("%Y-%m-%d_%H-%M").to_string())
        });
        utils::clear_or_create_dir(&log_dir).unwrap();

        Model {
            field: self.field.unwrap_or(Samples2d::new_filled(
                Vector::new(0.0, 0.0),
                1,
                1,
                boundary,
            )),
            potential: self
                .potential
                .unwrap_or(Samples2d::new_filled(0.0, 1, 1, boundary)),
            polymers,
            model_parameters: parameters,
            boundary,
            energies: Vec::new(),
            lower_count: 0,
            accepted_count: 0,
            rejected_count: 0,
            rates: Vec::new(),
            rng,
            log_dir,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Model {
    field: Samples2d<Vector>,
    potential: Samples2d<f32>,
    polymers: QuadTree<Polymer>,
    model_parameters: ModelParameters,
    boundary: Rect,
    energies: Vec<Energy>,
    lower_count: u32,
    accepted_count: u32,
    rejected_count: u32,
    rates: Vec<[f32; 3]>,
    #[serde(skip, default = "MyRng::from_entropy")]
    rng: MyRng,
    log_dir: PathBuf,
}

impl Model {
    pub fn new() -> ModelBuilder {
        ModelBuilder {
            field: None,
            potential: None,
            model_parameters: None,
            aspect_ratio: None,
            log_dir: None,
        }
    }

    pub fn log_energies(&mut self) {
        let mut summed_energy = Energy::zero();
        for polymer in self.polymers.iter() {
            summed_energy += polymer.all_energies(
                &self.potential,
                &self.field,
                self.polymers.query_intersects(
                    polymer
                        .bounding_box()
                        .add_radius(self.model_parameters.interaction_radius),
                ),
                self.boundary,
                &self.model_parameters,
            );
        }
        // TODO: this can be done better!
        self.energies.push(summed_energy.half_interaction())
    }

    pub fn calculate_energy_from(&self, polymer: &Polymer) -> f32 {
        polymer
            .all_energies(
                &self.potential,
                &self.field,
                self.polymers.query_intersects(
                    polymer
                        .bounding_box()
                        .add_radius(self.model_parameters.interaction_radius),
                ),
                self.boundary,
                &self.model_parameters,
            )
            .sum()
    }

    pub fn count_polymers(&self) -> usize {
        self.polymers.len()
    }

    pub fn take_mc_step(&mut self, temp: f32) {
        let old = self.polymers.pop_random(&mut self.rng);
        let new = old.vary(temp, &mut self.rng);

        let e_0 = self.calculate_energy_from(&old);
        let e_1 = self.calculate_energy_from(&new);
        let d_e = e_1 - e_0;

        if d_e < 0.0 {
            self.lower_count += 1;
            self.polymers.insert(new);
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
    }

    pub fn run(&mut self, format: (f32, f32), margin: f32) {
        if self.model_parameters.save_start_svg {
            svg::save(
                self.log_dir.join("img_start.svg"),
                &self.make_svg_doc(format, margin),
            )
            .unwrap();
        }

        for (i, temp) in self.model_parameters.get_temps().into_iter().enumerate() {
            println!(
                "Model running at temp = {} | {}/{}",
                temp,
                i + 1,
                self.model_parameters.temp_steps
            );
            std::io::stdout().flush().unwrap();
            self.clear_logs();
            for j in 1..=self.model_parameters.sweeps_per_temp {
                // TODO: dynamically adjust movement of polymers when total_acceptance_rate !~= 0.5
                if j % 50 == 0 {
                    print!(
                        "{}running {}/{}",
                        CLEAR_LINE, j, self.model_parameters.sweeps_per_temp
                    );
                    std::io::stdout().flush().unwrap();
                }
                self.make_mc_sweep(temp);
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
            std::io::stdout().flush().unwrap();
            if self.model_parameters.make_plots {
                self.make_all_plots(&format!("Temp {}", temp), &format!("{}", i))
                    .unwrap();
            }
            if self.model_parameters.save_step_svg {
                svg::save(
                    self.log_dir.join(&format!("img_{}_{}.svg", i, temp)),
                    &self.make_svg_doc(format, margin),
                )
                .unwrap();
            }
            print!("{}{}", MOVE_UP, CLEAR_LINE);
            std::io::stdout().flush().unwrap();
        }
        if self.model_parameters.save_end_svg && !self.model_parameters.save_step_svg {
            svg::save(
                self.log_dir.join(&format!("img_end.svg")),
                &self.make_svg_doc(format, margin),
            )
            .unwrap();
        }
        if self.model_parameters.save_full {
            let path = self.log_dir.join("full.ron");
            fs::write(path, ron::to_string(&self).unwrap()).unwrap()
        }
        println!("Finished Running")
    }

    pub fn clear_logs(&mut self) {
        self.energies = Vec::new();
        self.rates = Vec::new();
    }
}

impl Model {
    const LINE_WIDTH_FACTOR: f32 = 0.1;
    pub fn make_svg_group(&self) -> (Group, Rect) {
        let mut group = Group::new();
        let mut polymers = Group::new();
        for polymer in &self.polymers {
            polymers.append(
                polymer.as_path(Self::LINE_WIDTH_FACTOR * self.model_parameters.segment_len),
            )
        }
        group.append(polymers);
        group.append(
            self.boundary
                .as_svg(Self::LINE_WIDTH_FACTOR * self.model_parameters.segment_len),
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

    pub fn make_all_plots(&self, caption: &str, name: &str) -> Result<(), Box<dyn Error>> {
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
