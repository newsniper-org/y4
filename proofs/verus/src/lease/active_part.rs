// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! `0xE14` XLATE_ACTIVE_PART readback classification — switch, boot binding
//! and same-value probe (`docs/hiu_abi.md` §2.4 step 3 · §2.5 step 6 · §2.7,
//! `docs/lease_capability.md` I7).
//!
//! WaveTensor canonical `b16fc07` `e-defined.typ:332` fixes the (A, D)
//! pair: every bus-accepted `0xE14` write updates `[8]` A (applied) and
//! `[9]` D (drain verdict, sampled on the write's accept edge, same-cycle
//! intake included); any rejection — value check or drain verdict — has
//! A = 0, raises `[6]` and leaves the binding unchanged, so A = 0 ∧ D = 1
//! means a value rejection.  For a write of a *different* value the table
//! is D = 1 → A = 1 · binding replaced, D = 0 → A = 0 · rejected.
//!
//! Y4 writes `0xE14` for three purposes, each classified here:
//!
//! - **switch** (§2.4 step 3) — next ≠ cur and next < `n_parts` (the real
//!   partition count `0xFF8 [7:0]`, §1.4): a `recommends` of
//!   `switch_outcome` and `after_readback`, and a `requires` of the switch
//!   path once `hiu/` lands.  For next = cur the canonical answer is A = 1 ·
//!   no-op · no fault with D = 0 reported, which this classification would
//!   misread as a contradiction — that write is the probe below.
//! - **boot binding** (§2.5 step 6, `boot_bind_outcome`) — p0 < `n_parts`
//!   but no next ≠ cur precondition, since the device binding is unknown.
//! - **same-value probe** (§2.7 forced drain window, `probe_outcome`) —
//!   the write of cur while bound to cur.
//!
//! Every write is an aligned 32-bit full-width store under the `hiu_cfg`
//! lock, read back before any other configuration access.  The value
//! bound is a Y4 precondition (canonical `e-defined.typ:418` 「호스트는 그
//! 수를 `0xFF8` HIU_SHAPE `[7:0]` 로 읽어 거절을 일으키지 않고 피한다.」):
//! a value rejection is a Y4 ledger defect, and a value rejection with
//! D = 0 reads the same as a drain rejection (`e-defined.typ:557` 「값이
//! 적법했는지는 쓴 쪽이 알고」) — only the precondition makes A = 0 ∧ D = 0 a
//! genuine drain rejection (`conforming_device_never_contradicts` exposes
//! both cases).  A drain rejection is retried within the bound and then
//! quarantines the device.
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

/// The canonical value check of a `0xE14` write as Y4 issues it (RMW form,
/// `[31:10]` = 0): the partition value names a real partition
/// (`e-defined.typ:414-418`).  `n_parts` = `0xFF8` HIU_SHAPE `[7:0]`.
pub open spec fn partition_value_ok(v: nat, n_parts: nat) -> bool {
    v < n_parts
}

/// Precondition of the switch path (§2.4 step 3): next ≠ cur, and both
/// name real partitions.
pub open spec fn switch_pre(cur: nat, next: nat, n_parts: nat) -> bool {
    &&& cur != next
    &&& partition_value_ok(cur, n_parts)
    &&& partition_value_ok(next, n_parts)
}

/// Classify the readback (`bind` = `[7:0]`, `a` = `[8]`, `d` = `[9]`) of a
/// switch write of `next` while bound to `cur`.  Precondition of the
/// switch path: `switch_pre(cur, next, n_parts)`.
pub open spec fn switch_outcome(
    bind: nat,
    a: bool,
    d: bool,
    cur: nat,
    next: nat,
    n_parts: nat,
) -> SwitchOutcome
    recommends
        switch_pre(cur, next, n_parts),
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
    n_parts: nat,
    f6_seen_within_bound: bool,
) -> SwitchAction
    recommends
        switch_pre(cur, next, n_parts),
{
    if fault6_expected(a) && !f6_seen_within_bound {
        SwitchAction::Fail(Y4Error::DeviceFault)
    } else {
        outcome_action(switch_outcome(bind, a, d, cur, next, n_parts))
    }
}

