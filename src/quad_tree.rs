use svg::node::element::{path::Data, Path};

use crate::Vector;

#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

impl BoundingBox {
    pub fn new(x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Self {
        assert!(x_min <= x_max && y_min <= y_max);
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        !(self.x_max < other.x_min
            || self.x_min > other.x_max
            || self.y_max < other.y_min
            || self.y_min > other.y_max)
    }

    pub fn contains(&self, other: &BoundingBox) -> bool {
        self.x_min <= other.x_min
            && self.x_max >= other.x_max
            && self.y_min <= other.y_min
            && self.y_max >= other.y_max
    }

    pub fn combine(self, other: Self) -> Self {
        Self {
            x_min: self.x_min.min(other.x_min),
            x_max: self.x_max.max(other.x_max),
            y_min: self.y_min.min(other.y_min),
            y_max: self.y_max.max(other.y_max),
        }
    }
}

impl BoundingBox {
    pub fn signed_distance(&self, position: Vector) -> f32 {
        let x_dist = (self.x_min - position.x).max(position.x - self.x_max);
        let y_dist = (self.y_min - position.y).max(position.y - self.y_max);
        x_dist.max(y_dist)
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

impl BoundingBox {
    pub fn as_rect(&self, stroke_width: f32) -> Path {
        let data = Data::new()
            .move_to((self.x_min, self.y_min))
            .line_to((self.x_min, self.y_max))
            .line_to((self.x_max, self.y_max))
            .line_to((self.x_max, self.y_min))
            .line_to((self.x_min, self.y_min));
        Path::new()
            .set("fill", "none")
            .set("stroke", "red")
            .set("stroke-width", stroke_width)
            .set("d", data)
    }
}

pub trait Bounded {
    fn bounding_box(&self) -> BoundingBox;
}

pub struct QuadTree<T: Bounded> {
    root: Node<T>,
}

impl<T: Bounded> QuadTree<T> {
    pub fn new(objects: Vec<T>, objects_on_leafs: usize) -> Self {
        // TODO clean up testing
        let bounds = objects
            .iter()
            .map(|val| val.bounding_box())
            .reduce(|acc, val| val.combine(acc))
            .unwrap();
        let original_len = objects.len();
        let root = Node::new(objects, bounds, objects_on_leafs);
        debug_assert_eq!(original_len, root.count_objects());
        Self { root }
    }

    pub fn count_objects(&self) -> usize {
        self.root.count_objects()
    }
}

impl<T: Bounded> QuadTree<T> {
    pub fn query(&self, bounds: BoundingBox) -> Vec<&T> {
        let mut vec = Vec::new();
        self.root.query(bounds, &mut vec);
        vec
    }

    pub fn query_mut(&mut self, bounds: BoundingBox) -> Vec<&mut T> {
        let mut vec = Vec::new();
        self.root.query_mut(bounds, &mut vec);
        vec
    }
}

struct Node<T: Bounded> {
    bounds: BoundingBox,
    objects: Vec<T>,
    children: Option<[Box<Node<T>>; 4]>,
}

impl<T: Bounded> Node<T> {
    pub fn new(objects: Vec<T>, bounds: BoundingBox, objects_on_leafs: usize) -> Self {
        debug_assert!(objects
            .iter()
            .map(|x| bounds.contains(&x.bounding_box()))
            .reduce(|acc, x| acc && x)
            .unwrap_or(true));
        if objects.len() <= objects_on_leafs {
            dbg!(bounds);
            return Self {
                bounds,
                objects,
                children: None,
            };
        }
        let mut remaining = Vec::new();
        let boxes = bounds.get_quadrants();
        let mut sorted_obj: [Vec<T>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];

        'outer: for object in objects {
            for i in 0..4 {
                if boxes[i].contains(&object.bounding_box()) {
                    sorted_obj[i].push(object);
                    continue 'outer;
                }
            }
            remaining.push(object);
        }

        let children = std::array::from_fn(|i| {
            Box::new(Node::new(
                sorted_obj[i].drain(..).collect(),
                boxes[i],
                objects_on_leafs,
            ))
        });

        dbg!(remaining.len());
        Self {
            bounds,
            objects: remaining,
            children: Some(children),
        }
    }

    pub fn count_objects(&self) -> usize {
        let mut count = self.objects.len();
        if let Some(children) = self.children.as_ref() {
            for child in children {
                count += child.count_objects()
            }
        }
        count
    }

    pub fn query<'a>(&'a self, bounds: BoundingBox, vec: &mut Vec<&'a T>) {
        if !self.bounds.intersects(&bounds) {
            return;
        }
        for obj in &self.objects {
            if obj.bounding_box().intersects(&bounds) {
                vec.push(&obj)
            }
        }

        if let Some(children) = &self.children {
            for child in children {
                child.query(bounds, vec)
            }
        }
    }

    pub fn query_mut<'a>(&'a mut self, bounds: BoundingBox, vec: &mut Vec<&'a mut T>) {
        for obj in &mut self.objects {
            if obj.bounding_box().intersects(&bounds) {
                vec.push(obj)
            }
        }

        if let Some(children) = &mut self.children {
            for child in children {
                child.query_mut(bounds, vec)
            }
        }
    }
}

impl<T: Bounded> Into<Vec<T>> for QuadTree<T> {
    fn into(self) -> Vec<T> {
        self.root.into()
    }
}
impl<T: Bounded> Into<Vec<T>> for Node<T> {
    fn into(mut self) -> Vec<T> {
        let mut vec: Vec<_> = self.objects.drain(..).collect();
        if let Some(children) = self.children {
            for child in children {
                vec.append(&mut ((*child).into()))
            }
        }
        vec
    }
}
