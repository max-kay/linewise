use std::f32::consts::TAU;

use nalgebra::{Matrix2, Matrix2x3, Matrix2x4, Vector2, Vector3, Vector4};
use rand::Rng;
use svg::node::element::{
    path::{Command, Data},
    Path,
};

use crate::{
    gaussian_vector,
    quad_tree::{Bounded, Rect},
    rand_unit, MyRng, Rotation, Vector,
};

pub struct PolymerStorage {
    points_and_vecs: Vec<Vector>,
}

impl PolymerStorage {
    pub fn new() -> Self {
        Self {
            points_and_vecs: Vec::new(),
        }
    }

    pub fn add_polymer(&mut self, mut polymer: OwnedPolymer) -> PolymerRef {
        assert!(
            polymer.slice.is_none(),
            "trying to add an owned polymer which already is in the storage"
        );
        let storage_idx = self.points_and_vecs.len();
        let segments = polymer.count_segments();
        self.points_and_vecs.append(&mut polymer.points_and_vecs);
        PolymerRef {
            storage_idx,
            segments,
            bounds: polymer.bounds,
        }
    }

    pub fn shrink_to_fit(&mut self) {
        self.points_and_vecs.shrink_to_fit();
    }

    pub fn read(&self, polymer: PolymerRef) -> OwnedPolymer {
        OwnedPolymer {
            points_and_vecs: self.points_and_vecs
                [polymer.storage_idx..polymer.storage_idx + 2 * (polymer.segments + 1)]
                .to_vec(),
            bounds: polymer.bounds,
            slice: Some(polymer),
        }
    }

    pub fn revalidate_ref(&mut self, polymer: OwnedPolymer) -> PolymerRef {
        polymer
            .slice
            .expect("can only validate if the polymer is in the Storage")
    }
    pub fn overwrite_polymer(&mut self, polymer: OwnedPolymer) -> PolymerRef {
        let mut this_ref = polymer
            .slice
            .expect("can only overwrite if the polymer is already in the storage");
        this_ref.bounds = polymer.bounds;
        for (i, val) in polymer.points_and_vecs.into_iter().enumerate() {
            self.points_and_vecs[i + this_ref.storage_idx] = val
        }
        this_ref
    }

    pub fn get_borrowed_segments(
        &self,
        idx: &PolymerRef,
    ) -> impl Iterator<Item = BorrowedSegment<'_>> {
        debug_assert!(
            idx.storage_idx + (idx.segments + 1) * 2 <= self.points_and_vecs.len(),
            "polymer with idx {} out of bounds, storage_len: {}",
            idx.storage_idx,
            self.points_and_vecs.len()
        );
        self.points_and_vecs[idx.storage_idx..idx.storage_idx + (idx.segments + 1) * 2]
            .windows(4)
            .step_by(2)
            .map(|window| {
                let points_and_vecs = unsafe { &*(window.as_ptr() as *const [Vector; 4]) };
                BorrowedSegment { points_and_vecs }
            })
    }

    pub fn as_path(&self, polymer: &PolymerRef, stroke_width: f32) -> Path {
        let mut data = Data::new().move_to((
            self.points_and_vecs[polymer.storage_idx].x,
            self.points_and_vecs[polymer.storage_idx].y,
        ));
        // i points to the start of the segment
        for i in (polymer.storage_idx..polymer.storage_idx + polymer.segments * 2).step_by(2) {
            data.append(Command::CubicCurve(
                svg::node::element::path::Position::Absolute,
                (
                    (
                        self.points_and_vecs[i].x + self.points_and_vecs[i + 1].x,
                        self.points_and_vecs[i].y + self.points_and_vecs[i + 1].y,
                    ),
                    (
                        self.points_and_vecs[i + 2].x - self.points_and_vecs[i + 3].x,
                        self.points_and_vecs[i + 2].y - self.points_and_vecs[i + 3].y,
                    ),
                    (self.points_and_vecs[i + 2].x, self.points_and_vecs[i + 2].y),
                )
                    .into(),
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

pub struct PolymerRef {
    storage_idx: usize,
    segments: usize,
    bounds: Rect,
}

impl PolymerRef {
    pub fn duplicate(&self) -> Self {
        Self {
            storage_idx: self.storage_idx,
            segments: self.segments,
            bounds: self.bounds,
        }
    }
}

impl PartialEq for PolymerRef {
    fn eq(&self, other: &Self) -> bool {
        self.storage_idx == other.storage_idx
    }
}
impl Eq for PolymerRef {}

impl PartialOrd for PolymerRef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.storage_idx.partial_cmp(&other.storage_idx)
    }
}
impl Ord for PolymerRef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.storage_idx.cmp(&other.storage_idx)
    }
}

