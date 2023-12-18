// The unsafe being used gets tested with miri in the CI.

use std::cell::Cell;
use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::panic::catch_unwind;
use std::rc::Rc;
use std::str;

use once_cell::unsync::OnceCell;

use self_cell::self_cell;

#[derive(Debug, Eq, PartialEq)]
pub struct Ast<'input>(pub Vec<&'input str>);

impl<'x> From<&'x String> for Ast<'x> {
    fn from(body: &String) -> Ast<'_> {
        Ast(vec![&body[2..5], &body[1..3]])
    }
}

self_cell!(
    #[doc(hidden)]
    struct PackedAstCell {
        owner: String,

        #[covariant]
        dependent: Ast,
    }

    impl {Debug, PartialEq, Eq, Hash}
);

impl Clone for PackedAstCell {
    fn clone(&self) -> Self {
        PackedAstCell::new(self.borrow_owner().clone(), |owner| owner.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct PackedAst {
    ast_cell: PackedAstCell,
}

impl PackedAst {
    fn new(body: String) -> Self {
        Self {
            ast_cell: PackedAstCell::new(body, |owner| owner.into()),
        }
    }

    fn get_body(&self) -> &String {
        self.ast_cell.borrow_owner()
    }

    fn with_ast<'o>(&'o self, func: impl for<'a> std::ops::FnOnce(&'a String, &'o Ast<'a>)) {
        self.ast_cell.with_dependent(func)
    }

    #[allow(clippy::needless_lifetimes)]
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

// These are defined visible for the whole module. Because incorrect usage could affect many
// components it is not isolated to a specific test.
#[allow(non_snake_case, dead_code)]
fn Ok<T>(_: T) -> ! {
    panic!("Wrong Ok function called")
}

#[allow(non_snake_case, dead_code)]
fn Err<T>(_: T) -> ! {
    panic!("Wrong Err function called")
}

#[allow(dead_code)]
struct FnOnce<T> {
    marker: T,
}

mod core {
    pub mod ptr {
        #[allow(dead_code)]
        pub const unsafe fn read<T>(_src: *const T) -> T {
            panic!("Wrong ptr::read function called")
        }
    }
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
    let stripped_body = body.replace('\n', "");
    // Return Ast built from moved body, no lifetime hassle.
    PackedAst::new(stripped_body)
}

#[test]
fn return_self_ref_struct() {
    let body = String::from("a\nb\nc\ndef");
    let expected_body = body.replace('\n', "");

    // expected_ast is on the stack and lifetime dependent on body.
    let expected_ast = Ast::from(&expected_body);

    let packed_ast = make_ast_with_stripped_body(&body);
    assert_eq!(packed_ast.get_body(), &expected_body);
    assert_eq!(packed_ast.get_ast(), &expected_ast);
}

#[test]
fn failable_constructor_success() {
    let owner = String::from("This string is no trout");
    let expected_ast = Ast::from(&owner);

    let ast_cell_result: Result<PackedAstCell, i32> =
        PackedAstCell::try_new(owner.clone(), |owner| {
            std::result::Result::Ok(Ast::from(owner))
        });

    assert!(ast_cell_result.is_ok());

    let ast_cell = ast_cell_result.unwrap();
    assert_eq!(ast_cell.borrow_owner(), &owner);
    assert_eq!(ast_cell.borrow_dependent(), &expected_ast);
}

#[test]
fn failable_constructor_fail() {
    let owner = String::from("This string is no trout");

    let ast_cell_result: Result<PackedAstCell, i32> =
        PackedAstCell::try_new(owner.clone(), |_owner| std::result::Result::Err(22));

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
            owner: String,

            #[covariant]
            dependent: Dependent,
        }

        impl {Debug}
    );

    let mut extra_outside_state = None;

    assert_eq!(extra_outside_state, None);

    let expected_str = "small pink bike";

    let fn_cell = FnCell::new(expected_str.into(), |owner| {
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
            let stack_vec = vec![23, 44, 5];
            println!("{stack_vec:?}");
            panic!()
        }
    }

    self_cell!(
        struct NoLeakCell {
            owner: Owner,

            #[covariant]
            dependent: PanicCtor,

        }

        impl {Debug}
    );

