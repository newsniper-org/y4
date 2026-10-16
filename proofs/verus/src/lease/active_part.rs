// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! `0xE14` XLATE_ACTIVE_PART switch readback classification
//! (`docs/hiu_abi.md` §2.4 step 3, `docs/lease_capability.md` I7).
//!
//! WaveTensor canonical `e88f6f4` `e-defined.typ:332` fixes the (A, D)
//! pair: every bus-accepted `0xE14` write updates `[8]` A (applied) and
//! `[9]` D (drain verdict, sampled on the write's accept edge, same-cycle
//! intake included); any rejection — value check or drain verdict — has
//! A = 0, raises `[6]` and leaves the binding unchanged, so A = 0 ∧ D = 1
//! means a value rejection.  For a write of a *different* value the table
//! is D = 1 → A = 1 · binding replaced, D = 0 → A = 0 · rejected.
//!
//! Y4 writes `0xE14` only for a switch with next ≠ cur (a `recommends` of
//! `switch_outcome` and `after_readback` here, and a `requires` of the
//! switch path once `hiu/` lands: for next = cur the canonical answer is
//! A = 1 · no-op · no fault with D = 0 reported, which this classification
//! would misread as a contradiction; the boot binding of §2.5 step 6 has
//! no such precondition and is not classified here), as an aligned 32-bit
//! full-width store under the `hiu_cfg` lock, and reads it back before any
//! other configuration access.  Y4 never writes a value the device would
//! reject on content, so a value rejection is a Y4 ledger defect.  A value
//! rejection with D = 0 reads the same as a drain rejection; it is retried
//! within the bound and then quarantines the device.
//!
//! `[6]` is judged only after the readback (Y4 policy): when A = 0 Y4 polls
//! `0xF80[6]` = 1 within a platform-registry bound, attributes it to this
//! write and clears only `0xF84` `[6]`, before branching on the outcome.
//! The canonical text sets no cycle count for that visibility and Y4 takes
//! none as ABI.  A = 0 without `[6]` inside the bound is a missing loud
//! fault — the device's.  When A = 1 Y4 does not expect `[6]`.

use vstd::prelude::*;

use crate::error::Y4Error;

