use once_self_cell::unsync_once_self_cell;

#[derive(Debug, Eq, PartialEq)]
struct Ast<'a>(pub Vec<&'a str>);

impl<'a> From<&'a String> for Ast<'a> {
    fn from(body: &'a String) -> Self {
        Ast(vec![&body[2..5], &body[1..3]])
    }
}

unsync_once_self_cell!(LazyAstCell, String, Ast<'_>,);

// #[derive(Clone)]
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
fn playground() {
    let body = String::from("some longer string that ends now");

    // expected_ast is on the stack and lifetime dependent on body.
    let expected_ast = Ast::from(&body);

    // But LazyAst is struct and can be freely moved and copied.
    let lazy_ast = LazyAst::new(body.clone());
    assert_eq!(lazy_ast.get_body(), &body);
    assert_eq!(lazy_ast.get_ast(), &expected_ast);

    // let cloned_lazy_ast = lazy_ast.clone();
    // assert_eq!(cloned_lazy_ast.get_body(), &body);
    // assert_eq!(cloned_lazy_ast.get_ast(), &expected_ast);

    // let moved_lazy_ast = lazy_ast;
    // assert_eq!(moved_lazy_ast.get_body(), &body);
    // assert_eq!(moved_lazy_ast.get_ast(), &expected_ast);

    // // Assert that even though the original lazy_ast was moved, the clone of it is still valid.
    // assert_eq!(cloned_lazy_ast.get_body(), &body);
    // assert_eq!(cloned_lazy_ast.get_ast(), &expected_ast);
}
