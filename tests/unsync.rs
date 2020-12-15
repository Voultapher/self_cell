// Unfortunately some unsafe is being used, this gets tested with miri.

use core::fmt::Debug;

use once_self_cell::unsync::OnceSelfCell;

#[derive(Debug, Eq, PartialEq)]
struct Ast<'input>(pub Vec<&'input str>);

fn ast_from_string<'input>(owner: &'input String) -> Ast<'input> {
    Ast(vec![&owner[2..5], &owner[1..3]])
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct LazyAst {
    ast_cell: OnceSelfCell<String, Ast<'static>>,
}

impl LazyAst {
    fn new(body: String) -> Self {
        LazyAst {
            ast_cell: OnceSelfCell::new(body),
        }
    }

    fn get_body<'a>(&'a self) -> &'a String {
        self.ast_cell.get_owner()
    }

    fn get_ast<'a>(&'a self) -> &'a Ast<'a> {
        self.ast_cell.get_or_init_dependent(ast_from_string)
    }
}

#[test]
fn unsync_parse_ast() {
    let body = String::from("some longer string that ends now");

    // expected_ast is on the stack and lifetime dependent on body.
    let expected_ast = ast_from_string(&body);

    // But LazyAst is struct and can be freely moved and copied.
    let lazy_ast = LazyAst::new(body.clone());
    assert_eq!(lazy_ast.get_body(), &body);
    assert_eq!(lazy_ast.get_ast(), &expected_ast);

    let cloned_lazy_ast = lazy_ast.clone();
    assert_eq!(cloned_lazy_ast.get_body(), &body);
    assert_eq!(cloned_lazy_ast.get_ast(), &expected_ast);

    let moved_lazy_ast = lazy_ast;
    assert_eq!(moved_lazy_ast.get_body(), &body);
    assert_eq!(moved_lazy_ast.get_ast(), &expected_ast);

    // Assert that even though the original lazy_ast was moved, the clone of it is still valid.
    assert_eq!(cloned_lazy_ast.get_body(), &body);
    assert_eq!(cloned_lazy_ast.get_ast(), &expected_ast);
}

fn make_ast_with_stripped_body(body: &str) -> LazyAst {
    // This is created on the stack.
    let stripped_body = body.replace("\n", "");
    // Return Ast built from moved body, no lifetime hassle.
    LazyAst::new(stripped_body)
}

#[test]
fn return_self_ref_struct() {
    let body = String::from("a\nb\nc\ndef");
    let expected_body = body.replace("\n", "");

    // expected_ast is on the stack and lifetime dependent on body.
    let expected_ast = ast_from_string(&expected_body);

    let lazy_ast = make_ast_with_stripped_body(&body);
    assert_eq!(lazy_ast.get_body(), &expected_body);
    assert_eq!(lazy_ast.get_ast(), &expected_ast);
}

#[test]
fn no_derive_owner_type() {
    struct NoDerive<'a>(&'a i32);

    let no_derive = OnceSelfCell::<NoDerive, &'static i32>::new(NoDerive(&22));

    assert_eq!(no_derive.get_or_init_dependent(|x: &NoDerive| x.0), &&22);
}

#[test]
#[should_panic(
    expected = "assertion failed: `(left == right)`\n  left: `\"()\"`,\n right: `\"i32\"`"
)]
fn invalid_init_type() {
    let cell = OnceSelfCell::<String, ()>::new("".into());
    let _init_with_nothing = cell.get_or_init_dependent(|_| ());
    // Get with i32 should not work, because we expect nothing.
    let _get_with_i32 = cell.get_or_init_dependent(|_| 33);
}

#[test]
#[should_panic(
    expected = "assertion failed: `(left == right)`\n  left: `\"unsync::Ast\"`,\n right: `\"i32\"`"
)]
fn different_init_types() {
    let cell = OnceSelfCell::<String, Ast<'static>>::new("helllllo".into());
    let _init_with_ast = cell.get_or_init_dependent(ast_from_string);
    let _get_with_i32 = cell.get_or_init_dependent(|_| 33);
}

impl<'a, T: Debug> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        println!("{:?}", self.0);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Ref<'a, T: Debug>(&'a T);

#[derive(Debug, PartialEq, Eq)]
enum Void {}

#[test]
fn custom_drop() {
    type OV = Option<Vec<Void>>;
    let cell = OnceSelfCell::<OV, Ref<'static, OV>>::new(None);

    let expected_dependent = Ref::<'_, OV>(&None);

    assert_eq!(cell.get_or_init_dependent(Ref), &expected_dependent);
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
