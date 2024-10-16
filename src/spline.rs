use nalgebra::ComplexField;
use svg::node::element::{path::Data, Path};

use crate::{
    quad_tree::{Bounded, BoundingBox},
    MyRng, Vector,
};
pub struct BSpline {
    points: Vec<Vector>,
    vectors: Vec<Vector>,
}

impl BSpline {
    pub fn new(points: Vec<Vector>, vectors: Vec<Vector>) -> Self {
        assert_eq!(
            points.len(),
            vectors.len(),
            "points and vectors must be of same lenght"
        );
        assert!(
            points.len() >= 2,
            "must contain at least two points and vectors"
        );
        Self { points, vectors }
    }

    pub fn push(&mut self, point: Vector, vector: Vector) {
        self.points.push(point);
        self.vectors.push(vector);
    }

    pub fn count_segments(&self) -> usize {
        self.points.len() - 1
    }

    pub fn new_random(
        approx_center: Vector,
        approx_len_per_segment: f32,
        segments: usize,
        rng: &mut MyRng,
    ) -> Self {
        // reimplement TODO
        let up = Vector::new(0.0, approx_len_per_segment);
        let right = Vector::new(approx_len_per_segment, 0.0);
        let mut points = vec![approx_center];
        let mut vectors = vec![up];
        for i in 1..=segments {
            points.push(approx_center + i as f32 * right);
            vectors.push((-1.0).powi(i as i32) * up);
        }
        Self { points, vectors }
    }
}

impl BSpline {
    pub fn path(&self, t: f32) -> Vector {
        assert!(0.0 <= t && t <= self.count_segments() as f32);
        // TODO this may be bad
        if t == self.count_segments() as f32 {
            return *self.points.last().unwrap();
        }
        let floor = t.floor() as usize;
        let fract = t.fract();
        (1.0 - fract).powi(3) * self.points[floor]
            + (1.0 - fract).powi(2) * fract * (self.points[floor] + self.vectors[floor])
            + (1.0 - fract) * fract.powi(2) * (self.points[floor + 1] - self.vectors[floor + 1])
            + fract.powi(3) * self.points[floor + 1]
    }

    pub fn derivative(&self, t: f32) -> Vector {
        assert!(0.0 <= t && t < self.count_segments() as f32);
        let floor = t.floor() as usize;
        let fract = t.fract();
        3.0 * (1.0 - fract).powi(2) * self.vectors[floor]
            + 6.0
                * (1.0 - fract)
                * fract
                * (self.points[floor + 1]
                    - self.points[floor]
                    - self.vectors[floor + 1]
                    - self.vectors[floor])
            + 3.0 * fract.powi(2) * self.points[floor + 1]
    }

    pub fn derivative2(&self, t: f32) -> Vector {
        assert!(0.0 <= t && t < self.count_segments() as f32);
        let floor = t.floor() as usize;
        let fract = t.fract();
        6.0 * (1.0 - fract)
            * (self.points[floor + 1]
                - self.points[floor]
                - self.vectors[floor + 1]
                - 2.0 * self.vectors[floor])
            - 6.0
                * fract
                * (self.points[floor] - self.points[floor + 1]
                    + self.vectors[floor]
                    + 2.0 * self.vectors[floor + 1])
    }

    pub fn length(&self, steps_per_segment: usize) -> f32 {
        let mut sum = 0.0;
        let mut last_point = *self.points.first().unwrap();
        for t in (0..self.count_segments() * steps_per_segment)
            .map(|i| (i + 1) as f32 / steps_per_segment as f32)
        {
            let new_point = self.path(t);
            sum += (new_point - last_point).norm();
            last_point = new_point;
        }

        sum
    }

    pub fn curvature(&self, t: f32) -> f32 {
        let der = self.derivative(t);
        let der2 = self.derivative2(t);
        (der[0] * der2[1] - der2[0] * der[1]) / der.norm().powi(3)
    }
}

impl BSpline {
    pub fn as_path(&self, stroke_width: f32) -> Path {
        let mut data = Data::new().move_to((self.points[0].x, self.points[0].y));
        for i in 0..self.count_segments() {
            // TODO change to using append and command smooth cubic curve
            data = data.cubic_curve_to((
                (
                    self.points[i].x + self.vectors[i].x,
                    self.points[i].y + self.vectors[i].y,
                ),
                (
                    self.points[i + 1].x - self.vectors[i + 1].x,
                    self.points[i + 1].y - self.vectors[i + 1].y,
                ),
                (self.points[i + 1].x, self.points[i + 1].y),
            ));
        }
        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", stroke_width)
            .set("d", data);
        path
    }
}

impl Bounded for BSpline {
    fn bounding_box(&self) -> BoundingBox {
        let all_points = self
            .points
            .iter()
            .map(Bounded::bounding_box)
            .reduce(|acc, val| acc.combine(val))
            .unwrap();
        let first_control_points = (0..(self.points.len() - 1))
            .map(|i| (self.points[i] + self.vectors[i]).bounding_box())
            .reduce(|acc, val| acc.combine(val))
            .unwrap();
        let second_control_points = (1..(self.points.len()))
            .map(|i| (self.points[i] - self.vectors[i]).bounding_box())
            .reduce(|acc, val| acc.combine(val))
            .unwrap();
        all_points
            .combine(first_control_points)
            .combine(second_control_points)
    }
}
