// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! AMD-V (SVM) safety invariants for D1d.
//!
//! Spec source of truth: [`docs/amdv_safety.md`](../../../docs/amdv_safety.md).
//! AV1~AV20 + AV2-D = 21 invariant, 4 cluster:
//! - amdv lower (12 AV): intercept_floor / deadline / gif / tsc / nested /
//!   audit (AV11+12+13 shared) / vmcb_whitelist / cluster_dep / boundary /
//!   dispatch
//! - amdv upper (9 AV): npt (AV2+AV2-D shared) / cpu_pin / thread_group /
//!   bitmap_immut (AV9+10 shared) / lifetime (AV14+15 shared) / firmware
//!
//! 4-cluster file layout: av-proof-body-tracker §2.
//!
//! Bodies are filled per R7.11 milestone — first emit = AV1
//! `intercept_floor_holds` (Cluster 1 amdv lower PR-2a, 2026-06-03).
//! Subsequent bodies follow `av-proof-body-tracker.md` §5 의 6 step
//! (verify-adsmt → emit-isabelle → emit-rocq → cross-check).

pub mod lower;
// pub mod upper;  // Cluster 2 (amdv upper) — av-proof-body-tracker §6

use vstd::prelude::*;

verus! {

/// Top-level invariant: a vmrun is safe iff all AV1–AV20 + AV2-D
/// invariants hold at the moment of the syscall.  Once filled, callers
/// can invoke this as the single proof obligation guarding any
/// `seL4_X86_SVMVCPU_Run`.
pub proof fn vmrun_safe()
    ensures true,
{
}

} // verus!
