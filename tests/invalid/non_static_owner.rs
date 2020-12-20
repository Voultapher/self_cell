use once_self_cell::sync_once_self_cell;

struct Dependent<'a>(&'a String);

impl<'a> From<&&'a String> for Dependent<'a> {
    fn from(s: &&'a String) -> Self {
        Self(s)
    }
}

sync_once_self_cell!(BorrowFromStack, &'_ String, Dependent<'_>,);

fn main() {
    let c = BorrowFromStack::new(&String::from("abc"));
    let _ = c.get_or_init_dependent();
}
