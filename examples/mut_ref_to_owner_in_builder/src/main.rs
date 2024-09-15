use self_cell::{self_cell, MutBorrow};

type MutStringRef<'a> = &'a mut String;

self_cell!(
    struct MutStringCell {
        owner: MutBorrow<String>,

        #[covariant]
        dependent: MutStringRef,
    }
);

fn main() {
    let mut cell = MutStringCell::new(MutBorrow::new("abc".into()), |owner| owner.borrow_mut());

    cell.with_dependent_mut(|_owner, dependent| {
        println!("dependent before pop: {}", dependent);
        dependent.pop();
        println!("dependent after pop: {}", dependent);
    });

    let recovered_owner: String = cell.into_owner().into_inner();
    println!("recovered owner: {}", recovered_owner);
}
