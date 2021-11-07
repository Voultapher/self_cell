use std::cell::Cell;

use self_cell::self_cell;

type NotCovariant<'a> = Cell<&'a String>;

self_cell!(
    struct NoCov {
        owner: String,

        #[covariant]
        dependent: NotCovariant,
    }
);

fn main() {
    let _cell = NoCov::new("hi this is no good".into(), |owner| Cell::new(owner));
}
