// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! PCIe enumeration capsule specifications.
//!
//! PCIe enum is the most concrete non-HIU capsule we ship in Phase B
//! step 4 — it's how Y4 discovers the WaveTensor accelerator board on
//! the host bus.  Invariants below define the *contract* the enum
//! capsule must satisfy.
//!
//! v0 catalog:
//!   P1 — enum_deterministic              (same hardware ⇒ same list)
//!   P2 — enum_addrs_unique
//!   P3 — enum_requires_bus_enumerator_cap

use vstd::prelude::*;
use crate::capsules::state::*;

verus! {

/// Predicate form of P1: two enumeration calls against the same
/// hardware view return identical device sequences.  Hardware view is
/// modelled as the input parameter `hw`.
pub open spec fn p1_holds(hw: nat, r1: PcieEnumResult, r2: PcieEnumResult) -> bool {
    r1.devices == r2.devices
}

/// **P1 — enumeration determinism.**  Trusted boundary: the bus walk
/// algorithm is a pure function of config-space contents; we assume.
/// (Required by Y4 for stable lease cap binding to PCIe BARs.)
pub proof fn enum_deterministic(hw: nat, r1: PcieEnumResult, r2: PcieEnumResult)
    ensures p1_holds(hw, r1, r2)
{
    assume(p1_holds(hw, r1, r2));
}

/// Predicate form of P2: every two distinct entries in an enumeration
/// result occupy distinct (bus, dev, fn) addresses.
pub open spec fn p2_holds(r: PcieEnumResult) -> bool {
    forall|i: int, j: int|
        #![trigger r.devices.index(i), r.devices.index(j)]
        0 <= i < r.devices.len()
        && 0 <= j < r.devices.len()
        && i != j
        ==> r.devices.index(i).addr != r.devices.index(j).addr
}

/// **P2 — addresses unique.**  Trusted boundary on the bus-walk —
/// the algorithm visits each (b, d, f) at most once.
pub proof fn enum_addrs_unique(r: PcieEnumResult)
    ensures p2_holds(r)
{
    assume(p2_holds(r));
}

/// Predicate form of P3: the capsule that performed an enum call held
/// a `BusEnumerator` capability for the bus root.  Modelled as: there
/// exists a token in the capsule's cap_set whose kind is BusEnumerator.
pub open spec fn p3_holds(s: CapsulesState, capsule: CapsuleId) -> bool {
    s.capsules.dom().contains(capsule)
        && exists|t: CapTokenId|
            #![trigger s.capsules[capsule].cap_set.contains(t)]
            s.capsules[capsule].cap_set.contains(t)
                && s.tokens.dom().contains(t)
                && s.tokens[t].kind == ResourceKind::BusEnumerator
}

/// **P3 — enum requires BusEnumerator capability.**  Trusted boundary
/// on the enum syscall front-end (which checks the cap before
/// dispatching to the bus walk); we assume the precondition shape and
/// derive the predicate.
pub proof fn enum_requires_bus_enumerator_cap(s: CapsulesState, capsule: CapsuleId)
    requires
        s.capsules.dom().contains(capsule),
        exists|t: CapTokenId|
            #![trigger s.capsules[capsule].cap_set.contains(t)]
            s.capsules[capsule].cap_set.contains(t)
                && s.tokens.dom().contains(t)
                && s.tokens[t].kind == ResourceKind::BusEnumerator,
    ensures p3_holds(s, capsule)
{
}

} // verus!
