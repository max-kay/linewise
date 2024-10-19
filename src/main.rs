use linewise::Model;

fn main() {
    let mut model = Model::make_example();
    // svg::save("out/test_start.svg", &model.make_svg()).unwrap();
    for (i, temp) in (0..100).map(|i| (i, 1.0 / (i + 1) as f32)) {
        model.make_mc_sweep(temp, 10);
        // svg::save(format!("out/test_{}.svg", i), &model.make_svg()).unwrap();
    }
    // svg::save("out/test_end.svg", &model.make_svg()).unwrap();
}
