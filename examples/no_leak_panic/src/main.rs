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

self_cell!(
    struct NoLeakCell {
        owner: Owner,

        #[covariant]
        dependent: PanicCtor,
    }

    impl {Debug}
);

fn main() {
    let owner = Owner("This string is no trout".into());

    let err = NoLeakCell::try_new(owner, |owner| {
        std::panic::catch_unwind(|| PanicCtor::new(owner))
    })
    .unwrap_err()
    .downcast_ref::<&str>()
    .unwrap()
    .to_string();

    println!("PanicCtor::new panic -> {}", err);
    println!("But we keep going");
}
