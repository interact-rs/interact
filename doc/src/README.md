# Interact

Interact is a Rust framework for friendly online introspection of the running program state in an intuitive command-line *interact*ive way.

Interact is useful for server programs that otherwise receive no input. You can use Interact to make your server receive commands using the special prompt from the `interact_prompt` crate. The commands can be used to browse your server's internal state, modify it, and call method functions that were specified in `interact` derive attributes.

## Design

Using two traits, `Access` and `Deser`, Interact exposes types as trait objects, similarly to the `Any` trait, but with a functionality of reflection. The `Access` trait allows to probe the fields of structs and enums and modify them. It also allows to iterate arrays, vectors, and maps. It also allows to safely punch through `Rc`, `Arc`, and  `Mutex`. The traits can be derived using `#[derive(Interact)]`.

At the prompt side, predictive parsing is used for providing full auto-complete and hinting, while constructing access paths, and while constructing values used in field assignments and function calls.