    let owner = Owner("This string is no trout".into());

    let ast_cell_result = NoLeakCell::try_new(owner.clone(), |owner| {
        catch_unwind(|| PanicCtor::new(owner))
    });
    assert!(ast_cell_result.is_err());
}

#[test]
fn no_derive_owner_type() {
    #[derive(Debug)]
    struct NoDerive(i32);

    #[derive(Debug)]
    struct Dependent<'a>(&'a i32);

    self_cell!(
        struct NoDeriveCell {
            owner: NoDerive,

            #[covariant]
            dependent: Dependent,
        }

        impl {Debug}
    );
    let no_derive = NoDeriveCell::new(NoDerive(22), |owner| Dependent(&owner.0));
    assert_eq!(no_derive.borrow_dependent().0, &22);
}

#[test]
fn public_cell() {
    self_cell!(
        pub struct PubCell {
            owner: String,

            #[covariant]
            dependent: Ast,
        }

        impl {} // empty impl
    );

    #[allow(dead_code)]
    pub type PubTy = PubCell;
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

    self_cell!(
        struct CustomDrop {
            owner: OV,

            #[covariant]
            dependent: OvRef,
        }

        impl {Debug, PartialEq, Eq}
    );

    let cell = CustomDrop::new(None, |owner| Ref(owner));

    let expected_dependent = Ref::<'_, OV>(&None);

    assert_eq!(cell.borrow_dependent(), &expected_dependent);
}

#[test]
fn drop_order() {
    #[derive(Debug, PartialEq, Eq)]
    enum Dropped {
        Owner,
        Dependent,
    }

    struct Owner(Rc<RefCell<Vec<Dropped>>>);
    struct Dependent<'a>(&'a Owner, Rc<RefCell<Vec<Dropped>>>);

    impl Drop for Owner {
        fn drop(&mut self) {
            self.0.borrow_mut().push(Dropped::Owner)
        }
    }

    impl Drop for Dependent<'_> {
        fn drop(&mut self) {
            self.1.borrow_mut().push(Dropped::Dependent)
        }
    }

    self_cell! {
        struct DropOrder {
            owner: Owner,

            #[covariant]
            dependent: Dependent,
        }
    }

    let drops: Rc<RefCell<Vec<Dropped>>> = <_>::default();
    let cell = DropOrder::new(Owner(drops.clone()), |o| Dependent(o, drops.clone()));
    drop(cell);
    assert_eq!(&drops.borrow()[..], &[Dropped::Dependent, Dropped::Owner]);
}

#[test]
fn into_owner_drop_dependent_without_panic() {
    // This test resulted in a double-free in a previous version of self-cell
    type O = Cell<Option<Box<u8>>>;

    self_cell! {
        struct S {
            owner: O,

            #[covariant]
            dependent: D,
        }
    }

    struct D<'a>(&'a O);

    impl Drop for D<'_> {
        fn drop(&mut self) {
            self.0.take();
        }
    }

    let s = S::new(Cell::new(Some(Box::new(42))), |o| D(o));
    assert!(s.into_owner().into_inner().is_none());
}

#[test]
#[should_panic] // but should not leak or double-free
fn into_owner_drop_dependent_with_panic() {
    type O = Cell<Option<Box<u8>>>;

    self_cell! {
        struct S {
            owner: O,

            #[covariant]
            dependent: D,
        }
    }

    struct D<'a>(&'a O);

    impl Drop for D<'_> {
        fn drop(&mut self) {
            self.0.take();
            panic!();
        }
    }

    let s = S::new(Cell::new(Some(Box::new(42))), |o| D(o));
    s.into_owner();
}

#[test]
fn drop_panic_owner() {
    #[derive(Clone, Debug, PartialEq)]
    struct Owner(String);

    type Dependent<'a> = &'a Owner;

    self_cell! {
        struct DropPanicOwner {
            owner: Owner,

            #[covariant]
            dependent: Dependent,
        }
    }

    impl Drop for Owner {
        fn drop(&mut self) {
            panic!()
        }
    }

    let owner = Owner("s t e f f a h n <3 and some padding against sbo".into());

    let cell = DropPanicOwner::new(owner.clone(), |o| o);
    assert_eq!(cell.borrow_owner(), &owner);
    assert_eq!(cell.borrow_dependent(), &&owner);

    assert!(std::panic::catch_unwind(move || drop(cell)).is_err());
    assert!(std::panic::catch_unwind(move || drop(owner)).is_err());
}

