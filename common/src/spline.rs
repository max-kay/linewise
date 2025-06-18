use std::{f32::consts::TAU, usize};

use nalgebra::{Matrix2x4, Matrix4, Matrix4x2, Matrix4x3, Vector2, Vector3, Vector4};
use random::{MyRng, Rng};
use svg::node::element::{
    Path as SvgPath,
    path::{Command, Data, Position},
};
use tiny_skia::PathBuilder;

use crate::{
    Rotation, Vector,
    quad_tree::{Bounded, Rect},
};

use random::rand_unit;

pub struct Spline {
    points_and_vecs: Vec<Vector>,
    bounds: Rect,
}

impl Spline {
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
        };
        this.update_bounds();
        this
    }

    pub fn from_parts(points_and_vecs: &[Vector], bounds: Rect) -> Self {
        debug_assert!(
            points_and_vecs.len() >= 4 && points_and_vecs.len() % 2 == 0,
            "tried to create spline from invalid slice of vector."
        );
        assert!(
            BorrowedSpline(points_and_vecs).calculate_bounds() == bounds,
            "bounds were incorrect"
        );
        Self {
            points_and_vecs: points_and_vecs.to_vec(),
            bounds,
        }
    }

    pub fn into_vec(self) -> Vec<Vector> {
        self.points_and_vecs
    }

    pub fn as_slice(&self) -> &[Vector] {
        &self.points_and_vecs
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
        self.bounds = self.as_borrowed_spline().calculate_bounds()
    }
}

impl Spline {
    pub fn segments(&self) -> impl Iterator<Item = Segment> {
        self.points_and_vecs
            .windows(4)
            .step_by(2)
            .map(|window| Segment(Matrix2x4::from_columns(window)))
    }

    pub fn as_borrowed_spline(&self) -> BorrowedSpline<'_> {
        BorrowedSpline(&self.points_and_vecs)
    }
}

impl Spline {
    pub fn translate(&mut self, vector: Vector) {
        self.points_mut().for_each(|p| *p += vector);
        debug_assert!(
            self.points_and_vecs
                .iter()
                .map(|vec| vec.x.is_finite() && vec.y.is_finite())
                .reduce(|acc, val| acc && val)
                .unwrap_or(false),
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
                .unwrap_or(false),
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

    pub fn shortest_dist(&self, other: &Segment, precision: usize) -> f32 {
        let mut min = f32::INFINITY;
        for segment in self.segments() {
            let dist = segment.shortest_dist(other, precision);
            if dist < min {
                min = dist
            }
        }
        min
    }
}

impl Bounded for Spline {
    fn bounding_box(&self) -> Rect {
        self.bounds
    }
}

pub struct BorrowedSpline<'a>(&'a [Vector]);

impl<'a> BorrowedSpline<'a> {
    pub fn from_slice(slice: &'a [Vector]) -> Self {
        debug_assert!(
            slice.len() > 2 && slice.len() % 2 == 0,
            "tried create BorrowedSpline from an invalid slice len"
        );
        Self(slice)
    }
    pub fn count_segments(&self) -> usize {
        self.0.len() / 2 - 1
    }
    pub fn calculate_bounds(&self) -> Rect {
        let mut bounds = self.0[0].bounding_box();
        for i in 0..self.count_segments() {
            let c1 = self.0[2 * i] + self.0[2 * i + 1];
            let c2 = self.0[2 * i + 2] - self.0[2 * i + 3];
            let p2 = self.0[2 * i + 2];
            bounds = bounds.combine(Rect::from_points(&[c1, c2, p2]));
        }
        bounds
    }

    pub fn segments(&self) -> impl Iterator<Item = Segment> {
        self.0
            .windows(4)
            .step_by(2)
            .map(|window| Segment(Matrix2x4::from_columns(window)))
    }

    pub fn as_ts_path(&self) -> tiny_skia::Path {
        let mut path = PathBuilder::new();
        path.move_to(self.0[0].x, self.0[0].y);
        for i in 0..(self.0.len() / 2 - 1) {
            let segment_start = i * 2;
            let p1 = self.0[segment_start] + self.0[segment_start + 1];
            let p2 = self.0[segment_start + 2] - self.0[segment_start + 3];
            let p3 = self.0[segment_start + 2];
            path.cubic_to(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
        }
        path.finish().expect("paths are always valid")
    }

    pub fn as_svg_path(&self, color: &'static str, width: f32) -> SvgPath {
        let mut data = Data::new().move_to((self.0[0].x, self.0[0].y));
        // i points to the start of the segment
        for i in 0..self.0.len() / 2 - 1 {
            let c1 = self.0[2 * i as usize] + self.0[(2 * i + 1) as usize];
            let c2 = self.0[(2 * i + 2) as usize] - self.0[(2 * i + 3) as usize];
            let c3 = self.0[(2 * i + 2) as usize];
            data.append(Command::CubicCurve(
                Position::Absolute,
                ((c1.x, c1.y), (c2.x, c2.y), (c3.x, c3.y)).into(),
            ));
        }
        let path = SvgPath::new()
            .set("fill", "none")
            .set("stroke", color)
            .set("stroke-width", width)
            .set("d", data);
        path
    }

    pub fn as_slice(&self) -> &[Vector] {
        self.0
    }
}

pub struct Segment(Matrix2x4<f32>);

impl Segment {
    pub fn from_mat(mat: Matrix2x4<f32>) -> Self {
        Self(mat)
    }

