use std::cell::RefCell;
use std::fmt;

use self_cell::self_cell;

self_cell! {
    struct WrongVarianceExample {
        owner: (),

        #[covariant]
        dependent: Dependent,
    }
}

// this type is not covariant
type Dependent<'a> = RefCell<Box<dyn fmt::Display + 'a>>;

fn main() {
    let cell = WrongVarianceExample::new((), |_| RefCell::new(Box::new("")));
    let s = String::from("Hello World");

    // borrow_dependent unsound due to incorrectly checked variance
    *cell.borrow_dependent().borrow_mut() = Box::new(s.as_str());

    // s still exists
    cell.with_dependent(|_, d| println!("{}", d.borrow()));

    drop(s);

    // s is gone
    cell.with_dependent(|_, d| println!("{}", d.borrow()));
}