#[test]
fn drop_panic_dependent() {
    #[derive(Clone, Debug, PartialEq)]
    struct Owner(String);

    struct Dependent<'a>(&'a Owner);

    self_cell! {
        struct DropPanicDependent {
            owner: Owner,

            #[covariant]
            dependent: Dependent,
        }
    }

    impl Drop for Dependent<'_> {
        fn drop(&mut self) {
            panic!()
        }
    }

    let owner = Owner("s t e f f a h n <3".into());

    let cell = DropPanicDependent::new(owner.clone(), |o| Dependent(o));
    assert_eq!(cell.borrow_owner(), &owner);
    assert_eq!(cell.borrow_dependent().0, &owner);

    assert!(std::panic::catch_unwind(move || drop(cell)).is_err());
}

#[test]
fn dependent_mutate() {
    let mut ast_cell = PackedAstCell::new("Egal in welchen Farben ihr den ..".into(), |owner| {
        owner.into()
    });

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

    let mut ast_cell = PackedAstCell::new(before_input.clone(), |owner| owner.into());

    assert_eq!(ast_cell.borrow_owner(), &before_input);
    assert_eq!(ast_cell.borrow_dependent(), &before_ast_expected);

    ast_cell.with_dependent_mut(|owner, ast| {
        *ast = Ast(vec![&owner[0..2], &owner[0..3], &owner[5..9]]);
    });

    assert_eq!(ast_cell.borrow_dependent().0, vec!["Eg", "Ega", "in w"]);
}

#[test]
fn try_new_or_recover() {
    let original_input = String::from("Ein See aus Schweiß ..");

    // bad path
    let (input, err) =
        PackedAstCell::try_new_or_recover(original_input.clone(), |_| std::result::Result::Err(-1))
            .unwrap_err();

    assert_eq!(original_input, input);
    assert_eq!(err, -1);

    // happy path
    let cell = PackedAstCell::try_new_or_recover(original_input.clone(), |o| -> Result<_, ()> {
        std::result::Result::Ok(o.into())
    })
    .unwrap();
    assert_eq!(cell.borrow_owner(), &original_input);
    assert_eq!(cell.borrow_dependent(), &Ast::from(&original_input));
}

#[test]
fn into_owner() {
    // The Rc stuff here is somewhat tangential to what is being tested here.
    // The regular type system should enforce most of the invariants of into_owner
    // and miri should detect leaks.

    self_cell!(
        struct RcAstCell {
            owner: Rc<String>,

            #[covariant]
            dependent: Ast,
        }
    );

    let expected_body = Rc::new(String::from("Endless joy for you never 2"));

    // expected_ast is on the stack and lifetime dependent on body.
    let expected_ast = Ast::from(&*expected_body);

    let ast_cell = RcAstCell::new(Rc::clone(&expected_body), |s| Ast::from(&**s));
    assert_eq!(ast_cell.borrow_owner(), &expected_body);
    assert_eq!(ast_cell.borrow_dependent(), &expected_ast);

    let body_recovered: Rc<String> = ast_cell.into_owner();
    assert_eq!(&body_recovered, &expected_body);
    assert_eq!(Rc::strong_count(&expected_body), 2);

    // This shouldn't be possible anymore.
    // assert_eq!(ast_cell.borrow_owner(), &expected_body);
}

