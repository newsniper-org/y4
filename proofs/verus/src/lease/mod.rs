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
//! - `lnid` — LNID proxied-write check `lnid_write_ok` and step-load checks
//!   `fp_prog_ok` · `fp_bind_ok` (`docs/hiu_abi.md` §1.2.1 (C) · §2.7 · §2.8,
//!   `03-hiu.typ:1070-1077` · `:1096`, `e-defined.typ:459` · `:461`).
//! - `window` — global configuration window: the refusal set W = `[5]` ·
//!   `[6]` · `[7]` · `[15]`, pre-window disposition `dispose_pre_window` (가),
//!   post-window `window_verdict` (나) — only `[5]` · `[6]` halt — and the
//!   OWN_SET/OWN_CLR readback `own_readback` and FP gate `fp_gate_binds`
//!   (`docs/hiu_abi.md` §2.4 step 3 · §2.7 · §1.3).
//! - `nonce` — `FAULT_STATUS[12]` origin attribution against Y4's nonce
//!   ledger (`docs/hiu_abi.md` §2.6, `e-defined.typ:345` · `:347`).
//! - `active_part` — `0xE14` readback (A, D) classification for the switch
//!   (`switch_outcome`, next ≠ cur · next < `n_parts`), the boot binding
//!   (`boot_bind_outcome`, p0 < `n_parts`) and the same-value probe
//!   (`probe_outcome`), and the `[6]` gate (`docs/hiu_abi.md` §2.4 step 3 ·
//!   §2.5 step 6 · §2.7, `e-defined.typ:332`).

pub mod active_part;
pub mod lbid;
pub mod lnid;
pub mod nonce;
pub mod window;
