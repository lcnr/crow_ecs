use std::cmp;

use crate::{Joinable, Joined};

pub struct TupleJoin2<A: Joinable, B: Joinable>(A::Joined, B::Joined);

impl<A, B> Iterator for TupleJoin2<A, B>
where
    A: Joinable,
    B: Joinable,
{
    type Item = (A::Item, B::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.0.next();
        let b = self.1.next();

        match (a, b) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }
}

impl<A, B> Joinable for (A, B)
where
    A: Joinable,
    B: Joinable,
{
    type Joined = TupleJoin2<A, B>;
    type Item = (A::Item, B::Item);

    fn join(self) -> Joined<Self> {
        let a = self.0.join();
        let b = self.1.join();

        Joined::new(TupleJoin2(a.iter, b.iter), cmp::min(a.len, b.len))
    }
}

pub struct TupleJoin3<A: Joinable, B: Joinable, C: Joinable>(A::Joined, B::Joined, C::Joined);

impl<A, B, C> Iterator for TupleJoin3<A, B, C>
where
    A: Joinable,
    B: Joinable,
    C: Joinable,
{
    type Item = (A::Item, B::Item, C::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.0.next();
        let b = self.1.next();
        let c = self.2.next();

        match (a, b, c) {
            (Some(a), Some(b), Some(c)) => Some((a, b, c)),
            _ => None,
        }
    }
}

impl<A, B, C> Joinable for (A, B, C)
where
    A: Joinable,
    B: Joinable,
    C: Joinable,
{
    type Joined = TupleJoin3<A, B, C>;
    type Item = (A::Item, B::Item, C::Item);

    fn join(self) -> Joined<Self> {
        let a = self.0.join();
        let b = self.1.join();
        let c = self.2.join();

        Joined::new(
            TupleJoin3(a.iter, b.iter, c.iter),
            a.len.min(b.len).min(c.len),
        )
    }
}

pub struct TupleJoin4<A: Joinable, B: Joinable, C: Joinable, D: Joinable>(
    A::Joined,
    B::Joined,
    C::Joined,
    D::Joined,
);

impl<A, B, C, D> Iterator for TupleJoin4<A, B, C, D>
where
    A: Joinable,
    B: Joinable,
    C: Joinable,
    D: Joinable,
{
    type Item = (A::Item, B::Item, C::Item, D::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.0.next();
        let b = self.1.next();
        let c = self.2.next();
        let d = self.3.next();

        match (a, b, c, d) {
            (Some(a), Some(b), Some(c), Some(d)) => Some((a, b, c, d)),
            _ => None,
        }
    }
}

impl<A, B, C, D> Joinable for (A, B, C, D)
where
    A: Joinable,
    B: Joinable,
    C: Joinable,
    D: Joinable,
{
    type Joined = TupleJoin4<A, B, C, D>;
    type Item = (A::Item, B::Item, C::Item, D::Item);

    fn join(self) -> Joined<Self> {
        let a = self.0.join();
        let b = self.1.join();
        let c = self.2.join();
        let d = self.3.join();

        Joined::new(
            TupleJoin4(a.iter, b.iter, c.iter, d.iter),
            a.len.min(b.len).min(c.len).min(d.len),
        )
    }
}
