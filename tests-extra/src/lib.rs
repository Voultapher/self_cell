#![allow(dead_code)]
#![allow(unused_imports)]

use std::fs;
use std::process::Command;
use std::str;

use crossbeam_utils::thread;

use impls::impls;

use self_cell::self_cell;

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
// Not supported by miri isolation.
#[cfg_attr(miri, ignore)]
// Closure paths slashes show up as diff error on Windows.
#[cfg(not(target_os = "windows"))]
fn invalid_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("invalid/*.rs");
}

// Hacky custom version of try_build to support partial diffing.
fn try_build_manual(path: &str) {
    let output = Command::new("cargo")
        .arg("check")
        .arg("--color=never")
        .current_dir(path)
        .output()
        .unwrap();

    let compile_err = str::from_utf8(&output.stderr).unwrap();

    let expected_err = fs::read_to_string(format!("{}/expected.stderr", path))
        .unwrap()
        .replace("IGNORE", "");

    // Very naive approach.
    for expected_line in expected_err.split("\n").map(str::trim) {
        if !compile_err.contains(expected_line) {
            eprintln!("Expected: '{}'\n\nIN\n\n{}", expected_line, compile_err);
            panic!();
        }
    }
}

#[test]
// Not supported by miri isolation.
#[cfg_attr(miri, ignore)]
fn invalid_compile_manual() {
    try_build_manual("invalid_manual/wrong_covariance");
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
