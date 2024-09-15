# `mut_ref_to_owner_in_builder` Example

This example shows how to handle dependent types that want to reference the
owner by `&mut`. This works by using the wrapper type `MutBorrow` around the
owner type. This allows us to call `borrow_mut` in the builder function. This
example also shows how to recover the owner value if desired.

Run this example with `cargo run`, it should output:

```
dependent before pop: abc
dependent after pop: ab
recovered owner: ab
```