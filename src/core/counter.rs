use std::cell::Cell;

pub trait Count {
    type Item;
    fn next(&self) -> Self::Item;
}

#[derive(Default)]
pub struct Counter<A>(Cell<A>);
impl<A> Counter<A> {
    pub fn new(start: A) -> Self {
        Self(Cell::new(start))
    }
}

impl<A: Copy + num::Num> Count for Counter<A> {
    type Item = A;

    fn next(&self) -> A {
        let curr = self.0.get();
        self.0.set(curr + A::one());
        curr
    }
}
