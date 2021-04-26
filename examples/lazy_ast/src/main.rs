use self_cell::self_cell;

use once_cell::unsync::OnceCell;

#[derive(Debug, Eq, PartialEq)]
struct Ast<'input>(pub Vec<&'input str>);

impl<'a> From<&'a String> for Ast<'a> {
    fn from(code: &'a String) -> Self {
        dbg!("Parsing code");
        Ast(code.split(" ").collect())
    }
}

#[derive(Debug)]
struct LazyAstCell<'a>(OnceCell<Ast<'a>>);

impl<'a> From<&'a String> for LazyAstCell<'a> {
    fn from(_: &'a String) -> Self {
        Self(OnceCell::new())
    }
}

self_cell!(
    struct LazyAst {
        #[from]
        owner: String,

        #[not_covariant] // Because OnceCell uses UnsafeCell.
        dependent: LazyAstCell,
    }

    impl {Clone, Debug, PartialEq, Eq, Hash}
);

impl LazyAst {
    fn get_code(&self) -> &str {
        self.borrow_owner()
    }

    fn is_parsed(&self) -> bool {
        self.with_dependent(|_, dependent| dependent.0.get().is_some())
    }

    fn fmt_ast(&self) -> String {
        self.with_dependent(|owner, dependent| {
            format!("{:?}", dependent.0.get_or_init(|| owner.into()))
        })
    }
}

fn main() {
    let lazy_ast = LazyAst::new("a[i * x[y]] * sin(z)".into());

    dbg!(lazy_ast.get_code());
    dbg!(lazy_ast.is_parsed());

    dbg!(lazy_ast.fmt_ast());
    dbg!(lazy_ast.is_parsed());

    // This should not parse the Ast again, but use the existing one.
    dbg!(lazy_ast.fmt_ast());
}
