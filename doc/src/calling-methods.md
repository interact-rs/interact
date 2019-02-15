# Calling methods

By specifying Interact's special `mut_fn` or `immut_fn` [container attributes](attributes.html#container-attributes), you can add methods that would be reachable from the Interact prompt, upon reaching values that match the types on which the special methods are defined.

For example, given the following type:

```rust,ignore
#[derive(Interact)]
#[interact(mut_fn(add(param_a)))]
struct Baz(u32);

impl Baz {
    fn add(&mut self, param_a: u32) {
        self.0 += param_a;
    }
}
```

We can call the `add` methods:

```rust,ignore
>>> state.baz_val
Baz (1)

>>> state.baz_val.add(3)
>>> state.baz_val
Baz (4)
```
