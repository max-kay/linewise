use either::Either::{self, Left, Right};
use rand::Rng;
use svg::node::element::{path::Data, Path};

use crate::{MyRng, Vector};

const OBJECTS_ON_LEAVES: usize = 6;

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
}

impl BoundingBox {
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
    len: usize,
}

impl<T: Bounded> QuadTree<T> {
    pub fn new(objects: Vec<T>) -> Self {
        let len = objects.len();
        let bounds = objects
            .iter()
            .map(|val| val.bounding_box())
            .reduce(|acc, val| val.combine(acc))
            .unwrap();
        Self {
            root: Node::new(objects, bounds),
            len,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn insert(&mut self, val: T) {
        if !self.root.bounds.contains(&val.bounding_box()) {
            let new_bounds = self.root.bounds.combine(val.bounding_box());
            let mut as_vec: Vec<_> =
                std::mem::replace(&mut self.root, Node::new_placeholder()).into();
            as_vec.push(val);
            self.root = Node::new(as_vec, new_bounds);
            self.len += 1;
            return;
        }
        self.len += 1;
        self.root.insert(val);
    }
}

impl<T: Bounded> QuadTree<T> {
    pub fn query_intersects(&self, bounds: BoundingBox) -> Vec<&T> {
        let mut vec = Vec::new();
        self.root.query_intersects(bounds, &mut vec);
        vec
    }

    pub fn query_intersects_mut(&mut self, bounds: BoundingBox) -> Vec<&mut T> {
        let mut vec = Vec::new();
        self.root.query_intersects_mut(bounds, &mut vec);
        vec
    }

    pub fn query_not_contains(&self, bounds: BoundingBox) -> Vec<&T> {
        let mut vec = Vec::new();
        self.root.query_not_contains(bounds, &mut vec);
        vec
    }

    pub fn iter(&self) -> QuadTreeIter<'_, T> {
        QuadTreeIter {
            stack: vec![&self.root],
            index: 0,
        }
    }
}

impl<T: Bounded> QuadTree<T> {
    fn pop(&mut self, index: usize) -> T {
        self.len -= 1;
        self.root.pop(index).expect_left("out of bounds pop")
    }

    pub fn pop_random(&mut self, rng: &mut MyRng) -> T {
        self.pop(rng.gen_range(0..self.len))
    }
}

struct Node<T: Bounded> {
    bounds: BoundingBox,
    objects: Vec<T>,
    children: Option<[Box<Node<T>>; 4]>,
}

impl<T: Bounded> Node<T> {
    pub fn new(objects: Vec<T>, bounds: BoundingBox) -> Self {
        assert!(objects
            .iter()
            .map(|x| bounds.contains(&x.bounding_box()))
            .reduce(|acc, x| acc && x)
            .unwrap_or(true));

        if objects.len() <= OBJECTS_ON_LEAVES {
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
            Box::new(Node::new(sorted_obj[i].drain(..).collect(), boxes[i]))
        });

        Self {
            bounds,
            objects: remaining,
            children: Some(children),
        }
    }

    /// careful breaks assumptions
    pub fn new_placeholder() -> Self {
        Self {
            bounds: BoundingBox::new(0.0, 0.0, 0.0, 0.0),
            objects: Vec::new(),
            children: None,
        }
    }

    pub fn insert(&mut self, val: T) {
        if self.objects.len() < OBJECTS_ON_LEAVES {
            self.objects.push(val);
            return;
        }

        let bounds = val.bounding_box();
        if let Some(children) = &mut self.children {
            for child in children {
                if child.bounds.contains(&bounds) {
                    child.insert(val);
                    return;
                }
            }
        }

        let boxes = self.bounds.get_quadrants();

        for i in 0..4 {
            if boxes[i].contains(&bounds) {
                let mut children =
                    std::array::from_fn(|i| Box::new(Node::new(Vec::new(), boxes[i])));
                children[i].insert(val);
                self.children = Some(children);
                return;
            }
        }
        self.objects.push(val)
    }

    pub fn pop(&mut self, mut index: usize) -> Either<T, usize> {
        if index < self.objects.len() {
            return Left(self.objects.swap_remove(index));
        }
        index -= self.objects.len();
        if let Some(children) = &mut self.children {
            for child in children {
                match child.pop(index) {
                    Left(val) => return Left(val),
                    Right(idx) => index = idx,
                }
            }
        }
        return Right(index);
    }

    // pub fn count_objects(&self) -> usize {
    //     let mut count = self.objects.len();
    //     if let Some(children) = self.children.as_ref() {
    //         for child in children {
    //             count += child.count_objects()
    //         }
    //     }
    //     count
    // }

    pub fn query_intersects<'a>(&'a self, bounds: BoundingBox, vec: &mut Vec<&'a T>) {
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
                child.query_intersects(bounds, vec)
            }
        }
    }
    // TODO
    // in all querries pay attention to non commutativity of predicate
    pub fn query_intersects_mut<'a>(&'a mut self, bounds: BoundingBox, vec: &mut Vec<&'a mut T>) {
        if !self.bounds.intersects(&bounds) {
            return;
        }
        for obj in &mut self.objects {
            if obj.bounding_box().intersects(&bounds) {
                vec.push(obj)
            }
        }

        if let Some(children) = &mut self.children {
            for child in children {
                child.query_intersects_mut(bounds, vec)
            }
        }
    }

    pub fn query_not_contains<'a>(&'a self, bounds: BoundingBox, vec: &mut Vec<&'a T>) {
        if bounds.contains(&self.bounds) {
            return;
        }
        for obj in &self.objects {
            if !obj.bounding_box().contains(&bounds) {
                vec.push(obj)
            }
        }

        if let Some(children) = &self.children {
            for child in children {
                child.query_not_contains(bounds, vec)
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

pub struct QuadTreeIter<'a, T: Bounded> {
    stack: Vec<&'a Node<T>>,
    index: usize,
}

impl<'a, T: Bounded> Iterator for QuadTreeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        {
            // if the stack is empty this line is an early return
            let current_node = self.stack.last()?;
            self.index += 1;
            if let Some(item) = current_node.objects.get(self.index) {
                return Some(item);
            }
        }
        // only reached if the current_node has no objects left
        let empty_node = self
            .stack
            .pop()
            .expect("this should have been caught in the first line of the previous scope");

        // if the empty node has children push them on the stack
        if let Some(children) = &empty_node.children {
            for child in children {
                self.stack.push(&child)
            }
        }

        // set index to 0 for next node on stack
        self.index = 0;

        self.next()
    }
}

// struct QuadTreeIterMut<'a, T: Bounded> {
//     refs: Vec<&'a mut Node<T>>,
//     index: usize,
// }
//
// impl<'a, T: Bounded> Iterator for QuadTreeIterMut<'a, T> {
//     type Item = &'a mut T;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         {
//             // if the stack is empty this line is an early return
//             let current_node = self.refs.last_mut()?;
//             self.index += 1;
//             if let Some(item) = current_node.objects.get_mut(self.index) {
//                 return Some(item);
//             }
//         }
//         {
//             // only reached if the current_node has no objects left
//             let mut empty_node = self
//                 .refs
//                 .pop()
//                 .expect("this should have been caught in the first line of the previous scope");
//
//             // if the empty node has children push them on the stack
//             if let Some(children) = &empty_node.children {
//                 for mut child in children {
//                     self.refs.push(&mut child)
//                 }
//             }
//         }
//
//         // set index to 0 for next node on stack
//         self.index = 0;
//
//         self.next()
//     }
// }
