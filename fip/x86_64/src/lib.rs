// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-x86_64 — FIP 매크로, x86_64 특화
//!
//! [`y4_fip_core`] 의 FIP 계약을 **x86_64** 용으로 특화한다 (compiler-guideline
//! 튜닝).  seL4 `KernelSel4Arch=x86_64` 대응.  이 크레이트를 **x86_64 를 타깃할
//! 때** 사용한다.
//!
//! P3(hw_mechanism_abstraction): 계약은 `y4-fip-core` 가 공유, realization 은
//! 본 subcrate 의 [`arch`] 모듈.  설계: `docs/design_decisions.md` §2.1.
//!
//! 본 크레이트는 arch intrinsic(prefetch) 때문에 **narrow audited unsafe** 를
//! 허용한다(capsule §2.1 unsafe-audit gate — arch shim).  `y4-fip-core` 는
//! forbid-unsafe 유지.

#![no_std]

pub use y4_fip_core::*;

pub mod arch {
    //! x86_64 compiler-guideline 튜닝 파라미터·helper.  consts 는 어느
    //! 호스트에서도 읽을 수 있고, [`prefetch_read`] 의 intrinsic 은 x86_64
    //! 타깃에서만 emit 된다.

    /// arch 이름.
    pub const ARCH: &str = "x86_64";

    /// 전형적 L1 cache line 크기(bytes).  x86_64 = 64.
    pub const CACHE_LINE: usize = 64;

    /// 한 cache line 에 담기는 `T` 개수 — cache-friendly FIP batch 크기
    /// ([`y4_fip_core::compact`] 등 hot 루프의 배치 단위).  ZST 는 `usize::MAX`.
    #[must_use]
    pub const fn per_cache_line<T>() -> usize {
        match CACHE_LINE.checked_div(core::mem::size_of::<T>()) {
            Some(n) => n,
            None => usize::MAX,
        }
    }

    /// read prefetch hint — hot in-place 루프에서 다음 원소를 미리 당긴다.
    /// x86_64 에서 `PREFETCHT0` 한 명령으로 인라인; 그 외 타깃에선 no-op.
    ///
    /// `core::arch::x86_64::_mm_prefetch` 를 쓰지 않는다 — rustc 1.98 에서 그
    /// intrinsic 은 `#[target_feature(enable = "sse")]` safe fn 이라, SSE 를 끈
    /// 커널 타깃(`x86_64-unknown-none`)에서는 인라인되지 못하고 힌트마다
    /// out-of-line 호출이 된다.  `PREFETCHT0` 자체는 x86-64 기본 명령이라
    /// target feature 가 필요 없으므로 `asm!` 로 직접 낸다.
    #[inline(always)]
    pub fn prefetch_read<T>(p: *const T) {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: `PREFETCHT0` 은 x86-64 기본 ISA 의 순수 힌트다 — 아키텍처상
        // 어떤 주소에 대해서도 fault·예외를 내지 않고 메모리·레지스터·플래그
        // 상태를 바꾸지 않는다(SDM Vol. 2 PREFETCHh).  따라서 임의 raw 포인터로
        // 실행해도 안전하며, `nostack`·`preserves_flags`·`readonly` 가 그 사실을
        // 컴파일러에 알린다.
        unsafe {
            core::arch::asm!(
                "prefetcht0 [{0}]",
                in(reg) p,
                options(nostack, preserves_flags, readonly),
            );
        }
        #[cfg(not(target_arch = "x86_64"))]
        let _ = p;
    }
}
