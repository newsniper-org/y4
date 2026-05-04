// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Mirror of `proofs/verus/src/error.rs`.  Duplicated here pending the
//! shared `y4-error` crate that lands with `kernel/`.

/// Y4-wide error enum.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum Y4Error {
    /// Out of memory at the seL4 untyped or scudo backend.
    NoMemory,
    /// A capability slot was empty or wrong-type.
    BadCap,
    /// A blocking call did not complete within its bound.
    Timeout,
    /// Caller violated a precondition.
    InvalidArg,
    /// Lease invariant broke — paired with hypervisor panic.
    LeaseInvariant,
    /// Allocator detected a security-relevant violation.
    SecurityViolation,
}
