# src/sockets KNOWLEDGE BASE

## OVERVIEW
Localhost TCP helper exported as a C ABI surface for external simulation/testbench code.

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Socket owner | `mod.rs` | `SktWrapper` |
| Open/accept/close ABI | `mod.rs` | `c_skt_open`, `c_skt_accept`, `c_skt_close` |
| Numeric reads/writes | `mod.rs` | `c_skt_read_u*`, `c_skt_write_u*` |
| String ABI | `mod.rs` | `c_skt_read_string`, `c_skt_write_string` |

## CONVENTIONS
- Listens only on `127.0.0.1:{port}`.
- Streams are nonblocking; `WouldBlock` maps to `0` for poll-style callers.
- Client IDs are 1-based indexes into `stream`; hard failures usually map to `-1`.
- Integer writes are little-endian via `to_le_bytes()`.
- C strings use NUL termination at the ABI boundary.

## ANTI-PATTERNS
- Do not pass client ID `0`; checks only reject `id > len`, then index `id - 1`.
- Do not assume `c_skt_read_string` is payload-safe; current implementation is suspicious and leaks the returned CString to the caller.
- Do not expose this listener beyond localhost without a security review.

## NOTES
- No in-repo Rust callers were found; this is primarily an external FFI entrypoint.
- `SktWrapper` owns all streams; `c_skt_close` drops the boxed wrapper from a raw pointer.
- There is no child module split here; keep ABI changes localized in `mod.rs`.
- Prefer adding targeted tests or a driver before changing pointer/string ownership behavior.
