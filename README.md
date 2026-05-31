# json_parser

A JSON parser written from scratch in Rust with no external dependencies.

## How it works

Two stages:

- **Tokenizer** (`src/tokenize.rs`) walks the input character by character and produces a flat list of tokens. Handles strings, escape sequences, unicode (`\uXXXX`), numbers, scientific notation, and the structural punctuation.
- **Parser** (`src/parse.rs`) takes the token list and builds a typed value tree via recursive descent. Enforces strict JSON rules (no trailing commas, string-only keys, etc).

## Types

```rust
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}
```

## Usage

```rust
use json_parser::{from_str, Value};

let json = r#"{"name": "json_parser", "version": 1.0}"#;
let parsed = from_str(json).unwrap();
```

## Example

```bash
cargo run --example read_file -- examples/data.json
```

`examples/data.json` tests a range of edge cases empty objects and arrays, deep nesting, mixed types, escapes, unicode, and scientific notation.

## Tests

```bash
cargo test
```

Unit tests live alongside the code in each module. Integration tests for the full pipeline are in `src/lib.rs`.

## Limitations

- No surrogate pair support (`\uD800`–`\uDFFF`)
- Object insertion order is not preserved
- Numbers are `f64` large integers may lose precision
