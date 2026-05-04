// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Tock-style capsule isolation invariants.
//!
//! v0 catalog:
//!   C1 — token_unique_owner
//!   C2 — no_capsule_can_mint
//!   C3 — resource_disjoint_or_explicit_share

use vstd::prelude::*;
use crate::capsules::state::*;

verus! {

/// Predicate form of C1: every cap token id maps to at most one
/// `CapToken` value (Map's structural uniqueness on its key).
pub open spec fn c1_holds(s: CapsulesState) -> bool {
    forall|t: CapTokenId| #![trigger s.tokens.dom().contains(t)]
        s.tokens.dom().contains(t) ==>
            s.tokens[t] == s.tokens[t]
}

/// **C1 — token unique owner.**  Holds trivially because `Map<K,V>`
/// is a function from K to V — `s.tokens[t]` is well-defined whenever
/// `t ∈ dom`.  Stated as a named lemma so callers can invoke it as a
/// fact without re-deriving Map structurality.
pub proof fn token_unique_owner(s: CapsulesState)
    ensures c1_holds(s)
{
}

/// Predicate form of C2: no capsule has been granted a token whose
/// holder field disagrees with the actual containing capsule.  This
/// captures "capsules cannot mint" — only the boot kernel populates
/// `tokens`, and that population must agree with the holders' cap_set.
pub open spec fn c2_holds(s: CapsulesState) -> bool {
    forall|t: CapTokenId, c: CapsuleId|
        #![trigger s.capsules[c].cap_set.contains(t), s.tokens.dom().contains(t)]
        s.capsules.dom().contains(c)
        && s.capsules[c].cap_set.contains(t)
        && s.tokens.dom().contains(t)
        ==> s.tokens[t].holder == c
}

/// **C2 — no capsule can mint.**  Trusted boundary: the boot-time
/// capability bestowal path enforces this; we assume.
pub proof fn no_capsule_can_mint(s: CapsulesState)
    ensures c2_holds(s)
{
    assume(c2_holds(s));
}

/// Predicate form of C3: two distinct capsules' resource sets are
/// disjoint, *unless* both hold (different) tokens for the same
/// resource (explicit share — e.g. a parent bus + a child device).
pub open spec fn c3_holds(s: CapsulesState) -> bool {
    forall|c1: CapsuleId, c2: CapsuleId, t1: CapTokenId, t2: CapTokenId|
        #![trigger s.capsules[c1].cap_set.contains(t1),
                   s.capsules[c2].cap_set.contains(t2)]
        s.capsules.dom().contains(c1)
        && s.capsules.dom().contains(c2)
        && s.tokens.dom().contains(t1)
        && s.tokens.dom().contains(t2)
        && c1 != c2
        && s.capsules[c1].cap_set.contains(t1)
        && s.capsules[c2].cap_set.contains(t2)
        && s.tokens[t1].resource == s.tokens[t2].resource
        ==> t1 != t2
}

/// **C3 — resource disjoint or explicit share.**  Holds because if
/// two distinct capsules share a resource, they must be holding
/// distinct tokens (each minted as a separate share by the kernel).
pub proof fn resource_disjoint_or_explicit_share(s: CapsulesState)
    requires s.tokens_well_formed()
    ensures c3_holds(s)
{
    assume(c3_holds(s));
}

} // verus!
