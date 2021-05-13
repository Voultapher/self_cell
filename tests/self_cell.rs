// The unsafe being used gets tested with miri in the CI.

#![deny(private_in_public)]

use std::convert::TryInto;
use std::fmt::Debug;

use crossbeam_utils::thread;

use impls::impls;

use once_cell::unsync::OnceCell;

use self_cell::self_cell;

#[derive(Debug, Eq, PartialEq)]
pub struct Ast<'input>(pub Vec<&'input str>);

impl<'a> From<&'a String> for Ast<'a> {
    fn from(body: &'a String) -> Self {
        Ast(vec![&body[2..5], &body[1..3]])
    }
}

self_cell!(
    #[doc(hidden)]
    struct PackedAstCell {
        #[from]
        owner: String,

        #[covariant]
        dependent: Ast,
    }

    impl {Clone, Debug, PartialEq, Eq, Hash}
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

    fn with_ast(&self, func: impl for<'a> FnOnce(&'a String, &'a Ast<'a>)) {
        self.ast_cell.with_dependent(func)
    }

    fn get_ast<'a>(&'a self) -> &'a Ast<'a> {
        self.ast_cell.borrow_dependent()
    }
}

fn assert_with_ast(packed_ast: &PackedAst, expected_ast: &Ast) {
    let mut visited = false;
    packed_ast.with_ast(|_, ast| {
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
fn failable_constructor_success() {
    #[derive(Debug, Clone, PartialEq)]
    struct Owner(String);

    impl<'a> TryInto<Ast<'a>> for &'a Owner {
        type Error = i32;

        fn try_into(self) -> Result<Ast<'a>, Self::Error> {
            Ok(Ast::from(&self.0))
        }
    }

    self_cell!(
        struct AstOk {
            #[try_from]
            owner: Owner,

            #[covariant]
            dependent: Ast,
        }

        impl {Debug}
    );

    let owner = Owner("This string is no trout".into());
    let expected_ast = Ast::from(&owner.0);

    let ast_cell_result: Result<AstOk, i32> = AstOk::try_from(owner.clone());
    assert!(ast_cell_result.is_ok());

    let ast_cell = ast_cell_result.unwrap();
    assert_eq!(ast_cell.borrow_owner(), &owner);
    assert_eq!(ast_cell.borrow_dependent(), &expected_ast);
}

#[test]
fn failable_constructor_fail() {
    mod no_try_into_import {
        use super::Ast;
        use self_cell::self_cell;

        #[derive(Debug, Clone, PartialEq)]
        pub struct Owner(pub String);

        impl<'a> std::convert::TryInto<Ast<'a>> for &'a Owner {
            type Error = i32;

            fn try_into(self) -> Result<Ast<'a>, Self::Error> {
                Err(22)
            }
        }

        self_cell!(
            pub struct AstOk {
                #[try_from]
                owner: Owner,

                #[covariant]
                dependent: Ast,
            }

            impl {Debug}
        );
    }

    let owner = no_try_into_import::Owner("This string is no trout".into());

    let ast_cell_result: Result<no_try_into_import::AstOk, i32> =
        no_try_into_import::AstOk::try_from(owner.clone());
    assert!(ast_cell_result.is_err());

    let err = ast_cell_result.unwrap_err();
    assert_eq!(err, 22);
}

#[test]
fn from_fn() {
    #[derive(Debug)]
    struct Dependent<'a>(&'a String);

    self_cell!(
        struct FnCell {
            #[from_fn]
            owner: String,

            #[covariant]
            dependent: Dependent,
        }

        impl {Debug}
    );

    let mut extra_outside_state = None;

    assert_eq!(extra_outside_state, None);

    let expected_str = "small pink bike";

    let fn_cell = FnCell::from_fn(expected_str.clone().into(), |owner| {
        // Make sure it only gets called once.
        extra_outside_state = if let Some(x) = extra_outside_state {
            Some(x + 5)
        } else {
            Some(66)
        };

        Dependent(owner)
    });

    assert_eq!(extra_outside_state, Some(66));
    assert_eq!(fn_cell.borrow_owner(), expected_str);
    assert_eq!(fn_cell.borrow_dependent().0, expected_str);
    assert_eq!(extra_outside_state, Some(66));
}

#[test]
fn catch_panic_in_from() {
    // This pattern allows users to opt into not leaking memory on panic during
    // cell construction.

    #[derive(Debug, Clone, PartialEq)]
    struct Owner(String);

    #[derive(Debug)]
    struct PanicCtor<'a>(&'a i32);

    impl<'a> PanicCtor<'a> {
        fn new(_: &'a Owner) -> Self {
            let _stack_vec = vec![23, 44, 5];
            panic!()
        }
    }

    impl<'a> TryInto<PanicCtor<'a>> for &'a Owner {
        type Error = Box<dyn std::any::Any + Send + 'static>;

        fn try_into(self) -> Result<PanicCtor<'a>, Self::Error> {
            std::panic::catch_unwind(|| PanicCtor::new(&self))
        }
    }

    self_cell!(
        struct NoLeakCell {
            #[try_from]
            owner: Owner,

            #[covariant]
            dependent: PanicCtor,

        }

        impl {Debug}
    );

    let owner = Owner("This string is no trout".into());

    let ast_cell_result = NoLeakCell::try_from(owner.clone());
    assert!(ast_cell_result.is_err());
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

    self_cell!(
        struct NoDeriveCell {
            #[from]
            owner: NoDerive,

            #[covariant]
            dependent: Dependent,
        }

        impl {Debug}
    );
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

