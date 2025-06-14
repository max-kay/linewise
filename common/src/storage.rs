use std::{
    ops::{Index, IndexMut},
    usize,
};

use nalgebra::Matrix2x4;
use tiny_skia::{Color, Paint, Pixmap, Stroke, Transform};

use crate::{
    Vector,
    quad_tree::{Bounded, Rect},
    spline::{BorrowedSpline, Segment, Spline},
};

pub struct SplineStorage {
    points_and_vecs: Vec<Vector>,
    spline_starts: Vec<usize>,
    empty_slot: Option<SplineRef>,
}

impl Clone for SplineStorage {
    fn clone(&self) -> Self {
        Self {
            points_and_vecs: self.points_and_vecs.clone(),
            spline_starts: self.spline_starts.clone(),
            empty_slot: None,
        }
    }
}

impl SplineStorage {
    pub fn new() -> Self {
        Self {
            points_and_vecs: Vec::new(),
            spline_starts: Vec::new(),
            empty_slot: None,
        }
    }

    pub fn add_spline(&mut self, spline: Spline) -> SplineRef {
        let bounds = spline.bounding_box();
        let storage_idx = self.points_and_vecs.len();
        let list_idx = self.spline_starts.len();
        self.spline_starts.push(storage_idx);
        let segments = spline.count_segments();
        self.points_and_vecs.append(&mut spline.into_vec());
        SplineRef {
            storage_idx: storage_idx as u32,
            segments: segments as u32,
            list_idx: list_idx as u32,
            bounds,
        }
    }

    pub fn shrink_to_fit(&mut self) {
        self.points_and_vecs.shrink_to_fit();
        self.spline_starts.shrink_to_fit();
    }

    pub fn read(&mut self, spline: SplineRef) -> Spline {
        let owned = Spline::from_parts(
            &self.points_and_vecs[spline.storage_idx as usize
                ..(spline.storage_idx + 2 * (spline.segments + 1)) as usize],
            spline.bounds,
        );
        self.empty_slot = Some(spline);
        owned
    }

    pub fn is_empty(&self, spline: &SplineRef) -> bool {
        if let Some(ref this_spline) = self.empty_slot.as_ref() {
            if spline.storage_idx == this_spline.storage_idx {
                return true;
            }
        }
        false
    }

    pub fn revalidate_ref(&mut self, spline: Spline) -> SplineRef {
        debug_assert!(
            spline.count_segments() as u32
                == self
                    .empty_slot
                    .as_ref()
                    .expect("tried to overwrite spline but there was no empty slot")
                    .segments,
        );
        drop(spline);
        self.empty_slot.take().unwrap()
    }

    pub fn overwrite_spline(&mut self, spline: Spline) -> SplineRef {
        debug_assert!(
            spline.count_segments() as u32
                == self
                    .empty_slot
                    .as_ref()
                    .expect("tried to overwrite spline but there was no empty slot")
                    .segments,
        );
        let mut this_ref = self.empty_slot.take().unwrap();
        this_ref.bounds = spline.bounding_box();
        for (i, val) in spline.into_vec().into_iter().enumerate() {
            self.points_and_vecs[i + this_ref.storage_idx as usize] = val
        }
        this_ref
    }

    pub fn default_spline_info<T: Default>(&self) -> SplineInfo<T> {
        SplineInfo(
            (0..self.spline_starts.len())
                .map(|_| Default::default())
                .collect(),
        )
    }
    pub fn new_spline_info<T: Clone>(&self, val: T) -> SplineInfo<T> {
        SplineInfo(vec![val; self.spline_starts.len()])
    }

    pub fn make_spline_info<T>(&self, func: impl Fn(BorrowedSpline<'_>) -> T) -> SplineInfo<T> {
        let mut vec = Vec::with_capacity(self.spline_starts.len());
        for i in 0..self.spline_starts.len() - 1 {
            let start = self.spline_starts[i];
            let end = self.spline_starts[i + 1];
            vec.push(func(BorrowedSpline::from_slice(
                &self.points_and_vecs[start..end],
            )))
        }
        vec.push(func(BorrowedSpline::from_slice(
            &self.points_and_vecs[*self
                .spline_starts
                .last()
                .expect("spline starts is never empty")
                ..(self.points_and_vecs.len())],
        )));
        SplineInfo(vec)
    }

    pub fn default_segement_info<T: Default>(&self) -> SegmentInfo<T> {
        SegmentInfo(
            (0..self.points_and_vecs.len() - self.spline_starts.len())
                .map(|_| Default::default())
                .collect(),
        )
    }
    pub fn new_segment_info<T: Clone>(&self, val: T) -> SegmentInfo<T> {
        SegmentInfo(vec![
            val;
            self.points_and_vecs.len() - self.spline_starts.len()
        ])
    }

    pub fn make_segment_info<T>(&self, func: impl Fn(Segment) -> T) -> SegmentInfo<T> {
        SegmentInfo(self.all_segments().map(func).collect())
    }
}

impl SplineStorage {
    pub fn get_segments(&self, idx: &SplineRef) -> impl Iterator<Item = Segment> {
        debug_assert!(
            (idx.storage_idx + (idx.segments + 1) * 2) as usize <= self.points_and_vecs.len(),
            "spline with idx {} out of bounds, storage_len: {}",
            idx.storage_idx,
            self.points_and_vecs.len()
        );
        self.get_spline(idx).count_segments()
    }

