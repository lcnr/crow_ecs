use std::collections::BTreeMap;

use crate::{Entity, Join, Joinable, Joined, SparseStorage, Storage};

impl<T> Storage<T> {
    /// Removes all component of this storage
    pub fn drain(&mut self) -> Drain<T> {
        Drain(self, 0)
    }
}

/// The iterator returned by `Storage::drain`.
///
/// Using this struct in a `join` after mutating it
/// can easily lead to unexpected, but not *unsound* behavior.
pub struct Drain<'a, T>(&'a mut Storage<T>, usize);

impl<'a, T> Join for Drain<'a, T> {
    fn may_skip(&mut self, curr: usize) -> usize {
        self.0.inner[curr..]
            .iter()
            .take_while(|opt| opt.is_none())
            .count()
    }
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let item = self.0.remove(Entity(self.1));
        self.1 += 1;
        item
    }

    fn nth(&mut self, n: usize) -> Option<T> {
        for _ in 0..n {
            self.0.remove(Entity(self.1));
            self.1 += 1;
        }

        self.next()
    }
}

impl<'a, T> Joinable for Drain<'a, T> {
    type Joined = Drain<'a, T>;
    type Item = T;

    fn join(self) -> Joined<Self::Joined> {
        let len = self.0.inner.len();
        Joined::new(self, len)
    }
}

impl<T> SparseStorage<T> {
    /// Removes all component of this storage.
    pub fn drain(&mut self) -> SparseDrain<T> {
        SparseDrain {
            inner: &mut self.inner,
            position: 0,
        }
    }
}

/// The iterator returned by `SparseStorage::drain`.
///
/// Using this struct in a `join` after mutating it
/// can easily lead to unexpected, but not *unsound* behavior.
pub struct SparseDrain<'a, T> {
    inner: &'a mut BTreeMap<usize, T>,
    position: usize,
}

impl<'a, T> Drop for SparseDrain<'a, T> {
    fn drop(&mut self) {
        self.inner.clear()
    }
}

impl<'a, T> Join for SparseDrain<'a, T> {
    fn may_skip(&mut self, curr: usize) -> usize {
        self.position = curr;
        self.inner
            .range(self.position..)
            .next()
            .map_or(std::usize::MAX, |(&k, _)| k - self.position)
    }
}

impl<'a, T> Iterator for SparseDrain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let item = self.inner.remove(&self.position);
        self.position += 1;
        item
    }

    fn nth(&mut self, n: usize) -> Option<T> {
        self.position += n;
        self.next()
    }
}

impl<'a, T> Joinable for SparseDrain<'a, T> {
    type Joined = SparseDrain<'a, T>;
    type Item = T;

    fn join(self) -> Joined<Self::Joined> {
        let len = self.inner.keys().last().copied().map_or(0, |v| v + 1);
        Joined::new(self, len)
    }
}
