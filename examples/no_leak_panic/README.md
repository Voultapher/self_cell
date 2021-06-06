# `no_leak_panic` Example

Run this example with `cargo run`, this should output:

```
thread 'main' panicked at 'Oh noes, this is impossible', no_leak_panic/src/main.rs:14:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
PanicCtor::new panic -> Oh noes, this is impossible
But we keep going
```

Even though a panic occurred during `PanicCtor::new`, we are sure no
memory was leaked by `NoLeakCell`. You can confirm this by running `cargo miri run`.

You could either catch the panic as Error like in the example, or propagate the panic further but ensure no memory is leaked, like so:

```rust
let cell = match NoLeakCell::try_new(owner, |owner| {
    std::panic::catch_unwind(|| PanicCtor::new(owner))
}) {
    Ok(val) => val,
    Err(err) => {
        std::panic::panic_any(err);
    }
};
```