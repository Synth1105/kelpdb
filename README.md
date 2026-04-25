# kelpdb

`kelpdb` is a small in-memory database backed by a `ketheler` agent.
The database state lives behind a green-thread server, so cloned `DB`
handles share the same row storage.

## Features

- Stores row data inside a `ketheler::agent` state instead of a local `HashMap`
- Accepts mixed value types through `WriteValue`
- Reads values back either as typed collections with `get_by_type` or as strings with `get_display`
- Ships with a simple REPL binary and the optional `Scuver` command runner

## Installation

```toml
[dependencies]
kelpdb = "1.2.0"
```

## Basic Usage

```rust
use kelpdb::prelude::*;

fn main() {
    let mut db = DB::new("user", String::from("John"));

    db.set("user", 25i32);
    db.set("user", 180.5f64);
    db.set("posts", "hello");
    db.set("posts", "world");

    assert_eq!(db.get_by_type::<String>("user"), vec![String::from("John")]);
    assert_eq!(db.get_by_type::<i32>("user"), vec![25]);
    assert_eq!(db.get_by_type::<f64>("user"), vec![180.5]);
    assert_eq!(db.get_display("posts"), vec!["hello", "world"]);
}
```

## Shared Agent State

Every `DB` instance owns a handle to the same underlying agent state.
Cloning a `DB` clones the handle, not the stored rows.

```rust
use kelpdb::prelude::*;

fn main() {
    let db = DB::new("user", "John");
    let mut clone = db.clone();

    clone.set("user", 25i32);

    assert_eq!(db.get_by_type::<i32>("user"), vec![25]);
}
```

## REPL

Run the bundled REPL with:

```bash
cargo run
```

Supported commands:

- `SET <key> <value>`

- `GET <key>`
- `:exit`
- `:quit`

## Scuver

The optional `scuver` feature exposes a minimal SQL-like language runner:

```rust
use kelpdb::scuver::Scuver;

fn main() {
    let scuver = Scuver::new();

    scuver.run("SET user John".into()).unwrap();
    scuver.run("SET user 25".into()).unwrap();

    let output = scuver.run("GET user".into()).unwrap();
    assert_eq!(output, "John\n25");
}
```

## Notes

- `get` returns raw `Arc<dyn Any + Send + Sync>` values.
- `get_display` only stringifies built-in scalar types and strings.
- `remove` pops the last value in a row. When the row becomes empty, the row itself is removed.
