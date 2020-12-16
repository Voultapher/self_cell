use once_self_cell::sync::OnceSelfCell;

fn main() {
    let mut s = Ok(".".to_owned());

    let c: OnceSelfCell<(), &String> = OnceSelfCell::new((), |_| s.as_ref().unwrap());
}
