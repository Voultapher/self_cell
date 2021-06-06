use std::cell::Cell;

use self_cell::self_cell;

type NotCovariant<'a> = Cell<&'a String>;

self_cell!(
    struct NoCov {
        owner: String,

        #[not_covariant]
        dependent: NotCovariant,
    }
);

fn main() {
    let cell = NoCov::new("hi this is no good".into(), |owner| Cell::new(owner));
    let _leaked_ref = cell.with_dependent(|_, dependent| dependent);
}
