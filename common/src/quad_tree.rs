use either::Either::{self, Left, Right};

const OBJECTS_ON_LEAVES: usize = 8;

mod iter;
mod rect;
use random::{MyRng, Rng};
pub use rect::Rect;

pub trait Bounded {
    fn bounding_box(&self) -> Rect;
}

pub struct QuadTree<T: Bounded> {
    root: Node<T>,
    len: usize,
}

impl<T: Bounded> From<Vec<T>> for QuadTree<T> {
    fn from(objects: Vec<T>) -> Self {
        let len = objects.len();
        let bounds = objects
            .iter()
            .map(|val| val.bounding_box())
            .reduce(|acc, val| val.combine(acc))
            .expect("the vector should not be empty");
        Self {
            root: Node::new(objects, bounds),
            len,
        }
    }
}

impl<T: Bounded> QuadTree<T> {
    pub fn new() -> Self {
        Self {
            root: Node::new_placeholder(),
            len: 0,
        }
    }

    pub fn with_bounds(objects: Vec<T>, bounds: Rect) -> Self {
        let len = objects.len();
        Self {
            root: Node::new(objects, bounds),
            len,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn insert(&mut self, val: T) {
        if self.len == 0 {
            self.len += 1;
            self.root = Node::from_single(val);
            return;
        }
        if self.root.bounds.contains(&val.bounding_box()) {
            self.len += 1;
            self.root.insert(val);
        } else {
            let new_bounds = self.root.bounds.combine(val.bounding_box());
            let mut as_vec: Vec<_> =
                std::mem::replace(&mut self.root, Node::new_placeholder()).into();
            as_vec.push(val);
            self.root = Node::new(as_vec, new_bounds);
            self.len += 1;
        }
    }

    pub fn get_bounds(&self) -> Rect {
        self.root.bounds
    }
}

impl<T: Bounded> QuadTree<T> {
    fn pop(&mut self, index: usize) -> T {
        self.len -= 1;
        self.root.pop(index).expect_left("out of bounds pop")
    }

    pub fn pop_random(&mut self, rng: &mut MyRng) -> T {
        self.pop(rng.random_range(0..self.len))
    }
}

impl<T: Bounded> Into<Vec<T>> for QuadTree<T> {
    fn into(self) -> Vec<T> {
        self.root.into()
    }
}

#[derive(Clone)]
struct Node<T: Bounded> {
    bounds: Rect,
    objects: Vec<T>,
    children: Option<[Box<Node<T>>; 4]>,
}

impl<T: Bounded> Node<T> {
    fn new(objects: Vec<T>, bounds: Rect) -> Self {
        assert!(
            objects
                .iter()
                .map(|x| bounds.contains(&x.bounding_box()))
                .reduce(|acc, x| acc && x)
                .unwrap_or(true)
        );

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

    fn from_single(val: T) -> Self {
        Self {
            bounds: val.bounding_box(),
            objects: vec![val],
            children: None,
        }
    }

    /// careful breaks assumptions
    fn new_placeholder() -> Self {
        Self {
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            objects: Vec::new(),
            children: None,
        }
    }

    fn insert(&mut self, val: T) {
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
        } else {
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
        }
        self.objects.push(val)
    }

    fn pop(&mut self, mut index: usize) -> Either<T, usize> {
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

    // fn count_objects(&self) -> usize {
    //     let mut count = self.objects.len();
    //     if let Some(children) = self.children.as_ref() {
    //         for child in children {
    //             count += child.count_objects()
    //         }
    //     }
    //     count
    // }
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
