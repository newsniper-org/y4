// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Mirror of `proofs/verus/src/error.rs` — the Y4-wide error enum.
//!
//! Once `kernel/` lands the canonical Rust home for `Y4Error`, this
//! local copy will move to a `y4-error` workspace crate consumed by
//! both `y4-alloc` and `y4-ipc`.  Keeping a duplicate here for now
//! avoids a circular workspace dependency before that crate exists.

/// Y4-wide error enum.  Variants and intent match the Verus spec
/// (`proofs/verus/src/error.rs`); changes must update both sides.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum Y4Error {
    /// Out of memory at the seL4 untyped or scudo backend.
    NoMemory,
    /// A capability slot was empty or wrong-type.
    BadCap,
    /// A blocking call did not complete within its bound.
    Timeout,
    /// Caller violated a precondition (bad layout, double-free, etc).
    InvalidArg,
    /// Lease invariant broke — paired with hypervisor panic.
    LeaseInvariant,
    /// Allocator detected a security-relevant violation.
    SecurityViolation,
}