/// Each outcome holds exactly when its readback predicate holds, so the
/// four outcomes are mutually exclusive and cover every readback.
pub proof fn switch_outcome_exhaustive(
    bind: nat,
    a: bool,
    d: bool,
    cur: nat,
    next: nat,
    n_parts: nat,
)
    ensures
        (switch_outcome(bind, a, d, cur, next, n_parts) == SwitchOutcome::Applied) <==> (bind
            == next && a && d),
        (switch_outcome(bind, a, d, cur, next, n_parts) == SwitchOutcome::ValueRejected) <==> (!a
            && d && bind == cur),
        (switch_outcome(bind, a, d, cur, next, n_parts) == SwitchOutcome::DrainRejected) <==> (!a
            && !d && bind == cur),
        (switch_outcome(bind, a, d, cur, next, n_parts) == SwitchOutcome::DeviceContradiction)
            <==> !((bind == next && a && d) || (!a && d && bind == cur) || (!a && !d && bind
            == cur)),
{
}

/// A rejection outcome is one where `[6]` is expected.
pub proof fn rejection_implies_fault6_expected(
    bind: nat,
    a: bool,
    d: bool,
    cur: nat,
    next: nat,
    n_parts: nat,
)
    ensures
        switch_outcome(bind, a, d, cur, next, n_parts) == SwitchOutcome::ValueRejected
            || switch_outcome(bind, a, d, cur, next, n_parts) == SwitchOutcome::DrainRejected
            ==> fault6_expected(a),
{
}

/// An applied switch expects no `[6]`.
pub proof fn applied_implies_no_fault6(
    bind: nat,
    a: bool,
    d: bool,
    cur: nat,
    next: nat,
    n_parts: nat,
)
    ensures
        switch_outcome(bind, a, d, cur, next, n_parts) == SwitchOutcome::Applied
            ==> !fault6_expected(a),
{
}

/// A = 0 with no `[6]` inside the bound quarantines the device whatever
/// the rest of the readback says.
pub proof fn missing_fault6_is_device_fault(bind: nat, d: bool, cur: nat, next: nat, n_parts: nat)
    ensures
        after_readback(bind, false, d, cur, next, n_parts, false) == SwitchAction::Fail(
            Y4Error::DeviceFault,
        ),
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
/// when Y4 broke its own value precondition — value-rejected.  The D = 0
/// case is split to expose the assumption: with a bad value it still
/// classifies as a drain rejection — indistinguishable on the readback —
/// and only `switch_pre` excludes it (`switch_pre_excludes_value_rejection`).
pub proof fn conforming_device_never_contradicts(
    cur: nat,
    next: nat,
    n_parts: nat,
    value_ok: bool,
    d: bool,
)
    requires
        cur != next,
    ensures
        ({
            let r = device_readback(cur, next, value_ok, d);
            let o = switch_outcome(r.bind, r.a, r.d, cur, next, n_parts);
            &&& o != SwitchOutcome::DeviceContradiction
            &&& (value_ok && d) ==> o == SwitchOutcome::Applied
            &&& (value_ok && !d) ==> o == SwitchOutcome::DrainRejected
            &&& (!value_ok && !d) ==> o
                == SwitchOutcome::DrainRejected  // indistinguishable — the precondition excludes it
            &&& (!value_ok && d) ==> o == SwitchOutcome::ValueRejected
            &&& fault6_expected(r.a) <==> (o == SwitchOutcome::ValueRejected || o
                == SwitchOutcome::DrainRejected)
        }),
{
}

