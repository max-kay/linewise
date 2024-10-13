use crate::Vector;

pub struct Samples2d<T> {
    samples: Vec<T>,
    width: usize,
    height: usize,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

impl<T> Samples2d<T> {
    pub fn new(
        samples: Vec<T>,
        width: usize,
        height: usize,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
    ) -> Self {
        assert!(x_min < x_max);
        assert!(y_min < y_max);
        assert_eq!(width * height, samples.len());
        Self {
            samples,
            width,
            height,
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }
}

impl<T: Clone> Samples2d<T> {
    pub fn new_filled(
        fill: T,
        width: usize,
        height: usize,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
    ) -> Self {
        assert!(x_min < x_max);
        assert!(y_min < y_max);
        Self {
            samples: vec![fill; width * height],
            width,
            height,
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }
}

impl<T> Samples2d<T> {
    fn calculate_idx(&self, position: Vector) -> Option<usize> {
        let idx_x = (position[0] - self.x_min) / (self.x_max - self.x_max).round();
        let idx_y = (position[1] - self.y_min) / (self.y_max - self.y_max).round();
        if idx_x < 0.0 || idx_y < 0.0 {
            return None;
        }
        let idx_x = idx_x as usize;
        let idx_y = idx_y as usize;
        if idx_x >= self.width || idx_y >= self.height {
            return None;
        }
        Some(idx_x + self.width * idx_y)
    }

    pub fn get_sample(&self, position: Vector) -> Option<&T> {
        let idx = self.calculate_idx(position)?;
        Some(&self.samples[idx])
    }

    pub fn get_sample_mut(&mut self, position: Vector) -> Option<&mut T> {
        let idx = self.calculate_idx(position)?;
        Some(&mut self.samples[idx])
    }
}