#[test]
fn zero_size_cell() {
    struct ZeroSizeRef<'a>(PhantomData<&'a ()>);

    self_cell!(
        struct ZeroSizeCell {
            owner: (),

            #[covariant]
            dependent: ZeroSizeRef,
        }
    );

    assert!(catch_unwind(|| ZeroSizeCell::new((), |_| ZeroSizeRef(PhantomData))).is_err());

    assert!(
        catch_unwind(|| ZeroSizeCell::try_new((), |_| -> Result<_, i32> {
            std::result::Result::Ok(ZeroSizeRef(PhantomData))
        }))
        .is_err()
    );

    assert!(catch_unwind(
        || ZeroSizeCell::try_new_or_recover((), |_| -> Result<_, i32> {
            std::result::Result::Ok(ZeroSizeRef(PhantomData))
        })
    )
    .is_err());
}

#[test]
fn nested_cells() {
    // TODO allow automatic impls.
    // impl {Debug, PartialEq, Eq, Hash}
    self_cell!(
        struct ChildCell<'a> {
            owner: &'a String,

            #[covariant]
            dependent: Ast,
        }
    );

    self_cell!(
        struct ParentCell {
            owner: String,

            #[covariant]
            dependent: ChildCell,
        }
    );

    let parent_owner_expected = String::from("some string it is");
    let ast_expected = Ast::from(&parent_owner_expected);

    let parent_cell = ParentCell::new(parent_owner_expected.clone(), |parent| {
        ChildCell::new(parent, |child| Ast::from(*child))
    });

    assert_eq!(parent_cell.borrow_owner(), &parent_owner_expected);

    let child_cell = parent_cell.borrow_dependent();
    assert_eq!(*child_cell.borrow_owner(), &parent_owner_expected);
    assert_eq!(child_cell.borrow_dependent(), &ast_expected);
}

// partial nested cells

#[test]
fn panic_in_from_owner() {
    // panicking in user provided code shouldn't leak memory.

    type Dependent<'a> = &'a String;

    self_cell!(
        struct PanicCell {
            owner: String,

            #[covariant]
            dependent: Dependent,
        }
    );

    let owner = String::from("panic_in_from_owner");

    let new_result = std::panic::catch_unwind(|| {
        let _ = PanicCell::new(owner.clone(), |_| panic!());
    });
    assert!(new_result.is_err());

    let try_new_result = std::panic::catch_unwind(|| {
        let _ = PanicCell::try_new(owner.clone(), |_| -> Result<_, Box<i32>> { panic!() });
    });
    assert!(try_new_result.is_err());

    let try_new_or_recover_result = std::panic::catch_unwind(|| {
        let _ = PanicCell::try_new_or_recover(owner.clone(), |_| -> Result<_, i32> { panic!() });
    });
    assert!(try_new_or_recover_result.is_err());
}

#[test]
fn result_name_hygiene() {
    // See https://github.com/Voultapher/self_cell/issues/16
    #[allow(dead_code)]
    type Result<T> = std::result::Result<T, ()>;

    type VecRef<'a> = &'a Vec<u8>;

    self_cell!(
        struct SomeCell {
            owner: Vec<u8>,

            #[covariant]
            dependent: VecRef,
        }

        impl {Debug, PartialEq, Eq, Hash}
    );
}

#[test]
fn debug_impl() {
    // See https://github.com/Voultapher/self_cell/pull/22
    let ast_cell = PackedAstCell::new("xyz, abv".into(), |owner| owner.into());

    assert_eq!(
        format!("{:?}", &ast_cell),
        "PackedAstCell { owner: \"xyz, abv\", dependent: Ast([\"z, \", \"yz\"]) }"
    );

    let hash_fmt = r#"PackedAstCell {
    owner: "xyz, abv",
    dependent: Ast(
        [
            "z, ",
            "yz",
        ],
    ),
}"#;

    assert_eq!(format!("{:#?}", ast_cell), hash_fmt);
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
            owner: String,

            #[not_covariant]
            dependent: LazyAst,
        }

        impl {Debug, PartialEq, Eq, Hash}
    );

    let body = String::from("How thou shall not see what trouts shall see");

    let expected_ast = Ast::from(&body);

    let lazy_ast = LazyAstCell::new(body.clone(), |owner| owner.into());
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
fn cell_mem_size() {
    use std::mem::size_of;

    assert_eq!(size_of::<PackedAstCell>(), size_of::<*const u8>());
    assert_eq!(size_of::<Option<PackedAstCell>>(), size_of::<*const u8>());
}