/// Under the switch precondition the device's value check passes, so a
/// value rejection never occurs and every D = 0 is a genuine drain rejection.
pub proof fn switch_pre_excludes_value_rejection(cur: nat, next: nat, n_parts: nat, d: bool)
    requires
        switch_pre(cur, next, n_parts),
    ensures
        ({
            let r = device_readback(cur, next, partition_value_ok(next, n_parts), d);
            let o = switch_outcome(r.bind, r.a, r.d, cur, next, n_parts);
            &&& o != SwitchOutcome::ValueRejected
            &&& d ==> o == SwitchOutcome::Applied
            &&& !d ==> o == SwitchOutcome::DrainRejected
        }),
{
}

// --- boot binding (§2.5 step 6) --------------------------------------------

/// Precondition of the boot binding: p0 names a real partition.  There is
/// no next ≠ cur precondition — the device binding is unknown at boot.
pub open spec fn boot_bind_pre(p0: nat, n_parts: nat) -> bool {
    partition_value_ok(p0, n_parts)
}

/// Classify the readback of the boot write `0xE14` ← p0 (§2.5 step 6).
/// `r[7:0] == p0 ∧ A ∧ D` is required — even a same-value write must report
/// D = 1 (drain observed after the flush edge).  A = 0 ∧ D = 1 ∧ `[7:0]` ≠ p0
/// is a value rejection (Y4 defect).  D = 0 with A = 0 ∧ `[7:0]` ≠ p0 (a
/// different-value drain rejection) or A = 1 ∧ `[7:0]` == p0 (a same-value
/// write that reports D = 0) re-drains from step 2b — the former reads as a
/// drain rejection only under `boot_bind_pre`.  A = 0 ∧ `[7:0]` == p0 and
/// A = 1 ∧ `[7:0]` ≠ p0 are contradictions (a same-value write has A = 1 and
/// a rejection leaves the binding unchanged, `e-defined.typ:332`).
pub open spec fn boot_bind_outcome(
    bind: nat,
    a: bool,
    d: bool,
    p0: nat,
    n_parts: nat,
) -> SwitchOutcome
    recommends
        boot_bind_pre(p0, n_parts),
{
    if bind == p0 && a && d {
        SwitchOutcome::Applied
    } else if !a && d && bind != p0 {
        SwitchOutcome::ValueRejected
    } else if (!a && !d && bind != p0) || (a && !d && bind == p0) {
        SwitchOutcome::DrainRejected
    } else {
        SwitchOutcome::DeviceContradiction
    }
}

/// The boot binding's four outcomes are exclusive and cover every readback.
pub proof fn boot_bind_outcome_exhaustive(bind: nat, a: bool, d: bool, p0: nat, n_parts: nat)
    ensures
        ({
            let o = boot_bind_outcome(bind, a, d, p0, n_parts);
            &&& (o == SwitchOutcome::Applied) <==> (bind == p0 && a && d)
            &&& (o == SwitchOutcome::ValueRejected) <==> (!a && d && bind != p0)
            &&& (o == SwitchOutcome::DrainRejected) <==> ((!a && !d && bind != p0) || (a && !d
                && bind == p0))
            &&& (o == SwitchOutcome::DeviceContradiction) <==> ((!a && bind == p0) || (a && bind
                != p0))
        }),
{
}

/// The `[6]` gate of the boot binding is the same as the switch's
/// (§2.5 step 6 defers to §2.4 step 3): A = 0 without `[6]` inside the bound
/// is a missing loud fault.
pub open spec fn boot_after_readback(
    bind: nat,
    a: bool,
    d: bool,
    p0: nat,
    n_parts: nat,
    f6_seen_within_bound: bool,
) -> SwitchAction
    recommends
        boot_bind_pre(p0, n_parts),
{
    if fault6_expected(a) && !f6_seen_within_bound {
        SwitchAction::Fail(Y4Error::DeviceFault)
    } else {
        outcome_action(boot_bind_outcome(bind, a, d, p0, n_parts))
    }
}

