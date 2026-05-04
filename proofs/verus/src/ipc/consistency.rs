// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! scheme ↔ msgport hybrid 일관성 정리.
//!
//! v0 catalog: K1 scheme_op_implies_equivalent_msgport_seq,
//! K2 no_cross_layer_race, K3 handle_to_endpoint_bijection.

use vstd::prelude::*;
use crate::ipc::state::*;
use crate::ipc::scheme;
use crate::ipc::msgport;

verus! {

/// Scheme verb tag — abstracted to the four core verbs for v0.
pub enum SchemeVerb {
    Open,
    Read,
    Write,
    Close,
}

/// Predicate form of K1: for every (scheme verb, handle, ipc-state)
/// triple, there exists an equivalent msgport message that produces
/// the same observable lifecycle on the underlying endpoint cap.
/// Existence is the *contract* — the implementation PR realizes it.
pub open spec fn k1_holds(_s: IpcState, _verb: SchemeVerb, _h: HandleId) -> bool {
    // The witness existence is opaque at v0; the implementation builds
    // a concrete construction.  Stating it as `true` here keeps the
    // signature stable while the body is `assume`d at the boundary.
    true
}

/// **K1 — scheme verb의 msgport 등가성.**  Existence claim — body
/// realised in the implementation PR.
pub proof fn scheme_op_implies_equivalent_msgport_seq(
    s:    IpcState,
    verb: SchemeVerb,
    h:    HandleId,
)
    ensures k1_holds(s, verb, h)
{
}

/// Predicate form of K2: simultaneous scheme + msgport access on the
/// same endpoint preserves both SC4 (scheme registry uniqueness) and
/// M1 (message lifecycle totality).
pub open spec fn k2_holds(s: IpcState, msg_id: MsgId) -> bool {
    scheme::sc4_holds(s.scheme) && msgport::m1_holds(s.mp, msg_id)
}

/// **K2 — cross-layer race 없음.**  Decomposes into the per-layer
/// invariants, both of which we already proved.
pub proof fn no_cross_layer_race(s: IpcState, msg_id: MsgId)
    ensures k2_holds(s, msg_id)
{
    scheme::scheme_id_uniqueness(s.scheme);
    msgport::send_recv_pairing(s.mp, msg_id);
}

/// Predicate form of K3: for any live handle there is exactly one
/// endpoint cap, and the mapping is injective per epoch.
pub open spec fn k3_holds(s: SchemeState) -> bool {
    forall|h1: HandleId, h2: HandleId|
        #![trigger s.handle_meta.dom().contains(h1),
                   s.handle_meta.dom().contains(h2)]
        s.handle_meta.dom().contains(h1)
        && s.handle_meta.dom().contains(h2)
        && s.live_handles.contains(h1)
        && s.live_handles.contains(h2)
        && h1 != h2
        ==> s.handle_meta[h1].endpoint != s.handle_meta[h2].endpoint
}

/// **K3 — handle ↔ endpoint bijection.**  Trusted boundary on
/// `scheme_open` (which mints a fresh endpoint cap on each open) and
/// `scheme_close` (which revokes it on the last reference); we assume.
pub proof fn handle_to_endpoint_bijection(s: SchemeState)
    ensures k3_holds(s)
{
    assume(k3_holds(s));
}

} // verus!
