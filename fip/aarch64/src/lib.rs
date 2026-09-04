// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-aarch64 — FIP 매크로, AArch64 특화
//!
//! [`y4_fip_core`] 의 FIP 계약을 **AArch64**(little-endian) 용으로 특화한다
//! (compiler-guideline 튜닝 자리).  seL4 `KernelSel4Arch=aarch64` 대응.
//!
//! P3(hw_mechanism_abstraction): 계약은 `y4-fip-core` 가 공유, realization 은
//! 본 subcrate 가 arch-specific 으로 제공.  설계: `docs/design_decisions.md` §2.1.
//!
//! **scaffold** — 실제 arch 튜닝은 설계 진행 중.

#![no_std]
#![forbid(unsafe_code)]

pub use y4_fip_core::*;

#[cfg(target_arch = "aarch64")]
pub mod arch {
    //! AArch64 compiler-guideline 튜닝 (no-alloc hot 경로용 arch-specific
    //! helper 자리) — scaffold.
}
