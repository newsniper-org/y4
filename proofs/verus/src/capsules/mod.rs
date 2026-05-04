// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Y4 capsule subsystem specifications — Tock-style isolated drivers.
//!
//! Phase B step 4 ships PCIe enumeration in depth (Y4 needs it to find
//! the WaveTensor board) plus stubs for USB and CXL whose real driver
//! work follows in dependent PRs.  HIU is intentionally absent — HIU
//! capsule work is blocked on `docs/hiu_abi.md` v1.0 frozen.
//!
//! See `proofs/verus/src/capsules/README.md` for the v0 invariant catalog.

use vstd::prelude::*;

pub mod state;
pub mod isolation;
pub mod pcie;
pub mod usb;
pub mod cxl;

use state::*;

verus! {

/// Top-level capsule subsystem invariant: combination of per-token
/// well-formedness + the three isolation invariants from `isolation.rs`.
pub proof fn capsules_invariant(s: CapsulesState)
    requires s.tokens_well_formed()
    ensures
        isolation::c1_holds(s),
        isolation::c2_holds(s),
        isolation::c3_holds(s),
{
    isolation::token_unique_owner(s);
    isolation::no_capsule_can_mint(s);
    isolation::resource_disjoint_or_explicit_share(s);
}

} // verus!
