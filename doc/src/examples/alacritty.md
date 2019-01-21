# Alacritty

By enabling Interact for a program such as Alacritty, we can probe and modify its state while it runs (for example, the cursor's position).

### Summary of changes

The [changes in Alacritty](https://github.com/interact-rs/alacritty/compare/base...interact-rs:interact-demo) do the following:

* Add an invocation of the Interact prompt.
* Add `#[derive(Interact)]` for a small portion of the types.
* Add special `Access` and `Deser` deriving for the `FairMutex` type.

## Demo

Here is the interactive state it produces:

```rust.ignore
>>> term
Term {
    grid: Grid {
        cols: Column (80),
        lines: Line (24),
        display_offset: 0,
        scroll_limit: 0,
        max_scroll_limit: 100000
    },
    input_needs_wrap: false,
    next_title: None,
    next_mouse_cursor: None,
    alt_grid: Grid {
        cols: Column (80),
        lines: Line (24),
        display_offset: 0,
        scroll_limit: 0,
        max_scroll_limit: 0
    },
    alt: false,
    cursor: Cursor {
        point: Point { line: Line (5), col: Column (45) },
        template: Cell { c: ' ' },
        charsets: Charsets ([ Ascii, Ascii, Ascii, Ascii ])
    },
    dirty: false,
    next_is_urgent: None,
    cursor_save: Cursor {
        point: Point { line: Line (0), col: Column (0) },
        template: Cell { c: ' ' },
        charsets: Charsets ([ Ascii, Ascii, Ascii, Ascii ])
    },
    cursor_save_alt: Cursor {
        point: Point { line: Line (0), col: Column (0) },
        template: Cell { c: ' ' },
        charsets: Charsets ([ Ascii, Ascii, Ascii, Ascii ])
    },
    semantic_escape_chars: ",│`|:\"\' ()[]{}<>",
    dynamic_title: true,
    tabspaces: 8,
    auto_scroll: false
}
```
