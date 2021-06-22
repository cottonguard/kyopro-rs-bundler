# kyopro-rs-bundler

```
$ rs_bundler | rustfmt
```

from:
```rust
// src/lib.rs
mod a;
```

```rust
// src/a.rs
fn f() {}
```

to:
```rust
mod a {
    fn f() {}
}
```
