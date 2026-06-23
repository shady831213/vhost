# vhost

`vhost` is a Rust verification-host library for co-simulation flows. It bridges
`mailbox_rs`, SystemVerilog DPI/UVM code, optional Python callbacks, and helper
localhost sockets. It is library and FFI infrastructure, not a standalone CLI.

## What It Provides

- Mailbox bootstrapping through `mailbox_init`.
- Shared-memory parsing from YAML into direct, banked, or black-box memory models.
- Feature-selected memory backends for default logging, UVM HDL access, C memory API, or static export.
- Optional Python callback dispatch before falling back to SystemVerilog DPI calls.
- TCP socket FFI helpers exported as `c_skt_*` functions.
- Build-time staging of `python/` and `sv/` assets into Cargo `OUT_DIR`.

## Repository Layout

```text
vhost/
|-- src/             # Rust library, memory model, RPC bridge, socket ABI
|-- python/          # Python callback shims copied by build.rs
|-- sv/mailbox_sv/   # SystemVerilog mailbox/DPI runtime copied by build.rs
|-- build.rs         # Stages python/ and sv/ into Cargo OUT_DIR
|-- Cargo.toml       # Feature matrix and git dependencies
`-- README.md
```

Important Rust entry points:

- `src/lib.rs`: `mailbox_init`, `mb_server_run`, `__mb_call`.
- `src/mem/common.rs`: YAML parser and memory geometry.
- `src/mem/mod.rs`: memory backend feature selection.
- `src/mem/static_mem.rs`: static-memory export API.
- `src/rpcs/py_calls.rs`: optional PyO3 callback bridge.
- `src/sockets/mod.rs`: exported socket ABI.

## Toolchain

This crate requires nightly Rust. `src/lib.rs` enables nightly features:

```rust
#![feature(io_error_more)]
#![feature(map_try_insert)]
```

Use a command prefix such as `cargo +nightly ...` unless your default toolchain is
already nightly.

## Cargo Features

Pointer width is explicit. Any build that enables `mailbox_rs` must also choose
one of:

- `ptr32`
- `ptr64`

Memory backend features are mutually exclusive:

| Feature | Purpose |
| --- | --- |
| default, no memory backend feature | Diagnostic backend that logs sync/flush behavior. |
| `mem_uvm` | UVM HDL/DPI backend using `uvm_hdl_read` and `uvm_hdl_deposit`. |
| `mem_api` | C API backend for black-box memory via `__vhost_bb_mem_write/read`. |
| `mem_static` | In-process static buffers flushed through downstream `StaticMemSink`. |

Other features:

| Feature | Purpose |
| --- | --- |
| `python` | Enables PyO3 callback dispatch. |
| `cache_line_32`, `cache_line_64`, `cache_line_128`, `cache_line_256` | Forward cache-line selection to `mailbox_rs`. |

Examples:

```bash
cargo +nightly test --features ptr64
cargo +nightly test --features "mem_uvm ptr64"
cargo +nightly check --features "mem_api ptr64"
cargo +nightly test --features "mem_static ptr64"
cargo +nightly test --features "mem_static ptr32"
```

`cargo test` without `ptr32` or `ptr64` is not a valid configuration for this
crate because `mailbox_rs` does not define `MBPtrT` without a pointer-width
feature.

## Runtime Configuration

`mailbox_init` expects these environment variables to point at existing runtime
configuration files or directories:

| Variable | Meaning |
| --- | --- |
| `MEM_CFG_FILE` | YAML memory definition file consumed by `DPIShareMemParser`. |
| `MAILBOX_CFG_FILE` | Mailbox channel configuration consumed by `mailbox_rs`. |
| `MAILBOX_FS_ROOT` | Filesystem root used when building mailbox file services. |

Memory YAML supports direct memories, banked memories, optional `array_dims`, and
optional initialization metadata. Widths in YAML are bits; internal memory widths
are bytes.

## Static Memory Export

Enable `mem_static` when a downstream tool wants memory contents as raw static
buffers instead of simulator I/O. The backend registers one buffer per HDL memory
instance, identified by `(path, width, depth)`, and delegates serialization to the
caller.

```rust
use vhost::mem::{flush_static_mems, StaticMemDescriptor, StaticMemSink};

struct Sink;

impl StaticMemSink for Sink {
    fn write_static_mem(
        &mut self,
        descriptor: &StaticMemDescriptor,
        data: &[u8],
    ) -> Result<(), String> {
        println!(
            "{}: width={} depth={} bytes={}",
            descriptor.path,
            descriptor.width,
            descriptor.depth,
            data.len()
        );
        Ok(())
    }
}

fn export_static_mems() -> Result<(), String> {
    let mut sink = Sink;
    flush_static_mems(&mut sink)
}
```

`mem_static` intentionally does not choose a file layout, filename convention,
hex/bin format, or output directory. `BlackBox` memories are rejected by the
parser under `mem_static`; static export requires HDL instance paths.

## Build Assets

`build.rs` copies source assets from `python/` and `sv/` into Cargo output:

```rust
OtherDir::new("python").unwrap().add_dir("python").unwrap();
OtherDir::new("sv").unwrap().add_dir("sv").unwrap();
```

Edit the source directories, not generated files under `target/`.

## Validation

Useful local checks:

```bash
cargo +nightly fmt --all -- --check
cargo +nightly test --features ptr64
cargo +nightly test --features "mem_static ptr64"
cargo +nightly test --features "mem_static ptr32"
cargo +nightly check --features "mem_api ptr64"
cargo +nightly check --features "mem_uvm ptr64"
```

Known baseline warnings currently include unused nightly feature warnings for
`io_error_more` and `map_try_insert`.
