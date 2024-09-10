#![allow(unused_imports)]

use std::cell::RefCell;
use std::rc::Rc;
use std::str;

use crossbeam_utils::thread;

use impls::impls;

use self_cell::{self_cell, MutBorrow};

#[allow(dead_code)]
struct NotSend<'a> {
    ptr: *mut i32,
    derived: &'a String,
}

impl<'a> From<&'a String> for NotSend<'a> {
    fn from(s: &'a String) -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            derived: s,
        }
    }
}

self_cell!(
    struct NotSendCell {
        owner: String,

        #[covariant]
        dependent: NotSend,
    }
); // no impl

#[test]
fn not_send() {
    // If owner and or dependent are not Send then the created cell
    // should not be Send either.

    assert!(impls!(String: Send));
    assert!(!impls!(NotSend: Send));
    assert!(!impls!(NotSendCell: Send));
}

#[test]
fn not_sync() {
    // If owner and or dependent are not Sync then the created cell
    // should not be Sync either.

    assert!(impls!(String: Sync));
    assert!(!impls!(NotSend: Sync));
    assert!(!impls!(NotSendCell: Sync));
}

#[test]
fn mut_borrow_traits() {
    type MutBorrowString = MutBorrow<String>;
    assert!(impls!(MutBorrowString: Send));
    assert!(impls!(MutBorrowString: Sync));

    type MutBorrowRefCellString = MutBorrow<RefCell<String>>;
    assert!(impls!(MutBorrowRefCellString: Send));
    assert!(impls!(MutBorrowRefCellString: Sync));

    type MutBorrowRcString = MutBorrow<Rc<String>>;
    assert!(!impls!(MutBorrowRcString: Send));
    assert!(!impls!(MutBorrowRcString: Sync));

    type MutStringRef<'a> = &'a mut String;

    self_cell!(
        struct MutBorrowStringCell {
            owner: MutBorrow<String>,

            #[covariant]
            dependent: MutStringRef,
        }
    );

    assert!(impls!(MutBorrowStringCell: Send));
    assert!(impls!(MutBorrowStringCell: Sync));
}

#[test]
#[cfg(feature = "invalid_programs")]
// Not supported by miri isolation.
#[cfg_attr(miri, ignore)]
// Closure paths slashes show up as diff error on Windows.
#[cfg(not(target_os = "windows"))]
fn invalid_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("invalid/*.rs");
}

#[derive(Clone, Debug, PartialEq)]
struct Ast<'a>(Vec<&'a str>);

impl<'x> From<&'x String> for Ast<'x> {
    fn from<'a>(body: &'a String) -> Ast<'a> {
        Ast(vec![&body[0..1], &body[2..10]])
    }
}

self_cell!(
    struct AstCell {
        owner: String,

        #[covariant]
        dependent: Ast,
    }

    impl {Debug}
);

#[test]
fn share_across_threads() {
    // drop_joined takes &mut self, so that's not a thread concern anyway.
    // And get_or_init_dependent should be as thread compatible as OnceCell.
    // Owner never gets changed after init.

    let body = String::from("hy hyperspeed");

    // expected_ast is on the stack and lifetime dependent on body.
    let expected_ast = Ast::from(&body);

    // But PackedAst is struct and can be freely moved and copied.
    let packed_ast = AstCell::new(body.clone(), |o| o.into());

    thread::scope(|s| {
        s.spawn(|_| {
            assert_eq!(packed_ast.borrow_owner(), &body);
            assert_eq!(packed_ast.borrow_dependent(), &expected_ast);
        });

        s.spawn(|_| {
            assert_eq!(packed_ast.borrow_owner(), &body);
            assert_eq!(packed_ast.borrow_dependent(), &expected_ast);
        });

        assert_eq!(packed_ast.borrow_owner(), &body);
        assert_eq!(packed_ast.borrow_dependent(), &expected_ast);
    })
    .unwrap();
}
