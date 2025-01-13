use std::fs::File;
use std::io::BufReader;

use image::ImageFormat;
use linewise::{energy::Energy, Model, ModelParameters};

fn main() -> anyhow::Result<()> {
    let file = File::open("./in/fern.jpg")?;
    let buf = BufReader::new(file);
    let img = image::load(buf, ImageFormat::Jpeg)?;

    let width = img.width();
    let height = img.height();
    let format = (width as f32, height as f32);
    let default_energy = Energy {
        strain_energy: 1000.0,
        bending_energy: 0.01,
        potential_energy: 100.0,
        field_energy: 1000.0,
        interaction_energy: 500.0,
        boundary_energy: 0.0001,
    };

    let parameters = ModelParameters::new()
        .segment_len(0.01)
        .max_segments(8)
        .polymer_count(500)
        .sweeps_per_temp(400)
        .energy_factors(default_energy)
        .temp_steps(12)
        .set_save_start_svg()
        .set_save_step_svg()
        .build();
    let mut model = Model::new()
        .add_samples_from_img(img)
        .add_params(parameters)
        .build();
    model.run(format, 0.0)
}
