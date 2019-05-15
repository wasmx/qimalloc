# qimalloc

Quick Incremental (Wasteful) Memory Allocator.

This memory allocator will not release any memory. Its main use case is in short-lived environment, such as [WebAssembly](https://github.com/webassembly) binaries.

# Usage

To use it, add it as a dependency:
```toml
[dependencies]
qimalloc = "0.1"
```

And override the global allocator:
```rust
#[global_allocator]
static ALLOC: qimalloc::QIMalloc = qimalloc::QIMalloc::INIT;
```

## Maintainer(s)

- Sina Mahmoodi
- Alex Beregszaszi

## License

Apache 2.0
