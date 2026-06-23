# src/rpcs KNOWLEDGE BASE

## OVERVIEW
Optional Rust-to-Python RPC bridge used by `src/lib.rs::__mb_call` before falling back to the SystemVerilog mailbox call path.

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Feature gate | `mod.rs` | Exposes `py_calls` only with `feature = "pyo3"` |
| Python polling | `py_calls.rs` | `poll_py_call` |
| Python tick/init helpers | `py_calls.rs` | `py_tick`, `py_calls_init` |
| Dispatch caller | `../lib.rs` | `__mb_call` |
| Python implementations | `../../python/*/pymb_rpc.py` | Runtime-side callback registry |

## CONVENTIONS
- `poll_py_call` receives raw C strings and `MBPtrT[]`, then calls Python `pymb_rpcs.poll` under the GIL.
- Return values map to `MBCallStatus::Ready` or `MBCallStatus::Pending`.
- `py.new_pool()` is used so Python objects are freed at scope exit; preserve this lifetime boundary.
- `py_calls_init` treats missing `mbpy_calls_init.mbpy_calls_init()` as a warning, not a hard failure.

## ANTI-PATTERNS
- Do not assume this module exists unless PyO3/Python features are enabled.
- Do not rename Python module/function strings without updating `python/` runtime files and deployment paths.
- Do not swallow Python exceptions silently; current bridge prints errors before formatting them.

## COMMANDS
```bash
cargo build --features python
cargo test --features python
```

## NOTES
- Top-level `__mb_call` falls back to `mb_sv_call` when Python polling returns `Err` or Python is not compiled in.
- The Cargo feature is named `python`, but Rust cfg checks also use `pyo3` because the optional dependency creates that feature.
