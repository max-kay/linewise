use std::fs::File;
use std::io::BufReader;

use image::{ImageFormat, RgbImage};
use linewise::{Model, ModelParameters};

fn main() {
    let file = File::open("./in/fern.jpg").unwrap();
    let buf = BufReader::new(file);
    let img: RgbImage = image::load(buf, ImageFormat::Jpeg).unwrap().into();

    let width = img.width();
    let height = img.height();
    let format = (width as f32, height as f32);

    let parameters = ModelParameters::new()
        .segment_len(0.012)
        .polymer_count(5000)
        .sweeps_per_temp(200)
        .set_save_step_svg()
        .unset_make_plots()
        .build();
    let mut model = Model::new()
        .add_samples_from_img(img)
        .add_parameters(parameters)
        .build();
    model.run(format, 0.0)
}
