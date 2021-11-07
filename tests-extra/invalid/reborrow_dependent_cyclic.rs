use std::cell::RefCell;

use self_cell::self_cell;

struct Bar<'a>(RefCell<(Option<&'a Bar<'a>>, String)>);

self_cell! {
    struct Foo {
        owner: (),

        #[not_covariant]
        dependent: Bar,
    }
}

fn main() {
    let mut x = Foo::new((), |_| Bar(RefCell::new((None, "Hello".to_owned()))));

    x.with_dependent(|_, dep| {
        dep.0.borrow_mut().0 = Some(dep);
    });

    x.with_dependent_mut(|_, dep| {
        let r1 = dep.0.get_mut();
        let string_ref_1 = &mut r1.1;
        let mut r2 = r1.0.unwrap().0.borrow_mut();
        let string_ref_2 = &mut r2.1;

        let s = &string_ref_1[..];
        string_ref_2.clear();
        string_ref_2.shrink_to_fit();
        println!("{}", s); // prints garbage
    });
}
