use linewise::{energy::Energy, quad_tree::Rect, Model, ModelParameters, Vector};
use nalgebra::Rotation2;

pub const A4: (f32, f32) = (2480.0, 3508.0);
fn main() -> anyhow::Result<()> {
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

    let energy_factors = Energy {
        strain_energy: 100.0,
        bending_energy: 0.001,
        potential_energy: 10.0,
        field_energy: 100.0,
        interaction_energy: 0.00000001,
        boundary_energy: 0.00001,
    };

    let parameters = ModelParameters::new()
        .segment_len(0.01)
        .max_segments(5)
        .polymer_count(500)
        .unset_make_plots()
        .sweeps_per_temp(400)
        .energy_factors(energy_factors)
        .interaction_radius(0.3)
        .precision(30)
        .temp_steps(2)
        .set_save_start_svg()
        .set_save_step_svg()
        .build();
    let model = Model::new()
        .field_from_fn(field, bounds, (2000, 2000))
        .potential_from_fn(potential, bounds, (2000, 2000))
        .add_params(parameters)
        .build();
    model.run(None)
}
