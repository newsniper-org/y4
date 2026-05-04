// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Abstract state model used by the alloc subsystem invariants.
//!
//! The model is *spec-only* — no executable allocator code lives here.
//! Each invariant in `slab.rs`, `scudo.rs`, `boundary.rs`, and `mod.rs`
//! is a pure proof over these types.  The real allocator (Phase B step 4
//! onwards) must establish refinement against this model.

use vstd::prelude::*;

verus! {

/// Virtual address.
pub type VAddr = nat;

/// CPU identifier (SMP-first per C2; single-CPU configs treat this as
/// trivially {0}).
pub type CpuId = nat;

/// NUMA node identifier (B5).
pub type NumaNodeId = nat;

/// Half-open virtual address range `[start, end)`.  Well-formedness
/// requires `start < end`.  An ill-formed range is treated as empty.
pub struct Range {
    pub start: VAddr,
    pub end:   VAddr,
}

impl Range {
    /// `addr` falls inside the range.
    pub open spec fn contains(self, addr: VAddr) -> bool {
        self.start <= addr && addr < self.end
    }

    /// Number of bytes the range covers (0 if ill-formed).
    pub open spec fn len(self) -> nat {
        if self.start < self.end { (self.end - self.start) as nat } else { 0nat }
    }

    /// Two ranges are disjoint if they share no address.
    pub open spec fn disjoint(self, other: Range) -> bool {
        self.end <= other.start || other.end <= self.start
    }

    /// Well-formedness predicate.
    pub open spec fn well_formed(self) -> bool {
        self.start < self.end
    }
}

/// Allocation request layout: size + power-of-two alignment.
pub struct Layout {
    pub size:  nat,
    pub align: nat,
}

impl Layout {
    /// `align` is a power of two and `size > 0`.
    pub open spec fn well_formed(self) -> bool {
        self.size > 0 && self.align > 0 && Self::is_power_of_two(self.align)
    }

    /// Spec-side power-of-two test (recursive).  `1` is a power of two;
    /// `2k` is iff `k` is; odd `n > 1` is not.
    pub open spec fn is_power_of_two(n: nat) -> bool
        decreases n
    {
        if n == 0 {
            false
        } else if n == 1 {
            true
        } else if n % 2 == 0 {
            Self::is_power_of_two((n / 2) as nat)
        } else {
            false
        }
    }
}

/// A live allocation: covers `range`, satisfies `layout`, sourced from
/// `numa`, with `is_guarded` indicating B4 guard pages installed.
pub struct Allocation {
    pub range:      Range,
    pub layout:     Layout,
    pub numa:       NumaNodeId,
    pub is_guarded: bool,
}

impl Allocation {
    pub open spec fn aligned(self) -> bool {
        self.range.start % self.layout.align == 0
    }

    pub open spec fn well_formed(self) -> bool {
        self.range.well_formed()
            && self.layout.well_formed()
            && self.aligned()
            && self.range.len() >= self.layout.size
    }
}

// ------------------------------------------------------------------------
// SLAB front-end state
// ------------------------------------------------------------------------

/// Per-CPU magazine: the set of allocation ranges this CPU currently
/// holds for fast re-issue without going to scudo.
pub struct Magazine {
    pub ranges: Set<Range>,
}

/// Whole-allocator SLAB state: a magazine per CPU + zone-cache size
/// counters per zone (zones identified by allocation size class).
pub struct SlabState {
    pub magazines: Map<CpuId, Magazine>,
    /// `zone_size[size_class]` is the current count of cached objects.
    pub zone_size: Map<nat, nat>,
    /// Bound that any zone's cached count may not exceed (S2).
    pub z_max:     nat,
}

impl SlabState {
    /// Every magazine is internally well-formed (ranges all
    /// well-formed) — defined by recursion-free quantification.
    pub open spec fn magazines_well_formed(self) -> bool {
        forall|cpu: CpuId| #![trigger self.magazines.dom().contains(cpu)]
            self.magazines.dom().contains(cpu) ==>
                forall|r: Range| #![trigger self.magazines[cpu].ranges.contains(r)]
                    self.magazines[cpu].ranges.contains(r) ==> r.well_formed()
    }
}

// ------------------------------------------------------------------------
// scudo backend state
// ------------------------------------------------------------------------

/// scudo state: live allocations, quarantined allocations awaiting
/// release, and randomization seed evidence.
pub struct ScudoState {
    pub live:        Set<Allocation>,
    /// Quarantine entries paired with the count of free()s observed
    /// since the entry was added (used for B6 lifetime bound).
    pub quarantined: Map<Allocation, nat>,
    /// Bound on quarantine residency before forced release (B6).
    pub q_max:       nat,
    /// Witness that the randomization seed has been re-drawn at least
    /// once since boot — without it B3 cannot hold.
    pub randomized:  bool,
}

impl ScudoState {
    /// Every live allocation is well-formed.
    pub open spec fn live_well_formed(self) -> bool {
        forall|a: Allocation| #![trigger self.live.contains(a)]
            self.live.contains(a) ==> a.well_formed()
    }

    /// All addresses covered by some live allocation.
    pub open spec fn covered(self, addr: VAddr) -> bool {
        exists|a: Allocation| #![trigger self.live.contains(a)]
            self.live.contains(a) && a.range.contains(addr)
    }
}

// ------------------------------------------------------------------------
// Composed (alloc-public) view
// ------------------------------------------------------------------------

/// Combined allocator state visible to alloc-public callers.
pub struct AllocState {
    pub slab:  SlabState,
    pub scudo: ScudoState,
}

impl AllocState {
    /// Live allocations from the alloc-public view.  Currently equal
    /// to the scudo live set — SLAB magazines hold *cached* allocations
    /// that may have been freed by the public caller and recycled.
    pub open spec fn live(self) -> Set<Allocation> {
        self.scudo.live
    }
}

} // verus!
