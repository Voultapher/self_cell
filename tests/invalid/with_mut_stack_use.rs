use self_cell::self_cell;

type Dependent<'a> = &'a String;

self_cell!(
    struct MutStackUse {
        #[from_fn]
        owner: String,

        #[covariant]
        dependent: Dependent,
    }
);

fn main() {
    let outside_string = String::from("outside string");

    let mut cell = MutStackUse::from_fn("Crackle that thunder".into(), |owner| owner);

    cell.with_dependent_mut(|_, dependent| {
        *dependent = &outside_string;
    });
}
