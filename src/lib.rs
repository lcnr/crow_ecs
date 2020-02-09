#![forbid(unsafe_code)]
//! A simple ecs utility crate using no unsafe code.

use std::{
    collections::{btree_map, BTreeMap},
    iter::{self, Peekable},
    mem,
    ops::RangeFrom,
    slice,
};

mod tuple;

pub mod drain;
pub mod maybe;
pub mod not;

use maybe::Maybe;

/// This crate does currently not protect from generation missmatches.
///
/// To delete an entity one has to remove it from all storages, which is probably not the best approach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity(pub usize);

/// The default storage, use this if the component is fairly well used.
///
/// In case there exists only a few entities with the given component,
/// consider using a [`SparseStorage`] instead.
///
/// [`SparseStorage`]: struct.SparseStorage.html
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

/// A storage which can be used if the given component is not associated with
/// many entities.
#[derive(Debug, Clone)]
pub struct SparseStorage<T> {
    inner: BTreeMap<usize, T>,
}

impl<T> Default for SparseStorage<T> {
    fn default() -> Self {
        SparseStorage::new()
    }
}

impl<T> SparseStorage<T> {
    /// Creates a new `SparseStorage`.
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    /// Removes all components in this storage.
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Returns the component of the entity at `idx` in case it exists.
    pub fn get(&self, idx: Entity) -> Option<&T> {
        self.inner.get(&idx.0)
    }

    pub fn get_mut(&mut self, idx: Entity) -> Option<&mut T> {
        self.inner.get_mut(&idx.0)
    }

    /// Inserts a component for the entity at `idx`.
    ///
    /// In case the component was already present the previous
    /// one is returned.
    pub fn insert(&mut self, idx: Entity, c: T) -> Option<T> {
        self.inner.insert(idx.0, c)
    }

    /// Removes this component for the entity at `idx`.
    pub fn remove(&mut self, idx: Entity) -> Option<T> {
        self.inner.remove(&idx.0)
    }
}

pub struct SparseIter<'a, T> {
    inner: &'a BTreeMap<usize, T>,
    position: usize,
}

impl<'a, T> Clone for SparseIter<'a, T> {
    fn clone(&self) -> Self {
        SparseIter {
            inner: self.inner,
            position: self.position,
        }
    }
}

impl<'a, T> Join for SparseIter<'a, T> {
    fn may_skip(&mut self, curr: usize) -> usize {
        self.position = curr;
        self.inner
            .range(self.position..)
            .next()
            .map_or(std::usize::MAX, |(&k, _)| k - self.position)
    }
}

impl<'a, T> Iterator for SparseIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let item = self.inner.get(&self.position);
        self.position += 1;
        item
    }

    fn nth(&mut self, n: usize) -> Option<&'a T> {
        self.position += n;
        self.next()
    }
}

impl<'a, T> Joinable for &'a SparseStorage<T> {
    type Joined = SparseIter<'a, T>;
    type Item = &'a T;

    fn join(self) -> Joined<Self::Joined> {
        Joined::new(
            SparseIter {
                inner: &self.inner,
                position: 0,
            },
            self.inner.keys().last().copied().unwrap_or(0),
        )
    }
}

pub struct SparseIterMut<'a, T> {
    inner: Peekable<btree_map::IterMut<'a, usize, T>>,
    position: usize,
}

impl<'a, T> Join for SparseIterMut<'a, T> {
    fn may_skip(&mut self, curr: usize) -> usize {
        self.position = curr;
        self.inner
            .peek()
            .map_or(std::usize::MAX, |&(&k, _)| k - curr)
    }
}

impl<'a, T> Iterator for SparseIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        let position = self.position;
        while self.inner.peek().map_or(false, |&(&k, _)| k < position) {
            self.inner.next();
        }

        let item = if self.inner.peek().map_or(false, |&(&k, _)| k == position) {
            self.inner.next().map(|(_, v)| v)
        } else {
            None
        };
        self.position += 1;
        item
    }

    fn nth(&mut self, n: usize) -> Option<&'a mut T> {
        self.position += n;
        self.next()
    }
}

