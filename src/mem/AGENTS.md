# src/mem KNOWLEDGE BASE

## OVERVIEW
Memory subsystem: YAML parser, memory geometry, buffer slicing, and feature-selected backends for default logging, UVM HDL, C mem API, or static memory export.

## STRUCTURE
```text
src/mem/
|-- mod.rs       # feature gate: mem_uvm, mem_api, mem_static, else default
|-- common.rs    # shared types, parser, geometry, inline tests
|-- default.rs   # println-backed non-FFI backend
|-- mem_api.rs   # black-box C API backend
|-- static_mem.rs # static per-instance buffer + downstream sink
|-- static_mem_tests.rs # mem_static backend tests
`-- uvm_mem.rs    # UVM HDL/DPI backend
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| YAML schema / parsing | `common.rs` | `DPIShareMemParser::parse` |
| Direct memory | `common.rs` | `DPIDirectShareMem`, `DPIMemArray` |
| Banked memory | `common.rs` | `DPIBankedShareMem`, `DPIBankedMemHDLBuffers` |
| Backend selection | `mod.rs` | Mutually overlays shared types |
| Local/mock behavior | `default.rs` | Logs sync/flush actions |
| UVM HDL behavior | `uvm_mem.rs` | `uvm_hdl_read`, `uvm_hdl_deposit` |
| C API behavior | `mem_api.rs` | `__vhost_bb_mem_write/read` only for black-box memory |
| Static export | `static_mem.rs` | `StaticMemSink`, `flush_static_mems` |
| Tests | `common.rs`, `static_mem_tests.rs` | Shared geometry/parser tests plus mem_static backend tests |

## CONVENTIONS
- Widths in YAML arrive in bits; internal array/bank widths are bytes via `>> 3`.
- Missing `array_dims` means `rows = 1`, `cols = size / bytes_per_word`.
- Address math uses row/col/bank helpers plus power-of-two masks; preserve those invariants when editing.
- Public read/write dispatch in `DPIShareMem` clamps accesses at memory end before hitting variant implementations.
- `mem_uvm`, `mem_api`, and `mem_static` are mutually exclusive implementations over shared types, not independent models.
- `mem_static` identity is `(HDL path, width, depth)` so reused paths with distinct geometry do not collide.
- `mem_static` does not choose `ptr32` or `ptr64`; test/build commands should include the intended pointer-width feature.

## ANTI-PATTERNS
- Do not bypass `check()` when constructing direct/banked memory; size, dimensions, and bank grid shape must agree.
- Do not call buffer constructors with zero length; many paths compute `offset + len - 1`.
- Do not assume `banks[0][0]` exists unless bank YAML was validated.
- Do not make `mem_api.rs` handle direct/banked memory without replacing its current intentional panics.
- Do not put hex/bin/file naming policy into `mem_static`; downstream crates own serialization.
- Do not expect `BlackBox` to work under `mem_static`; parser rejects memories without HDL `path`.

## TESTS
```bash
cargo test
cargo test --features mem_uvm
cargo test --features mem_api
cargo test --features "mem_static ptr64"
```

Current tests cover direct memory, direct arrays, 1D banked vertical/horizontal layouts, array banked layouts, 2D banked layouts, cross-row/cross-bank reads/writes, mem_static per-instance flushes, and mem_static black-box rejection.

## NOTES
- `common.rs` mixes model types, parser logic, buffer slicing, and tests; keep edits narrow.
- `InitMethod::Hex` expands shell paths and checks file existence at parse time.
- Default backend is diagnostic/logging behavior; UVM and mem API paths are simulator/FFI behavior.
