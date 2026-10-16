// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Lease capability specifications (`docs/lease_capability.md` I1–I7).
//!
//! The LeaseCap invariants themselves are deferred until `docs/hiu_abi.md`
//! is `v1.0 frozen`.  What lands here early are Y4-side preconditions and
//! attribution / classification specs whose need is already fixed by the
//! canonical text (WaveTensor `e88f6f4`, the v1.0 freeze candidate)
//! regardless of the final surface:
//!
//! - `lbid` — LBID proxied-write check (`docs/hiu_abi.md` §2.8).
//! - `nonce` — `FAULT_STATUS[12]` origin attribution against Y4's nonce
//!   ledger (`docs/hiu_abi.md` §2.6, `e-defined.typ:345` · `:347`).
//! - `active_part` — `0xE14` switch readback (A, D) classification and the
//!   `[6]` gate (`docs/hiu_abi.md` §2.4 step 3, `e-defined.typ:332`).

pub mod active_part;
pub mod lbid;
pub mod nonce;
