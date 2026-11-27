use std::collections::{HashSet, VecDeque};

/// A simple deque backed by a set for efficient duplication filtering
pub struct ProcessQueue<T: Copy + PartialEq + Eq + std::hash::Hash> {
    deque: VecDeque<T>,
    set: HashSet<T>,
}

impl<T: Copy + PartialEq + Eq + std::hash::Hash> ProcessQueue<T> {
    pub fn new() -> Self {
        Self::from(HashSet::default())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { deque: VecDeque::with_capacity(capacity), set: HashSet::with_capacity(capacity) }
    }

    pub fn from(set: HashSet<T>) -> Self {
        Self { deque: set.iter().copied().collect(), set }
    }

    pub fn enqueue_chunk<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            if self.set.insert(item) {
                self.deque.push_back(item);
            }
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if let Some(item) = self.deque.pop_front() {
            self.set.remove(&item);
            Some(item)
        } else {
            None
        }
    }
}

impl<T: Copy + PartialEq + Eq + std::hash::Hash> IntoIterator for ProcessQueue<T> {
    type Item = T;
    type IntoIter = <std::collections::VecDeque<T> as std::iter::IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.deque.into_iter()
    }
}

impl<T: Copy + PartialEq + Eq + std::hash::Hash> FromIterator<T> for ProcessQueue<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut q = Self::with_capacity(iter.size_hint().0);
        q.enqueue_chunk(iter);
        q
    }
}

#[cfg(test)]
mod tests {
    use super::ProcessQueue;

    #[test]
    fn test_process_queue() {
        let mut q = ProcessQueue::new();
        q.enqueue_chunk([1, 2, 3, 4, 5, 5, 6]);
        for expected in 1..=6 {
            assert_eq!(q.dequeue(), Some(expected));
        }
        assert_eq!(q.dequeue(), None);
        q.enqueue_chunk([7, 8, 7, 1]);
        assert_eq!(q.into_iter().collect::<Vec<_>>(), vec![7, 8, 1]);
    }
}
