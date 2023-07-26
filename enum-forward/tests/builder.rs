use std::process::Output;
use enum_forward::{Forward, From, TryInto};

struct A {}
struct B {}

#[derive(Forward)]
enum Foo { A(A), B(B), }

trait GetName {
    fn name(&self) -> &'static str;
}

impl GetName for A {
    fn name(&self) -> &'static str { "A" }
}

impl GetName for B {
    fn name(&self) -> &'static str { "B" }
}

trait FooForwarder<R> {
    fn build<T: GetName>(&self) -> fn(&T) -> R;
}

impl Foo {
    fn forward<'a, D: FooForwarder<R>, R>(&self, fwd: D) -> R {
        return match self {
            Foo::A(a) => { fwd.build()(a) }
            Foo::B(b) => { fwd.build()(b) }
        };
    }
}

trait Visit<I> {
    type Output;
    fn visit(&self, input : I) -> Self::Output;
}

impl<I, R> Visit<I> for Foo where A : Visit<I, Output=R>, B : Visit<I, Output=R>{
    type Output = R;

    fn visit(&self, input: I) -> R {
        return match self {
            Foo::A(val) => {Visit::visit(val, input)}
            Foo::B(val) => {Visit::visit(val, input)}
        }
    }
}

// everything from here on has no knowledge of the variants of Foo

struct GetNameFwd {}

impl<T> Visit<GetNameFwd> for T where T : GetName {
    type Output = &'static str;
    fn visit(&self, input: GetNameFwd) -> &'static str {
        self.name()
    }
}

impl Foo {
    fn get_name(&self) -> &'static str {
        Self::visit(self, GetNameFwd{})
    }
}

