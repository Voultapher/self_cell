# Benchmarks for `self_cell`

To run time based criterion benchmarks:

```
cargo bench --bench time
```

To run instruction based iai benchmarks:

```
cargo bench --bench instructions
```

Interpret the iai cycle and instruction count with care, as they are volatile especially for the smaller benchmarks.