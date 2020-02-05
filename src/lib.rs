#![forbid(unsafe_code)]
//! A simple ecs utility crate using no unsafe code.

use std::{iter, mem, ops::RangeFrom, slice};

mod tuple;

/// This crate does currently not protect from generation missmatches.
///
/// To delete an entity one has to remove it from all storages, which is probably not the best approach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity(pub usize);

#[derive(Debug, Clone)]
pub struct Storage<T> {
    inner: Vec<Option<T>>,
}

impl<T> Storage<T> {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Removes all components in this storage.
    pub fn clear(&mut self) {
        // We don't just clear `inner`, as we
        // don't want to resize the vector the next time
        // a component is inserted.
        for c in self.inner.iter_mut() {
            *c = None;
        }
    }

    /// Inserts a component for the entity at `idx`.
    ///
    /// In case the component was already present the previous
    /// one is returned.
    pub fn insert(&mut self, idx: Entity, c: T) -> Option<T> {
        if idx.0 >= self.inner.len() {
            self.inner.resize_with(idx.0 + 1, || None);
        }

        mem::replace(&mut self.inner[idx.0], Some(c))
    }

    /// Removes this component for the entity at `idx`.
    pub fn remove(&mut self, idx: Entity) -> Option<T> {
        self.inner.get_mut(idx.0).map(Option::take).flatten()
    }

    /// Removes all compo
    pub fn drain(&mut self) -> Drain<T> {
        Drain(self)
    }
}

pub struct Drain<'a, T>(&'a mut Storage<T>);

pub struct DrainIter<'a, T>(&'a mut Storage<T>, usize);

impl<'a, T> Iterator for DrainIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.1 < self.0.inner.len() {
            let item = self.0.remove(Entity(self.1));
            self.1 += 1;
            item
        } else {
            None
        }
    }
}

impl<'a, T> Joinable for Drain<'a, T> {
    type Joined = DrainIter<'a, T>;
    type Item = T;

    fn join(self) -> Joined<Self> {
        let len = self.0.inner.len();
        Joined::new(DrainIter(self.0, 0), len)
    }
}

pub struct Iter<'a, T>(slice::Iter<'a, Option<T>>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.0.next().map(Option::as_ref).flatten()
    }
}

impl<'a, T> Joinable for &'a Storage<T> {
    type Joined = Iter<'a, T>;
    type Item = &'a T;

    fn join(self) -> Joined<Self> {
        Joined::new(Iter(self.inner.iter()), self.inner.len())
    }
}

pub struct IterMut<'a, T>(slice::IterMut<'a, Option<T>>);

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        self.0.next().map(Option::as_mut).flatten()
    }
}

impl<'a, T> Joinable for &'a mut Storage<T> {
    type Joined = IterMut<'a, T>;
    type Item = &'a mut T;

    fn join(self) -> Joined<Self> {
        let len = self.inner.len();
        Joined::new(IterMut(self.inner.iter_mut()), len)
    }
}

pub struct Entities;

impl Joinable for Entities {
    type Joined = iter::Map<RangeFrom<usize>, fn(usize) -> Entity>;
    type Item = Entity;

    fn join(self) -> Joined<Self> {
        Joined::new((0..).map(Entity as fn(usize) -> Entity), std::usize::MAX)
    }
}

pub struct Joined<T: Joinable + ?Sized> {
    iter: T::Joined,
    len: usize,
    pos: usize,
}

impl<T: Joinable + ?Sized> Joined<T> {
    pub fn new(iter: T::Joined, len: usize) -> Self {
        Self { iter, len, pos: 0 }
    }
}

impl<T: Joinable + ?Sized> Iterator for Joined<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<T::Item> {
        while self.pos < self.len {
            if let Some(item) = self.iter.next() {
                return Some(item);
            } else {
                self.pos += 1;
            }
        }

        None
    }
}

pub trait Joinable {
    type Joined: Iterator<Item = Self::Item> + Sized;
    type Item;

    fn join(self) -> Joined<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_join() {
        let a = Entity(0);
        let b = Entity(1);
        let c = Entity(2);

        let mut d: Storage<u32> = Storage::new();
        let mut e: Storage<u8> = Storage::new();

        d.insert(a, 7);
        d.insert(b, 12);
        e.insert(b, 17);
        e.insert(c, 0);

        for (&d_entry, &e_entry) in (&d, &e).join() {
            assert_eq!(d_entry, 12);
            assert_eq!(e_entry, 17);
        }
    }

    #[test]
    fn entities() {
        let a = Entity(0);
        let b = Entity(1);
        let c = Entity(2);

        let mut d: Storage<u32> = Storage::new();
        let mut e: Storage<u8> = Storage::new();

        d.insert(a, 7);
        d.insert(b, 12);
        e.insert(b, 17);
        e.insert(c, 0);

        for (&d_entry, &e_entry, entity) in (&d, &e, Entities).join() {
            assert_eq!(d_entry, 12);
            assert_eq!(e_entry, 17);
            assert_eq!(entity, b);
        }
    }
}