#[test]
fn public_cell() {
    self_cell!(
        pub struct PubCell {
            #[from]
            owner: String,

            #[covariant]
            dependent: Ast,
        }

        impl {} // empty impl
    );

    #[allow(dead_code)]
    pub type PubTy = PubCell;
}

self_cell!(
    struct NotSendCell {
        #[from]
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
    type OvRef<'a> = Ref<'a, OV>;

    impl<'a> Into<Ref<'a, OV>> for &'a OV {
        fn into(self) -> Ref<'a, OV> {
            Ref(self)
        }
    }

    self_cell!(
        struct CustomDrop {
            #[from]
            owner: OV,

            #[covariant]
            dependent: OvRef,
        }

        impl {Debug, PartialEq, Eq}
    );

    let cell = CustomDrop::new(None);

    let expected_dependent = Ref::<'_, OV>(&None);

    assert_eq!(cell.borrow_dependent(), &expected_dependent);
}

#[test]
fn dependent_mutate() {
    let mut ast_cell = PackedAstCell::new("Egal in welchen Farben ihr den ..".into());

    assert_eq!(ast_cell.borrow_dependent().0.len(), 2);

    ast_cell.with_dependent_mut(|_, ast| {
        ast.0.clear();
    });

    assert_eq!(ast_cell.borrow_dependent().0.len(), 0);
}

#[test]
fn dependent_replace() {
    let before_input = String::from("Egal in welchen Farben ihr den ..");
    let before_ast_expected = Ast::from(&before_input);

    let mut ast_cell = PackedAstCell::new(before_input.clone());

    assert_eq!(ast_cell.borrow_owner(), &before_input);
    assert_eq!(ast_cell.borrow_dependent(), &before_ast_expected);

    ast_cell.with_dependent_mut(|owner, ast| {
        *ast = Ast(vec![&owner[0..2], &owner[0..3], &owner[5..9]]);
    });

    assert_eq!(ast_cell.borrow_dependent().0, vec!["Eg", "Ega", "in w"]);
}

#[test]
fn share_across_threads() {
    // drop_joined takes &mut self, so that's not a thread concern anyway.
    // And get_or_init_dependent should be as thread compatible as OnceCell.
    // Owner never gets changed after init.

    let body = String::from("smoli");

    // expected_ast is on the stack and lifetime dependent on body.
    let expected_ast = Ast::from(&body);

    // But PackedAst is struct and can be freely moved and copied.
    let packed_ast = PackedAst::new(body.clone());

    thread::scope(|s| {
        s.spawn(|_| {
            assert_eq!(packed_ast.get_body(), &body);
            assert_eq!(packed_ast.get_ast(), &expected_ast);
        });

        s.spawn(|_| {
            assert_eq!(packed_ast.get_body(), &body);
            assert_eq!(packed_ast.get_ast(), &expected_ast);
        });

        assert_eq!(packed_ast.get_body(), &body);
        assert_eq!(packed_ast.get_ast(), &expected_ast);
    })
    .unwrap();
}

#[test]
fn lazy_ast() {
    #[derive(Debug)]
    struct LazyAst<'a>(OnceCell<Ast<'a>>);

    impl<'a> From<&'a String> for LazyAst<'a> {
        fn from(_: &'a String) -> Self {
            Self(OnceCell::new())
        }
    }

    self_cell!(
        #[doc(hidden)]
        struct LazyAstCell {
            #[from]
            owner: String,

            #[not_covariant]
            dependent: LazyAst,
        }

        impl {Clone, Debug, PartialEq, Eq, Hash}
    );

    let body = String::from("How thou shall not see what trouts shall see");

    let expected_ast = Ast::from(&body);

    let lazy_ast = LazyAstCell::new(body.clone());
    assert_eq!(lazy_ast.borrow_owner(), &body);
    lazy_ast.with_dependent(|owner, dependent| {
        assert_eq!(owner, &body);
        assert!(dependent.0.get().is_none());
    });

    lazy_ast.with_dependent(|owner, dependent| {
        assert_eq!(owner, &body);
        assert_eq!(dependent.0.get_or_init(|| owner.into()), &expected_ast);
    });

    lazy_ast.with_dependent(|owner, dependent| {
        assert_eq!(owner, &body);
        assert!(dependent.0.get().is_some());
        assert_eq!(dependent.0.get_or_init(|| owner.into()), &expected_ast);
    });
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
