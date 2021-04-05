use std::cell::Cell;

use self_cell::self_cell;

struct NotCovariant<'a>(Cell<&'a String>);

impl<'a> From<&'a String> for NotCovariant<'a> {
    fn from(s: &'a String) -> Self {
        Self(Cell::new(s))
    }
}

self_cell!(NoCov, {}, from, String, NotCovariant, covariant);

fn main() {
    let _cell = NoCov::new("hi this is no good".into());
}
