use std::ops::Not;

use crate::{Iter, Join, Joinable, Joined, SparseIter, SparseStorage, Storage};

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

pub struct NegatedSparseStorage<'a, T>(&'a SparseStorage<T>);

impl<'a, T> Not for &'a SparseStorage<T> {
    type Output = NegatedSparseStorage<'a, T>;

    fn not(self) -> NegatedSparseStorage<'a, T> {
        NegatedSparseStorage(self)
    }
}

impl<'a, T> Not for &'a &SparseStorage<T> {
    type Output = NegatedSparseStorage<'a, T>;

    fn not(self) -> NegatedSparseStorage<'a, T> {
        NegatedSparseStorage(self)
    }
}

impl<'a, T> Not for &'a &mut SparseStorage<T> {
    type Output = NegatedSparseStorage<'a, T>;

    fn not(self) -> NegatedSparseStorage<'a, T> {
        NegatedSparseStorage(self)
    }
}

pub struct NegatedSparseIter<'a, T>(SparseIter<'a, T>);

// TODO: Might be more performant to actually check if the next entity exists,
// TODO: but tbh I don't think this is not worth the effort
impl<'a, T> Join for NegatedSparseIter<'a, T> {
    fn may_skip(&mut self, _curr: usize) -> usize {
        0
    }
}

impl<'a, T> Clone for NegatedSparseIter<'a, T> {
    fn clone(&self) -> Self {
        NegatedSparseIter(self.0.clone())
    }
}

impl<'a, T> Iterator for NegatedSparseIter<'a, T> {
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

impl<'a, T> Joinable for NegatedSparseStorage<'a, T> {
    type Joined = NegatedSparseIter<'a, T>;
    type Item = ();

    fn join(self) -> Joined<Self::Joined> {
        let storage = self.0.join();
        Joined::new(NegatedSparseIter(storage.iter), std::usize::MAX)
    }
}
