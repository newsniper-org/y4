// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! AMD-V (SVM) safety invariants for D1d.
//!
//! Spec source of truth: [`docs/amdv_safety.md`](../../../docs/amdv_safety.md).
//! 13 invariants AV1–AV13, one per safety constraint S1–S13 in the spec.
//!
//! Bodies are placeholders until the seL4 raw-SVM patch + `y4-hypercall`
//! Rust impl land — at which point each invariant is filled the same way
//! `alloc/refinement.rs` and `ipc/refinement.rs` were filled (executable
//! state model + inductive proof).

use vstd::prelude::*;

verus! {

/// Top-level invariant: a vmrun is safe iff all S1–S13 invariants hold
/// at the moment of the syscall.  Once filled, callers can invoke this
/// as the single proof obligation guarding any `seL4_X86_SVMVCPU_Run`.
pub proof fn vmrun_safe()
    ensures true,
{
}

} // verus!
