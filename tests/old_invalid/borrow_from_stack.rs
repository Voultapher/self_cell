use once_self_cell::sync_once_self_cell;

struct Owner(());
struct Dependent<'a>(&'a String);

impl<'a> From<&Owner> for Dependent<'a> {
    fn from(_: &Owner) -> Self {
        let stack_string = String::from("hello stack");
        Self(&stack_string)
    }
}

sync_once_self_cell!(BorrowFromStack, Owner, Dependent<'_>,);

fn main() {
    let c = BorrowFromStack::new(Owner(()));
    let _ = c.get_or_init_dependent();
}
