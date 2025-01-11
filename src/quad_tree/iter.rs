use super::{Bounded, Node, Rect};

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

impl<T: Bounded + Clone> IntoIterator for super::QuadTree<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            stack: vec![self.root],
        }
    }
}

impl<T: Bounded + Clone> super::QuadTree<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            stack: vec![&self.root],
            index: 0,
        }
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

impl<'a, T: Bounded + Clone> IntoIterator for &'a super::QuadTree<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct QuerryIter<'a, T, F>
where
    T: Bounded,
    F: Fn(Rect) -> bool,
{
    pub(super) stack: Vec<&'a Node<T>>,
    pub(super) predicate: F,
    pub(super) index: usize,
}

impl<T: Bounded + Clone> super::QuadTree<T> {
    pub fn querry_iter<F: Fn(Rect) -> bool>(&self, func: F) -> QuerryIter<'_, T, F> {
        QuerryIter {
            stack: vec![&self.root],
            predicate: func,
            index: 0,
        }
    }

    pub fn query_intersects(&self, bounds: Rect) -> QuerryIter<T, impl Fn(Rect) -> bool> {
        self.querry_iter(move |rect| rect.intersects(&bounds))
    }
}

impl<'a, T, F> Iterator for QuerryIter<'a, T, F>
where
    T: Bounded,
    F: Fn(Rect) -> bool,
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
                if (self.predicate)(item.bounding_box()) {
                    return Some(item);
                }
            }
        }
        // only reached if the current_node has no objects left
        let empty_node = self.stack.pop().expect("checked above");

        // if the empty node has children push them on the stack
        if let Some(children) = &empty_node.children {
            for child in children {
                if (self.predicate)(child.bounds) {
                    self.stack.push(&child)
                }
            }
        }

        // set index to 0 for next node on stack
        self.index = 0;

        self.next()
    }
}
