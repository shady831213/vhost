# sv/mailbox_sv KNOWLEDGE BASE

## OVERVIEW
SystemVerilog/UVM mailbox runtime that exports DPI functions consumed by Rust and calls back into Rust for memory and server progress.

## STRUCTURE
```text
sv/mailbox_sv/
|-- mailbox.sv     # DPI imports/exports, Mailbox classes, state machine
|-- mailbox.svh    # macro defaults
`-- mailbox_if.sv  # clock/reset interface
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| DPI surface | `mailbox.sv` | Imports Rust functions, exports SV functions/tasks |
| RPC state machine | `mailbox.sv` | `MailboxSvCallState::{INIT,PENDING,READY}` |
| Channel registry | `mailbox.sv` | `Mailbox`, `MailboxCh`, `MbRoot` |
| Black-box memory | `mailbox.sv` | `MbBBMem`, `MbBBMemFactory` |
| Pointer width / args | `mailbox.svh` | `MB_PTR`, `SV_CALL_MAX_ARGS` |
| Clock stepping | `mailbox_if.sv` | Minimal `clk`/`reset` interface |

## CONVENTIONS
- `mailbox.sv` includes `mailbox.svh`; keep macro names stable for Rust/SV agreement.
- `mb_step` advances on `posedge inf.clk`; Rust calls it from `mb_tick()`.
- `mb_sv_call` returns method results through `Mailbox::poll` and a status out-param.
- Registration APIs use existing project spelling `registery`; preserve unless doing a deliberate rename across boundaries.

## ANTI-PATTERNS
- Do not edit generated copies under `target/debug/build/*/out/vhost/mailbox_sv`.
- Do not change `MB_PTR` or `SV_CALL_MAX_ARGS` without checking Rust `MBPtrT` and call argument expectations.
- Do not add standalone SV files without ensuring `build.rs` staging and simulator include paths see them.

## NOTES
- `MailboxSvCall::run()` forks an infinite execution loop gated by events.
- Missing channel/method/black-box memory registrations call `uvm_fatal`.
- Source tree is copied by Cargo build but compiled by the simulator/UVM flow, not by rustc.
