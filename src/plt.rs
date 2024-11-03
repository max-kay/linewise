use plotters::prelude::*;
use std::{error::Error, path::Path};

use crate::polymer::Energy;
const PLOT_FORMAT: (u32, u32) = (4048, 3027);

const STROKE_WIDTH: u32 = 2;
const S_STROKE_WIDTH: u32 = 1;
const FONT: u32 = 160;
const S_FONT: u32 = 80;
const T_FONT: u32 = 60;

pub fn simple_line(
    values: &[f32],
    caption: &str,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(&path, PLOT_FORMAT).into_drawing_area();
    root.fill(&WHITE)?;

    let x_range = 0..values.len();
    let y_range = min(&values)..max(&values);
    let mut chart = ChartBuilder::on(&root)
        .margin(200)
        .caption(caption, ("sans-serif", FONT))
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(x_range, y_range)?;

    chart
        .configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_mesh()
        .x_label_formatter(&|v| format!("{:.1}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .label_style(("sans-serif", T_FONT))
        .y_desc("Energy")
        .x_desc("Sweeps")
        .draw()?;
    chart.draw_series(LineSeries::new(
        values.iter().cloned().enumerate(),
        BLACK.stroke_width(STROKE_WIDTH),
    ))?;
    root.present()?;

    Ok(())
}

pub fn divergent_chart(
    energies: &[Energy],
    caption: &str,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(&path, PLOT_FORMAT).into_drawing_area();
    root.fill(&WHITE)?;

    let first = energies.first().unwrap().as_array();
    let y_min = energies
        .iter()
        .map(|val| {
            val.as_array()
                .iter()
                .enumerate()
                .map(|(i, val)| val - first[i])
                .min_by(|a, b| a.total_cmp(b))
                .unwrap()
        })
        .min_by(|a, b| a.total_cmp(b))
        .unwrap();
    let y_max = energies
        .iter()
        .map(|val| {
            val.as_array()
                .iter()
                .enumerate()
                .map(|(i, val)| val - first[i])
                .max_by(|a, b| a.total_cmp(b))
                .unwrap()
        })
        .max_by(|a, b| a.total_cmp(b))
        .unwrap();

    let mut chart = ChartBuilder::on(&root)
        .margin(200)
        .caption(caption, ("sans-serif", FONT))
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(0..energies.len(), y_min..y_max)?;

    chart
        .configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_mesh()
        .x_label_formatter(&|v| format!("{:.1}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .label_style(("sans-serif", T_FONT))
        .y_desc("Energy")
        .x_desc("Sweeps")
        .draw()?;

    for i in 0..6 {
        chart
            .draw_series(LineSeries::new(
                energies
                    .iter()
                    .map(|val| val.as_array()[i] - first[i])
                    .enumerate(),
                Palette99::pick(i).stroke_width(STROKE_WIDTH),
            ))?
            .label(format!("{} energy", Energy::NAMES[i]))
            .legend(move |(x, y)| {
                Rectangle::new(
                    [(x - 15, y - 15), (x + 15, y + 15)],
                    Palette99::pick(i).stroke_width(STROKE_WIDTH).filled(),
                )
            });
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.2))
        .label_font(("sans-serif", S_FONT))
        .border_style(BLACK.stroke_width(S_STROKE_WIDTH))
        .draw()?;
    root.present()?;

    Ok(())
}

pub fn rate_plot<const N: usize>(
    values: &[[f32; N]],
    caption: &str,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(&path, PLOT_FORMAT).into_drawing_area();
    root.fill(&WHITE)?;

    let x_range = 0..values.len();
    let y_range = 0.0..1.0_f32;
    let mut chart = ChartBuilder::on(&root)
        .margin(200)
        .caption(caption, ("sans-serif", FONT))
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(x_range, y_range)?;

    chart
        .configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_mesh()
        .x_label_formatter(&|v| format!("{:.1}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .label_style(("sans-serif", T_FONT))
        .x_desc("Sweeps")
        .y_desc("Rate")
        .draw()?;
    let names = ["lower", "accepted", "rejected"];
    for i in 0..N {
        let this = values.iter().map(|val| val[i]).enumerate();
        chart
            .draw_series(LineSeries::new(
                this,
                Palette99::pick(i).stroke_width(STROKE_WIDTH),
            ))?
            .label(names[i])
            .legend(move |(x, y)| {
                Rectangle::new(
                    [(x - 20, y - 20), (x + 20, y + 20)],
                    Palette99::pick(i).stroke_width(STROKE_WIDTH).filled(),
                )
            });
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .label_font(("sans-serif", S_FONT))
        .border_style(BLACK.stroke_width(S_STROKE_WIDTH))
        .draw()?;
    root.present()?;

    Ok(())
}

fn min(values: &[f32]) -> f32 {
    *values.iter().min_by(|a, b| (**a).total_cmp(*b)).unwrap()
}
fn max(values: &[f32]) -> f32 {
    *values.iter().max_by(|a, b| (**a).total_cmp(*b)).unwrap()
}
