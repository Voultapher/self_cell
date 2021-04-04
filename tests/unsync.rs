// Unfortunately some unsafe is being used, this gets tested with miri.

use core::fmt::Debug;

use once_self_cell::unsync_once_self_cell;

#[derive(Debug, Eq, PartialEq)]
struct Ast<'input>(pub Vec<&'input str>);

impl<'a> From<&'a String> for Ast<'a> {
    fn from(body: &'a String) -> Self {
        Ast(vec![&body[2..5], &body[1..3]])
    }
}

unsync_once_self_cell!(
    LazyAstCell,
    String,
    Ast<'_>,
    derive(Clone, Debug, Eq, PartialEq)
);

#[derive(Debug, Clone, Eq, PartialEq)]
struct LazyAst {
    ast_cell: LazyAstCell,
}

impl LazyAst {
    fn new(body: String) -> Self {
        LazyAst {
            ast_cell: LazyAstCell::new(body),
        }
    }

    fn get_body<'a>(&'a self) -> &'a String {
        self.ast_cell.get_owner()
    }

    fn get_ast<'a>(&'a self) -> &'a Ast<'a> {
        self.ast_cell.get_or_init_dependent()
    }
}

#[test]
fn unsync_parse_ast() {
    let body = String::from("some longer string that ends now");

    // expected_ast is on the stack and lifetime dependent on body.
    let expected_ast = Ast::from(&body);

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
    let expected_ast = Ast::from(&expected_body);

    let lazy_ast = make_ast_with_stripped_body(&body);
    assert_eq!(lazy_ast.get_body(), &expected_body);
    assert_eq!(lazy_ast.get_ast(), &expected_ast);
}

#[test]
fn no_derive_owner_type() {
    struct NoDerive(i32);

    impl<'a> Into<&'a i32> for &'a NoDerive {
        fn into(self) -> &'a i32 {
            &self.0
        }
    }

    unsync_once_self_cell!(NoDeriveCell, NoDerive, &'_ i32);
    let no_derive = NoDeriveCell::new(NoDerive(22));
    assert_eq!(no_derive.get_or_init_dependent(), &&22);
}

#[test]
fn multi_derive_generated_type() {
    // While the derives could be part of the same meta expression,
    // this tests that multiple derive statements can show up.
    unsync_once_self_cell!(
        NoDeriveCell,
        String,
        &'_ String,
        derive(Clone),
        derive(Debug),
        derive(PartialEq)
    );

    let multi_derive = NoDeriveCell::new("abc".into());
    assert_eq!(**multi_derive.get_or_init_dependent(), String::from("abc"));

    let multi_derive_clone = multi_derive.clone();
    assert!(format!("{:?}", multi_derive)
        .starts_with("NoDeriveCell { unsafe_self_cell: UnsafeOnceSelfCell { owner"));
    assert!(format!("{:?}", multi_derive_clone)
        .starts_with("NoDeriveCell { unsafe_self_cell: UnsafeOnceSelfCell { owner"));
    assert_eq!(multi_derive, multi_derive_clone);
}

#[test]
fn custom_drop() {
    #[derive(Debug, PartialEq, Eq)]
    struct Ref<'a, T: Debug>(&'a T);

    impl<'a, T: Debug> Drop for Ref<'a, T> {
        fn drop(&mut self) {
            println!("{:?}", self.0);
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum Void {}

    type OV = Option<Vec<Void>>;

    impl<'a> Into<Ref<'a, OV>> for &'a OV {
        fn into(self) -> Ref<'a, OV> {
            Ref(self)
        }
    }

    unsync_once_self_cell!(CustomDrop, OV, Ref<'_, OV>, derive(Debug, PartialEq, Eq));

    let cell = CustomDrop::new(None);

    let expected_dependent = Ref::<'_, OV>(&None);

    assert_eq!(cell.get_or_init_dependent(), &expected_dependent);
}

// #[test]
// // Not supported by miri isolation.
// #[cfg_attr(miri, ignore)]
// // Closure paths slashes show up as diff error on Windows.
// #[cfg(not(target_os = "windows"))]
// fn invalid_compile() {
//     let t = trybuild::TestCases::new();
//     t.compile_fail("tests/invalid/*.rs");
// }
