use once_self_cell::unsync_once_self_cell;

struct CustomType<'a>(&'a i32);

impl<'a> From<&'a i32> for CustomType<'a> {
    fn from(x: &'a i32) -> Self {
        Self(x)
    }
}

unsync_once_self_cell!(WrongFromCell, String, CustomType<'_>,);

fn main() {
    let _ = WrongFromCell::new("abc".into());
}
