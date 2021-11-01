# `owner_with_lifetime` Example

Here we want to optionally escape an input string and then compute an AST based
on this escaped string. This means we either borrow the 'input lifetime
reference or the function stack local owned string produced as result of
escaping the input. This can only be done with a self-referential struct, that
still is statically lifetime dependent on the input.

Run this example with `cargo run`, it should output:

```
input: "au bx"
not_escaped.borrow_dependent() -> ["au", "bx"]
input: "au bz"
escaping input (owned alloc)
escaped.borrow_dependent() -> ["az", "bz"]
```

Notice for the first input that contains an 'x' no escaping is done, so the
dummy AST is a Vec of pointers into the original input. The second time with
input "au bz" there is no 'x' so we need want to escape the input and build the
AST based of that as seen "az" instead of "au" in the input.