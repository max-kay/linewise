use std::usize;

#[derive(Copy, Clone)]
pub struct BoundingBox {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

impl BoundingBox {
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
}

impl BoundingBox {
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

pub trait Bounded {
    fn bounding_box(&self) -> BoundingBox;
}

pub struct QuadTree<T: Bounded> {
    root: Node<T>,
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
            .fold(true, |acc, x| acc && x));
        if objects.len() <= objects_on_leafs {
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

        // Use a closure to create the children nodes for each quadrant
        let children = std::array::from_fn(|i| {
            Box::new(Node::new(
                sorted_obj[i].drain(..).collect(),
                boxes[i],
                objects_on_leafs,
            ))
        });
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
