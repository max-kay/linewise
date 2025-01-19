use crate::Vector;

use super::{Bounded, Node, Rect};

pub trait Query {
    fn predicate(&self, bounds: Rect) -> bool;
}

pub struct IntersectsRect(Rect);

impl Query for IntersectsRect {
    fn predicate(&self, bounds: Rect) -> bool {
        bounds.intersects(&self.0)
    }
}

pub struct ContainsPoint(Vector);

impl Query for ContainsPoint {
    fn predicate(&self, bounds: Rect) -> bool {
        bounds.contains_point(self.0)
    }
}

pub struct IntoIter<T: Bounded> {
    pub(super) stack: Vec<Node<T>>,
}

impl<T: Bounded> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }
        {
            let current_node = self.stack.last_mut().expect("checked above");
            if let Some(item) = current_node.objects.pop() {
                return Some(item);
            }
        }
        let empty_node = self.stack.pop().expect("checked above");
        if let Some(children) = empty_node.children {
            for child in children {
                self.stack.push(*child)
            }
        }
        self.next()
    }
}

impl<T: Bounded> IntoIterator for super::QuadTree<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let mut stack = Vec::new();
        stack.push(self.root);
        IntoIter { stack }
    }
}

impl<T: Bounded> super::QuadTree<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        let mut stack = Vec::new();
        stack.push(&self.root);
        Iter { stack, index: 0 }
    }
}

pub struct Iter<'a, T: Bounded> {
    pub(super) stack: Vec<&'a Node<T>>,
    pub(super) index: usize,
}

impl<'a, T: Bounded> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }
        {
            let current_node = self.stack.last().expect("checked above");

            // try to get next item from current node
            // fails if index >= objects.len()
            if let Some(item) = current_node.objects.get(self.index) {
                self.index += 1; // setup to get next item
                return Some(item);
            }
        }
        // only reached if the current_node has no objects left
        let empty_node = self.stack.pop().expect("checked above");

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

impl<'a, T: Bounded> IntoIterator for &'a super::QuadTree<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct QueryIter<'a, T, Q>
where
    T: Bounded,
    Q: Query,
{
    pub(super) stack: Vec<&'a Node<T>>,
    pub(super) query: Q,
    pub(super) index: usize,
}

impl<T: Bounded> super::QuadTree<T> {
    /// returns an iterator over all elements for which func(element.bounding_box()) is true
    pub fn query_iter<Q: Query>(&self, query: Q) -> QueryIter<'_, T, Q> {
        let mut stack = Vec::new();
        stack.push(&self.root);
        QueryIter {
            stack,
            query,
            index: 0,
        }
    }

    pub fn query_contains_point(&self, point: Vector) -> QueryIter<T, ContainsPoint> {
        self.query_iter(ContainsPoint(point))
    }

    pub fn query_intersects(&self, bounds: Rect) -> QueryIter<T, IntersectsRect> {
        self.query_iter(IntersectsRect(bounds))
    }
}

impl<'a, T, F> Iterator for QueryIter<'a, T, F>
where
    T: Bounded,
    F: Query,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }
        {
            let current_node = self.stack.last().expect("checked above");

            // try to get next item from current node
            // this fails if all objects still in the vec do not fullfill the predicate
            while let Some(item) = current_node.objects.get(self.index) {
                self.index += 1; // setup to get next item
                if self.query.predicate(item.bounding_box()) {
                    return Some(item);
                }
            }
        }
        // only reached if the current_node has no objects left
        let empty_node = self.stack.pop().expect("checked above");

        // if the empty node has children push them on the stack
        if let Some(children) = &empty_node.children {
            for child in children {
                if self.query.predicate(child.bounds) {
                    self.stack.push(&child)
                }
            }
        }

        // set index to 0 for next node on stack
        self.index = 0;

        self.next()
    }
}
