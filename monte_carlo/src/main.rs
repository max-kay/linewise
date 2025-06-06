use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::{fs::File, thread};

use common::energy::Energy;
use image::ImageFormat;
use minifb::{Key, Window, WindowOptions};

use monte_carlo::{Model, ModelParameters};

fn main() -> anyhow::Result<()> {
    let file = File::open("./in/fern.jpg")?;
    let buf = BufReader::new(file);
    let img = image::load(buf, ImageFormat::Jpeg)?;

    let default_energy = Energy {
        strain_energy: 1000.0,
        bending_energy: 0.01,
        potential_energy: 100.0,
        field_energy: 1000.0,
        interaction_energy: 500.0,
        boundary_energy: 0.0001,
    };

    let parameters = ModelParameters::new()
        .segment_len(0.005)
        .interaction_radius(0.02)
        .max_segments(8)
        .spline_count(2000)
        .sweeps_per_temp(500)
        .energy_factors(default_energy)
        .temp_steps(12)
        .precision(30)
        .unset_make_plots()
        .set_save_start_svg()
        .set_save_step_svg()
        .build();
    let model = Model::new()
        .add_samples_from_img(img)
        .add_params(parameters)
        .build()?;

    run_in_window(model)?;
    // model.run(None)?;
    Ok(())
}

fn run_in_window(model: Model) -> anyhow::Result<()> {
    let bounds = model.get_bounds();
    let line_width = model.calc_linewidth();
    let (tx, rx) = mpsc::channel();

    let window_scale = 700.0;
    let width = (bounds.width() * window_scale) as usize;
    let height = (bounds.height() * window_scale) as usize;
    let mut window = Window::new(
        "Simulation Display",
        width,
        height,
        WindowOptions::default(),
    )?;

    let stop_flag = Arc::new(AtomicBool::new(false));
    let sim_flag = Arc::clone(&stop_flag);

    let sim_thread = thread::spawn(move || -> anyhow::Result<()> {
        model.run(Some((tx, Arc::clone(&sim_flag))))?;
        sim_flag.store(true, Ordering::Relaxed);
        Ok(())
    });

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Ok(splines) = rx.try_recv() {
            let pixmap = splines.rasterize(line_width, window_scale, width as u32, height as u32);
            let buffer: Vec<_> = pixmap
                .data()
                .chunks(4)
                .map(|rgba| {
                    let r = rgba[0] as u32;
                    let g = rgba[1] as u32;
                    let b = rgba[2] as u32;
                    let a = rgba[3] as u32;
                    (a << 24) | (r << 16) | (g << 8) | b
                })
                .collect();
            window.update_with_buffer(&buffer, width, height).unwrap();
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }
        }
    }
    stop_flag.store(true, Ordering::Relaxed);
    match sim_thread.join() {
        Err(_) => anyhow::bail!("failed to join simulation thread"),
        Ok(_) => Ok(()),
    }
}
