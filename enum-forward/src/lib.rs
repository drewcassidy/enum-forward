use std::marker::PhantomData;
pub use enum_forward_macros::*;

pub trait Forward<I> {
    type Output;
    fn forward(&self, input : &I) -> Self::Output;
}

struct EnumIterator<'a, I : Clone> {
    counter : usize,
    input : &'a I,
}

trait ForwardIter<I : Clone> {
    type Output;
    fn forward_iter(input: &I) -> EnumIterator<I>;
}