impl Bounded for PolymerRef {
    fn bounding_box(&self) -> Rect {
        self.bounds
    }
}

pub struct OwnedPolymer {
    points_and_vecs: Vec<Vector>,
    bounds: Rect,
    slice: Option<PolymerRef>,
}

impl OwnedPolymer {
    pub fn new(points: Vec<Vector>, vectors: Vec<Vector>) -> Self {
        assert_eq!(
            points.len(),
            vectors.len(),
            "points and vectors must be of same length"
        );
        assert!(
            points.len() >= 2,
            "must contain at least two points and vectors"
        );
        let mut points_and_vecs = Vec::new();
        for (p, v) in points.into_iter().zip(vectors.into_iter()) {
            points_and_vecs.push(p);
            points_and_vecs.push(v)
        }
        let mut this = Self {
            points_and_vecs,
            bounds: Rect::default(),
            slice: None,
        };
        this.update_bounds();
        this
    }

    fn duplicate(&self) -> Self {
        Self {
            points_and_vecs: self.points_and_vecs.clone(),
            bounds: self.bounds,
            slice: self.slice.as_ref().map(PolymerRef::duplicate),
        }
    }

    pub fn count_segments(&self) -> usize {
        self.points_and_vecs.len() / 2 - 1
    }

    pub fn points(&self) -> impl Iterator<Item = &Vector> {
        self.points_and_vecs.iter().step_by(2)
    }
    pub fn vecs(&self) -> impl Iterator<Item = &Vector> {
        self.points_and_vecs.iter().skip(1).step_by(2)
    }

    pub fn points_mut(&mut self) -> impl Iterator<Item = &mut Vector> {
        self.points_and_vecs.iter_mut().step_by(2)
    }
    pub fn vecs_mut(&mut self) -> impl Iterator<Item = &mut Vector> {
        self.points_and_vecs.iter_mut().skip(1).step_by(2)
    }

    pub fn new_random(
        approx_center: Vector,
        approx_len_per_segment: f32,
        segments: usize,
        rng: &mut MyRng,
    ) -> Self {
        let mut points = vec![Vector::zeros()];
        for _ in 0..segments {
            let mut translation = rand_unit(rng);
            while translation.dot(&Vector::new(1.0, 0.0)) < 0.2 {
                translation = rand_unit(rng)
            }
            points
                .push(points.last().expect("is non empty") + translation * approx_len_per_segment);
        }

        let mut vectors = vec![Vector::new(approx_len_per_segment * 0.5, 0.0)];
        for i in 0..segments - 1 {
            vectors.push((points[i + 1] - points[i]).normalize() * 0.5 * approx_len_per_segment)
        }
        vectors.push(Vector::new(approx_len_per_segment * 0.5, 0.0));

        let mut this = Self::new(points, vectors);
        this.translate(approx_center - this.bounds.get_center());
        this.rotate(rng.gen::<f32>() * TAU);
        this
    }

    pub fn update_bounds(&mut self) {
        let all_points = self
            .points()
            .map(Bounded::bounding_box)
            .reduce(|acc, val| acc.combine(val))
            .expect("is non empty");
        let first_control_points = self.points_and_vecs[0..self.points_and_vecs.len() - 2]
            .iter()
            .step_by(2)
            .zip(
                self.points_and_vecs[0..self.points_and_vecs.len() - 2]
                    .iter()
                    .skip(1)
                    .step_by(2),
            )
            .map(|(p, v)| (p + v).bounding_box())
            .reduce(|acc, val| acc.combine(val))
            .expect("is non empty");
        let second_control_points = self
            .points()
            .skip(1)
            .zip(self.vecs().skip(1))
            .map(|(p, v)| (p - v).bounding_box())
            .reduce(|acc, val| acc.combine(val))
            .expect("is non empty");
        self.bounds = all_points
            .combine(first_control_points)
            .combine(second_control_points);
    }
}

impl OwnedPolymer {
    pub fn get_borrowed_segments(&self) -> impl Iterator<Item = BorrowedSegment<'_>> {
        self.points_and_vecs.windows(4).step_by(2).map(|window| {
            let points_and_vecs = unsafe { &*(window.as_ptr() as *const [Vector; 4]) };

            BorrowedSegment { points_and_vecs }
        })
    }
}

