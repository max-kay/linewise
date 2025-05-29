use image::Luma;
use serde::{Deserialize, Serialize};

use crate::{Vector, quad_tree::Rect};

#[derive(Serialize, Deserialize)]
pub struct Samples2d<T> {
    samples: Vec<T>,
    width: usize,
    height: usize,
    bounds: Rect,
}

impl<T> Samples2d<T> {
    pub fn new(samples: Vec<T>, width: usize, height: usize, bounds: Rect) -> Self {
        assert_eq!(width * height, samples.len());
        Self {
            samples,
            width,
            height,
            bounds,
        }
    }

    pub fn from_fn(func: impl Fn(Vector) -> T, width: usize, height: usize, bounds: Rect) -> Self {
        let mut samples = Vec::with_capacity(width * height);
        for j in 0..height {
            for i in 0..width {
                let position =
                    bounds.from_box_coords((i as f32 / width as f32, j as f32 / height as f32));
                samples.push(func(position))
            }
        }
        Self {
            samples,
            width,
            height,
            bounds,
        }
    }
}

impl<T: Clone> Samples2d<T> {
    pub fn new_filled(fill: T, width: usize, height: usize, bounds: Rect) -> Self {
        Self {
            samples: vec![fill; width * height],
            width,
            height,
            bounds,
        }
    }

    pub fn get_bounds(&self) -> Rect {
        self.bounds
    }

    pub fn map<S>(&self, func: impl Fn(&T) -> S) -> Samples2d<S> {
        Samples2d {
            samples: self.samples.iter().map(func).collect(),
            width: self.width,
            height: self.height,
            bounds: self.bounds,
        }
    }

    pub fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds
    }
}

impl<T> Samples2d<T> {
    fn calculate_idx(&self, position: Vector) -> Option<usize> {
        let (frac_x, frac_y) = self.bounds.to_box_coords(position);
        if frac_x < 0.0 || frac_y < 0.0 {
            return None;
        }
        let idx_x = (frac_x * self.width as f32).round() as usize;
        let idx_y = (frac_y * self.height as f32).round() as usize;
        if idx_x >= self.width || idx_y >= self.height {
            return None;
        }
        Some(idx_x + self.width * idx_y)
    }

    pub fn get_sample(&self, position: Vector) -> Option<&T> {
        let idx = self.calculate_idx(position)?;
        // SAFETY calculate_idx produces valid ideces only
        unsafe { Some(self.samples.get_unchecked(idx)) }
    }

    pub fn get_sample_mut(&mut self, position: Vector) -> Option<&mut T> {
        let idx = self.calculate_idx(position)?;
        Some(&mut self.samples[idx])
    }
}

impl Samples2d<f32> {
    pub fn as_img(&self, path: &str) {
        let img = image::ImageBuffer::<Luma<u8>, Vec<u8>>::from_vec(
            self.width as u32,
            self.height as u32,
            self.samples.iter().map(|val| (val * 256.0) as u8).collect(),
        )
        .unwrap();
        img.save(path).unwrap();
    }
}
