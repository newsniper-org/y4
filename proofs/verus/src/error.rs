// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! `Y4Error` — single error enum shared by every Y4 subsystem (C3 in
//! `MEMORY/y4_ipc_alloc_preflight.md`).
//!
//! The enum is the canonical return type for every fallible Y4 path:
//! cap invocations, alloc requests, IPC sends, lease ops, etc.  Any
//! seL4 capability invocation that fails maps into one of the variants
//! below; a missing variant means the failure mode is currently
//! unspecified — open a PR to add it rather than reusing a near-miss.

use vstd::prelude::*;

verus! {

/// Y4-wide error enum.  Variants are ordered by "how often this fires"
/// (NoMemory first because alloc paths are the densest source of errors).
#[derive(PartialEq, Eq, Structural)]
pub enum Y4Error {
    /// `seL4_NoError` after `seL4_Untyped_Retype` failed for lack of
    /// untyped capacity, or scudo / SLAB front-end ran out of memory.
    NoMemory,

    /// A capability slot was empty, the wrong type, or revoked between
    /// lookup and use.  Includes IPC endpoint cap mismatches.
    BadCap,

    /// A blocking syscall (Recv / Wait / shared-frame map reservation)
    /// did not complete within its caller-specified bound.
    Timeout,

    /// The caller violated a precondition the kernel surface or Y4
    /// invariant requires (e.g. mapping a non-frame cap, double-free,
    /// scheme path with bad encoding).
    InvalidArg,

    /// Lease invariant broke *on the Y4 side* (partition disjointness,
    /// (key, nonce) reuse — two mask-valid partitions sharing a nonce, or
    /// a rekeyed partition given the nonce of a mask-valid other partition
    /// by Y4 — `FAULT_STATUS[12]` attributed to `FaultOrigin::Y4Intent`
    /// or `Y4Procedure` — translation entry written outside the lease's
    /// partition window, a `0xE14` switch or boot-binding value rejection —
    /// `lease::active_part::SwitchOutcome::ValueRejected` — or a
    /// global-configuration-window write refused with `FAULT_STATUS[5]` or
    /// `[6]` — `lease::window::WindowVerdict::Y4Defect` — or a step fault
    /// `[15]` while Y4's own step ledger is inconsistent
    /// (`step_fault_origin` → `FaultOrigin::Y4Intent`, the `[19]` precedent);
    /// `[7]` never maps here, and a `[15]` under a consistent step ledger
    /// does not either — `lease::window::window_halts_only_on_y4_evidence` ·
    /// `lease::step::step_path_halts_only_on_ledger`).
    /// Y4's own verified bookkeeping is wrong, so it is always paired with
    /// a panic in the hypervisor — the variant exists for spec clients that
    /// observe the failure model.
    LeaseInvariant,

    /// The accelerator reported a state that Y4's verified bookkeeping
    /// did not produce (e.g. `FAULT_STATUS[12]` while Y4's nonce ledger
    /// shows no collision — none among mask-valid partitions, none with
    /// the rekeyed partition's new nonce — and every captured partition
    /// written by Y4 this boot — `FaultOrigin::DeviceSide`), or the origin
    /// cannot be attributed (`FaultOrigin::Unattributed`), or a `0xE14` readback
    /// the canonical (A, D) table does not allow / a rejection without
    /// `[6]` inside the bound / a same-value probe that is not A = 1 with the
    /// binding unchanged (`lease::active_part`), or a window the device
    /// contradicted — `[7]` not explained by a refused `OWN_SET` readback of
    /// the same window, an `OWN_SET` refused on readback without `[7]`
    /// (silent refusal), an `OWN_CLR` still device-owned on readback
    /// (`lease::window::WindowVerdict::DeviceContradiction`,
    /// `OwnReadback::Contradiction`) — or a step fault `[15]` under a
    /// consistent step ledger (`step_fault_origin` → `Unattributed`, user
    /// decision 2026-10-22; the FP gate included — `lease::step::StepGate`),
    /// or a `0xF0C` FP_SHAPE
    /// word that contradicts major 1 or the platform registry
    /// (`lease::step::FpShapeVerdict::Contradiction` — encoding number ≠ 1,
    /// P = 0, P or stepMax > 256, registry mismatch — user decision U2).  Paired with *device*
    /// quarantine — every lease on that HIU revoked and the device marked
    /// faulty — not with a hypervisor panic: Y4's state is consistent, so
    /// other devices and guests keep running.  The device is reset only
    /// through a platform-attested reset path (device state is snapshotted
    /// first); without one it stays out of service until a host reboot or
    /// an attested reset (user decision 2026-10-09).  A host reboot is not
    /// a device reset, so a persistent device defect may be re-admitted
    /// after reboot and re-detected by the same attribution.
    /// `docs/hiu_abi.md` §2.6.
    DeviceFault,

    /// Allocator reported a security-relevant violation (use-after-free
    /// detected by scudo, guard-page write, randomization disabled).
    /// Distinct from InvalidArg because the invariant is enforced at
    /// the alloc boundary, not the call site.
    SecurityViolation,
}

/// Specification-only: every Y4Error value is *observable* —
/// no variant is a placeholder for "unknown".  Callers can pattern-
/// match exhaustively without a wildcard arm.
pub proof fn y4error_is_total(e: Y4Error)
    ensures
        e == Y4Error::NoMemory
            || e == Y4Error::BadCap
            || e == Y4Error::Timeout
            || e == Y4Error::InvalidArg
            || e == Y4Error::LeaseInvariant
            || e == Y4Error::DeviceFault
            || e == Y4Error::SecurityViolation,
{
}

/// Where a device-observed lease fault came from (`docs/hiu_abi.md` §2.6).
/// Decided by comparing the device report against Y4's nonce ledger.
#[derive(PartialEq, Eq, Structural)]
pub enum FaultOrigin {
    /// The ledger itself holds two mask-valid partitions with the same
    /// nonce, or the rekeyed partition's new nonce equals a mask-valid
    /// other partition's ledger nonce (canonical `b16fc07`
    /// `e-defined.typ:345`) — Y4's allocator / bookkeeping broke
    /// (`lease::nonce::nonce_dup_origin` step 2).
    Y4Intent,
    /// The colliding partition had no key/nonce written by Y4 this boot —
    /// Y4 skipped or mis-ran the boot neutralisation (`docs/hiu_abi.md`
    /// §2.5 step 4: every partition's key and nonce written, mask validity
    /// dropped and checked) or wrote without updating the ledger.
    Y4Procedure,
    /// The ledger is consistent and was written this boot, yet the device
    /// reports a collision — write path, storage or comparator fault.
    DeviceSide,
    /// The ledger is consistent but the device report carries no capture
    /// that would split `Y4Procedure` from `DeviceSide`.  Not reachable for
    /// `[12]` on a v1.0 device (canonical `b16fc07` `e-defined.typ:347`
    /// invariant `[16]` ⇔ `[12]`, so `[12]` with `[16]` = 0 is a device
    /// contract violation — `DeviceSide`); it is the origin of a blocked non-owned frame access
    /// under a consistent ownership ledger (`nonowned_access_origin`).
    Unattributed,
}

/// Disposition of an attributed fault.  Y4-side origins panic the
/// hypervisor; device-side and unattributed origins quarantine the device.
pub open spec fn fault_disposition(o: FaultOrigin) -> Y4Error {
    match o {
        FaultOrigin::Y4Intent => Y4Error::LeaseInvariant,
        FaultOrigin::Y4Procedure => Y4Error::LeaseInvariant,
        FaultOrigin::DeviceSide => Y4Error::DeviceFault,
        FaultOrigin::Unattributed => Y4Error::DeviceFault,
    }
}

/// Origin of a blocked non-owned frame access — FAULT_STATUS `[19]`, which
/// WaveTensor split out of `[4]` at Y4's request before the v1.0 freeze
/// (canonical `b16fc07` `e-defined.typ:523`; `docs/hiu_abi.md` §1.3).  The
/// entitlement check comes first, so Y4's deny-fill entries raise `[4]` only.  A tenant cannot cause it: Y4 maps
/// only frames whose `OWN_SET` the `OWN_STATUS` readback confirmed device-owned
/// (canonical E.1.1 host-interpretation rule 5, `e-defined.typ:208`;
/// `lease::window::own_readback` · `ledger_device_owned_only_if_confirmed` — a
/// refused `OWN_SET` (E.7 item 6, `e-defined.typ:1387`) leaves the frame
/// host-owned in the ledger and unmapped).
/// If Y4's ownership ledger itself is
/// inconsistent (a frame in some `dva_map` or `pending_reclaim` that the
/// ledger does not hold device-owned) the origin is Y4's; otherwise the
/// device gives no capture, so Y4 procedure vs device stays undecided.
pub open spec fn nonowned_access_origin(ownership_ledger_consistent: bool) -> FaultOrigin {
    if ownership_ledger_consistent {
        FaultOrigin::Unattributed
    } else {
        FaultOrigin::Y4Intent
    }
}

/// A non-owned access panics the hypervisor exactly when Y4's ownership
/// ledger is inconsistent; with a consistent ledger it quarantines the
/// device.  (That it is never reported to the tenant is the disposition
/// table's choice in `docs/hiu_abi.md` §1.3, not a property proved here.)
pub proof fn nonowned_access_panics_iff_ledger_broken(c: bool)
    ensures
        (fault_disposition(nonowned_access_origin(c)) == Y4Error::LeaseInvariant) <==> !c,
        (fault_disposition(nonowned_access_origin(c)) == Y4Error::DeviceFault) <==> c,
{
}

/// Origin of a step-program fault — FAULT_STATUS `[15]` (canonical `b16fc07`
/// `e-defined.typ:464`; `docs/hiu_abi.md` §1.3, user decision 2026-10-22 —
/// the `[19]` precedent).  Y4 avoids sources (1)–(3) before writing
/// (`lease::step::fp_prog_ok` · `fp_bind_ok`), and source (4) runaway is
/// excluded by the bind precondition and the reload order Y4 keeps
/// (`e-defined.typ:208` · `:466` — "a `[15]` it sees is a load/bind duty
/// violation on its own side (driver · platform) or another own-side duty
/// violation").  Tenants supply no images (U1) and Y4 is the only writer, so
/// a tenant cannot cause it.  If Y4's step ledger is itself inconsistent
/// (`lease::step::fp_mirror` / `fp_ledger_wf` broken, or a write in this
/// reset generation that broke its precondition), the fault is Y4's intent
/// defect.  With a consistent ledger, Y4's procedure, the platform (a device
/// reset Y4 was not told of) and a non-conforming device cannot be told
/// apart — unattributed.
pub open spec fn step_fault_origin(step_ledger_consistent: bool) -> FaultOrigin {
    if step_ledger_consistent {
        FaultOrigin::Unattributed
    } else {
        FaultOrigin::Y4Intent
    }
}

/// A step fault panics the hypervisor exactly when Y4's step ledger is
/// inconsistent; with a consistent ledger it quarantines the device.
pub proof fn step_fault_panics_iff_ledger_broken(c: bool)
    ensures
        (fault_disposition(step_fault_origin(c)) == Y4Error::LeaseInvariant) <==> !c,
        (fault_disposition(step_fault_origin(c)) == Y4Error::DeviceFault) <==> c,
{
}

/// A hypervisor panic (`LeaseInvariant`) is reached exactly when the
/// origin is Y4's own; a device-side or unattributed origin never panics.
pub proof fn fault_disposition_separates_origin(o: FaultOrigin)
    ensures
        (fault_disposition(o) == Y4Error::LeaseInvariant)
            <==> (o == FaultOrigin::Y4Intent || o == FaultOrigin::Y4Procedure),
        (fault_disposition(o) == Y4Error::DeviceFault)
            <==> (o == FaultOrigin::DeviceSide || o == FaultOrigin::Unattributed),
{
}

} // verus!
