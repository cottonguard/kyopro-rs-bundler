# kyopro-rs-bundler

from:
```rust
// lib.rs
mod a;
```

```rust
// a.rs
fn f() {}
```

to:
```rust
mod a {
    fn f() {}
}
```
