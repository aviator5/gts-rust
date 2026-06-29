# gts-dylint

A [Dylint](https://github.com/trailofbits/dylint) lint that flags hard-coded GTS identifier prefixes in string literals.

## What it does

String literals starting with a configured prefix (default: `"gts."`) are flagged with a warning. These should use the `GTS_ID_PREFIX` constant or the `gts_id!` macro instead, so that the prefix remains overridable at **compile time** via the `GTS_ID_PREFIX` environment variable.

The set of flagged prefixes can be customized via the `GTS_DYLINT_PREFIXES` environment variable (comma-separated, e.g. `GTS_DYLINT_PREFIXES="gts.,acme."`). Defaults to `gts.`.

### Suggested replacements

| Pattern | Replacement |
|---------|-------------|
| `"gts.x.core.events.topic.v1~"` | `GTS_ID_PREFIX` compile-time constant from the `gts-id` crate |
| Constructing GTS IDs at compile time | `gts_id!` macro from the `gts-macros` crate |

### Suppressing

Use `#[allow(gts_id_hardcoded_prefix)]` on specific items or `#![allow(gts_id_hardcoded_prefix)]` at the crate level. Since the lint is only known when dylint is loaded, pair it with `#[allow(unknown_lints)]` to avoid "unknown lint" warnings during normal `cargo check`:

```rust
#[allow(unknown_lints, gts_id_hardcoded_prefix)]
pub const DEFAULT_GTS_ID_PREFIX: &str = "gts.";
```

For test code, add at the crate root:

```rust
#![cfg_attr(test, allow(unknown_lints, gts_id_hardcoded_prefix))]
```

## Examples

### `gts_id!` as a standalone expression

Expands to `concat!(GTS_ID_PREFIX, "suffix")` — a `&'static str` with the configured prefix prepended at compile time:

```rust
use gts_macros::gts_id;

// With the default prefix "gts.":
let id: &str = gts_id!("x.core.events.topic.v1~");
assert_eq!(id, "gts.x.core.events.topic.v1~");
```

### `gts_id!` inside `gts_instance!`

The `gts_id!("...")` marker is recognized inside `gts_instance!` — write the suffix without the prefix:

```rust
use gts_macros::{gts_id, gts_instance};

let t: TopicV1 = gts_instance!(TopicV1 {
    id: gts_id!("x.core.events.topic.v1~vendor.app.orders.created.v1"),
    name: "orders".to_owned(),
    retention: "P30D".to_owned(),
});
```

### `gts_id!` inside `#[struct_to_gts_schema]`

The same marker works in the `type_id` argument of `#[struct_to_gts_schema]`:

```rust
use gts_macros::{struct_to_gts_schema, gts_id};

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    type_id = gts_id!("x.core.events.topic.v1~"),
    description = "Topic type",
    properties = "id,name"
)]
pub struct TopicV1 {
    pub id: gts::GtsInstanceId,
    pub name: String,
}
```

### `gts_id!` inside `gts_instance_raw!`

For JSON-shaped instances without a Rust struct:

```rust
use gts_macros::{gts_id, gts_instance_raw};

let v: serde_json::Value = gts_instance_raw!({
    "id": gts_id!("x.core.events.topic.v1~vendor.app.events.audit.v1"),
    "name": "audit",
});
```

## Requirements

- **Nightly Rust** with `rustc-dev` and `llvm-tools-preview` components:
  ```bash
  rustup toolchain install nightly
  rustup component add rustc-dev llvm-tools-preview --toolchain nightly
  ```

- **cargo-dylint** and **dylint-link**:
  ```bash
  cargo install cargo-dylint dylint-link
  ```

## Usage

### In a project that depends on gts-rust

Add to your workspace `Cargo.toml`:

```toml
[workspace.metadata.dylint]
libraries = [
    { name = "gts-dylint" },
]
```

Add `gts-dylint` as a dependency:

```bash
cargo add gts-dylint
```

Run the lint:

```bash
cargo +nightly dylint --all
```

To also lint test code, examples, and benchmarks:

```bash
cargo +nightly dylint --all -- --all-targets
```

### In this repository

```bash
make dylint
```

## Testing

UI tests use [`dylint_testing`](https://docs.rs/dylint_testing):

```bash
cd gts-dylint && cargo +nightly test
```

## License

Same as the gts-rust project.
