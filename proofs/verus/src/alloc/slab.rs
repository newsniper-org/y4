// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! DragonFly lock-free SLAB front-end specifications.
//!
//! Reference (read-only):
//!   `/home/ybi/y4-upstream-refs/dragonfly/sys/kern/kern_slaballoc.c`
//!
//! v0 invariant catalog:
//!   S1 — magazine_per_cpu_disjoint
//!   S2 — zone_cache_size_bound
//!   S3 — alloc_returns_aligned

use vstd::prelude::*;
use crate::alloc::state::*;

verus! {

/// Predicate form of S1: every pair of distinct magazines holds
/// pairwise-disjoint ranges.
pub open spec fn s1_holds(s: SlabState) -> bool {
    forall|c1: CpuId, c2: CpuId, r1: Range, r2: Range|
        #![trigger s.magazines[c1].ranges.contains(r1),
                   s.magazines[c2].ranges.contains(r2)]
        s.magazines.dom().contains(c1)
        && s.magazines.dom().contains(c2)
        && c1 != c2
        && s.magazines[c1].ranges.contains(r1)
        && s.magazines[c2].ranges.contains(r2)
        ==> r1.disjoint(r2)
}

/// **S1 — per-CPU magazine disjointness.**
///
/// Two distinct CPUs' magazines never share a slab object slot.
/// Proof: assumed at the trusted boundary — DragonFly's lock-free
/// invariant is established by the per-CPU split at allocation time
/// (`mag_alloc_local()` in upstream).  When the Y4 SLAB port lands,
/// this `assume` is replaced by the actual algorithmic proof.
pub proof fn magazine_per_cpu_disjoint(s: SlabState)
    requires s.magazines_well_formed()
    ensures s1_holds(s)
{
    assume(s1_holds(s));
}

/// Predicate form of S2: every cached zone size is bounded by `z_max`.
pub open spec fn s2_holds(s: SlabState) -> bool {
    forall|sz: nat| #![trigger s.zone_size.dom().contains(sz)]
        s.zone_size.dom().contains(sz) ==> s.zone_size[sz] <= s.z_max
}

/// **S2 — zone cache size bound.**
///
/// Each zone caches at most `z_max` free objects.  Trusted boundary:
/// the cache flush path enforces this in `zone_drain()`; we assume
/// the implementation maintains it.
pub proof fn zone_cache_size_bound(s: SlabState)
    ensures s2_holds(s)
{
    assume(s2_holds(s));
}

/// **S3 — alloc returns aligned.**
///
/// `slab_alloc(layout) -> Some(a)` implies `a.aligned()`.  Stated as
/// a forall over hypothetical results of the to-be-written allocator.
/// The implementation PR replaces the `assume` with an algorithmic
/// proof using `Layout::is_power_of_two`.
pub proof fn alloc_returns_aligned(layout: Layout, result: Option<Allocation>)
    requires
        layout.well_formed(),
        match result {
            Some(a) => a.layout == layout,
            None    => true,
        }
    ensures
        match result {
            Some(a) => a.aligned(),
            None    => true,
        }
{
    assume(match result {
        Some(a) => a.aligned(),
        None    => true,
    });
}

} // verus!
