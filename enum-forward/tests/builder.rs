use enum_forward::{forwarding, From, TryInto};

struct A {}
struct B {}

#[forwarding(GetName)]
#[derive(TryInto, From)]
enum Foo { A, B, }

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

trait Visit<I, R> {
    fn visit(&self, input : I) -> R;
}

impl<I, R> Visit<I,R> for Foo where A : Visit<I,R>, B : Visit<I,R>{
    fn visit(&self, input: I) -> R {
        return match self {
            Foo::A(val) => {Visit::visit(val, input)}
            Foo::B(val) => {Visit::visit(val, input)}
        }
    }
}

// everything from here on has no knowledge of the variants of Foo

struct GetNameFwd {}

impl<T> Visit<GetNameFwd, &'static str> for T where T : GetName {
    fn visit(&self, input: GetNameFwd) -> &'static str {
        self.name()
    }
}

impl Foo {
    fn get_name(&self) -> &'static str {
        <Self as Visit<GetNameFwd, &'static str>>::visit(self, GetNameFwd{})
    }
}

