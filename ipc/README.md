<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# y4-ipc — Y4 IPC subsystem

Redox scheme verbs (control plane) + DragonFly LWKT msgport (data
plane) hybrid (decision I-P2 in [`MEMORY/y4_ipc_alloc_preflight.md`](../.claude-memories/y4_ipc_alloc_preflight.md)).
Spec lives at [`/proofs/verus/src/ipc/`](../proofs/verus/src/ipc/).

ipc/ and alloc/ are independent (decision C1) — messages are
caller-allocated (LWKT pattern), so the IPC layer never calls into
alloc/.

## What ships

| Module | Purpose | Spec correspondence |
|--------|---------|---------------------|
| `types` | `Msg` / `MsgState` / `Endpoint` / `Handle` — Rust mirror of the Verus state model | `state.rs` |
| `error` | `Y4Error` — common enum (mirrored from `proofs/verus/src/error.rs`) | C3 |
| `sel4_backend` | `Sel4Backend` trait abstracting `seL4_Send/Recv/Call/Reply`, `seL4_CNode_Mint`, `seL4_Untyped_Retype`; `MockSel4Backend` (per-endpoint FIFO) for tests | C4 trusted boundary |
| `msgport` | `Msgport` — LWKT-style dispatcher; `open` / `close` / `send` / `wait` / `forward` / `abort` / `reply` / `time_out`; per-CPU `owner_cpu` tagging; per-endpoint priority high-water mark | M1 / M2 / M3 / M4 / M5 |
| `scheme` | `SchemeRegistry` — `register` / `lookup` / `open` / `close` / `read` / `write` / `dup` / `fevent`; per-handle inline buffer + event mask; caller-context strict | SC1 / SC2 / SC3 / SC4 + K3 carve-out via `dup` |

## Tests

```sh
cargo test -p y4-ipc       # 18 tests
```

Coverage:

- `msgport`: open/send/wait round-trip, FIFO ordering, empty-wait
  None, close revokes, **forward re-targets (M2)**, **abort
  owner-only (M3)**, **priority high-water mark (M5)**, **per-CPU
  ports disjoint (M4)**, **lifecycle terminals settable (M1)**
- `scheme`: open/close round-trip, unknown scheme rejected,
  **distinct handles → distinct endpoints (K3)**, **write-then-read
  byte round-trip**, **read on stale handle BadCap (SC2)**, **read
  with wrong caller InvalidArg (SC3)**, **dup shares endpoint (K3
  carve-out)**, **fevent records mask**, **lookup deterministic (SC1)**

## Refinement against the Verus spec

`proofs/verus/src/ipc/refinement.rs` adds executable spec functions
and discharges five of the spec's `assume()`s by direct construction:

- **M5 priority_inversion_avoidance** via `priority_step` +
  `m5_high_water_after_step` + `m5_monotone`
- **SC4 scheme_id_uniqueness** via `register_step` +
  `sc4_preserved_by_register`
- **M1 send_recv_pairing** via `m1_state_is_total`
- **K2 cross-layer race absence** via `k2_via_layers`

Remaining `assume()`s (M2, M3, M4, SC2, K1, K3) sit at the trusted
boundary — the contract this Rust impl is intended to honour but not
yet machine-checked against by Verus.

## Roadmap

| Stage | Deliverable | Spec assume → discharge |
|-------|-------------|-------------------------|
| **0 (shipped)** | mock-backed scheme + msgport with full verb set | M5, M1, SC4 (constructive); rest at trusted boundary |
| 1 | Real `Sel4Backend` impl in `kernel/` | trusted boundary closes for real cap invocations |
| 2 | Per-CPU sharded msgport queues (true M4 by structure) | M4 |
| 3 | Priority-inheritance / ceiling implementation | M5 implementation choice locked in |
| 4 | Cross-layer consistency proof (scheme verb ↔ msgport seq) | K1 |
