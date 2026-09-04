// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-aarch64 — FIP 매크로, AArch64 특화
//!
//! [`y4_fip_core`] 의 FIP 계약을 **AArch64**(little-endian) 용으로 특화한다
//! (compiler-guideline 튜닝).  seL4 `KernelSel4Arch=aarch64` 대응.  이 크레이트를
//! **AArch64 를 타깃할 때** 사용한다.
//!
//! P3(hw_mechanism_abstraction): 계약은 `y4-fip-core` 가 공유, realization 은
//! 본 subcrate 의 [`arch`] 모듈.  설계: `docs/design_decisions.md` §2.1.

#![no_std]
#![forbid(unsafe_code)]

pub use y4_fip_core::*;

pub mod arch {
    //! AArch64 compiler-guideline 튜닝 파라미터·helper.

    /// arch 이름.
    pub const ARCH: &str = "aarch64";

    /// 전형적 L1 cache line 크기(bytes).  ARMv8 통상 = 64 (Apple M-계열·일부
    /// Cavium 은 128 — 플랫폼별 override 대상).
    pub const CACHE_LINE: usize = 64;

    /// 한 cache line 에 담기는 `T` 개수 — cache-friendly FIP batch 크기.
    /// ZST 는 `usize::MAX`.
    #[must_use]
    pub const fn per_cache_line<T>() -> usize {
        match CACHE_LINE.checked_div(core::mem::size_of::<T>()) {
            Some(n) => n,
            None => usize::MAX,
        }
    }

    /// read prefetch hint — 현재 **no-op**.  AArch64 `PRFM PLDL1KEEP` 을 `asm!`
    /// 로 emit 하는 실제 튜닝은 후속(안전 wrapper + audited unsafe).
    #[inline(always)]
    pub fn prefetch_read<T>(p: *const T) {
        let _ = p;
    }
}
