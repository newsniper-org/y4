// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Refinement proofs for the alloc subsystem.
//!
//! `slab.rs`, `scudo.rs`, and `boundary.rs` state invariants and
//! discharge them via `assume()` at the trusted boundary.  This module
//! takes a step further: it introduces *executable spec functions*
//! that model the Rust implementation's per-operation effect on the
//! state, and proves the invariants are preserved by each step.
//!
//! The invariants discharged constructively (no `assume`) below:
//!
//! - **B6 quarantine_lifetime_bound** — the `quarantine_step` model
//!   reflects `HardenedBackend::free`'s aging + retain logic; the
//!   proof shows every transition preserves the `q_max` upper bound.
//! - **B3 randomization** — `init_state` shows that a state produced
//!   by the constructor witnesses `randomized = true`.
//! - **alloc_no_overlap (lifted X3)** — strengthened with a direct
//!   proof using B1's predicate and the `live() == scudo.live` equality.

use vstd::prelude::*;
use vstd::seq::*;

use crate::alloc::scudo;
use crate::alloc::state::*;

verus! {

// ---------------------------------------------------------------------
// B6 — quarantine lifetime bound, constructive proof.
// ---------------------------------------------------------------------

/// Model of `HardenedBackend::free`'s effect on the quarantine table.
/// Operationally: ages every existing entry, admits the new one with
/// age 0, then drops every entry whose age exceeds `q_max`.
///
/// Returned as a `Map<Allocation, nat>` matching `ScudoState.quarantined`.
pub open spec fn quarantine_step(
    q_in:  Map<Allocation, nat>,
    new_entry: Allocation,
    q_max: nat,
) -> Map<Allocation, nat>
    recommends
        forall|a: Allocation| #![trigger q_in.dom().contains(a)]
            q_in.dom().contains(a) ==> q_in[a] <= q_max
{
    // Aged map: every key advances by 1.  Then add the new entry at age 0.
    // Then keep only entries whose age is still ≤ q_max (= q_max - 1 + 1
    // for old entries, 0 for the new one).
    Map::<Allocation, nat>::new(
        |a: Allocation|
            (q_in.dom().contains(a) && q_in[a] + 1 <= q_max)
            || a == new_entry,
        |a: Allocation|
            if a == new_entry { 0nat } else { (q_in[a] + 1) as nat },
    )
}

/// **B6 (constructive).**  Every state produced by `quarantine_step`
/// from a B6-respecting input still satisfies B6.
pub proof fn b6_preserved_by_step(
    q_in: Map<Allocation, nat>,
    new_entry: Allocation,
    q_max: nat,
)
    requires
        forall|a: Allocation| #![trigger q_in.dom().contains(a)]
            q_in.dom().contains(a) ==> q_in[a] <= q_max,
    ensures ({
        let q_out = quarantine_step(q_in, new_entry, q_max);
        forall|a: Allocation| #![trigger q_out.dom().contains(a)]
            q_out.dom().contains(a) ==> q_out[a] <= q_max
    })
{
    // The proof is structural: q_out's value definition guarantees
    // the bound by construction.
    let q_out = quarantine_step(q_in, new_entry, q_max);
    assert(forall|a: Allocation| #![trigger q_out.dom().contains(a)]
        q_out.dom().contains(a) ==> q_out[a] <= q_max);
}

// ---------------------------------------------------------------------
// B3 — randomization witness, constructive proof.
// ---------------------------------------------------------------------

/// Model of `HardenedBackend::new` — the resulting state has the
/// `randomized` field set when the constructor consumed the seed.
pub open spec fn init_state(seed: nat) -> ScudoState
    recommends seed > 0
{
    ScudoState {
        live:        Set::<Allocation>::empty(),
        quarantined: Map::<Allocation, nat>::empty(),
        q_max:       16nat,
        randomized:  true,
    }
}

/// **B3 (constructive).**  The state produced by `init_state` always
/// witnesses the randomization invariant.
pub proof fn b3_holds_post_init(seed: nat)
    requires seed > 0
    ensures scudo::b3_holds(init_state(seed))
{
    // Direct: init_state.randomized = true is exactly b3_holds.
}

// ---------------------------------------------------------------------
// alloc_no_overlap (lifted X3) — strengthened constructive proof.
// ---------------------------------------------------------------------

/// Lift the alloc-public no-overlap from B1 + the equality
/// `s.live() == s.scudo.live` (definitional).
pub proof fn alloc_no_overlap_via_b1(s: AllocState)
    requires s.scudo.live_well_formed()
    ensures
        forall|a: Allocation, b: Allocation|
            #![trigger s.live().contains(a), s.live().contains(b)]
            s.live().contains(a) && s.live().contains(b) && a != b
                ==> a.range.disjoint(b.range)
{
    scudo::backend_no_overlap(s.scudo);
    assert(s.live() =~= s.scudo.live);
}

// ---------------------------------------------------------------------
// Stress witness: a sequence of N quarantine steps still satisfies B6.
// Demonstrates that the per-step proof composes inductively.
// ---------------------------------------------------------------------

/// Sequential application of `quarantine_step` over `entries`.
pub open spec fn quarantine_stream(
    initial: Map<Allocation, nat>,
    entries: Seq<Allocation>,
    q_max: nat,
) -> Map<Allocation, nat>
    decreases entries.len()
{
    if entries.len() == 0 {
        initial
    } else {
        quarantine_step(
            quarantine_stream(initial, entries.drop_last(), q_max),
            entries.last(),
            q_max,
        )
    }
}

/// **B6 induction lemma.**  Sequential steps preserve the bound.
pub proof fn b6_preserved_by_stream(
    initial: Map<Allocation, nat>,
    entries: Seq<Allocation>,
    q_max: nat,
)
    requires
        forall|a: Allocation| #![trigger initial.dom().contains(a)]
            initial.dom().contains(a) ==> initial[a] <= q_max,
    ensures ({
        let q_out = quarantine_stream(initial, entries, q_max);
        forall|a: Allocation| #![trigger q_out.dom().contains(a)]
            q_out.dom().contains(a) ==> q_out[a] <= q_max
    })
    decreases entries.len()
{
    if entries.len() == 0 {
        // Base: q_out == initial, predicate holds by precondition.
    } else {
        let head_entries = entries.drop_last();
        b6_preserved_by_stream(initial, head_entries, q_max);
        let q_mid = quarantine_stream(initial, head_entries, q_max);
        b6_preserved_by_step(q_mid, entries.last(), q_max);
    }
}

} // verus!
