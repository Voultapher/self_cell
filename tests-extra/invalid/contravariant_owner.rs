use self_cell::self_cell;

type X<'a> = &'a str;

self_cell! {
    pub struct Foo<'a> {
        owner: fn(&'a ()),

        #[covariant]
        dependent: X,
    }
}

fn transmute_lifetime<'a, 'b>(x: &'a str) -> &'b str {
    fn helper<'x>(s: &'x str) -> impl for<'z> FnOnce(&'z fn(&'x ())) -> &'z str {
        move |_| s
    }
    let x: Foo<'a> = Foo::new(|_| (), helper(x));
    let x: Foo<'static> = x; // coerce using variance
    let y = Box::leak(Box::new(x));
    y.borrow_dependent()
}

fn main() {
    let r;
    {
        let s = "Hello World".to_owned();
        r = transmute_lifetime(s.as_str());
        dbg!(r); // "Hello World"
    }
    dbg!(r); // prints garbage :-)
}
