# Attributes

## Container attributes

Following `#[derive(Interact)]`, methods to be called from the prompt can be
specified by name, along with their parameters, and whether they take in `&self`
or `&mut self`.

```rust,ignore
#[interact(mut_fn(function_name(param_a, param_b)))
```

```rust,ignore
#[interact(immut_fn(function_name(param_a, param_b)))
```

For example:

```rust
#[derive(Interact)]
#[interact(mut_fn(add(param_a)))]
struct Baz(u32);

impl Baz {
	fn add(&mut self, param_a: u32) {
        self.baz += param_a;
    }
}
```

## Field attributes

The `skip` attribute allows to make some fields invisible:
```rust,ignore
#[interact(skip))
```

The downside is that having any skipped field on a type means that it is
unbuildable, and therefore cannot be passed as value to functions or to be
assigned using `=` in an expression.