    pub fn get_spline(&self, idx: &SplineRef) -> BorrowedSpline {
        BorrowedSpline::from_slice(
            &self.points_and_vecs
                [idx.storage_idx as usize..(idx.storage_idx + 2 * idx.segments) as usize],
        )
    }

    pub fn all_splines(&self) -> impl Iterator<Item = BorrowedSpline> {
        SplineIter {
            points_and_vecs: &self.points_and_vecs,
            spline_starts: &self.spline_starts[1..],
            next_start: 0,
        }
    }

    pub fn all_segments(&self) -> impl Iterator<Item = Segment> {
        (0..self.points_and_vecs.len() / 2 - 1).flat_map(|i| {
            let st_idx = i * 2;
            // if the end point of the window I'm trying to construct is the start of a segment
            // then that window is no valid segment and should be skipped
            if self.spline_starts.binary_search(&(st_idx + 2)).is_ok() {
                None
            } else {
                // SAFETY: the highest value i can take is len / 2 - 2
                // so the st_idx is at most len - 4
                // the slice [len - 4..len] is allways valid
                let window = &self.points_and_vecs[st_idx..st_idx + 4];
                Some(Segment::from_mat(Matrix2x4::from_columns(window)))
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
        let mut pix_map = Pixmap::new(width, height).expect("pixmap size is valid");
        pix_map.fill(Color::from_rgba8(255, 255, 255, 255));
        for spline in self.all_splines() {
            pix_map.stroke_path(
                &spline.as_ts_path(),
                &paint,
                &Stroke {
                    width: line_width,
                    ..Default::default()
                },
                Transform::from_scale(scaling_factor, scaling_factor),
                None,
            );
        }
        pix_map
    }
}

pub struct SplineIter<'a> {
    points_and_vecs: &'a [Vector],
    spline_starts: &'a [usize],
    next_start: usize,
}

impl<'a> Iterator for SplineIter<'a> {
    type Item = BorrowedSpline<'a>;

    // FIXME: this can be done nicer
    fn next(&mut self) -> Option<Self::Item> {
        if self.spline_starts.is_empty() {
            if self.next_start == 0 {
                return None;
            }
            let item = Some(BorrowedSpline::from_slice(
                &self.points_and_vecs[self.next_start..],
            ));
            self.next_start = 0;
            return item;
        }
        let next = self.spline_starts[0];
        self.spline_starts = &self.spline_starts[1..];
        let item = BorrowedSpline::from_slice(&self.points_and_vecs[self.next_start..next]);
        self.next_start = next;
        Some(item)
    }
}

pub struct SplineRef {
    storage_idx: u32,
    segments: u32,
    list_idx: u32,
    bounds: Rect,
}

impl PartialEq for SplineRef {
    fn eq(&self, other: &Self) -> bool {
        self.storage_idx == other.storage_idx
    }
}

impl Eq for SplineRef {}

impl PartialOrd for SplineRef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.storage_idx.partial_cmp(&other.storage_idx)
    }
}
impl Ord for SplineRef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.storage_idx.cmp(&other.storage_idx)
    }
}

impl Bounded for SplineRef {
    fn bounding_box(&self) -> Rect {
        self.bounds
    }
}

pub struct SplineInfo<T>(Vec<T>);

impl<T> SplineInfo<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.0.iter_mut()
    }
}

impl<T> IndexMut<&SplineRef> for SplineInfo<T> {
    fn index_mut(&mut self, index: &SplineRef) -> &mut Self::Output {
        &mut self.0[index.list_idx as usize]
    }
}

impl<T> Index<&SplineRef> for SplineInfo<T> {
    type Output = T;

    fn index(&self, index: &SplineRef) -> &Self::Output {
        &self.0[index.list_idx as usize]
    }
}

pub struct SegmentInfo<T>(Vec<T>);

impl<T> IndexMut<&SplineRef> for SegmentInfo<T> {
    fn index_mut(&mut self, index: &SplineRef) -> &mut Self::Output {
        let start = index.storage_idx - index.list_idx;
        &mut self.0[start as usize..(start + index.segments) as usize]
    }
}

impl<T> Index<&SplineRef> for SegmentInfo<T> {
    type Output = [T];

    fn index(&self, index: &SplineRef) -> &Self::Output {
        let start = index.storage_idx - index.list_idx;
        &self.0[start as usize..(start + index.segments) as usize]
    }
}
