# python KNOWLEDGE BASE

## OVERVIEW
Two Python callback shims implement the same `poll(callback_name, id, *args)` contract for different async runtimes.

## STRUCTURE
```text
python/
|-- async/pymb_rpc.py   # asyncio variant
`-- cocotb/pymb_rpc.py  # cocotb variant with DUT access
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Plain async callbacks | `async/pymb_rpc.py` | Uses `asyncio.Event` and event-loop tasks |
| cocotb callbacks | `cocotb/pymb_rpc.py` | Uses `cocotb.fork`, `RunningTask`, DUT globals |
| Rust bridge | `../src/rpcs/py_calls.rs` | Imports `pymb_rpcs.poll` through PyO3 |
| Build staging | `../build.rs` | Copies this tree to `target/.../out/vhost` |

## CONVENTIONS
- `register_callback(callback)` stores by `callback.__name__`; Rust passes method names as strings.
- `poll(...)` returns `(exists, finished, result)` and recursively starts a missing task before polling it.
- Keys are `callback_name + "_" + id`; IDs are strings from Rust channel names.
- The cocotb variant passes `g_dut` as the first callback argument.

## ANTI-PATTERNS
- Do not edit generated copies under `target/debug/build/*/out/vhost/{async,cocotb}`.
- Do not add package-import assumptions: there is no `__init__.py`, `pyproject.toml`, or `setup.py`.
- Do not rename `pymb_rpc.py` casually; Rust hardcodes import text for `pymb_rpcs.poll` at the bridge boundary.

## COMMANDS
```bash
cargo build --features python
cargo test --features python
```

## NOTES
- No Python test runner is configured in this repo.
- `src/rpcs/py_calls.rs` also looks for optional `mbpy_calls_init.mbpy_calls_init()` and only warns if absent.
