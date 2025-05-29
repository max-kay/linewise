use std::{f32::consts::TAU, usize};

use nalgebra::{Matrix2, Matrix2x3, Matrix2x4, Vector2, Vector3, Vector4};
use random::{MyRng, Rng};
use svg::node::element::{
    Path,
    path::{Command, Data, Position},
};
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::{
    Rotation, Vector,
    quad_tree::{Bounded, Rect},
};

use random::rand_unit;

#[derive(Clone)]
pub struct PolymerStorage {
    points_and_vecs: Vec<Vector>,
    seg_starts: Vec<usize>,
}

impl PolymerStorage {
    pub fn new() -> Self {
        Self {
            points_and_vecs: Vec::new(),
            seg_starts: Vec::new(),
        }
    }

    pub fn add_polymer(&mut self, mut polymer: OwnedPolymer) -> PolymerRef {
        assert!(
            polymer.st_ref.is_none(),
            "trying to add an owned polymer which already is in the storage"
        );
        let storage_idx = self.points_and_vecs.len();
        self.seg_starts.push(storage_idx);
        let segments = polymer.count_segments();
        self.points_and_vecs.append(&mut polymer.points_and_vecs);
        PolymerRef {
            storage_idx: storage_idx as u32,
            segments: segments as u32,
            bounds: polymer.bounds,
        }
    }

    pub fn shrink_to_fit(&mut self) {
        self.points_and_vecs.shrink_to_fit();
        self.seg_starts.shrink_to_fit();
    }

    pub fn read(&self, polymer: PolymerRef) -> OwnedPolymer {
        OwnedPolymer {
            points_and_vecs: self.points_and_vecs[polymer.storage_idx as usize
                ..(polymer.storage_idx + 2 * (polymer.segments + 1)) as usize]
                .to_vec(),
            bounds: polymer.bounds,
            st_ref: Some(polymer),
        }
    }

    pub fn revalidate_ref(&mut self, polymer: OwnedPolymer) -> PolymerRef {
        polymer
            .st_ref
            .expect("can only validate if the polymer is in the Storage")
    }
    pub fn overwrite_polymer(&mut self, polymer: OwnedPolymer) -> PolymerRef {
        let mut this_ref = polymer
            .st_ref
            .expect("can only overwrite if the polymer is already in the storage");
        this_ref.bounds = polymer.bounds;
        for (i, val) in polymer.points_and_vecs.into_iter().enumerate() {
            self.points_and_vecs[i + this_ref.storage_idx as usize] = val
        }
        this_ref
    }

