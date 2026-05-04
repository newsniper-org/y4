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

    /// Lease invariant broke (partition disjointness, nonce reuse,
    /// shadow-slot collision).  Always paired with a panic in the
    /// hypervisor — the variant exists for spec clients that observe
    /// the failure model.
    LeaseInvariant,

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
            || e == Y4Error::SecurityViolation,
{
}

} // verus!
