// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Y4 allocator specifications — `DragonFly lock-free SLAB` front-end +
//! `LLVM scudo` backend (decision A-P1).
//!
//! Layered structure:
//!
//! ```text
//! caller
//!   │
//!   ▼
//! [SLAB front-end]   ← per-CPU magazine, hot-path object cache
//!   │
//!   ▼
//! [scudo backend]    ← NUMA-aware quarantine, randomization,
//!                      guard pages, UAF detection
//!   │
//!   ▼
//! seL4 kernel API:
//!   seL4_X86_Page_Map / Unmap, seL4_X86_PageTable_Map,
//!   seL4_Untyped_Retype                       (C4 trusted boundary)
//! ```
//!
//! Each layer has its own spec file in this module.  See
//! `proofs/verus/src/alloc/README.md` for the v0 invariant catalog.

use vstd::prelude::*;

pub mod slab;
pub mod scudo;
pub mod boundary;

verus! {

/// Top-level no-overlap invariant.  Holds across the whole alloc
/// subsystem — both the SLAB and scudo layers individually preserve
/// it, and the boundary contract ensures it under composition.
///
/// ⚠ TODO Phase B step 3 — body intentionally empty, awaiting concrete
///       heap state model.  The signature stands so dependent specs
///       can call `alloc_no_overlap(...)` without recompile breakage.
pub proof fn alloc_no_overlap()
    ensures true,
{
}

} // verus!
