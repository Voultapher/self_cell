use once_self_cell::unsync_once_self_cell;

unsync_once_self_cell!(NoFromCell, String, (),);

fn main() {
    let _ = NoFromCell::new("abc".into());
}
