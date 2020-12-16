use std::cell::Cell;

use once_self_cell::unsync::OnceSelfCell;

fn main() {
    static STRING: String = String::new();

    let mut owner = Ok(".".to_owned());
    let c: OnceSelfCell<_, Cell<&String>> = OnceSelfCell::new(&owner);
    c.get_or_init_dependent(|own| Cell::new(own.as_ref().unwrap()));
    let cell = c.get_or_init_dependent(|_| Cell::new(&STRING));
    let uh = cell.replace(&STRING);
    drop(c);
    owner = Err(());
    let _ = owner;
    println!("{}", uh);
}
