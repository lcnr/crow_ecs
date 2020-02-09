#![forbid(unsafe_code)]
//! A simple ecs utility crate using no unsafe code.

use std::{
    iter, mem,
    ops::{Not, RangeFrom},
    slice,
};

mod tuple;

/// This crate does currently not protect from generation missmatches.
///
/// To delete an entity one has to remove it from all storages, which is probably not the best approach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity(pub usize);

#[derive(Debug, Clone)]
pub struct Storage<T> {
    inner: Vec<Option<T>>,
}

impl<T> Default for Storage<T> {
    fn default() -> Self {
        Storage::new()
    }
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

    /// Returns the component of the entity at `idx` in case it exists.
    pub fn get(&self, idx: Entity) -> Option<&T> {
        if let Some(i) = self.inner.get(idx.0) {
            i.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, idx: Entity) -> Option<&mut T> {
        if let Some(i) = self.inner.get_mut(idx.0) {
            i.as_mut()
        } else {
            None
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

pub struct NegatedStorage<'a, T>(&'a Storage<T>);

impl<'a, T> Not for &'a Storage<T> {
    type Output = NegatedStorage<'a, T>;

    fn not(self) -> NegatedStorage<'a, T> {
        NegatedStorage(self)
    }
}

impl<'a, T> Not for &'a &Storage<T> {
    type Output = NegatedStorage<'a, T>;

    fn not(self) -> NegatedStorage<'a, T> {
        NegatedStorage(self)
    }
}

impl<'a, T> Not for &'a &mut Storage<T> {
    type Output = NegatedStorage<'a, T>;

    fn not(self) -> NegatedStorage<'a, T> {
        NegatedStorage(self)
    }
}

pub struct NegatedIter<'a, T>(Iter<'a, T>);

impl<'a, T> Join for NegatedIter<'a, T> {
    fn may_skip(&mut self, _curr: usize) -> usize {
        self.0.slice.iter().take_while(|opt| opt.is_some()).count()
    }
}

impl<'a, T> Clone for NegatedIter<'a, T> {
    fn clone(&self) -> Self {
        NegatedIter(self.0.clone())
    }
}

impl<'a, T> Iterator for NegatedIter<'a, T> {
    type Item = ();

    fn next(&mut self) -> Option<()> {
        if let Some(_) = self.0.next() {
            None
        } else {
            Some(())
        }
    }

    fn nth(&mut self, n: usize) -> Option<()> {
        if let Some(_) = self.0.nth(n) {
            None
        } else {
            Some(())
        }
    }
}

impl<'a, T> Joinable for NegatedStorage<'a, T> {
    type Joined = NegatedIter<'a, T>;
    type Item = ();

    fn join(self) -> Joined<Self::Joined> {
        let storage = self.0.join();
        Joined::new(NegatedIter(storage.iter), std::usize::MAX)
    }
}

pub struct Drain<'a, T>(&'a mut Storage<T>);

pub struct DrainIter<'a, T>(&'a mut Storage<T>, usize);

impl<'a, T> Join for DrainIter<'a, T> {
    fn may_skip(&mut self, curr: usize) -> usize {
        self.0.inner[curr..]
            .iter()
            .take_while(|opt| opt.is_none())
            .count()
    }
}

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

    fn join(self) -> Joined<Self::Joined> {
        let len = self.0.inner.len();
        Joined::new(DrainIter(self.0, 0), len)
    }
}

pub struct Iter<'a, T> {
    slice: &'a [Option<T>],
}

impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Iter { slice: self.slice }
    }
}

impl<'a, T> Join for Iter<'a, T> {
    fn may_skip(&mut self, _curr: usize) -> usize {
        self.slice.iter().take_while(|opt| opt.is_none()).count()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.nth(0)
    }

    fn nth(&mut self, n: usize) -> Option<&'a T> {
        if self.slice.len() > n {
            let (start, end) = self.slice.split_at(n + 1);
            self.slice = end;
            start.last().unwrap().as_ref()
        } else {
            self.slice = &[];
            None
        }
    }
}

impl<'a, T> Joinable for &'a Storage<T> {
    type Joined = Iter<'a, T>;
    type Item = &'a T;

    fn join(self) -> Joined<Self::Joined> {
        Joined::new(Iter { slice: &self.inner }, self.inner.len())
    }
}

impl<'a, T> Joinable for &'a &Storage<T> {
    type Joined = Iter<'a, T>;
    type Item = &'a T;

    fn join(self) -> Joined<Self::Joined> {
        <&Storage<T>>::join(self)
    }
}

impl<'a, T> Joinable for &'a &mut Storage<T> {
    type Joined = Iter<'a, T>;
    type Item = &'a T;

    fn join(self) -> Joined<Self::Joined> {
        <&Storage<T>>::join(self)
    }
}

pub struct IterMut<'a, T> {
    iter: slice::IterMut<'a, Option<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        self.iter.next().map(Option::as_mut).flatten()
    }

    fn nth(&mut self, n: usize) -> Option<&'a mut T> {
        self.iter.nth(n).map(Option::as_mut).flatten()
    }
}

impl<'a, T> Join for IterMut<'a, T> {
    fn may_skip(&mut self, _curr: usize) -> usize {
        let slice = mem::replace(&mut self.iter, [].iter_mut()).into_slice();
        let next = slice.iter().take_while(|opt| opt.is_none()).count();
        self.iter = slice.iter_mut();
        next
    }
}

