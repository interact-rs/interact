# Actix

To get a taste of Interact as applied to actual servers, you can try the Interact-enable Actix chat demo (originally from [here](https://github.com/actix/actix/tree/master/examples/chat)).

While the state of an Actix program is spread across a stack of Futures that may exist in multiple process thread, Interact has no difficulty in traversing it and presenting a whole picture.

### Summary of changes

To enable this example, there were two changes:

* Changes in Actix core ([Github link](https://github.com/interact-rs/actix/compare/interact-rs:base...interact-rs:interact-addr)), that enable Interact for the `Addr<T>` Actor messaging proxy.
* Changes to Actix chat app ([Github link](https://github.com/interact-rs/actix/compare/interact-rs:interact-addr...interact-rs:interact-chat)), which add `#[derive(Interact)]` for its types, and invocation of the Interact prompt.


## Demo

```shell
git clone https://github.com/interact-rs/actix
cd actix/examples/chat
cargo run --bin server
```

Executing the server presents a prompt in a dedicated Interact thread, while the server functionality runs in the process's background:

```shell
Running chat server on 127.0.0.1:12345
Rust `interact`, type '?' for more information
>>>
```

You can examine the server state:

```rust,ignore
>>> server
ChatServer { sessions: HashMap {}, rooms: HashMap { "Main": HashSet {} } }
```

In parallel, run two clients using `cargo run --bin client`, and re-examine the server's state:

```rust,ignore
>>> server
[#1] ChatServer {
    sessions: HashMap {
        8426954607288880898: ChatSession { id: 8426954607288880898,
			addr: [#1], hb: 374.307146ms, room: "Main" },
        9536033526192464616: ChatSession { id: 9536033526192464616,
			addr: [#1], hb: 513.580812ms, room: "Main" }
    },
    rooms: HashMap { "Main": HashSet { 8426954607288880898, 9536033526192464616 } }
}
```

The reason for `#[1]` is the loop that is detected by traversal of ChatSession's `addr`, which loops back into `ChatServer`.

You can use Interact to print only field of rooms:
```rust,ignore
>>> server.rooms
HashMap { "Main": HashSet { 8426954607288880898, 9536033526192464616 } }
```

Or access one of the sessions:

```rust,ignore
>>> server.sessions[8426954607288880898]
[#1] ChatSession {
    id: 8426954607288880898,
    addr: [#2] ChatServer {
        sessions: HashMap {
            8426954607288880898: [#1],
            9536033526192464616: ChatSession { id: 9536033526192464616,
				 addr: [#2], hb: 759.986694ms, room: "Main" }
        },
        rooms: HashMap { "Main": HashSet { 8426954607288880898, 9536033526192464616 } }
    },
    hb: 632.849822ms,
    room: "Main"
}
```

See the 'hb' field get updated:

```rust,ignore
>>> server.sessions[8426954607288880898].hb
716.16972ms
>>> server.sessions[8426954607288880898].hb
10.398845ms
```

Modify the room's name:

```rust,ignore
>>> server.sessions[8426954607288880898].room = "Boo"
```

See that it was indeed modified:

```rust,ignore
>>> server.sessions[8426954607288880898]
[#1] ChatSession {
    id: 8426954607288880898,
    addr: [#2] ChatServer {
        sessions: HashMap {
            8426954607288880898: [#1],
            9536033526192464616: ChatSession { id: 9536033526192464616,
				addr: [#2], hb: 219.435076ms, room: "Main" }
        },
        rooms: HashMap { "Main": HashSet { 8426954607288880898, 9536033526192464616 } }
    },
    hb: 112.667608ms,
    room: "Boo"
}

```
