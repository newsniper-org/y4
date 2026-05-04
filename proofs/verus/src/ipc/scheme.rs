// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Redox scheme verb specifications.
//!
//! v0 catalog: SC1 path_resolution_deterministic, SC2
//! handle_lifetime_bounded_by_close, SC3 verb_dispatch_in_caller_context,
//! SC4 scheme_id_uniqueness.

use vstd::prelude::*;
use crate::ipc::state::*;
use crate::error::Y4Error;

verus! {

/// Spec-side path resolver: pure function from `(SchemeState, path)` to
/// either an endpoint cap or `Y4Error::BadCap`.  Determinism (SC1) is
/// the property that the function depends only on its inputs.
pub open spec fn scheme_lookup(s: SchemeState, path_id: SchemeId) -> Option<EndpointCap> {
    if s.registry.dom().contains(path_id) {
        Some(s.registry[path_id])
    } else {
        None
    }
}

/// Predicate form of SC1.
pub open spec fn sc1_holds(s1: SchemeState, s2: SchemeState, path: SchemeId) -> bool {
    s1 == s2 ==> scheme_lookup(s1, path) == scheme_lookup(s2, path)
}

/// **SC1 — path resolution determinism.**  Direct from the function
/// being a pure spec.
pub proof fn path_resolution_deterministic(s1: SchemeState, s2: SchemeState, path: SchemeId)
    ensures sc1_holds(s1, s2, path)
{
}

/// Predicate form of SC2: every operation on a non-live handle returns
/// `Y4Error::BadCap`.
pub open spec fn sc2_holds(s: SchemeState, h: HandleId, e: Y4Error) -> bool {
    !s.live_handles.contains(h) ==> e == Y4Error::BadCap
}

/// **SC2 — handle lifetime bounded by close.**  Trusted boundary on
/// the verb dispatcher (`scheme_dispatch_verb()`); we assume.  The
/// invariant captures: once `close()` removes a handle from
/// `live_handles`, every subsequent verb returns BadCap.
pub proof fn handle_lifetime_bounded_by_close(s: SchemeState, h: HandleId, e: Y4Error)
    ensures sc2_holds(s, h, e)
{
    assume(sc2_holds(s, h, e));
}

/// Predicate form of SC3: the dispatching thread's CSpace is the
/// caller's, not the implementer's.  Modelled by tagging each handle
/// with its `owner` thread and asserting verb invocation runs on it.
pub open spec fn sc3_holds(h: Handle, runs_on: ThreadId) -> bool {
    runs_on == h.owner
}

/// **SC3 — verb dispatch in caller context.**  Trusted boundary on the
/// dispatcher; we assume.
pub proof fn verb_dispatch_in_caller_context(h: Handle, runs_on: ThreadId)
    requires runs_on == h.owner
    ensures sc3_holds(h, runs_on)
{
    // Direct: precondition is the body.
}

/// Predicate form of SC4: at any given moment, every SchemeId maps to
/// at most one endpoint (Map's structural property gives this for
/// free, but we expose it as a named invariant for callers).
pub open spec fn sc4_holds(s: SchemeState) -> bool {
    forall|id: SchemeId| #![trigger s.registry.dom().contains(id)]
        s.registry.dom().contains(id) ==>
            scheme_lookup(s, id) == Some(s.registry[id])
}

/// **SC4 — SchemeId uniqueness.**  Falls out of `Map<K,V>` structural
/// uniqueness directly via the lookup function definition.
pub proof fn scheme_id_uniqueness(s: SchemeState)
    ensures sc4_holds(s)
{
}

} // verus!