    pub fn from_slice(slice: &[Vector]) -> Self {
        Self(Matrix2x4::from_columns(slice))
    }
}

impl Segment {
    pub fn position(&self, s: f32) -> Vector {
        self.0 * MatrixGenerator::position(s)
    }
    pub fn derivative(&self, s: f32) -> Vector {
        self.0 * MatrixGenerator::derivative(s)
    }
    pub fn derivative2(&self, s: f32) -> Vector {
        self.0 * MatrixGenerator::derivative2(s)
    }

    pub fn pos_iter(&self, steps: usize) -> impl Iterator<Item = Vector> {
        MatrixGenerator::s_iter(steps).map(|s| self.position(s))
    }
    pub fn der_iter(&self, steps: usize) -> impl Iterator<Item = Vector> {
        MatrixGenerator::s_iter(steps).map(|s| self.derivative(s))
    }
    pub fn der2_iter(&self, steps: usize) -> impl Iterator<Item = Vector> {
        MatrixGenerator::s_iter(steps).map(|s| self.derivative2(s))
    }
    pub fn pos_and_der_iter(&self, steps: usize) -> impl Iterator<Item = (Vector, Vector)> {
        MatrixGenerator::s_iter(steps).map(|s| (self.position(s), self.derivative(s)))
    }
    pub fn all_iters(&self, steps: usize) -> impl Iterator<Item = (Vector, Vector, Vector)> {
        MatrixGenerator::s_iter(steps)
            .map(|s| (self.position(s), self.derivative(s), self.derivative2(s)))
    }

    pub fn pos_iter_p(&self, precomp: &Precomputed) -> impl Iterator<Item = Vector> {
        precomp.position().map(|mat| self.0 * mat)
    }
    pub fn der_iter_p(&self, precomp: &Precomputed) -> impl Iterator<Item = Vector> {
        precomp.derivative().map(|mat| self.0 * mat)
    }
    pub fn der2_iter_p(&self, precomp: &Precomputed) -> impl Iterator<Item = Vector> {
        precomp.derivative2().map(|mat| self.0 * mat)
    }
    pub fn pos_and_der_iter_p(
        &self,
        precomp: &Precomputed,
    ) -> impl Iterator<Item = (Vector, Vector)> {
        precomp
            .position()
            .zip(precomp.derivative())
            .map(|(pos, der)| (self.0 * pos, self.0 * der))
    }
    pub fn all_iters_p(
        &self,
        precomp: &Precomputed,
    ) -> impl Iterator<Item = (Vector, Vector, Vector)> {
        precomp
            .position()
            .zip(precomp.derivative())
            .zip(precomp.derivative2())
            .map(|((pos, der), der2)| (self.0 * pos, self.0 * der, self.0 * der2))
    }
}

impl Segment {
    pub fn shortest_dist(&self, other: &Self, precision: usize) -> f32 {
        let mut min = f32::INFINITY;
        for p1 in self.pos_iter(precision) {
            for p2 in other.pos_iter(precision) {
                let dist = (p1 - p2).norm();
                if dist < min {
                    min = dist
                }
            }
        }
        min
    }
}
pub struct MatrixGenerator;

impl MatrixGenerator {
    pub fn s_iter(steps: usize) -> impl Iterator<Item = f32> {
        (0..steps).map(move |i| i as f32 / steps as f32)
    }
    pub fn s_iter_end(steps: usize) -> impl Iterator<Item = f32> {
        (0..steps + 1).map(move |i| i as f32 / steps as f32)
    }

    pub fn get_bernstein_3(s: f32) -> Vector4<f32> {
        Vector4::new(
            (1.0 - s).powi(3),
            3.0 * (1.0 - s).powi(2) * s,
            3.0 * (1.0 - s) * s.powi(2),
            s.powi(3),
        )
    }
    pub fn get_bernstein_2(s: f32) -> Vector3<f32> {
        Vector3::new((1.0 - s).powi(2), 2.0 * (1.0 - s) * s, s.powi(2))
    }
    pub fn get_bernstein_1(s: f32) -> Vector2<f32> {
        Vector2::new(1.0 - s, s)
    }

