use std::cell::Cell;

use self_cell::self_cell;

struct NotCovariant<'a>(Cell<&'a String>);

impl<'a> From<&'a String> for NotCovariant<'a> {
    fn from(s: &'a String) -> Self {
        Self(Cell::new(s))
    }
}

self_cell!(
    struct NoCov {
        #[from]
        owner: String,

        #[not_covariant]
        dependent: NotCovariant,
    }
);

fn main() {
    let cell = NoCov::new("hi this is no good".into());
    let _leaked_ref = cell.with_dependent(|_, dependent| dependent);
}
