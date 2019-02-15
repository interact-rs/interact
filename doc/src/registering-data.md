# Registering data

The `interact_prompt` crate provides helpers for registering data to be examined. Currently, data must be owned, so if it is shared by threads running in the background in parallel to the prompt, it needs to be provided using `Arc<_>`. If the data is expeced to be shared _and_ mutable by the Interact prompt, then it should be wrapped in `Arc<Mutex<_>>`.

For example:

```rust,ignore
use interact_prompt::{SendRegistry};

fn register_global_mutable_data(global_state: Arc<Mutex<MyData>>) {
	SendRegistry::insert("global", Box::new(global_state));
}

fn register_global_readable_data(readonly: Arc<MyData>) {
	SendRegistry::insert("readonly", Box::new(readonly));
}
```
