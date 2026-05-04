<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# y4-capsules — Y4 isolated drivers

Tock-style isolated drivers (non-HIU).  HIU capsule work is blocked
until `docs/hiu_abi.md` is `v1.0 frozen`.

Spec lives at [`/proofs/verus/src/capsules/`](../proofs/verus/src/capsules/).

## Current state

| Module | Purpose | Spec |
|--------|---------|------|
| `types` | `Capsule`, `CapToken`, `ResourceKind` mirrors of the Verus state model | `state.rs` |
| `isolation` | `CapsulesState` + runtime predicates `c1_holds`/`c2_holds`/`c3_holds` + `well_formed()` | `isolation.rs` |
| `config_space` | `ConfigSpace` trait abstracting PCIe config-space access; `MockConfigSpace` for tests | (used by `pcie.rs`) |
| `pcie` | `PcieEnumerator` — first concrete capsule.  Recursive bus walk, bridge descent, multi-function support, P3 cap check | `pcie.rs` |

## Tests

```sh
cargo test -p y4-capsules
```

16 tests cover:
- Empty-state and post-mint well-formedness (C1/C2/C3)
- Mint into unknown capsule (BadCap)
- Explicit-share variants (C3 carve-out)
- Single-device topology (P1 baseline)
- Bridged topology with descent into secondary bus (P1 + P2 + recursion)
- P3 cap-required (without cap → `BadCap`; unknown caller → `BadCap`)
- Determinism across repeated calls (P1)
- Self-loop bridge guard (no infinite recursion)
- Multi-function device per-function enumeration

## Roadmap

| Stage | Deliverable | Spec assume → discharge |
|-------|-------------|-------------------------|
| **0 (this PR)** | mock-backed PCIe enum + isolation runtime | (none yet) |
| 1 | x86 ECAM-backed `ConfigSpace` impl in `kernel/` | trusted boundary closes |
| 2 | USB host capsule (NetBSD anykernel via rump per `licensing.md` §"Linux driver tier") | U1, U2 |
| 3 | CXL.io capsule | X1c, X2c |

## Non-goals

- HIU capsule — blocked on `docs/hiu_abi.md` v1.0.
- Real ECAM access — Phase B step 5+ once `kernel/` provides the
  MMIO mapping primitive.
- Hot-plug events — Phase C topic.
