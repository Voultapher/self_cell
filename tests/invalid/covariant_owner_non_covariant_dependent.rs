use std::{cell::Cell, marker::PhantomData};

use self_cell::self_cell;

self_cell! {
    struct Foo<'a> {
        owner: PhantomData<&'a ()>,
        #[not_covariant]
        dependent: Dependent,
    }
}

type Dependent<'q> = Cell<&'q str>;

fn main() {
    let foo = Foo::new(PhantomData, |_| Cell::new(""));
    let s: String = "Hello World".into();
    foo.with_dependent(|_, d| {
        d.set(&s);
    });
    drop(s);
    foo.with_dependent(|_, d| println!("{}", d.get()));
}