impl<'a, T> Joinable for &'a mut Storage<T> {
    type Joined = IterMut<'a, T>;
    type Item = &'a mut T;

    fn join(self) -> Joined<Self::Joined> {
        let len = self.inner.len();
        Joined::new(
            IterMut {
                iter: self.inner.iter_mut(),
            },
            len,
        )
    }
}

impl<'a, T> Joinable for &'a mut &mut Storage<T> {
    type Joined = IterMut<'a, T>;
    type Item = &'a mut T;

    fn join(self) -> Joined<Self::Joined> {
        <&mut Storage<T>>::join(self)
    }
}

pub struct Entities;

impl Join for iter::Map<RangeFrom<usize>, fn(usize) -> Entity> {
    fn may_skip(&mut self, _curr: usize) -> usize {
        0
    }
}

impl Joinable for Entities {
    type Joined = iter::Map<RangeFrom<usize>, fn(usize) -> Entity>;
    type Item = Entity;

    fn join(self) -> Joined<Self::Joined> {
        Joined::new((0..).map(Entity as fn(usize) -> Entity), std::usize::MAX)
    }
}

pub struct Joined<T> {
    iter: T,
    len: usize,
    pos: usize,
}

impl<T: Iterator + Join> Joined<T> {
    pub fn new(iter: T, len: usize) -> Self {
        Self { iter, len, pos: 0 }
    }
}

impl<T> Clone for Joined<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            len: self.len,
            pos: self.pos,
        }
    }
}

impl<T: Join + Iterator> Iterator for Joined<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<T::Item> {
        while self.pos < self.len {
            let nth = self.iter.may_skip(self.pos);
            self.pos += nth;
            if let Some(item) = self.iter.nth(nth) {
                return Some(item);
            } else {
                self.pos += 1;
            }
        }

        None
    }
}

pub struct Maybe<T: Joinable>(T::Joined);

pub struct MaybeJoined<T: Joinable>(T::Joined);

impl<T: Joinable> Iterator for MaybeJoined<T> {
    type Item = Option<T::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.next())
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        Some(self.0.nth(n))
    }
}

impl<T: Joinable> Join for MaybeJoined<T> {
    fn may_skip(&mut self, _curr: usize) -> usize {
        0
    }
}

impl<T: Joinable> Joinable for Maybe<T> {
    type Joined = MaybeJoined<T>;
    type Item = Option<T::Item>;

    fn join(self) -> Joined<Self::Joined> {
        Joined::new(MaybeJoined(self.0), std::usize::MAX)
    }
}

pub trait Joinable: Sized {
    type Joined: Iterator<Item = Self::Item> + Join + Sized;
    type Item;

    fn join(self) -> Joined<Self::Joined>;

    fn maybe(self) -> Maybe<Self> {
        Maybe(self.join().iter)
    }
}

pub trait Join {
    fn may_skip(&mut self, curr: usize) -> usize;
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

    #[test]
    fn negate() {
        let a = Entity(0);
        let b = Entity(1);
        let c = Entity(2);

        let mut d: Storage<u32> = Storage::new();
        let mut e: Storage<u8> = Storage::new();

        d.insert(a, 7);
        d.insert(b, 12);
        e.insert(b, 17);
        e.insert(c, 0);

        for (&d_entry, e_entry, entity) in (&d, !&e, Entities).join() {
            assert_eq!(d_entry, 7);
            assert_eq!(e_entry, ());
            assert_eq!(entity, a);
        }
    }

    #[test]
    fn negate_len() {
        let b = Entity(1);

        let mut d: Storage<u32> = Storage::new();
        let e: Storage<u8> = Storage::new();

        d.insert(b, 12);

        for (&d_entry, e_entry, entity) in (&d, !&e, Entities).join() {
            assert_eq!(d_entry, 12);
            assert_eq!(e_entry, ());
            assert_eq!(entity, b);
        }
    }

    #[test]
    fn maybe() {
        let b = Entity(1);

        let mut d: Storage<u32> = Storage::new();
        let mut e: Storage<u8> = Storage::new();

        d.insert(b, 12);

        for (&d_entry, e_entry, entity) in (&d, (&e).maybe(), Entities).join() {
            assert_eq!(d_entry, 12);
            assert_eq!(e_entry, None);
            assert_eq!(entity, b);
        }

        e.insert(b, 32);

        for (&d_entry, e_entry, entity) in (&d, (&e).maybe(), Entities).join() {
            assert_eq!(d_entry, 12);
            assert_eq!(e_entry, Some(&32));
            assert_eq!(entity, b);
        }
    }

    #[test]
    fn entities_clone() {
        let _ = Entities.join().clone();
        let _ = (Entities, Entities).join().clone();
        let _ = (Entities, Entities, Entities).join().clone();
        let _ = (Entities, Entities, Entities, Entities).join().clone();
    }

    #[test]
    fn may_skip() {
        let mut s = Storage::new();
        s.insert(Entity(0), 17);
        s.insert(Entity(4), 3);
        s.insert(Entity(5), 4);

        let mut iter = (&s).join();
        assert_eq!(iter.next(), Some(&17));
        assert_eq!(iter.iter.may_skip(1), 3);
    }
}
