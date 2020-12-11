# OnceSelfCell

`once_self_cell` provides two new cell-like types, `unsync::OnceSelfCell` and `sync::OnceSelfCell`. `OnceSelfCell` might store arbitrary non-Copy types,
can be assigned to at most once and provide direct access to the stored contents,
**and** can store a dependent value, with a lifetime depending on the owner.
This enables safe to use macro free self referential structs in stable Rust,
without leaking the struct internal lifetime. In a nutshell,
the API looks *roughly* like this:

```rust
impl OnceSelfCell<Owner> {
    fn new(owner: Owner) -> OnceSelfCell<Owner> { ... }
    fn get_owner<'a>(&'a self) -> &'a Owner { ... }
    fn get_or_init_dependent<'a, Dependent>(
        &'a self,
        make_dependent: impl FnOnce(&'a Owner) -> Dependent,
    ) -> &'a Dependent { ... }
}
```

Self referential structs are currently not supported with safe vanilla Rust.
The only reasonable safe alternative is to expect the user to juggle 2 separate
data structures which is a mess. The library solution rental is both no longer
maintained and really heavy to compile due to its use of procedural macros.

This alternative is `no-std`, uses no macros, a total of ~10 lines unsafe and works on stable Rust, and is miri tested. With a total of less than 200 lines of implementation code, which consists mostly of type and trait wrangling, this crate aims to be a good minimal solution to the problem of self referential structs.

A motivating use case:

```rust
use once_self_cell::unsync::OnceSelfCell;

#[derive(Debug, Eq, PartialEq)]
struct Ast<'a>(pub Vec<&'a str>);

fn ast_from_string<'a>(owner: &'a String) -> Ast<'a> {
    Ast(vec![&owner[2..5], &owner[1..3]])
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct LazyAst {
    ast_cell: OnceSelfCell<String>,
}

impl LazyAst {
    fn new(body: String) -> Self {
        LazyAst {
            ast_cell: OnceSelfCell::<String>::new(body),
        }
    }

    fn get_body<'a>(&'a self) -> &'a String {
        self.ast_cell.get_owner()
    }

    fn get_ast<'a>(&'a self) -> &'a Ast<'a> {
        self.ast_cell.get_or_init_dependent(ast_from_string)
    }
}
```

A node data structure derived from some input for example:

```rust
struct Ast<'a>(pub Vec<&'a str>);
```

Classically if for example you to want to read the input files from the filesystem
and then parse them lazy as needed, the caller would have to first call the
filesystem read so that they can own the input `String`, and then lookup the
Ast in a hash map, and initialize it as needed, the hash map and is lifetime
bound to all input files, so they need to be in a container again, and then
this needs to be juggled in every place its used. That's neither easy to use
nor efficient.

A more natural programming approach would be something like this:

```rust
#[derive(Debug, Clone, Eq, PartialEq)]
struct LazyAst {
    body: String,
    ast: OnceCell<Ast<'body>>,
}
```

Putting the things that belong together, together. Yet that's currently not
possible in Rust. `OnceSelfCell` aims to fill this gap.

With `OnceSelfCell` the above becomes:

```rust
#[derive(Debug, Clone, Eq, PartialEq)]
struct LazyAst {
    ast_cell: OnceSelfCell<String>,
}
```

Notice, that `LasyAst` is free of lifetime annotations, and can be safely used
like any other struct.

Wait, but where did Ast go?

Behind the scenes `OnceSelfCell` performs type erasure, which allows it to be
initialized once dynamically.

```rust
impl LazyAst {
    fn new(body: String) -> Self {
        LazyAst {
            ast_cell: OnceSelfCell::<String>::new(body),
        }
    }

    fn get_body<'a>(&'a self) -> &'a String {
        self.ast_cell.get_owner()
    }

    fn get_ast<'a>(&'a self) -> &'a Ast<'a> {
        self.ast_cell.get_or_init_dependent(ast_from_string)
    }
}

fn ast_from_string<'a>(owner: &'a String) -> Ast<'a> {
    Ast(vec![&owner[2..5], &owner[1..3]])
}
```

The key is `get_or_init_dependent` uses the type described in `ast_from_string`
which contains a valid lifetime for `Ast`, to construct, read and drop the
dependent value.

### Installing

[See cargo docs](https://doc.rust-lang.org/cargo/guide/).

## Running the tests

```
cargo test

cargo miri test
```

### Related crates

[once_cell](https://github.com/matklad/once_cell)

[rental](https://github.com/jpernst/rental)

## Contributing

Please respect the [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) when contributing.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available,
see the [tags on this repository](https://github.com/Voultapher/once_self_cell/tags).

## Authors

* **Lukas Bergdoll** - *Initial work* - [Voultapher](https://github.com/Voultapher)

See also the list of [contributors](https://github.com/Voultapher/once_self_cell/contributors)
who participated in this project.

## License

This project is licensed under the Apache License, Version 2.0 -
see the [LICENSE.md](LICENSE.md) file for details.
