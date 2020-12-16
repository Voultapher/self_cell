use once_self_cell::sync::OnceSelfCell;

fn borrow_from_stack<'a>(_: &()) -> &'a String {
    let stack_string = String::from("hello stack");
    &stack_string
}

fn main() {
    let c: OnceSelfCell<(), &String> = OnceSelfCell::new((), borrow_from_stack);
    let _ = c.get_or_init_dependent::<String>();
}
