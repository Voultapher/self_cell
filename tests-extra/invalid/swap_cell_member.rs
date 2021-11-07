use self_cell::self_cell;

type Dep1<'a> = (&'a str, &'static str);

self_cell! {
    pub struct Struct1 {
        owner: String,
        #[covariant]
        dependent: Dep1,
    }
}

type Dep2<'a> = (&'static str, &'a str);

self_cell! {
    pub struct Struct2 {
        owner: String,
        #[covariant]
        dependent: Dep2,
    }
}

fn main() {
    let hello: &'static str;
    {
        let mut x1 = Struct1::new(String::from("Hello World"), |s| (s, ""));
        let mut x2 = Struct2::new(String::new(), |_| ("", ""));
        std::mem::swap(&mut x1.unsafe_self_cell, &mut x2.unsafe_self_cell);
        hello = x2.borrow_dependent().0;

        dbg!(hello); // "Hello World"
                     // hello is now a static reference in to the "Hello World" string
    }
    // the String is dropped at the end of the block above

    dbg!(hello); // prints garbage, use-after-free
}
