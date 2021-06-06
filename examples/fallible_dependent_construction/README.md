# `fallible_dependent_construction` Example

Most of the code is setup to showcase a motivated use case. The `self_cell`
relevant parts is using the `try_new` constructor.

Run this example with `cargo run`, it should output:

```
'this is good' -> Ok(NameCell { owner: "this is good", dependent: Names([Name("this"), Name("is"), Name("good")]) })
'this is bad' -> Err(Banned)
```