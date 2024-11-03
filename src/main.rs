use std::{fs, path::Path};

use linewise::{format::A4, Model, Vector, MM};
use nalgebra::Rotation2;

const LINE_WIDTH: f32 = 1.0 * MM;
const PRECISION: usize = 15;

const T_START: f32 = 1.0;
const T_END: f32 = 0.05;

const T_STEPS: u32 = 10;

fn temp(i: u32) -> f32 {
    T_START * ((T_END / T_START).ln() / ((T_STEPS - 1) as f32) * i as f32).exp()
}

const SWEEPS: i32 = 200;

fn clear_or_create_dir(path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = path.as_ref();

    if !(path.exists() && path.is_dir()) {
        return fs::create_dir_all(path);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            fs::remove_file(entry_path)?;
        } else if entry_path.is_dir() {
            fs::remove_dir_all(entry_path)?;
        }
    }

    Ok(())
}

fn main() {
    clear_or_create_dir("./out").unwrap();
    let center = Vector::new(A4.0 / 2.0, A4.1 / 2.0);

    let potential = |pos: Vector| -> f32 { ((pos - center).norm() / 100.0).sin().powi(2) };
    let field = |pos: Vector| -> Vector {
        let result = Rotation2::new(std::f32::consts::FRAC_PI_2) * (pos - center).normalize();
        if result.x.is_nan() || result.y.is_nan() {
            Vector::new(0.0, 0.0)
        } else {
            result
        }
    };

    let mut model = Model::from_fns(potential, field, A4, 40.0 * MM, PRECISION);

    svg::save("out/test_start.svg", &model.make_svg(LINE_WIDTH)).unwrap();

    for (i, temp) in (0..T_STEPS).map(temp).enumerate() {
        println!("Model running at temp = {} | {}/{}", temp, i + 1, T_STEPS);
        for j in 1..=SWEEPS {
            if j % 50 == 0 {
                println!("running {}/{}", j, SWEEPS);
            }
            model.make_mc_sweep(temp);
        }
        model
            .make_all_plots(&format!("Temp {}", temp), &format!("{}", i))
            .unwrap();
        model.clear_logs();
        svg::save(
            format!("out/img_{}_{}.svg", i, temp),
            &model.make_svg(LINE_WIDTH),
        )
        .unwrap();
    }
    // model.make_all_plots(&format!("Test",), "test").unwrap();
}
