use std::cell::Cell;

use once_self_cell::unsync::OnceSelfCell;

fn main() {
    let owner = String::from("bleib");
    let c: OnceSelfCell<&String, Cell<&String>> = OnceSelfCell::new(&owner, |_| panic!());
}
