# Modifying data

Types that expose a mutable interface, for example via `Arc<Mutex<_>>`, can have their fields be assigned and modified from the Interact prompt.

Interact knows the basic types, and is also able to construct values of derived types for which the `#[interact(skip)]` attribute was _not_ used for any field.

## Assignments

Assignments are done using `=` at the prompt.

For example, check `cargo run --example large-example`:

```rust,ignore
>>> complex.tuple
((690498389, VarUnit, (193, 38)), 1262478744)

>>> complex.tuple.0.2
(193, 38)

>>> complex.tuple.0.2 = (1, 1)
>>> complex.tuple
((690498389, VarUnit, (1, 1)), 1262478744)

>>> complex.tuple.0.1 = VarNamed { a: 3, b: 10}
>>> complex.tuple
((690498389, VarNamed { a: 3, b: 10 }, (1, 1)), 1262478744)
```

## Wrapper types

The wrapper types `Rc`, `RefCell`, `Mutex`, `Box` are transparent to construction of values, and need not be specified.

```rust,ignore
>>> complex.boxed = VarNamed { a: 3, b: 10}

>>> complex.boxed
VarNamed { a: 3, b: 10 }
```
