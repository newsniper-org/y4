// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Y4 Verus specifications.
//!
//! This crate hosts machine-checked specifications for every privileged
//! Y4 code path. Each subsystem gets its own module:
//!
//! ```text
//! src/
//! ├── lib.rs          ← this file (root; placeholder until specs land)
//! ├── ipc/            ← LWKT + scheme IPC invariants (Phase B step 3)
//! ├── alloc/          ← SLUB + SLAB + mmap-only allocator (Phase B step 3)
//! ├── kernel/         ← seL4 root-task and capability bootstrap
//! └── lease/          ← LeaseCap I1–I6 (deferred — needs hiu_abi v1.0)
//! ```
//!
//! See `proofs/README.md` for the harness contract and
//! `docs/lease_capability.md` §2 for the invariant catalogue.
//!
//! This file is verified by the `verus` CLI, NOT by `cargo build` —
//! it depends on `vstd` (Verus standard library) bundled with the
//! Verus install and uses the `verus!{}` macro syntax.

use vstd::prelude::*;

verus! {

/// Placeholder identity function used only to give the harness a target
/// it can verify in absence of any real spec. Real specs replace this.
pub const fn placeholder(x: u32) -> (r: u32)
    ensures r == x
{
    x
}

/// Trivial proof so the harness has something to discharge.
proof fn placeholder_trivial()
    ensures 1nat + 1nat == 2nat
{
}

} // verus!
