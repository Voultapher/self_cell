[<img alt="github" src="https://img.shields.io/badge/github-once__self__cell-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/Voultapher/once_self_cell)
[<img alt="crates.io" src="https://img.shields.io/badge/dynamic/json?color=fc8d62&label=crates.io&query=%24.crate.max_version&url=https%3A%2F%2Fcrates.io%2Fapi%2Fv1%2Fcrates%2Fonce_self_cell&style=for-the-badge&logo=rust" height="20">](https://crates.io/crates/once_self_cell)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-once__self__cell-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/once_self_cell)

# OnceSelfCell

`once_self_cell` provides two macros, `unsync_once_self_cell` and `sync_once_self_cell`. These generate a struct that can be assigned to at most once and provide direct reference access to the stored contents,
**and** can store a dependent value, with a lifetime depending on the owner.
This enables safe-to-use proc-macro-free self-referential structs in stable Rust,
without leaking the struct internal lifetime. In a nutshell,
the API looks *roughly* like this:

```rust
unsync_once_self_cell!(NewStructName, Owner, Dependent<'_>, derive(Clone));

// Generated.
impl NewStructName {
    fn new(owner: Owner) -> NewStructName { ... }
    fn get_owner<'a>(&'a self) -> &'a Owner { ... }
    fn get_or_init_dependent<'a>(&'a self) -> &'a Dependent<'a> { ... }
}
```

Self-referential structs are currently not supported with safe vanilla Rust.
The only reasonable safe alternative is to expect the user to juggle 2 separate
data structures which is a mess. The library solution rental is both no longer
maintained and really heavy to compile due to its use of procedural macros.

This alternative is `no-std`, uses no proc-macros, a total of ~20 lines unsafe and works on stable Rust, and is miri tested. With a total of less than 300 lines of implementation code, which consists mostly of type and trait wrangling, this crate aims to be a good minimal solution to the problem of self-referential structs.

### Fast compile times

```
$ rm -rf target && cargo +nightly build -Z timings

Compiling once_cell v1.5.2
Compiling once_self_cell v0.6.0
Completed once_cell v1.5.2 in 0.3s
Completed once_self_cell v0.5.0 in 0.3s
```

Measurements done a slow and old laptop.

Because it does **not** use proc-macros, compile-times are fast.

### A motivating use case

```rust
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
    derive(Clone, Debug, Eq, PartialEq) // optional
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

With `once_self_cell` the above becomes:

```rust
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
```

Notice, that `LasyAst` is free of lifetime annotations, and can be safely used
like any other struct.

What is this `'_` lifetime? This is a sort of higher kinded lifetime, that
with the help of a macro becomes the appropriate lifetime in the place of
use, in this case `get_or_init_dependent` and `drop`.

Behind the scenes type and lifetime erasure is performed.
which allows it to be initialized once dynamically.

For this Dependent has to implement `From<&Owner>` or `Into<Dependent<'a>`
has to be implemented.

```rust
#[derive(Debug, Eq, PartialEq)]
struct Ast<'input>(pub Vec<&'input str>);

impl<'a> From<&'a String> for Ast<'a> {
    fn from(body: &'a String) -> Self {
        Ast(vec![&body[2..5], &body[1..3]])
    }
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
```

### Installing

[See cargo docs](https://doc.rust-lang.org/cargo/guide/).

## Running the tests

```
cargo test

cargo miri test
```

### Related projects

[once_cell](https://github.com/matklad/once_cell)

[rental](https://github.com/jpernst/rental)

[Schroedinger](https://github.com/dureuill/sc)

[owning_ref](https://github.com/Kimundi/owning-ref-rs)

[ouroboros](https://github.com/joshua-maros/ouroboros)

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
