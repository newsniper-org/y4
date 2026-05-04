// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! USB capsule specifications — STUB for Phase B step 4.
//!
//! USB host controller (XHCI) work follows PCIe enum (USB controllers
//! ARE PCIe devices on x86_64).  Y4's first USB capsule reuses NetBSD's
//! anykernel USB stack via rump (`licensing.md` §"Linux driver tier"
//! priority 2), so the Y4-side spec surface is small: enumerate ports
//! and forward urb requests.
//!
//! v0 catalog (stubs):
//!   U1 — port_count_static
//!   U2 — urb_completion_total

use vstd::prelude::*;
use crate::capsules::state::*;

verus! {

/// Predicate form of U1: every USB host controller exposes a
/// non-decreasing port count over its lifetime (USB allows hot-plug
/// of *devices*, not *ports*).
pub open spec fn u1_holds(prev_count: nat, next_count: nat) -> bool {
    next_count >= prev_count
}

pub proof fn port_count_static(prev_count: nat, next_count: nat)
    requires next_count >= prev_count
    ensures u1_holds(prev_count, next_count)
{
}

/// Predicate form of U2: every URB submitted to a port reaches a
/// terminal completion code (Success / Stall / Timeout).  Stubbed —
/// the real spec lands when rump kernel integration begins.
pub open spec fn u2_holds() -> bool {
    true
}

pub proof fn urb_completion_total()
    ensures u2_holds()
{
}

} // verus!