    pub fn get_borrowed_segments(
        &self,
        idx: &PolymerRef,
    ) -> impl Iterator<Item = BorrowedSegment<'_>> {
        debug_assert!(
            (idx.storage_idx + (idx.segments + 1) * 2) as usize <= self.points_and_vecs.len(),
            "polymer with idx {} out of bounds, storage_len: {}",
            idx.storage_idx,
            self.points_and_vecs.len()
        );
        self.points_and_vecs
            [idx.storage_idx as usize..(idx.storage_idx + (idx.segments + 1) * 2) as usize]
            .windows(4)
            .step_by(2)
            .map(|window| {
                let points_and_vecs = unsafe { &*(window.as_ptr() as *const [Vector; 4]) };
                BorrowedSegment { points_and_vecs }
            })
    }

    pub fn segments<'a>(&'a self) -> impl Iterator<Item = BorrowedSegment<'a>> {
        (0..self.points_and_vecs.len() / 2 - 1).flat_map(|i| {
            let st_idx = i * 2;
            // if the end point of the window I'm trying to construct is the start of a segment
            // then that window is no valid segment and should be skipped
            if self.seg_starts.binary_search(&(st_idx + 2)).is_ok() {
                None
            } else {
                // SAFETY: the highest value i can take is len / 2 - 2
                // so the st_idx is at most len - 4
                // the slice [len - 4..len] is allways valid
                let window = &self.points_and_vecs[st_idx..st_idx + 4];
                let points_and_vecs = unsafe { &*(window.as_ptr() as *const [Vector; 4]) };
                Some(BorrowedSegment { points_and_vecs })
            }
        })
    }

    pub fn rasterize(
        &self,
        line_width: f32,
        scaling_factor: f32,
        width: u32,
        height: u32,
    ) -> Pixmap {
        let paint = Paint {
            shader: tiny_skia::Shader::SolidColor(Color::from_rgba8(0, 0, 0, 255)),
            ..Default::default()
        };
        let mut pix_map = Pixmap::new(width, height).unwrap();
        pix_map.fill(Color::from_rgba8(255, 255, 255, 255));
        let mut path = PathBuilder::new();
        let mut index_iter = self.seg_starts.iter();
        path.move_to(self.points_and_vecs[0].x, self.points_and_vecs[0].y);
        index_iter.next();
        let mut next_start = *index_iter.next().unwrap();
        for i in 0..(self.points_and_vecs.len() / 2 - 1) {
            let segment_start = i * 2;
            if segment_start + 2 == next_start {
                let mut place_holder = PathBuilder::new();
                std::mem::swap(&mut place_holder, &mut path);
                pix_map.stroke_path(
                    &place_holder.finish().unwrap(),
                    &paint,
                    &Stroke {
                        width: line_width,
                        ..Default::default()
                    },
                    Transform::from_scale(scaling_factor, scaling_factor),
                    None,
                );
                if segment_start + 4 == self.points_and_vecs.len() {
                    break;
                }
                let next_pos = self.points_and_vecs[segment_start + 2];
                path.move_to(next_pos.x, next_pos.y);
                next_start = index_iter
                    .next()
                    .cloned()
                    .unwrap_or(self.points_and_vecs.len());
                continue;
            }
            let p1 = self.points_and_vecs[segment_start] + self.points_and_vecs[segment_start + 1];
            let p2 =
                self.points_and_vecs[segment_start + 2] - self.points_and_vecs[segment_start + 3];
            let p3 = self.points_and_vecs[segment_start + 2];
            path.cubic_to(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
        }
        pix_map
    }

    pub fn as_path(&self, polymer: &PolymerRef, stroke_width: f32) -> Path {
        let mut data = Data::new().move_to((
            self.points_and_vecs[polymer.storage_idx as usize].x,
            self.points_and_vecs[polymer.storage_idx as usize].y,
        ));
        // i points to the start of the segment
        for i in (polymer.storage_idx..polymer.storage_idx + polymer.segments * 2).step_by(2) {
            data.append(Command::CubicCurve(
                Position::Absolute,
                (
                    (
                        self.points_and_vecs[i as usize].x
                            + self.points_and_vecs[(i + 1) as usize].x,
                        self.points_and_vecs[i as usize].y
                            + self.points_and_vecs[(i + 1) as usize].y,
                    ),
                    (
                        self.points_and_vecs[(i + 2) as usize].x
                            - self.points_and_vecs[(i + 3) as usize].x,
                        self.points_and_vecs[(i + 2) as usize].y
                            - self.points_and_vecs[(i + 3) as usize].y,
                    ),
                    (
                        self.points_and_vecs[(i + 2) as usize].x,
                        self.points_and_vecs[(i + 2) as usize].y,
                    ),
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
    storage_idx: u32,
    segments: u32,
    bounds: Rect,
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
    st_ref: Option<PolymerRef>,
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
            st_ref: None,
        };
        this.update_bounds();
        this
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
        this.rotate(rng.random::<f32>() * TAU);
        this
    }

    pub fn update_bounds(&mut self) {
        let mut bounds = self.points_and_vecs[0]
            .bounding_box()
            .combine(self.points_and_vecs[self.points_and_vecs.len() - 2].bounding_box());
        for i in 0..self.count_segments() {
            let c1 = self.points_and_vecs[2 * i] + self.points_and_vecs[2 * i + 1];
            let c2 = self.points_and_vecs[2 * i + 2] - self.points_and_vecs[2 * i + 3];
            bounds = bounds.combine(c1.bounding_box());
            bounds = bounds.combine(c2.bounding_box());
        }
        self.bounds = bounds;
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
    pub fn translate(&mut self, vector: Vector) {
        self.points_mut().for_each(|p| *p += vector);
        debug_assert!(
            self.points_and_vecs
                .iter()
                .map(|vec| vec.x.is_finite() && vec.y.is_finite())
                .reduce(|acc, val| acc && val)
                .expect("is non empty"),
            "translate{:?}",
            vector
        );
        self.update_bounds();
    }

    pub fn rotate(&mut self, radians: f32) {
        let center = self.bounding_box().get_center();
        let rot = Rotation::new(radians);
        self.points_mut()
            .for_each(|point| *point = center + rot * (*point - center));
        self.vecs_mut().for_each(|vec| *vec = rot * *vec);
        debug_assert!(
            self.points_and_vecs
                .iter()
                .map(|vec| vec.x.is_finite() && vec.y.is_finite())
                .reduce(|acc, val| acc && val)
                .expect("is non empty"),
            "rotate({})",
            radians
        );
        self.update_bounds();
    }

    pub fn rotate_segment(&mut self, segment: usize, radians: f32) {
        if let [p0, v0, p1, v1] = &mut self.points_and_vecs[2 * segment..2 * segment + 4] {
            let mid_point = (*p0 + *p1) / 2.0;
            *p0 = Rotation::new(radians) * (*p0 - mid_point) + mid_point;
            *p1 = Rotation::new(radians) * (*p1 - mid_point) + mid_point;
            *v0 = Rotation::new(radians / 2.0) * *v0;
            *v1 = Rotation::new(radians / 2.0) * *v1;
        } else {
            unreachable!()
        }
        self.update_bounds();
    }

    pub fn scales_vecs(&mut self, factor: f32) {
        self.vecs_mut().for_each(|vec| *vec = factor * *vec);
        self.update_bounds();
    }

    pub fn scales_vecs_random(&mut self, factor: f32, rng: &mut MyRng) {
        self.vecs_mut()
            .for_each(|vec| *vec = (rng.random::<f32>() - 0.5) * 2.0 * factor * *vec);
        self.update_bounds();
    }

    pub fn stretch(&mut self, factor: f32) {
        let origin =
            (self.points_and_vecs[0] + self.points_and_vecs[self.points_and_vecs.len() - 2]) / 2.0;
        self.points_mut()
            .for_each(|point| *point = factor * (*point - origin) + origin);
        self.vecs_mut().for_each(|vec| *vec = *vec * factor);
        self.update_bounds();
    }

    pub fn intersects(&self, other: &BorrowedSegment, precision: usize) -> bool {
        let mut out = false;
        for segment in self.get_borrowed_segments() {
            out |= other.interscets(&segment, precision);
        }
        out
    }
}

impl Bounded for OwnedPolymer {
    fn bounding_box(&self) -> Rect {
        self.bounds
    }
}

impl PartialEq<PolymerRef> for OwnedPolymer {
    fn eq(&self, other: &PolymerRef) -> bool {
        self.st_ref.as_ref().expect("index initialised").storage_idx == other.storage_idx
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
    pub fn position_iter_with_end(&self, steps: usize) -> impl Iterator<Item = Vector> {
        let coord_matrix = self.get_coord_matrix_0();
        (0..steps + 1)
            .map(move |i| i as f32 / steps as f32)
            .map(move |s| coord_matrix * Self::get_bernstein_3(s))
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

    pub fn pos_and_der_iter(&self, steps: usize) -> impl Iterator<Item = (Vector, Vector)> {
        self.position_iter(steps).zip(self.derivative_iter(steps))
    }

    pub fn all_iters(&self, steps: usize) -> impl Iterator<Item = (Vector, Vector, Vector)> {
        self.position_iter(steps)
            .zip(self.derivative_iter(steps))
            .zip(self.derivative2_iter(steps))
            .map(|((path, der), der2)| (path, der, der2))
    }
}

fn line_intersects(line_1: &[Vector; 2], line_2: &[Vector; 2]) -> bool {
    let mat = match Matrix2::from_columns(&[line_2[1] - line_2[0], line_1[0] - line_1[1]])
        .try_inverse()
    {
        Some(inv) => inv,
        None => return false,
    };
    let intersection_vec = mat * (line_1[0] - line_2[0]);
    (0.0 <= intersection_vec.x && intersection_vec.x <= 1.0)
        && (0.0 <= intersection_vec.y && intersection_vec.y <= 1.0)
}

impl BorrowedSegment<'_> {
    pub fn interscets(&self, other: &Self, precision: usize) -> bool {
        let points_1: Vec<_> = self.position_iter_with_end(precision).collect();
        let points_2: Vec<_> = other.position_iter_with_end(precision).collect();
        let mut intersects = false;
        for line_1 in points_1.windows(2) {
            let line_1 = unsafe { &*(line_1.as_ptr() as *const [Vector; 2]) };
            for line_2 in points_2.windows(2) {
                let line_2 = unsafe { &*(line_2.as_ptr() as *const [Vector; 2]) };
                intersects |= line_intersects(line_1, line_2);
            }
        }
        intersects
    }
}

impl BorrowedSegment<'_> {
    pub fn as_triangles(&self, precision: usize, line_width: f32) -> Vec<[Vector; 3]> {
        let mut out = Vec::new();
        for i in 0..precision + 1 {
            let s_0 = i as f32 / precision as f32;
            let s_1 = (i + 1) as f32 / precision as f32;
            let p_0 = self.get_coord_matrix_0() * Self::get_bernstein_3(s_0);
            let offset_0 = Rotation::new(TAU / 4.0)
                * (self.get_coord_matrix_1() * Self::get_bernstein_2(s_1)).normalize()
                * line_width
                / 2.0;
            let p_1 = self.get_coord_matrix_0() * Self::get_bernstein_3(s_1);
            let offset_1 = Rotation::new(TAU / 4.0)
                * (self.get_coord_matrix_1() * Self::get_bernstein_2(s_1)).normalize()
                * line_width
                / 2.0;
            out.push([p_0 + offset_0, p_0 - offset_0, p_1 + offset_1]);
            out.push([p_0 - offset_0, p_1 - offset_1, p_1 + offset_1]);
        }
        out
    }
}
