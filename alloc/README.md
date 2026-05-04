<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# y4-alloc — Y4 memory allocator

DragonFly lock-free SLAB front-end + LLVM scudo backend (decision A-P1
in [`MEMORY/y4_ipc_alloc_preflight.md`](../.claude-memories/y4_ipc_alloc_preflight.md)).
Spec lives at [`/proofs/verus/src/alloc/`](../proofs/verus/src/alloc/).

## What ships

| Module | Purpose | Spec correspondence |
|--------|---------|---------------------|
| `types` | `Range` / `Layout` / `Allocation` — Rust mirror of the Verus state model | `state.rs` |
| `error` | `Y4Error` — common enum (mirrored from `proofs/verus/src/error.rs`) | C3 |
| `page_backend` | `PageBackend` trait abstracting `seL4_X86_Page_Map` / `Unmap` / `Untyped_Retype`; `MockPageBackend` for tests | C4 trusted boundary |
| `bump` | `BumpAllocator` — minimum-correct allocator over `PageBackend` | alloc_no_overlap |
| `slab` | DragonFly `SlabAllocator` Rust port — per-CPU zones, 9 size classes, refill, cross-CPU reject | S1 / S2 / S3 |
| `hardened` | `HardenedBackend` — Rust impl satisfying scudo's B1–B6 contract (UAF detect, randomization, guard pages, NUMA round-robin, quarantine) | B1 / B2 / B3 / B4 / B5 / B6 |
| `integrated` | `IntegratedAllocator` — SLAB + hardened composition; small via SLAB, large bypasses to hardened | X1 / X2 / X3 |
| `scudo_ffi` *(opt-in via `scudo` feature)* | `ScudoFfiBackend` — wraps `y4-scudo-sys` so the LLVM C++ allocator can be the production backend on hosted Linux | B1 (operational) |

## Tests

```sh
cargo test -p y4-alloc                  # 22 tests (without scudo feature)
cargo test -p y4-alloc --features scudo # 24 tests (adds scudo_ffi smokes)
```

Coverage:

- `bump`: alignment, no-overlap under pressure, OOM
- `slab`: class lookup, small alloc round-trip, per-CPU disjointness,
  cross-CPU free reject (S1), large bypass, Z_MAX cap (S2)
- `hardened`: randomization witness (B3), distinct seeds → distinct
  starts, live no-overlap (B1), UAF detection (B2), guard fault on
  overflow (B4), NUMA round-robin (B5), quarantine release after q_max (B6)
- `integrated`: small-via-SLAB, large-via-hardened, X1 chunks inside
  backend region, X3 no-overlap under pressure
- `scudo_ffi` *(feature)*: aligned pointer, many-allocs disjoint

## scudo as the production backend

The `hardened` module is a Rust contract twin — small enough to read in
one sitting and good enough for hosted unit tests.  The production
hardened backend is **LLVM scudo** linked through `y4-scudo-sys` and
gated behind the `scudo` cargo feature:

```sh
just scudo-fetch                  # materialise pinned C++ source (~1 MB)
cargo test -p y4-alloc --features scudo
```

The pin (commit SHA + LLVM repo) lives in
[`/third_party/scudo/PIN.toml`](../third_party/scudo/PIN.toml).
Bumping it is a one-line edit + `just scudo-fetch` + re-test —
see [`/third_party/scudo/README.md`](../third_party/scudo/README.md).

## Refinement against the Verus spec

`proofs/verus/src/alloc/refinement.rs` introduces *executable spec
functions* (e.g. `quarantine_step`, `init_state`, `alloc_no_overlap_via_b1`)
and discharges five of the spec's `assume()`s by induction:

- **B6 quarantine_lifetime_bound** via `b6_preserved_by_step` /
  `b6_preserved_by_stream`
- **B3 randomization** via `b3_holds_post_init`
- **alloc_no_overlap (lifted X3)** via `alloc_no_overlap_via_b1`

The remaining `assume()`s (S1, S2, S3, B1, B2, B4, B5, X1) sit at
the trusted boundary — the contract this Rust impl is *intended* to
honour but not yet machine-checked against by Verus.  Closing them
requires either lifting the Rust impl into Verus (`verus!{}` macro
inside the alloc crate) or adding a Kani harness for each.

## Roadmap

| Stage | Deliverable | Spec assume → discharge |
|-------|-------------|-------------------------|
| **0 (shipped)** | bump + slab + hardened + integrated + scudo FFI | B3, B6, X3 (constructive); rest at trusted boundary |
| 1 | seL4 backend `impl PageBackend` in `kernel/` | C4 trusted boundary closes for real cap invocations |
| 2 | Verus refinement of S1 (lift SlabAllocator into `verus!`) | S1 |
| 3 | Cross-CPU async free via seL4 IPI | depends on `kernel/` IPI cap |
