// Unfortunately some unsafe is being used, this gets tested with miri.

use once_self_cell::unsync::OnceSelfCell;

#[derive(Debug, Eq, PartialEq)]
struct Ast<'input>(pub &'input str);

fn ast_from_string<'input>(owner: &'input String) -> Ast<'input> {
    Ast(&owner[2..5])
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct LazyAst {
    once_ast: OnceSelfCell<String>,
}

impl LazyAst {
    fn new(body: String) -> Self {
        LazyAst{once_ast: OnceSelfCell::<String>::new(body)}
    }

    fn get_body<'a>(&'a self) -> &'a String {
        self.once_ast.get_owner()
    }

    fn get_ast<'a>(&'a self) -> &'a Ast<'a> {
        // The user has to make sure that the return type of ast_from_string and the generic
        // parameter of get_or_init_dependent are the same.
        self.once_ast.get_or_init_dependent(ast_from_string)
    }
}

impl Drop for LazyAst {
    fn drop<'a>(&'a mut self) {
        unsafe { self.once_ast.drop_dependent_unconditional::<Ast<'a>>() };
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
fn drop_multiple_times<'a>() {
    let body = String::from("some longer string that ends now");
    let expected_ast = ast_from_string(&body);

    let mut once_self_cell = OnceSelfCell::<String>::new(body.clone());

    assert_eq!(once_self_cell.get_owner(), &body);

    // drop now should do nothing.
    unsafe { once_self_cell.drop_dependent_unconditional::<Ast<'a>>() };

    assert_eq!(once_self_cell.get_owner(), &body);

    // fill the dependent
    let ast = once_self_cell.get_or_init_dependent(ast_from_string);
    assert_eq!(once_self_cell.get_owner(), &body);
    assert_eq!(ast, &expected_ast);

    // First drop should actually drop the thing, the next one should be a no-op.
    unsafe { once_self_cell.drop_dependent_unconditional::<Ast<'a>>() };
    unsafe { once_self_cell.drop_dependent_unconditional::<Ast<'a>>() };

    assert_eq!(once_self_cell.get_owner(), &body);

    // fill the dependent again
    let ast = once_self_cell.get_or_init_dependent(ast_from_string);
    assert_eq!(once_self_cell.get_owner(), &body);
    assert_eq!(ast, &expected_ast);

    unsafe { once_self_cell.drop_dependent_unconditional::<Ast<'a>>() };
    unsafe { once_self_cell.drop_dependent_unconditional::<Ast<'a>>() };
    unsafe { once_self_cell.drop_dependent_unconditional::<Ast<'a>>() };
}