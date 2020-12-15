use once_self_cell::sync::OnceSelfCell;

fn main() {
    let c: OnceSelfCell<(), &String> = OnceSelfCell::new(());
    let mut s = Ok(".".to_owned());
    c.get_or_init_dependent(|_| s.as_ref().unwrap());
    let uh: &&String = c.get_or_init_dependent(|_| panic!());
    let p = s.as_ref().unwrap().as_ptr();
    let _ = std::mem::replace(&mut s, Err([p; 3]));
    println!("{}", &uh[..100]);
}
