# Using derive

## Cargo.toml

The Interact dependency is needed:

```toml
[dependencies]
interact = "0.3"
```

## Source

* The a crate's top level, `extern crate interact` is needed.
* At places `#[derive(Interact)]` is needed, import the needed proc macro: `use interact::Interact;`

An example:

```rust
extern crate interact;

use interact::Interact;

#[derive(Interact)]
struct Point {
    x: i32,
    y: i32,
}
```
