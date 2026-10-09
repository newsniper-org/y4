// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Lease capability specifications (`docs/lease_capability.md` I1–I7).
//!
//! The LeaseCap invariants themselves are deferred until `docs/hiu_abi.md`
//! is `v1.0 frozen`.  What lands here early are Y4-side preconditions
//! whose need is already fixed by the canonical text regardless of the
//! final surface — today the LBID proxied-write check (`lbid`).

pub mod lbid;
