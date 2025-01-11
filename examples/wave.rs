use linewise::{
    format::A4, quad_tree::Rect, utils::clear_or_create_dir, Model, ModelParameters, Vector,
};
use nalgebra::Rotation2;

const SWEEPS: i32 = 50;
const T_STEPS: u32 = 10;

const T_START: f32 = 1.0;
const T_END: f32 = 0.005;

fn temp(i: u32) -> f32 {
    T_START * ((T_END / T_START).ln() / ((T_STEPS - 1) as f32) * i as f32).exp()
}

fn main() {
    let parameters = ModelParameters::new().build();
    let format = A4;

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
    let bounds = Rect::new(0.0, A4.0, 0.0, A4.1);

    let mut model = Model::new()
        .field_from_fn(field, bounds, (2000, 2000))
        .potential_from_fn(potential, bounds, (2000, 2000))
        .add_parameters(parameters)
        .build();
    model.run(format, 20.0)
}