impl<'a, T> Joinable for &'a mut SparseStorage<T> {
    type Joined = SparseIterMut<'a, T>;
    type Item = &'a mut T;

    fn join(self) -> Joined<Self::Joined> {
        let len = self.inner.keys().last().copied().unwrap_or(0);
        Joined::new(
            SparseIterMut {
                inner: self.inner.iter_mut().peekable(),
                position: 0,
            },
            len,
        )
    }
}

/// A joinable struct returning the currently iterated `Entity`.
///
/// The returned iterator is unbounded.
pub struct Entities;

/// The iterator created by [`Entities::join`].
///
/// [`Entities::join`]: struct.Entities.html
#[derive(Debug, Clone)]
pub struct EntitiesIter(iter::Map<RangeFrom<usize>, fn(usize) -> Entity>);

impl Iterator for EntitiesIter {
    type Item = Entity;

    fn next(&mut self) -> Option<Entity> {
        self.0.next()
    }

    fn nth(&mut self, n: usize) -> Option<Entity> {
        self.0.nth(n)
    }
}

impl Join for EntitiesIter {
    fn may_skip(&mut self, _curr: usize) -> usize {
        0
    }
}

impl Joinable for Entities {
    type Joined = EntitiesIter;
    type Item = Entity;

    fn join(self) -> Joined<Self::Joined> {
        Joined::new(
            EntitiesIter((0..).map(Entity as fn(usize) -> Entity)),
            std::usize::MAX,
        )
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

pub trait Join {
    fn may_skip(&mut self, curr: usize) -> usize;
}

/// Join multiple storages for easy iteration.
///
/// # Examples
///
/// ```rust
/// use crow_ecs::{Entity, Storage, Joinable, Entities};
///
/// let a = Entity(0);
/// let b = Entity(1);
///
/// let mut names = Storage::new();
/// names.insert(a, "Foo");
/// names.insert(b, "Bar");
///
/// let mut health = Storage::new();
/// health.insert(a, 17);
/// health.insert(b, 3);
///
/// // take 1 HP of each named player
/// for (name, health, id) in (&names, &mut health, Entities).join() {
///     *health -= 1;
///     println!("The player `{}` with ID {} now has {} health", name, id.0, health);
/// }
/// ```
pub trait Joinable: Sized {
    type Joined: Iterator<Item = Self::Item> + Join + Sized;
    type Item;

    fn join(self) -> Joined<Self::Joined>;

    fn maybe(self) -> Maybe<Self::Joined> {
        Maybe::new(self.join().iter)
    }
}

impl<'a, T> Joinable for &'a &T
where
    &'a T: Joinable,
{
    type Joined = <&'a T as Joinable>::Joined;
    type Item = <&'a T as Joinable>::Item;

    fn join(self) -> Joined<Self::Joined> {
        <&T>::join(self)
    }
}

impl<'a, T> Joinable for &'a &mut T
where
    &'a T: Joinable,
{
    type Joined = <&'a T as Joinable>::Joined;
    type Item = <&'a T as Joinable>::Item;

    fn join(self) -> Joined<Self::Joined> {
        <&T>::join(self)
    }
}

impl<'a, T> Joinable for &'a mut &mut T
where
    &'a mut T: Joinable,
{
    type Joined = <&'a mut T as Joinable>::Joined;
    type Item = <&'a mut T as Joinable>::Item;

    fn join(self) -> Joined<Self::Joined> {
        <&'a mut T>::join(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_join() {
        let a = Entity(0);
        let b = Entity(1);
        let c = Entity(4);

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
    fn simple_join_sparse() {
        let a = Entity(0);
        let b = Entity(1);
        let c = Entity(4);

        let mut d = SparseStorage::<u32>::new();
        let mut e = SparseStorage::<u8>::new();

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
        let c = Entity(4);

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
    fn entities_sparse() {
        let a = Entity(0);
        let b = Entity(1);
        let c = Entity(4);

        let mut d = SparseStorage::<u32>::new();
        let mut e = SparseStorage::<u8>::new();

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
        let c = Entity(4);

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
