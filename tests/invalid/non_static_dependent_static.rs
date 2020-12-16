use once_self_cell::sync::OnceSelfCell;

struct Ast<'a>(&'a str);

fn non_static_dependent_static<'a>() {
    let _: OnceSelfCell<String, Ast<'a>> = OnceSelfCell::new("33".into(), |_| panic!());
}

fn main() {
    non_static_dependent_static();
}
