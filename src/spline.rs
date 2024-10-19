use nalgebra::ComplexField;
use svg::node::element::{path::Data, Path};

use crate::{
    quad_tree::{Bounded, BoundingBox},
    MyRng, Rotation, Vector,
};

#[derive(Clone)]
pub struct BSpline {
    points: Vec<Vector>,
    vectors: Vec<Vector>,
    bounds: BoundingBox,
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
        let bounds = calc_bounding_box(&points, &vectors);
        Self {
            points,
            vectors,
            bounds,
        }
    }

    pub fn push(&mut self, point: Vector, vector: Vector) {
        self.points.push(point);
        self.vectors.push(vector);
        self.bounds = calc_bounding_box(&self.points, &self.vectors);
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
        let bounds = calc_bounding_box(&points, &vectors);
        Self {
            points,
            vectors,
            bounds,
        }
    }
}

impl BSpline {
    pub fn first_point(&self) -> Vector {
        *self.points.first().unwrap()
    }

    pub fn path(&self, t: f32) -> Vector {
        debug_assert!(
            0.0 <= t && t < self.count_segments() as f32,
            "{t} < {}",
            self.count_segments()
        );
        let floor = t.floor() as usize;
        let fract = t.fract();
        (1.0 - fract).powi(3) * self.points[floor]
            + (1.0 - fract).powi(2) * fract * (self.points[floor] + self.vectors[floor])
            + (1.0 - fract) * fract.powi(2) * (self.points[floor + 1] - self.vectors[floor + 1])
            + fract.powi(3) * self.points[floor + 1]
    }

    pub fn derivative(&self, t: f32) -> Vector {
        debug_assert!(0.0 <= t && t < self.count_segments() as f32);
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
        debug_assert!(0.0 <= t && t < self.count_segments() as f32);
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
        for t in (0..self.count_segments() * steps_per_segment)
            .map(|i| i as f32 / steps_per_segment as f32)
        {
            sum += self.derivative(t).norm();
        }

        sum / steps_per_segment as f32
    }

    pub fn curvature(&self, t: f32) -> f32 {
        let der = self.derivative(t);
        let der2 = self.derivative2(t);
        (der[0] * der2[1] - der2[0] * der[1]) / der.norm().powi(3)
    }
}

impl BSpline {
    pub fn translate(&mut self, vector: Vector) {
        self.bounds = self.bounds.translate(vector);
        self.points.iter_mut().for_each(|p| *p += vector)
    }

    pub fn rotate(&mut self, radians: f32) {
        let center = self.bounding_box().get_center();
        let rot = Rotation::new(radians);
        self.points
            .iter_mut()
            .for_each(|point| *point = center + rot * (*point - center));
        self.vectors.iter_mut().for_each(|vec| *vec = rot * *vec);
        self.bounds = calc_bounding_box(&self.points, &self.vectors);
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
        self.bounds
    }
}

fn calc_bounding_box(points: &[Vector], vectors: &[Vector]) -> BoundingBox {
    let all_points = points
        .iter()
        .map(Bounded::bounding_box)
        .reduce(|acc, val| acc.combine(val))
        .unwrap();
    let first_control_points = (0..(points.len() - 1))
        .map(|i| (points[i] + vectors[i]).bounding_box())
        .reduce(|acc, val| acc.combine(val))
        .unwrap();
    let second_control_points = (1..(points.len()))
        .map(|i| (points[i] - vectors[i]).bounding_box())
        .reduce(|acc, val| acc.combine(val))
        .unwrap();
    all_points
        .combine(first_control_points)
        .combine(second_control_points)
}
