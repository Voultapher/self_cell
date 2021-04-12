use std::convert::TryInto;

use self_cell::self_cell;

#[derive(Debug, Clone, PartialEq)]
struct Owner(String);

#[derive(Debug)]
struct PanicCtor<'a>(&'a i32);

impl<'a> PanicCtor<'a> {
    fn new(_: &'a Owner) -> Self {
        let _stack_vec = vec![23, 44, 5];
        panic!("Oh noes, this is impossible")
    }
}

impl<'a> TryInto<PanicCtor<'a>> for &'a Owner {
    type Error = Box<dyn std::any::Any + Send + 'static>;

    fn try_into(self) -> Result<PanicCtor<'a>, Self::Error> {
        std::panic::catch_unwind(|| PanicCtor::new(&self))
    }
}

self_cell!(NoLeakCell, { Debug }, try_from, Owner, PanicCtor, covariant);

fn main() {
    let owner = Owner("This string is no trout".into());

    dbg!(NoLeakCell::try_from(owner)
        .unwrap_err()
        .downcast_ref::<&str>());
    dbg!("But we keep going");
}
