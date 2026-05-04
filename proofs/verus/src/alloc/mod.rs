// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Y4 allocator specifications — `DragonFly lock-free SLAB` front-end +
//! `LLVM scudo` backend (decision A-P1).
//!
//! See `proofs/verus/src/alloc/README.md` for the v0 invariant catalog.

use vstd::prelude::*;

pub mod state;
pub mod slab;
pub mod scudo;
pub mod boundary;
pub mod refinement;

use state::*;

verus! {

/// Top-level no-overlap invariant.  Lifts X3 (composed_no_overlap) to
/// the alloc-public surface so callers can reason about safety without
/// reaching into either layer.
pub proof fn alloc_no_overlap(s: AllocState)
    requires
        s.scudo.live_well_formed(),
        s.slab.magazines_well_formed(),
    ensures boundary::alloc_no_overlap_holds(s)
{
    boundary::composed_no_overlap(s);
}

} // verus!
