use crate::{Joinable, Joined};

#[derive(Debug, Clone)]
pub struct TupleJoin<T>(T);

macro_rules! tuple_join {
    ($($par:ident $var:ident: $e:tt),*) => {
        impl<$($par: Iterator),*> Iterator for TupleJoin<($($par),*)>
        {
            type Item = ($($par::Item),*);

            fn next(&mut self) -> Option<Self::Item> {
                $(let $var = (self.0).$e.next();)*

                match ($($var),*) {
                    ($(Some($var)),*) => Some(($($var),*)),
                    _ => None,
                }
            }
        }

        impl<$($par: Joinable),*> Joinable for ($($par),*)
        {
            type Joined = TupleJoin<($($par::Joined),*)>;
            type Item = ($($par::Item),*);

            fn join(self) -> Joined<Self> {
                $(let $var = self.$e.join();)*

                Joined::new(TupleJoin(($($var.iter),*)), std::usize::MAX.$(min($var.len)).*)
            }
        }
    }
}

tuple_join!(A a: 0, B b: 1);
tuple_join!(A a: 0, B b: 1, C c: 2);
tuple_join!(A a: 0, B b: 1, C c: 2, D d: 3);
tuple_join!(A a: 0, B b: 1, C c: 2, D d: 3, E e: 4);
tuple_join!(A a: 0, B b: 1, C c: 2, D d: 3, E e: 4, F f: 5);