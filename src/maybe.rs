use crate::{Join, Joinable, Joined};

/// The iterator returned by calling `T::maybe()` on a `T` which implements `Joinable`.
pub struct Maybe<T>(T);

impl<T> Maybe<T> {
    pub(crate) fn new(inner: T) -> Self {
        Maybe(inner)
    }
}

impl<T: Iterator> Iterator for Maybe<T> {
    type Item = Option<T::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.next())
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        Some(self.0.nth(n))
    }
}

impl<T> Join for Maybe<T> {
    fn may_skip(&mut self, _curr: usize) -> usize {
        0
    }
}

impl<T: Iterator> Joinable for Maybe<T> {
    type Joined = Maybe<T>;
    type Item = Option<T::Item>;

    fn join(self) -> Joined<Self::Joined> {
        Joined::new(self, std::usize::MAX)
    }
}
