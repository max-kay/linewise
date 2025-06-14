use serde::{Deserialize, Serialize};
use svg::node::element::{Path, path::Data};

use crate::Vector;
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            x_min: 0.0,
            x_max: 0.0,
            y_min: 0.0,
            y_max: 0.0,
        }
    }
}

impl Rect {
    pub fn new(x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Self {
        assert!(
            x_min <= x_max && y_min <= y_max,
            "with x: {} {} and y: {} {}",
            x_min,
            x_max,
            y_min,
            y_max
        );
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        !(self.x_max < other.x_min
            || self.x_min > other.x_max
            || self.y_max < other.y_min
            || self.y_min > other.y_max)
    }

    pub fn contains(&self, other: &Rect) -> bool {
        self.x_min <= other.x_min
            && self.x_max >= other.x_max
            && self.y_min <= other.y_min
            && self.y_max >= other.y_max
    }

    pub fn contains_point(&self, point: Vector) -> bool {
        self.x_min <= point.x
            && self.x_max >= point.x
            && self.y_min <= point.y
            && self.y_max >= point.y
    }

    pub fn combine(self, other: Self) -> Self {
        Self {
            x_min: self.x_min.min(other.x_min),
            x_max: self.x_max.max(other.x_max),
            y_min: self.y_min.min(other.y_min),
            y_max: self.y_max.max(other.y_max),
        }
    }

    pub fn from_points(points: &[Vector]) -> Self {
        debug_assert!(!points.is_empty());
        let mut x_min = points[0].x;
        let mut x_max = points[0].x;
        let mut y_min = points[0].y;
        let mut y_max = points[0].y;
        for p in &points[1..] {
            if p.x < x_min {
                x_min = p.x
            } else if p.x > x_max {
                x_max = p.x
            }

            if p.y < y_min {
                y_min = p.y
            } else if p.y > y_max {
                y_max = p.y
            }
        }
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    pub fn translate(self, vector: Vector) -> Self {
        Self {
            x_min: self.x_min + vector.x,
            x_max: self.x_max + vector.x,
            y_min: self.y_min + vector.y,
            y_max: self.y_max + vector.y,
        }
    }

    pub fn add_radius(self, radius: f32) -> Self {
        Self::new(
            self.x_min - radius,
            self.x_max + radius,
            self.y_min - radius,
            self.y_max + radius,
        )
    }

    pub fn width(&self) -> f32 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> f32 {
        self.y_max - self.y_min
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width() / self.height()
    }
}

impl Rect {
    pub fn to_box_coords(&self, position: Vector) -> (f32, f32) {
        (
            (position[0] - self.x_min) / (self.x_max - self.x_min),
            (position[1] - self.y_min) / (self.y_max - self.y_min),
        )
    }

    pub fn from_box_coords(&self, box_coords: (f32, f32)) -> Vector {
        Vector::new(
            box_coords.0 * (self.x_max - self.x_min) + self.x_min,
            box_coords.1 * (self.y_max - self.y_min) + self.y_min,
        )
    }

    pub fn signed_distance(&self, position: Vector) -> f32 {
        let x_dist = (self.x_min - position.x).max(position.x - self.x_max);
        let y_dist = (self.y_min - position.y).max(position.y - self.y_max);
        x_dist.max(y_dist)
    }

    pub fn get_center(&self) -> Vector {
        Vector::new(
            (self.x_min + self.x_max) / 2.0,
            (self.y_min + self.y_max) / 2.0,
        )
    }

    pub fn get_width(&self) -> f32 {
        self.x_max - self.x_min
    }

    pub fn get_height(&self) -> f32 {
        self.y_max - self.y_min
    }

    pub fn get_quadrants(&self) -> [Self; 4] {
        [
            Self {
                x_min: self.x_min,
                x_max: (self.x_min + self.x_max) / 2.0,
                y_min: self.y_min,
                y_max: (self.y_min + self.y_max) / 2.0,
            },
            Self {
                x_min: self.x_min,
                x_max: (self.x_min + self.x_max) / 2.0,
                y_min: (self.y_min + self.y_max) / 2.0,
                y_max: self.y_max,
            },
            Self {
                x_min: (self.x_min + self.x_max) / 2.0,
                x_max: self.x_max,
                y_min: self.y_min,
                y_max: (self.y_min + self.y_max) / 2.0,
            },
            Self {
                x_min: (self.x_min + self.x_max) / 2.0,
                x_max: self.x_max,
                y_min: (self.y_min + self.y_max) / 2.0,
                y_max: self.y_max,
            },
        ]
    }
}

impl Rect {
    pub fn as_svg(&self, stroke_width: f32) -> Path {
        let data = Data::new()
            .move_to((self.x_min, self.y_min))
            .line_to((self.x_min, self.y_max))
            .line_to((self.x_max, self.y_max))
            .line_to((self.x_max, self.y_min))
            .close();
        Path::new()
            .set("fill", "none")
            .set("stroke", "red")
            .set("stroke-width", stroke_width)
            .set("d", data)
    }
}
