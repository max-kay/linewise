use linewise::Model;

const PRECISION: usize = 10;

fn main() {
    let mut model = Model::make_example(PRECISION);
    // svg::save("out/test_start.svg", &model.make_svg()).unwrap();
    for (_, temp) in (0..100).map(|i| (i, 1.0 / (i + 1) as f32)) {
        model.make_mc_sweep(temp, PRECISION);
        model.log_energies(PRECISION)

        // svg::save(format!("out/test_{}.svg", i), &model.make_svg()).unwrap();
    }
    // svg::save("out/test_end.svg", &model.make_svg()).unwrap();
}
