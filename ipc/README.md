<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# y4-ipc — Y4 IPC subsystem

Redox scheme verbs (control plane) + DragonFly LWKT msgport (data
plane) hybrid (decision I-P2).  Spec lives at
[`/proofs/verus/src/ipc/`](../proofs/verus/src/ipc/).

## Current state

- **Public types**: `Msg`, `MsgState`, `Endpoint`, `Handle` — Rust
  mirror of the Verus state model.
- **`Sel4Backend` trait**: abstracts the seL4 endpoint / notification
  syscalls.  `MockSel4Backend` for tests (per-endpoint FIFO).
- **`Msgport`**: caller-allocated-message dispatch (LWKT pattern,
  satisfies C1 ipc/alloc independence).
- **`SchemeRegistry`**: scheme-id → endpoint dispatcher (`open` /
  `close`), handle table with `MAX_HANDLES = 64`.

## Roadmap

| Stage | Deliverable | Spec assume → discharge |
|-------|-------------|-------------------------|
| **0 (this PR)** | mock-backed scheme + msgport skeleton | (none yet) |
| 1 | Real `Sel4Backend` impl in `kernel/` | trusted boundary closes |
| 2 | Per-CPU sharded msgport queues | M4 (per-CPU isolation) |
| 3 | Priority-inversion handling (PI / ceiling) | M5 |
| 4 | Cross-layer consistency proof (scheme ↔ msgport) | K1, K2, K3 |

## Tests

```sh
cargo test -p y4-ipc          # msgport + scheme unit tests
just ci                        # workspace-wide gate including this crate
```
