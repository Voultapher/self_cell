use std::{cell::Cell, marker::PhantomData};

use self_cell::self_cell;

type NotCovariant<'a> = Cell<&'a str>;

struct Owner<'a>(String, PhantomData<&'a ()>);

self_cell!(
    struct NoCov<'a> {
        owner: Owner<'a>,

        #[not_covariant]
        dependent: NotCovariant,
    }
);

fn main() {
    let cell = NoCov::new(Owner("hi this is no good".into(), PhantomData), |owner| {
        Cell::new(&owner.0)
    });
    let leaked_ref = cell.with_dependent(|_, dependent| dependent);
    leaked_ref.set(&String::from("temporary"));

    cell.with_dependent(|_, dep| {
        println!("{}", dep.replace(""));
    });
}
