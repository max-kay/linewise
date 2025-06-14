use std::path::{Path, PathBuf};

use chrono::Utc;
use convolve2d::{convolve2d, kernel};
use image::DynamicImage;

use super::{AcceptanceCounter, METHODS, Model, ModelParameters, SvgParams, TransitionScales};
use common::energy::Energy;
use common::quad_tree::{Bounded, QuadTree, Rect};
use common::sampler::Samples2d;
use common::storage::SplineStorage;
use common::{Spline, SplineRef, Vector};
use random::Rng;

pub struct ParamBuilder {
    segment_len: Option<f32>,
    max_segments: Option<usize>,
    spline_count: Option<usize>,
    interaction_radius: Option<f32>,

    energy_factors: Option<Energy>,

    precision: Option<usize>,
    temp_range: Option<(f32, f32)>,
    temp_steps: Option<usize>,
    sweeps_per_temp: Option<usize>,

    save_parameters: bool,
    save_start_svg: bool,
    save_step_svg: bool,
    save_end_svg: bool,
    make_plots: bool,
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
            interaction_radius: self.interaction_radius.unwrap_or(0.01),
            spline_count: self.spline_count.unwrap_or(900),
            segment_len: self.segment_len.unwrap_or(0.03),
            max_segments: self.max_segments.unwrap_or(4),

            energy_factors: self.energy_factors.unwrap_or(default_energy),
            temp_range: self.temp_range.unwrap_or((1.0, 0.005)),

            precision: self.precision.unwrap_or(12),
            temp_steps: self.temp_steps.unwrap_or(10),
            sweeps_per_temp: self.sweeps_per_temp.unwrap_or(150),

            save_parameters: self.save_parameters,
            save_start_svg: self.save_start_svg,
            save_step_svg: self.save_step_svg,
            save_end_svg: self.save_end_svg,
            make_plots: self.make_plots,
        }
    }
    pub fn spline_count(mut self, spline_count: usize) -> Self {
        self.spline_count = Some(spline_count);
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
        self.save_parameters = true;
        self
    }
    pub fn unset_save_full(mut self) -> Self {
        self.save_parameters = false;
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
            save_parameters: true,
            save_start_svg: false,
            save_step_svg: false,
            save_end_svg: true,
            spline_count: None,
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

pub struct ModelBuilder {
    field: Option<Samples2d<Vector>>,
    potential: Option<Samples2d<f32>>,
    params: Option<ModelParameters>,
    svg_params: Option<SvgParams>,
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

    pub fn add_params(mut self, params: ModelParameters) -> Self {
        self.params = Some(params);
        self
    }

    pub fn add_svg_params(mut self, params: SvgParams) -> Self {
        self.svg_params = Some(params);
        self
    }

    pub fn build(self) -> anyhow::Result<Model> {
        let params = self.params.unwrap_or(ModelParameters::new().build());
        let aspect = self.aspect_ratio.unwrap_or(1.0);
        let boundary = Rect::new(0.0, aspect.sqrt(), 0.0, 1.0 / aspect.sqrt());
        let mut rng = random::new_rng();

        let log_dir = self.log_dir.unwrap_or_else(|| {
            Path::new("out").join(Utc::now().format("%Y-%m-%d_%H-%M").to_string())
        });
        std::fs::create_dir_all(&log_dir)?;

        let mut storage = SplineStorage::new();
        let mut splines: QuadTree<SplineRef> = QuadTree::new();
        let max_iterations = params.spline_count * 100;
        // TODO: think about the influence of this algorithm for length distr of the splines
        // and if I even care
        for _ in 0..max_iterations {
            if splines.len() == params.spline_count {
                break;
            }
            let spline = Spline::new_random(
                boundary
                    .add_radius(-(params.max_segments as f32) * params.segment_len)
                    .from_box_coords((rng.random(), rng.random())),
                params.segment_len,
                rng.random_range(1..=params.max_segments),
                &mut rng,
            );
            let mut intersection = false;
            // FIXME: checking for intersections is not enough needs to be same as energy
            // calcualtion
            'outer: for other in splines.query_intersects(spline.bounding_box()) {
                for o_segment in storage.get_segments(other) {
                    if spline.intersects(&o_segment, params.precision * 20) {
                        intersection = true;
                        break 'outer;
                    }
                }
            }
            if !intersection {
                let polyref = storage.add_spline(spline);
                splines.insert(polyref);
            }
        }
        if splines.len() != params.spline_count {
            anyhow::bail!(
                "couldn't place {} nonintersecting splines in {} iterations",
                params.spline_count,
                max_iterations
            )
        }
        Ok(Model {
            field: self.field.unwrap_or(Samples2d::new_filled(
                Vector::new(0.0, 0.0),
                1,
                1,
                boundary,
            )),
            potential: self
                .potential
                .unwrap_or(Samples2d::new_filled(0.0, 1, 1, boundary)),

            splines: splines,
            markings: storage.default_spline_info(),
            storage: storage,
            params,
            svg_params: self.svg_params.unwrap_or(SvgParams {
                format: (
                    // with aspect = 1/sqrt(2) this is a4
                    100.0 / 4.0 * aspect.sqrt(),
                    100.0 / 4.0 / aspect.sqrt(),
                ),
                margins: (1.2, 1.2),
            }),
            boundary,
            energies: Vec::new(),
            acceptance_couter: AcceptanceCounter::zeros(),
            transition_scales: TransitionScales([0.005; METHODS]),
            rates: Vec::new(),
            rng,
            log_dir,
        })
    }
}

impl Default for ModelBuilder {
    fn default() -> Self {
        Self {
            field: None,
            potential: None,
            params: None,
            svg_params: None,
            aspect_ratio: None,
            log_dir: None,
        }
    }
}
