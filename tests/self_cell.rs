// The unsafe being used gets tested with miri in the CI.

use core::fmt::Debug;

use impls::impls;

use once_self_cell::self_cell;

#[derive(Debug, Eq, PartialEq)]
struct Ast<'input>(pub Vec<&'input str>);

impl<'a> From<&'a String> for Ast<'a> {
    fn from(body: &'a String) -> Self {
        Ast(vec![&body[2..5], &body[1..3]])
    }
}

self_cell!(
    PackedAstCell,
    {Clone, Debug, PartialEq, Eq, Hash},
    String,
    Ast,
    covariant,
    doc(hidden)
);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct PackedAst {
    ast_cell: PackedAstCell,
}

impl PackedAst {
    fn new(body: String) -> Self {
        Self {
            ast_cell: PackedAstCell::new(body),
        }
    }

    fn get_body<'a>(&'a self) -> &'a String {
        self.ast_cell.borrow_owner()
    }

    fn with_ast(&self, func: impl for<'a> FnOnce(&'a Ast<'a>)) {
        self.ast_cell.with_dependent(func)
    }

    fn get_ast<'a>(&'a self) -> &'a Ast<'a> {
        self.ast_cell.borrow_dependent()
    }
}

fn assert_with_ast(packed_ast: &PackedAst, expected_ast: &Ast) {
    let mut visited = false;
    packed_ast.with_ast(|ast| {
        assert_eq!(ast, expected_ast);
        visited = true;
    });
    assert!(visited);
}

#[test]
fn parse_ast() {
    let body = String::from("some longer string that ends now");

    // expected_ast is on the stack and lifetime dependent on body.
    let expected_ast = Ast::from(&body);

    // But PackedAst is struct and can be freely moved and copied.
    let packed_ast = PackedAst::new(body.clone());
    assert_eq!(packed_ast.get_body(), &body);
    assert_with_ast(&packed_ast, &expected_ast);
    assert_eq!(packed_ast.get_ast(), &expected_ast);

    assert_eq!(
        format!("{:?}", &packed_ast),
        "PackedAst { ast_cell: PackedAstCell { owner: \"some longer string that ends now\", dependent: Ast([\"me \", \"om\"]) } }"
    );

    let cloned_packed_ast = packed_ast.clone();
    assert_eq!(cloned_packed_ast.get_body(), &body);
    assert_with_ast(&cloned_packed_ast, &expected_ast);
    assert_eq!(cloned_packed_ast.get_ast(), &expected_ast);

    let moved_packed_ast = packed_ast;
    assert_eq!(moved_packed_ast.get_body(), &body);
    assert_with_ast(&moved_packed_ast, &expected_ast);
    assert_eq!(moved_packed_ast.get_ast(), &expected_ast);

    // Assert that even though the original packed_ast was moved, the clone of it is still valid.
    assert_eq!(cloned_packed_ast.get_body(), &body);
    assert_with_ast(&cloned_packed_ast, &expected_ast);
    assert_eq!(cloned_packed_ast.get_ast(), &expected_ast);

    // Assert that even though the original packed_ast was dropped, the clone of it is still valid.
    drop(moved_packed_ast);
    assert_eq!(cloned_packed_ast.get_body(), &body);
    assert_with_ast(&cloned_packed_ast, &expected_ast);
    assert_eq!(cloned_packed_ast.get_ast(), &expected_ast);
}

fn make_ast_with_stripped_body(body: &str) -> PackedAst {
    // This is created on the stack.
    let stripped_body = body.replace("\n", "");
    // Return Ast built from moved body, no lifetime hassle.
    PackedAst::new(stripped_body)
}

#[test]
fn return_self_ref_struct() {
    let body = String::from("a\nb\nc\ndef");
    let expected_body = body.replace("\n", "");

    // expected_ast is on the stack and lifetime dependent on body.
    let expected_ast = Ast::from(&expected_body);

    let packed_ast = make_ast_with_stripped_body(&body);
    assert_eq!(packed_ast.get_body(), &expected_body);
    assert_eq!(packed_ast.get_ast(), &expected_ast);
}

#[test]
fn no_derive_owner_type() {
    #[derive(Debug)]
    struct NoDerive(i32);

    #[derive(Debug)]
    struct Dependent<'a>(&'a i32);

    impl<'a> From<&'a NoDerive> for Dependent<'a> {
        fn from(d: &'a NoDerive) -> Self {
            Self(&d.0)
        }
    }

    self_cell!(NoDeriveCell, { Debug }, NoDerive, Dependent, covariant);
    let no_derive = NoDeriveCell::new(NoDerive(22));
    assert_eq!(no_derive.borrow_dependent().0, &22);
}

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

self_cell!(NotSendCell, {}, String, NotSend, covariant);

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
    t.compile_fail("tests/invalid/*.rs");
}

// TODO panic in from
// TODO try_new
