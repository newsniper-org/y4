// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-x86_64 — FIP 매크로, x86_64 특화
//!
//! [`y4_fip_core`] 의 FIP 계약을 **x86_64** 용으로 특화한다 (compiler-guideline
//! 튜닝 자리).  seL4 `KernelSel4Arch=x86_64` 대응.
//!
//! P3(hw_mechanism_abstraction): 계약은 `y4-fip-core` 가 공유, realization 은
//! 본 subcrate 가 arch-specific 으로 제공.  설계: `docs/design_decisions.md` §2.1.
//!
//! **scaffold** — 실제 arch 튜닝은 설계 진행 중.

#![no_std]
#![forbid(unsafe_code)]

pub use y4_fip_core::*;

#[cfg(target_arch = "x86_64")]
pub mod arch {
    //! x86_64 compiler-guideline 튜닝 (no-alloc hot 경로용 arch-specific
    //! helper 자리) — scaffold.
}
