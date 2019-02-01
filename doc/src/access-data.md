# Accessing data

## Whole state

Suppose a value of the following simple state is registered:

```rust
#[derive(Interact)]
struct Point {
	x: u32,
	y: u32,
}
```

The whole of it can be printed, and the result is similar to a pretty printed `Debug`:

```shell
>>> state
Point { x: 3, y: 4 }
```

Tuple structs are accessed similarly using Rust's `.0`, `.1`, etc.

## Field access

The syntax for field access is similar to Rust's. For example, accessing one of the fields of the previous example:

```shell
>>> state.x
3
```

## Enum access

```rust
struct OptPoint {
	x: Option<u32>,
	y: Option<u32>,
}
```

Suppose that we have an instance of this struct with the following value:

`OptPoint { x: None, y: Some(3) }`

Unlike in Rust, we can have a full path to the variant's value through variant's name:

```shell
>>> state.y.Some
(3)
>>> state.y.Some.0
3
>>> state.x
None
>>> basic.x.None
None
```

## Vec, HashMap, and BTreeMap access

Accessing vectors and maps are done like you'd expected via `[]`. Currently, ranges are _not_ supported in vectors and sorted maps.

## Access via `Mutex`, `Rc`, `Arc`, `RefCell`, `Box`

Interact elides complexity to access paths when wrapper types are used. For Mutex, it uses `.try_lock()` behind the scenes. For `RefCell` it uses `try_borrow()`.
