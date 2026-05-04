<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# y4-alloc — Y4 memory allocator

DragonFly lock-free SLAB front-end + LLVM scudo backend (decision A-P1).
Spec lives at [`/proofs/verus/src/alloc/`](../proofs/verus/src/alloc/).

## Current state

- **Public types**: `Range`, `Layout`, `Allocation` — Rust mirror of the
  Verus state model.
- **`PageBackend` trait**: abstracts seL4_X86_Page_Map / Unmap /
  Untyped_Retype.  `MockPageBackend` for tests.
- **`BumpAllocator`**: simplest correct allocator over the trait.
  Satisfies the no-overlap invariant; no free, no NUMA, no quarantine.

## Roadmap

| Stage | Deliverable | Spec assume → discharge |
|-------|-------------|-------------------------|
| **0 (this PR)** | bump skeleton + trait + types | (none yet) |
| 1 | DragonFly SLAB front-end Rust port | S1, S2, S3 |
| 2 | scudo FFI binding + `third_party/scudo` submodule | B1–B6 |
| 3 | SLAB ↔ scudo composition | X1–X3 |
| 4 | seL4 backend implementation in `kernel/` | trusted boundary closes |

## Tests

```sh
cargo test -p y4-alloc        # unit tests on the bump allocator
just ci                        # workspace-wide gate including this crate
```

## Refinement against the spec

The spec at `proofs/verus/src/alloc/` uses `assume()` at every trusted
boundary.  Each Roadmap stage above closes a subset of those `assume`s
by adding a refinement proof inside the Verus crate that points at the
matching Rust impl.  Until that proof lands, the spec is a *contract*
this crate is *intended* to honour but not yet machine-checked against.
