# PROJECT KNOWLEDGE BASE

**Generated:** 2026-06-23  
**Commit:** 99a51d3  
**Branch:** master

## OVERVIEW
`vhost` is a Rust verification-host library that bridges `mailbox_rs`, SystemVerilog DPI/UVM code, optional Python callbacks, and localhost socket helpers. It is library/FFI infrastructure, not a CLI app.

## STRUCTURE
```text
vhost/
|-- src/             # Rust crate: public API, memory model, RPC bridge, socket ABI
|-- python/         # Source Python callback shims copied by build.rs
|-- sv/mailbox_sv/  # Source SystemVerilog mailbox/DPI runtime copied by build.rs
|-- build.rs        # Stages python/ and sv/ into Cargo OUT_DIR
|-- Cargo.toml      # Feature matrix and git dependencies
`-- target/         # Generated Cargo output; never edit as source
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Public Rust API / boot flow | `src/lib.rs` | `mailbox_init`, `mb_server_run`, `__mb_call` |
| Memory model / parser | `src/mem/common.rs` | Core hotspot; YAML to `DPIShareMem` |
| Memory backend selection | `src/mem/mod.rs` | `mem_uvm`, `mem_api`, `mem_static`, else default |
| Python callback bridge | `src/rpcs/py_calls.rs`, `python/*/pymb_rpc.py` | Feature-gated through PyO3 |
| Socket ABI | `src/sockets/mod.rs` | Exported `c_skt_*` functions |
| SV mailbox runtime | `sv/mailbox_sv/mailbox.sv` | DPI imports/exports and state machine |
| Build asset staging | `build.rs` | Copies `python/` and `sv/` into `target/.../out/vhost` |

## CODE MAP
| Symbol | Type | Location | Refs | Role |
|--------|------|----------|------|------|
| `mailbox_init` | function | `src/lib.rs` | unmeasured | Builds memory spaces/channels from env-config files |
| `__mb_call` | `extern "C"` | `src/lib.rs` | unmeasured | Routes mailbox calls to Python first, SV fallback |
| `mb_server_run` | function | `src/lib.rs` | unmeasured | Returns wake + serve futures for mailbox server |
| `DPIShareMemParser` | parser | `src/mem/common.rs` | unmeasured | Parses YAML memory definitions |
| `DPIShareMem` | enum | `src/mem/common.rs` | unmeasured | Dispatches `BlackBox`, `Direct`, `Banked` memory |
| `DPIBankedShareMem` | struct | `src/mem/common.rs` | unmeasured | Bank geometry and address mapping |
| `StaticMemSink` | trait | `src/mem/static_mem.rs` | unmeasured | Downstream-defined static memory output sink |
| `poll_py_call` | function | `src/rpcs/py_calls.rs` | unmeasured | Calls Python `pymb_rpcs.poll` under GIL |
| `SktWrapper` | struct | `src/sockets/mod.rs` | unmeasured | TCP listener and 1-based client streams |
| `Mailbox` | SV class | `sv/mailbox_sv/mailbox.sv` | unmeasured | SV-side channel/method registry and polling |

`rust-analyzer` was unavailable in this environment and no `codegraph_*` tools were registered, so symbol references are intentionally marked unmeasured.

## CONVENTIONS
- Requires nightly Rust: `src/lib.rs` uses `#![feature(io_error_more)]` and `#![feature(map_try_insert)]`.
- Runtime env contract: `MEM_CFG_FILE`, `MAILBOX_CFG_FILE`, `MAILBOX_FS_ROOT` must exist before `mailbox_init`.
- Cargo features define behavior: `python`, `mem_uvm`, `mem_api`, `mem_static`, `ptr32`, `ptr64`, `cache_line_{32,64,128,256}`.
- Memory backend features still require an explicit mailbox pointer-width feature such as `ptr32` or `ptr64`; `mem_static` does not select one implicitly.
- `python/` and `sv/` are source inputs even though they are copied into `target/` during builds.
- Memory tests are inline Rust unit tests in `src/mem/common.rs` and feature-gated static backend tests in `src/mem/static_mem_tests.rs`; no separate `tests/` tree exists.

## ANTI-PATTERNS (THIS PROJECT)
- Do not edit `target/debug/build/**`, `target/debug/.fingerprint/**`, or `.omo/**` as source.
- Do not treat generated `target/.../out/vhost/**` copies as authoritative; edit `python/` or `sv/mailbox_sv/` instead.
- Do not assume `src/rpcs` is active unless PyO3/Python features are enabled.
- Do not add generic Python packaging assumptions; there is no `pyproject.toml`, `setup.py`, or package scaffold.

## UNIQUE STYLES
- FFI boundaries use `#[no_mangle] extern "C"` in Rust and DPI imports/exports in SV.
- Parser errors are `Result<_, String>`; socket errors use `anyhow`.
- Several unsupported feature/backend combinations intentionally `panic!`; do not replace those with broad fallback paths without changing the contract.
- The default memory backend logs sync/flush activity with `println!` rather than performing simulator I/O.
- `mem_static` buffers writes per HDL memory instance and flushes through downstream `StaticMemSink`; it does not choose hex/bin/file layout.

## COMMANDS
```bash
cargo build
cargo test
cargo build --features python
cargo test --features mem_uvm
cargo test --features mem_api
cargo test --features "mem_static ptr64"
```

## NOTES
- `Cargo.lock` exists but `.gitignore` also lists it; treat this as a library-style crate unless policy changes.
- `build.rs` uses `v_build_utils::OtherDir`; adding assets under `python/` or `sv/` affects Cargo build output.
- `py_calls_init()` only warns when optional `mbpy_calls_init.mbpy_calls_init()` is missing.
- `src/mem/common.rs` is the main maintainability hotspot: 1k+ lines, parser, geometry, buffer math, and tests in one file.