/// Against a conforming device whose prior binding `prev` is a real
/// partition, the boot binding under `boot_bind_pre` is applied when D = 1
/// and re-drains when D = 0 — never a contradiction, never a value
/// rejection.  `[6]` is expected exactly when A = 0.
pub proof fn conforming_device_boot_bind(prev: nat, p0: nat, n_parts: nat, d: bool)
    requires
        boot_bind_pre(p0, n_parts),
        partition_value_ok(prev, n_parts),
    ensures
        ({
            let r = device_readback(prev, p0, partition_value_ok(p0, n_parts), d);
            let o = boot_bind_outcome(r.bind, r.a, r.d, p0, n_parts);
            &&& d ==> o == SwitchOutcome::Applied
            &&& !d ==> o == SwitchOutcome::DrainRejected
            &&& fault6_expected(r.a) ==> o == SwitchOutcome::DrainRejected
        }),
{
}

// --- same-value probe (§2.7 forced drain window) ---------------------------

/// Classification of the same-value probe `0xE14` ← cur while bound to cur.
/// A same-value write is A = 1 · no-op · no fault, D reported either way
/// (`e-defined.typ:332` table), so Y4 expects no `[6]` from it.
#[derive(PartialEq, Eq, Structural)]
pub enum ProbeOutcome {
    /// `[7:0]` == cur ∧ A ∧ D — drained; resume intake.
    Resume,
    /// `[7:0]` == cur ∧ A ∧ ¬D — not drained now; bounded re-drain, exhaustion
    /// quarantines the device (`drain_retry_exhausted`).
    Redrain,
    /// Anything else (A = 0, or `[7:0]` ≠ cur) — not a response the canonical
    /// table allows for a same-value write.  `DeviceFault`.
    DeviceContradiction,
}

/// Classify the probe readback.  Precondition: cur is a real partition
/// (it is the device's current binding, established by §2.5 step 6 or §2.4).
pub open spec fn probe_outcome(bind: nat, a: bool, d: bool, cur: nat, n_parts: nat) -> ProbeOutcome
    recommends
        partition_value_ok(cur, n_parts),
{
    if bind == cur && a && d {
        ProbeOutcome::Resume
    } else if bind == cur && a && !d {
        ProbeOutcome::Redrain
    } else {
        ProbeOutcome::DeviceContradiction
    }
}

/// What Y4 does with a probe outcome.
pub open spec fn probe_action(o: ProbeOutcome) -> SwitchAction {
    match o {
        ProbeOutcome::Resume => SwitchAction::Proceed,
        ProbeOutcome::Redrain => SwitchAction::RetryDrain,
        ProbeOutcome::DeviceContradiction => SwitchAction::Fail(Y4Error::DeviceFault),
    }
}

/// The three probe outcomes are exclusive and cover every readback; the
/// probe never reaches `LeaseInvariant`.
pub proof fn probe_outcome_exhaustive(bind: nat, a: bool, d: bool, cur: nat, n_parts: nat)
    ensures
        ({
            let o = probe_outcome(bind, a, d, cur, n_parts);
            &&& (o == ProbeOutcome::Resume) <==> (bind == cur && a && d)
            &&& (o == ProbeOutcome::Redrain) <==> (bind == cur && a && !d)
            &&& (o == ProbeOutcome::DeviceContradiction) <==> !(bind == cur && a)
            &&& probe_action(o) != SwitchAction::Fail(Y4Error::LeaseInvariant)
        }),
{
}

/// Against a conforming device the probe resumes when D = 1 and re-drains
/// when D = 0; it never contradicts and never has A = 0, so no `[6]` is
/// expected.
pub proof fn conforming_device_probe(cur: nat, n_parts: nat, d: bool)
    requires
        partition_value_ok(cur, n_parts),
    ensures
        ({
            let r = device_readback(cur, cur, partition_value_ok(cur, n_parts), d);
            let o = probe_outcome(r.bind, r.a, r.d, cur, n_parts);
            &&& d ==> o == ProbeOutcome::Resume
            &&& !d ==> o == ProbeOutcome::Redrain
            &&& !fault6_expected(r.a)
        }),
{
}

} // verus!