impl OwnedPolymer {
    pub fn vary(&self, transition_scale: f32, rng: &mut MyRng) -> Self {
        let mut this = self.duplicate();
        match rng.gen_range(0..=1) {
            0 => this.translate(gaussian_vector(rng) * transition_scale),
            1 => this.rotate((rng.gen::<f32>() - 0.5) * TAU * transition_scale / 10.0),
            _ => unreachable!(),
        }
        this
    }

    pub fn translate(&mut self, vector: Vector) {
        self.bounds = self.bounds.translate(vector);
        self.points_mut().for_each(|p| *p += vector)
    }

    pub fn rotate(&mut self, radians: f32) {
        let center = self.bounding_box().get_center();
        let rot = Rotation::new(radians);
        self.points_mut()
            .for_each(|point| *point = center + rot * (*point - center));
        self.vecs_mut().for_each(|vec| *vec = rot * *vec);
        self.update_bounds();
    }
}

impl Bounded for OwnedPolymer {
    fn bounding_box(&self) -> Rect {
        self.bounds
    }
}

impl PartialEq<PolymerRef> for OwnedPolymer {
    fn eq(&self, other: &PolymerRef) -> bool {
        self.slice.as_ref().expect("index initialised").storage_idx == other.storage_idx
    }
}

pub struct BorrowedSegment<'a> {
    points_and_vecs: &'a [Vector; 4],
}

impl BorrowedSegment<'_> {
    pub fn s_iter(steps: usize) -> impl Iterator<Item = f32> {
        (0..steps).map(move |i| i as f32 / steps as f32)
    }

    pub fn get_coord_matrix_0(&self) -> Matrix2x4<f32> {
        let columns = [
            self.points_and_vecs[0],
            self.points_and_vecs[0] + self.points_and_vecs[1],
            self.points_and_vecs[2] - self.points_and_vecs[3],
            self.points_and_vecs[2],
        ];
        Matrix2x4::from_columns(&columns)
    }
    pub fn get_bernstein_3(s: f32) -> Vector4<f32> {
        Vector4::new(
            (1.0 - s).powi(3),
            3.0 * (1.0 - s).powi(2) * s,
            3.0 * (1.0 - s) * s.powi(2),
            s.powi(3),
        )
    }
    pub fn position_iter(&self, steps: usize) -> impl Iterator<Item = Vector> {
        let coord_matrix = self.get_coord_matrix_0();
        Self::s_iter(steps).map(move |s| coord_matrix * Self::get_bernstein_3(s))
    }

    pub fn get_coord_matrix_1(&self) -> Matrix2x3<f32> {
        let columns = [
            self.points_and_vecs[1],
            self.points_and_vecs[2]
                - self.points_and_vecs[0]
                - self.points_and_vecs[3]
                - self.points_and_vecs[1],
            self.points_and_vecs[3],
        ];
        Matrix2x3::from_columns(&columns)
    }
    pub fn get_bernstein_2(s: f32) -> Vector3<f32> {
        Vector3::new((1.0 - s).powi(2), 2.0 * (1.0 - s) * s, s.powi(2))
    }
    pub fn derivative_iter(&self, steps: usize) -> impl Iterator<Item = Vector> {
        let coord_matrix = self.get_coord_matrix_1();
        Self::s_iter(steps).map(move |s| 3.0 * coord_matrix * Self::get_bernstein_2(s))
    }

    pub fn get_coord_matrix_2(&self) -> Matrix2<f32> {
        let columns = [
            self.points_and_vecs[2]
                - self.points_and_vecs[0]
                - self.points_and_vecs[3]
                - 2.0 * self.points_and_vecs[1],
            self.points_and_vecs[0] - self.points_and_vecs[2]
                + self.points_and_vecs[1]
                + 2.0 * self.points_and_vecs[3],
        ];
        Matrix2::from_columns(&columns)
    }
    pub fn get_bernstein_1(s: f32) -> Vector2<f32> {
        Vector2::new(1.0 - s, s)
    }
    pub fn derivative2_iter(&self, steps: usize) -> impl Iterator<Item = Vector> {
        let coord_matrix = self.get_coord_matrix_2();
        Self::s_iter(steps).map(move |s| 6.0 * coord_matrix * Self::get_bernstein_1(s))
    }

    pub fn all_iters(&self, steps: usize) -> impl Iterator<Item = (Vector, Vector, Vector)> {
        self.position_iter(steps)
            .zip(self.derivative_iter(steps))
            .zip(self.derivative2_iter(steps))
            .map(|((path, der), der2)| (path, der, der2))
    }
}
