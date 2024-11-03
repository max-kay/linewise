use std::f32::consts::TAU;

use rand::Rng;
use svg::node::element::{path::Data, Path};

use crate::{
    quad_tree::{Bounded, Rect},
    rand_unit, MyRng, Rotation, Vector,
};

#[derive(Clone)]
pub struct BSpline {
    points: Vec<Vector>,
    vectors: Vec<Vector>,
    bounds: Rect,
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
        let mut points = vec![Vector::zeros()];
        for _ in 0..segments {
            let mut translation = rand_unit(rng);
            while translation.dot(&Vector::new(1.0, 0.0)) < 0.2 {
                translation = rand_unit(rng)
            }
            points.push(points.last().unwrap() + translation * approx_len_per_segment);
        }

        let mut vectors = vec![Vector::new(approx_len_per_segment * 0.5, 0.0)];
        for i in 0..segments - 1 {
            vectors.push((points[i + 2] - points[i]).normalize() * 0.5 * approx_len_per_segment)
        }
        vectors.push(Vector::new(approx_len_per_segment * 0.5, 0.0));

        let bounds = calc_bounding_box(&points, &vectors);
        let mut this = Self {
            points,
            vectors,
            bounds,
        };
        this.translate(approx_center - bounds.get_center());
        this.rotate(rng.gen::<f32>() * TAU);
        this
    }
}

impl BSpline {
    /// SAFETY assumes segment < self.count_segments.len() - 1 && 0.0 <= t < 1.0
    pub unsafe fn path(&self, segment: usize, t: f32) -> Vector {
        (1.0 - t).powi(3) * self.points.get_unchecked(segment)
            + 3.0
                * (1.0 - t).powi(2)
                * t
                * (self.points.get_unchecked(segment) + self.vectors.get_unchecked(segment))
            + 3.0
                * (1.0 - t)
                * t.powi(2)
                * (self.points.get_unchecked(segment + 1) - self.vectors.get_unchecked(segment + 1))
            + t.powi(3) * self.points.get_unchecked(segment + 1)
    }

    /// SAFETY assumes segment < self.count_segments.len() - 1 && 0.0 <= t < 1.0
    pub unsafe fn derivative(&self, segment: usize, t: f32) -> Vector {
        3.0 * (1.0 - t).powi(2) * self.vectors.get_unchecked(segment)
            + 6.0
                * (1.0 - t)
                * t
                * (self.points.get_unchecked(segment + 1)
                    - self.points.get_unchecked(segment)
                    - self.vectors.get_unchecked(segment + 1)
                    - self.vectors.get_unchecked(segment))
            + 3.0 * t.powi(2) * self.vectors.get_unchecked(segment + 1)
    }

    /// SAFETY assumes segment < self.count_segments.len() - 1 && 0.0 <= t < 1.0
    pub unsafe fn derivative2(&self, segment: usize, t: f32) -> Vector {
        6.0 * (1.0 - t)
            * (self.points.get_unchecked(segment + 1)
                - self.points.get_unchecked(segment)
                - self.vectors.get_unchecked(segment + 1)
                - 2.0 * self.vectors.get_unchecked(segment))
            - 6.0
                * t
                * (self.points.get_unchecked(segment) - self.points.get_unchecked(segment + 1)
                    + self.vectors.get_unchecked(segment)
                    + 2.0 * self.vectors.get_unchecked(segment + 1))
    }

    /// returns an iter of item segment, t
    /// for the use in the path function and its derivative
    pub fn index_iter(&self, steps_per_segment: usize) -> impl Iterator<Item = (usize, f32)> {
        (0..self.count_segments() * steps_per_segment).map(move |i| {
            (
                i / steps_per_segment,
                (i % steps_per_segment) as f32 / steps_per_segment as f32,
            )
        })
    }

    pub fn length(&self, steps_per_segment: usize) -> f32 {
        let mut sum = 0.0;
        for (seg, t) in self.index_iter(steps_per_segment) {
            unsafe {
                sum += self.derivative(seg, t).norm();
            }
        }

        sum / steps_per_segment as f32
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

    pub fn bend_at_segment(&mut self, segment: usize, angle: f32) {
        debug_assert!(segment < self.points.len() - 1);
        let center = unsafe { self.path(segment, 0.5) };
        let rot = Rotation::new(angle);
        self.points[segment + 1..]
            .iter_mut()
            .for_each(|point| *point = rot * (*point - center) + center);
        self.vectors[segment + 1..]
            .iter_mut()
            .for_each(|vec| *vec = rot * *vec);
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
    fn bounding_box(&self) -> Rect {
        self.bounds
    }
}

fn calc_bounding_box(points: &[Vector], vectors: &[Vector]) -> Rect {
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
