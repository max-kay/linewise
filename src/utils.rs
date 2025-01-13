use rand::prelude::*;
use std::{f32::consts::TAU, fs, io::Result, path::Path};

use crate::{MyRng, Vector};

pub fn clear_or_create_dir(path: impl AsRef<Path>) -> Result<()> {
    if !(path.as_ref().exists() && path.as_ref().is_dir()) {
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

pub fn gaussian_vector(rng: &mut MyRng) -> Vector {
    // Box Muller transform
    let theta = TAU * rng.gen::<f32>();
    let radius = (-2.0 * rng.gen::<f32>().ln()).sqrt();
    match rng.gen_range(0..4) {
        0 => Vector::new(radius * theta.cos(), radius * theta.sin()),
        1 => -Vector::new(radius * theta.cos(), radius * theta.sin()),
        2 => Vector::new(radius * theta.sin(), radius * theta.cos()),
        3 => -Vector::new(radius * theta.sin(), radius * theta.cos()),
        _ => unreachable!(),
    }
}

pub fn rand_unit(rng: &mut MyRng) -> Vector {
    gaussian_vector(rng).normalize()
}

#[cfg(test)]
mod test {
    use super::*;
    const N: usize = 100_000;
    #[test]
    fn gauss_vec_test() {
        let mut rng = MyRng::from_entropy();
        let mut vec = Vector::zeros();
        for _ in 0..N {
            vec = vec + gaussian_vector(&mut rng);
        }
        assert!(
            (vec.norm() / N as f32) < 0.01,
            "was {}",
            vec.norm() / N as f32
        )
    }
}
