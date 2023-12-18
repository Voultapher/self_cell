#![no_std]

use self_cell::self_cell;

// Not using alloc is on purpose, self_cell should also work in such scenarios.

const SCRATCH_REGION: [u8; 4096] = [0u8; 4096];

#[derive(Eq, PartialEq)]
struct StaticString {
    region: &'static [u8],
}

const MAX_NODES: usize = 8;

#[derive(Eq, PartialEq)]
struct Ast<'a>([Option<&'a [u8]>; MAX_NODES]);

self_cell!(
    struct AstCell {
        owner: StaticString,

        #[covariant]
        dependent: Ast,
    }

    impl {Eq, PartialEq}
);

fn build_ast_cell() -> AstCell {
    // Yes in a static setting self_cell is not terribly useful, this could be solved differently.
    // This is only a test.
    let pre_processed_code = StaticString {
        region: &SCRATCH_REGION[4000..4015],
    };

    AstCell::new(pre_processed_code, |code| {
        let mut ast_nodes = [None; MAX_NODES];
        ast_nodes[0] = Some(&code.region[3..7]);
        ast_nodes[1] = Some(&code.region[10..12]);

        Ast(ast_nodes)
    })
}

#[test]
fn self_cell_works_in_no_std_env() {
    let ast_cell = build_ast_cell();
    assert_eq!(ast_cell.borrow_owner().region.len(), 15);
    assert_eq!(
        ast_cell
            .borrow_dependent()
            .0
            .iter()
            .filter(|val| val.is_some())
            .count(),
        2
    );
}
