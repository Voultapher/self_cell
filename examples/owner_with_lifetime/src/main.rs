// This example demonstrates a use-case where a lifetime in the owner is required.

use std::borrow::Cow;

use self_cell::self_cell;

type Ast<'a> = Vec<&'a str>;

self_cell!(
    struct AstCell<'input> {
        owner: Cow<'input, str>,

        #[covariant]
        dependent: Ast,
    }
);

impl<'input> AstCell<'input> {
    // Escape input if necessary before constructing AST.
    fn from_un_escaped(input: &'input str) -> Self {
        println!("input: {:?}", input);

        let escaped_input = if input.contains("x") {
            Cow::from(input)
        } else {
            println!("escaping input (owned alloc)");
            let owned: String = input.replace('u', "z");
            Cow::from(owned)
        };

        // This would be impossible without a self-referential struct.
        // escaped_input could either be a pointer to the input or an owned
        // string on stack.
        // We only want to depend on the input lifetime and encapsulate the
        // string escaping logic in a function. Which we can't do with
        // vanilla Rust.
        // Non self-referential version https://godbolt.org/z/3Pcc9a5za error.
        Self::new(escaped_input, |escaped_input| {
            // Dummy for expensive computation you don't want to redo.
            escaped_input.split(' ').collect()
        })
    }
}

fn main() {
    let not_escaped = AstCell::from_un_escaped("au bx");

    println!(
        "not_escaped.borrow_dependent() -> {:?}",
        not_escaped.borrow_dependent()
    );

    let escaped = AstCell::from_un_escaped("au bz");

    println!(
        "escaped.borrow_dependent() -> {:?}",
        escaped.borrow_dependent()
    );
}
