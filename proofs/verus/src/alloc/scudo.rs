// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! LLVM scudo backend specifications.
//!
//! scudo is the hardened allocator from the LLVM project (Apache-2.0).
//! Y4 uses it as the page-grain backend below DragonFly SLAB.  The
//! scudo C++ source is *linked*, not ported — Verus invariants here
//! describe the *contract* Y4 demands of scudo, not its internal state.
//!
//! v0 invariant catalog (placeholders, bodies deferred):
//!   B1 — backend_no_overlap
//!   B2 — uaf_detection                (A-P3: use-after-free)
//!   B3 — randomization                (A-P3: randomized base)
//!   B4 — guard_page_alignment         (A-P3: guard pages)
//!   B5 — numa_node_locality
//!   B6 — quarantine_lifetime_bound
//!
//! When scudo's contract is violated at runtime, the alloc layer
//! returns `Y4Error::SecurityViolation` (see `proofs/verus/src/error.rs`).

use vstd::prelude::*;

verus! {

/// **B1 — backend non-overlap.**  Distinct live allocations occupy
/// disjoint address ranges.  Holds across NUMA nodes.
pub proof fn backend_no_overlap()
    ensures true,
{
}

/// **B2 — use-after-free detection (A-P3).**
///
/// Reading a freed allocation either returns `Y4Error::SecurityViolation`
/// (caught by scudo's quarantine check) OR causes a guard-page fault
/// (B4).  Silent UAF — the freed page being silently reused with
/// stale contents readable — is forbidden.
pub proof fn uaf_detection()
    ensures true,
{
}

/// **B3 — address randomization (A-P3).**
///
/// Successive allocations of the same size do not return predictable
/// virtual addresses.  Spec form: there exists no constant `c` such
/// that `alloc(s) == prev_alloc(s) + c` for all sequences — i.e.
/// scudo's per-allocation jitter is part of the spec.
pub proof fn randomization()
    ensures true,
{
}

/// **B4 — guard page alignment (A-P3).**
///
/// Every live allocation is bracketed by guard pages whose mappings
/// fault on access.  Catches sequential overflow / underflow before
/// it can hit another live allocation.
pub proof fn guard_page_alignment()
    ensures true,
{
}

/// **B5 — NUMA locality.**
///
/// `alloc_on_node(s, n)` returns memory whose backing physical frames
/// are sourced from NUMA node `n`.  Required by SLAB front-end's
/// per-CPU magazine to keep allocations local.  (No-op on uniprocessor
/// configs but spec'd for SMP-first per C2.)
pub proof fn numa_node_locality()
    ensures true,
{
}

/// **B6 — quarantine lifetime bound.**
///
/// A freed allocation stays in scudo's quarantine for at most `Q_MAX`
/// successive `free()` operations before being released back to the
/// OS via `seL4_X86_Page_Unmap` and `seL4_Untyped_Retype`.  Bounds
/// the worst-case quarantine memory footprint.
pub proof fn quarantine_lifetime_bound()
    ensures true,
{
}

} // verus!
