use std::cell::Cell;

use once_self_cell::self_cell;

struct NotCovariant<'a>(Cell<&'a String>);

impl<'a> From<&'a String> for NotCovariant<'a> {
    fn from(s: &'a String) -> Self {
        Self(Cell::new(s))
    }
}

self_cell!(NoCov, {}, from, String, NotCovariant, not_covariant);

fn main() {
    let cell = NoCov::new("hi this is no good".into());
    let _leaked_ref = cell.with_dependent(|_, dependent| dependent);
}
