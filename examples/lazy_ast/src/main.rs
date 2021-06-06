use self_cell::self_cell;

use once_cell::unsync::OnceCell;

#[derive(Debug, Eq, PartialEq)]
struct Ast<'input>(pub Vec<&'input str>);

impl<'a> From<&'a String> for Ast<'a> {
    fn from(code: &'a String) -> Self {
        println!("[parsing code]");
        Ast(code.split(" ").collect())
    }
}

type LazyAstCell<'a> = OnceCell<Ast<'a>>;

self_cell!(
    struct LazyAst {
        owner: String,

        #[not_covariant] // Because OnceCell uses UnsafeCell.
        dependent: LazyAstCell,
    }

    impl {Debug, PartialEq, Eq, Hash}
);

impl LazyAst {
    fn get_code(&self) -> &str {
        self.borrow_owner()
    }

    fn is_parsed(&self) -> bool {
        self.with_dependent(|_, dependent| dependent.get().is_some())
    }

    fn fmt_ast(&self) -> String {
        self.with_dependent(|owner, dependent| {
            format!("{:?}", dependent.get_or_init(|| owner.into()))
        })
    }
}

impl Clone for LazyAst {
    fn clone(&self) -> Self {
        Self::new(self.borrow_owner().clone(), |_| OnceCell::new())
    }
}

fn main() {
    let lazy_ast = LazyAst::new("a[i * x[y]] * sin(z)".into(), |_| OnceCell::new());

    println!("lazy_ast.get_code() -> {}", lazy_ast.get_code());
    println!("lazy_ast.is_parsed() -> {}", lazy_ast.is_parsed());

    println!("lazy_ast.fmt_ast() -> {}", lazy_ast.fmt_ast());
    println!("lazy_ast.is_parsed() -> {}", lazy_ast.is_parsed());

    // This should not parse the Ast again, but use the existing one.
    println!("lazy_ast.fmt_ast() -> {}", lazy_ast.fmt_ast());
}
