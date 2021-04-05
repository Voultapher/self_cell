use once_self_cell::unsync_once_self_cell;

struct CustomType(&'static str);

impl From<&String> for CustomType {
    fn from(x: &String) -> Self {
        Self(x.as_ref())
    }
}

unsync_once_self_cell!(derive(), InvalidLifetime, String, CustomType);

fn main() {
    let v: &'static str = {
        let c = InvalidLifetime::new(".".repeat(4000));
        *c.get_or_init_dependent().0
    };
    println!("{}", v);
}
