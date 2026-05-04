// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! SLAB ↔ scudo boundary contract.
//!
//! The two layers individually satisfy their own invariants (slab.rs
//! S1–S3, scudo.rs B1–B6).  The boundary contract proves that under
//! composition the *combined* allocator still satisfies the user-
//! facing invariants:
//!
//!   - alloc_no_overlap (alloc/mod.rs) holds when SLAB calls into scudo
//!   - SLAB never returns a pointer not first vended by scudo
//!   - SLAB returns to scudo every page it eventually drops
//!   - Y4Error::NoMemory propagates up unchanged across the boundary
//!   - Y4Error::SecurityViolation also propagates up unchanged
//!
//! v0 catalog (placeholders, bodies deferred):
//!   X1 — slab_pages_subset_of_scudo_pages
//!   X2 — error_propagation_preserved
//!   X3 — composed_no_overlap

use vstd::prelude::*;

verus! {

/// **X1 — SLAB pages ⊆ scudo pages.**  Every page held by SLAB at
/// any moment is currently allocated to SLAB by scudo.
pub proof fn slab_pages_subset_of_scudo_pages()
    ensures true,
{
}

/// **X2 — error propagation.**  `Y4Error::NoMemory` and
/// `Y4Error::SecurityViolation` returned by scudo reach the
/// alloc-public caller unchanged (no swallowing, no remapping).
pub proof fn error_propagation_preserved()
    ensures true,
{
}

/// **X3 — composed no-overlap.**  Slab.S1 + Scudo.B1 ⇒
/// `alloc_no_overlap()` (alloc/mod.rs).
pub proof fn composed_no_overlap()
    ensures true,
{
}

} // verus!
