// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Lease capability specifications (`docs/lease_capability.md` I1–I7).
//!
//! The LeaseCap invariants themselves are deferred until `docs/hiu_abi.md`
//! is `v1.0 frozen`.  What lands here early are Y4-side preconditions and
//! attribution / classification specs whose need is already fixed by the
//! canonical text (WaveTensor `b16fc07` — the freeze sha, the citation basis of
//! `docs/hiu_abi.md` v1.0-rc4) regardless of the final surface:
//!
//! - `lbid` — LBID proxied-write check (`docs/hiu_abi.md` §2.8).
//! - `lnid` — LNID proxied-write check `lnid_write_ok` (`docs/hiu_abi.md`
//!   §1.2.1 (C) · §2.7 · §2.8, `03-hiu.typ:1070-1077` · `:1072` · `:1096`).
//! - `step` — step program load: FP_PROG / FP_BIND preconditions
//!   `fp_prog_ok` · `fp_bind_ok` · `fp_unbind_ok` against the closed four
//!   `[15]` sources (`fp_prog_refused15` · `fp_bind_refused15`), the `0xF0C`
//!   FP_SHAPE decode `fp_shape_verdict`, the image policy `fp_image_ok`, the
//!   slot ledger `SlotState` / `fp_mirror` (Unloaded → Loaded → Bound →
//!   Unbound → Quiesced), `fp_quiesce_action` and the FP gate `StepGate`
//!   (`docs/hiu_abi.md` §1.2.1 (A) 4 · (C) · §1.4 · §2.5 step 5 · §2.7,
//!   `e-defined.typ:208` · `:459` · `:461` · `:463` · `:464` · `:466`,
//!   `03-hiu.typ:1692`).
//! - `window` — global configuration window: the refusal set W = `[5]` ·
//!   `[6]` · `[7]` · `[15]`, pre-window disposition `dispose_pre_window` (가),
//!   post-window `window_verdict` (나) — only Y4-side evidence halts
//!   (`[5]` · `[6]`, or `[15]` with Y4's step ledger broken), `[7]` explained
//!   by a refused OWN_SET is `OwnSetRefused`, `[15]` under a consistent step
//!   ledger is unattributed → `DeviceFault` (`step_fault_verdict`, the `[19]`
//!   precedent) — and the OWN_SET/OWN_CLR readback `own_readback`
//!   (`docs/hiu_abi.md` §2.4 step 3 · §2.7 · §1.3, `e-defined.typ:527` ·
//!   `:1387` · `:1388`).
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
pub mod step;
pub mod window;