    #[rustfmt::skip]
    const CHAR_MAT_POS: Matrix4<f32> = Matrix4::new(
        1.0, 1.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 1.0,
        0.0, 0.0, -1.0, 0.0,
    );
    #[rustfmt::skip]
    const CHAR_MAT_DER: Matrix4x3<f32> = Matrix4x3::new(
        0.0, -1.0, 0.0,
        1.0, -1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, -1.0, 1.0
    );
    #[rustfmt::skip]
    const CHAR_MAT_DER_2: Matrix4x2<f32> = Matrix4x2::new(
        -1.0, 1.0,
        -2.0, 1.0,
        1.0, -1.0,
        -1.0, 2.0,
    );

    pub fn position(s: f32) -> Vector4<f32> {
        Self::CHAR_MAT_POS * Self::get_bernstein_3(s)
    }
    pub fn derivative(s: f32) -> Vector4<f32> {
        3.0 * Self::CHAR_MAT_DER * Self::get_bernstein_2(s)
    }
    pub fn derivative2(s: f32) -> Vector4<f32> {
        6.0 * Self::CHAR_MAT_DER_2 * Self::get_bernstein_1(s)
    }

    pub fn precompute_mats(steps: usize) -> Precomputed {
        Precomputed {
            steps,
            position: Self::s_iter_end(steps).map(|s| Self::position(s)).collect(),
            derivative: Self::s_iter_end(steps)
                .map(|s| Self::derivative(s))
                .collect(),
            derivative2: Self::s_iter_end(steps)
                .map(|s| Self::derivative2(s))
                .collect(),
        }
    }
}

pub struct Precomputed {
    steps: usize,
    position: Vec<Vector4<f32>>,
    derivative: Vec<Vector4<f32>>,
    derivative2: Vec<Vector4<f32>>,
}

impl Precomputed {
    pub fn position(&self) -> impl Iterator<Item = &Vector4<f32>> {
        self.position[0..self.steps - 1].iter()
    }
    pub fn position_end(&self) -> impl Iterator<Item = &Vector4<f32>> {
        self.position.iter()
    }
    pub fn derivative(&self) -> impl Iterator<Item = &Vector4<f32>> {
        self.derivative[0..self.steps - 1].iter()
    }
    pub fn derivative_end(&self) -> impl Iterator<Item = &Vector4<f32>> {
        self.derivative.iter()
    }
    pub fn derivative2(&self) -> impl Iterator<Item = &Vector4<f32>> {
        self.derivative2[0..self.steps - 1].iter()
    }
    pub fn derivative2_end(&self) -> impl Iterator<Item = &Vector4<f32>> {
        self.derivative2.iter()
    }
}

#[cfg(test)]
mod test {
    use svg::{
        Node,
        node::element::{Circle, Group, Line},
    };

    use super::*;
    #[test]
    fn visual() {
        let side_len = 100.0;
        let s_width = side_len / 500.0;
        let center = Vector::new(side_len / 2.0, side_len / 2.0);

        let points = 4;
        let rot = Rotation::new(-TAU / 18.0);
        let off_set = side_len / points as f32;
        let spline = Spline::new(
            (0..points)
                .map(|i| {
                    let v = Vector::new((i as f32 + 0.5) * off_set, 0.5 * side_len);
                    rot * (v - center) + center
                })
                .collect(),
            (0..points)
                .map(|i| (off_set / 3.0) * (rot * Vector::new(1.0, -(-1.0_f32).powi(i))))
                .collect(),
        );

        let mut doc = svg::Document::new()
            .set("width", side_len)
            .set("height", side_len);
        doc.append(spline.as_borrowed_spline().as_svg_path("black", s_width));

        let steps = 10;

        let mut poss = Group::new();
        let mut ders = Group::new();
        for segment in spline.segments() {
            for (pos, mut der) in segment.pos_and_der_iter(steps) {
                der = der / steps as f32;
                poss.append(
                    Circle::new()
                        .set("cx", pos.x)
                        .set("cy", pos.y)
                        .set("r", s_width)
                        .set("fill", "black"),
                );
                ders.append(
                    Line::new()
                        .set("x1", pos.x)
                        .set("x2", pos.x + der.x)
                        .set("y1", pos.y)
                        .set("y2", pos.y + der.y)
                        .set("stroke-width", s_width)
                        .set("stroke", "blue"),
                )
            }
        }
        doc.append(spline.bounds.as_svg(s_width * 0.7));
        doc.append(ders);
        doc.append(poss);
        svg::save("test.svg", &doc).unwrap()
    }
}
