# `no_leak_panic` Example

Run this example with `cargo run`, this should output:

```
thread 'main' panicked at 'Oh noes, this is impossible', no_leak_panic/src/main.rs:14:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
[no_leak_panic/src/main.rs:31] NoLeakCell::try_from(owner).unwrap_err().downcast_ref::<&str>() = Some(
    "Oh noes, this is impossible",
)
[no_leak_panic/src/main.rs:34] "But we keep going" = "But we keep going"
```

Even though a panic occured during `Dependen::from(&Owner)`, we are sure no
memory was leaked by `NoLeakCell`. You can confirm this by running `cargo miri run`.

You could either catch the panic as Error like in the example, or propagate the panic further but ensure no memory is leaked, like so:

```rust
let cell = match NoLeakCell::try_from(owner) {
    Ok(val) => val,
    Err(err) => std::panic::panic_any(err),
};
```