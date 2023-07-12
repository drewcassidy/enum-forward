struct A {}
struct B {}

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
    fn forward<D: FooForwarder<R>, R>(&self, fwd: D) -> R {
        return match self {
            Foo::A(a) => { fwd.build::<A>()(a) }
            Foo::B(b) => { fwd.build::<B>()(b) }
        };
    }
}

// everything from here on has no knowledge of the variants of Foo

struct GetNameFwd {}

impl FooForwarder<&'static str> for GetNameFwd {
    fn build<T: GetName>(&self) -> fn(&T) -> &'static str {
        |t| t.name()
    }
}

impl Foo {
    fn get_name(&self) -> &'static str {
        self.forward(GetNameFwd{})
    }
}