verus! {

/// Classification of one `0xE14` readback after a switch write.
#[derive(PartialEq, Eq, Structural)]
pub enum SwitchOutcome {
    /// `[7:0]` == next ∧ A ∧ D — the binding moved to next.
    Applied,
    /// ¬A ∧ D ∧ `[7:0]` == cur — value rejection: Y4 wrote a value the
    /// device rejects on content.
    ValueRejected,
    /// ¬A ∧ ¬D ∧ `[7:0]` == cur — drain-verdict rejection.
    DrainRejected,
    /// Any other combination (A ∧ `[7:0]` ≠ next, A ∧ ¬D, ¬A ∧ `[7:0]` ≠ cur)
    /// — not a response the canonical table allows for next ≠ cur.
    DeviceContradiction,
}

/// What Y4 does with a classified readback.
#[derive(PartialEq, Eq, Structural)]
pub enum SwitchAction {
    /// Resume intake for the new lease (§2.4 step 4).
    Proceed,
    /// Bounded re-drain retry from the drain loop (§2.4 step 2: drain loop,
    /// V=0 of denied VPNs, `0xE10`); exhaustion quarantines the device.
    RetryDrain,
    /// Stop with this error.
    Fail(Y4Error),
}

/// Classify the readback (`bind` = `[7:0]`, `a` = `[8]`, `d` = `[9]`) of a
/// switch write of `next` while bound to `cur`.  Precondition of the
/// switch path: `cur != next`.
pub open spec fn switch_outcome(bind: nat, a: bool, d: bool, cur: nat, next: nat) -> SwitchOutcome
    recommends
        cur != next,
{
    if bind == next && a && d {
        SwitchOutcome::Applied
    } else if !a && d && bind == cur {
        SwitchOutcome::ValueRejected
    } else if !a && !d && bind == cur {
        SwitchOutcome::DrainRejected
    } else {
        SwitchOutcome::DeviceContradiction
    }
}

/// Disposition of each outcome: a value rejection is a Y4 ledger defect
/// (`LeaseInvariant`); a drain rejection is retried within the bound in
/// v1.0 (the work source is external presentation and Y4 cannot stop it);
/// a contradiction quarantines the device (`DeviceFault`).
pub open spec fn outcome_action(o: SwitchOutcome) -> SwitchAction {
    match o {
        SwitchOutcome::Applied => SwitchAction::Proceed,
        SwitchOutcome::ValueRejected => SwitchAction::Fail(Y4Error::LeaseInvariant),
        SwitchOutcome::DrainRejected => SwitchAction::RetryDrain,
        SwitchOutcome::DeviceContradiction => SwitchAction::Fail(Y4Error::DeviceFault),
    }
}

/// Retry exhaustion of a drain rejection quarantines the whole device.
pub open spec fn drain_retry_exhausted() -> Y4Error {
    Y4Error::DeviceFault
}

/// Every rejection raises `[6]`; Y4 expects it exactly when A = 0.
pub open spec fn fault6_expected(a: bool) -> bool {
    !a
}

/// The `[6]` gate that runs right after the readback and before the
/// branch: A = 0 without `[6]` seen within the bound is a missing loud
/// fault (`DeviceFault`); otherwise the readback is classified.
pub open spec fn after_readback(
    bind: nat,
    a: bool,
    d: bool,
    cur: nat,
    next: nat,
    f6_seen_within_bound: bool,
) -> SwitchAction
    recommends
        cur != next,
{
    if fault6_expected(a) && !f6_seen_within_bound {
        SwitchAction::Fail(Y4Error::DeviceFault)
    } else {
        outcome_action(switch_outcome(bind, a, d, cur, next))
    }
}

/// Each outcome holds exactly when its readback predicate holds, so the
/// four outcomes are mutually exclusive and cover every readback.
pub proof fn switch_outcome_exhaustive(bind: nat, a: bool, d: bool, cur: nat, next: nat)
    ensures
        (switch_outcome(bind, a, d, cur, next) == SwitchOutcome::Applied) <==> (bind == next && a
            && d),
        (switch_outcome(bind, a, d, cur, next) == SwitchOutcome::ValueRejected) <==> (!a && d
            && bind == cur),
        (switch_outcome(bind, a, d, cur, next) == SwitchOutcome::DrainRejected) <==> (!a && !d
            && bind == cur),
        (switch_outcome(bind, a, d, cur, next) == SwitchOutcome::DeviceContradiction) <==> !(
        (bind == next && a && d) || (!a && d && bind == cur) || (!a && !d && bind == cur)),
{
}

/// A rejection outcome is one where `[6]` is expected.
pub proof fn rejection_implies_fault6_expected(bind: nat, a: bool, d: bool, cur: nat, next: nat)
    ensures
        switch_outcome(bind, a, d, cur, next) == SwitchOutcome::ValueRejected
            || switch_outcome(bind, a, d, cur, next) == SwitchOutcome::DrainRejected
            ==> fault6_expected(a),
{
}

/// An applied switch expects no `[6]`.
pub proof fn applied_implies_no_fault6(bind: nat, a: bool, d: bool, cur: nat, next: nat)
    ensures
        switch_outcome(bind, a, d, cur, next) == SwitchOutcome::Applied ==> !fault6_expected(a),
{
}

/// A = 0 with no `[6]` inside the bound quarantines the device whatever
/// the rest of the readback says.
pub proof fn missing_fault6_is_device_fault(bind: nat, d: bool, cur: nat, next: nat)
    ensures
        after_readback(bind, false, d, cur, next, false) == SwitchAction::Fail(Y4Error::DeviceFault),
{
}

/// One readback as the device reports it.
pub struct Readback {
    pub bind: nat,
    pub a: bool,
    pub d: bool,
}

/// The canonical (A, D) table (`e-defined.typ:332`) for a bus-accepted
/// aligned full-width write of `next` while bound to `cur`.  `value_ok` is
/// the value check; `d` is the drain verdict sampled on the accept edge.
pub open spec fn device_readback(cur: nat, next: nat, value_ok: bool, d: bool) -> Readback {
    if !value_ok {
        Readback { bind: cur, a: false, d: d }
    } else if next == cur {
        Readback { bind: cur, a: true, d: d }
    } else if d {
        Readback { bind: next, a: true, d: true }
    } else {
        Readback { bind: cur, a: false, d: false }
    }
}

/// Against a conforming device, a switch with next ≠ cur is never
/// classified as a contradiction: it is applied, drain-rejected, or — only
/// when Y4 broke its own value precondition — value-rejected.
pub proof fn conforming_device_never_contradicts(cur: nat, next: nat, value_ok: bool, d: bool)
    requires
        cur != next,
    ensures
        ({
            let r = device_readback(cur, next, value_ok, d);
            let o = switch_outcome(r.bind, r.a, r.d, cur, next);
            &&& o != SwitchOutcome::DeviceContradiction
            &&& (value_ok && d) ==> o == SwitchOutcome::Applied
            &&& (!d) ==> o == SwitchOutcome::DrainRejected
            &&& (!value_ok && d) ==> o == SwitchOutcome::ValueRejected
            &&& fault6_expected(r.a) <==> (o == SwitchOutcome::ValueRejected || o
                == SwitchOutcome::DrainRejected)
        }),
{
}

} // verus!
