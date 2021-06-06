# `lazy_ast` Example

The lazy part comes from the use of `OnceCell`. Because
`OnceCell` uses interior mutability `LazyAst` has to be marked `not_covariant`,
which in turn means we have to use the `with_dependent` accessor function.

Run this example with `cargo run`, it should output:

```
lazy_ast.get_code() -> a[i * x[y]] * sin(z)
lazy_ast.is_parsed() -> false
[parsing code]
lazy_ast.fmt_ast() -> Ast(["a[i", "*", "x[y]]", "*", "sin(z)"])
lazy_ast.is_parsed() -> true
lazy_ast.fmt_ast() -> Ast(["a[i", "*", "x[y]]", "*", "sin(z)"])
```

Notice how at the beginning `is_parsed` returns `false` because we haven't
accessed the `Ast` yet. Once we call `fmt_ast` for the first time 'Parsing code'
shows up. However on the second call to `fmt_ast` this doesn't show up anymore.
This enables us to create a struct with internal lifetime that doesn't leak
outside, is cheap to create and only does the parsing work only once when
needed.