// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-x86_64 — FIP 매크로, `x86_64` 특화
//!
//! [`y4_fip_core`] 의 FIP 계약을 **`x86_64`** 용으로 특화한다 (compiler-guideline
//! 튜닝).  seL4 `KernelSel4Arch=x86_64` 대응.  이 크레이트를 **`x86_64` 를 타깃할
//! 때** 사용한다.
//!
//! P3(`docs/hw_mechanism_abstraction.md` §2 「trait 뒤의 arch-neutral 로직」 · §1 규칙 1):
//! arch-neutral 조합자와 Y4-FIP 정의는 `y4-fip-core` 에 있고(재수출), 이 크레이트의
//! [`arch`] 모듈은 arch 별 튜닝 상수 · 힌트만 둔다.  세 arch 크레이트의 `arch`
//! 모듈은 같은 모양이지만 그 모양을 묶는 공통 trait 은 아직 없다(결정 대기 —
//! `.claude-notes/fip-isa-subcrates-open-items.md` §4-4).  설계:
//! `docs/design_decisions.md` §2.1.
//!
//! 본 크레이트는 `prefetcht0` 인라인 `asm!` 하나 때문에 **좁게 감사된 unsafe** 를
//! 허용한다 — 크레이트는 `deny(unsafe_code)` 이고 [`arch::prefetch_read`] 에만
//! `allow` 를 둔다(Y4 가 쓴 1차 코드의 감사 근거:
//! `.brainstormings/20260812-174402-capsule-fault-isolation-restart.md`).
//! `y4-fip-core` 는 `forbid(unsafe_code)` 그대로다.

#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]

pub use y4_fip_core::*;

pub mod arch {
    //! `x86_64` compiler-guideline 튜닝 파라미터·helper.  consts 는 어느
    //! 호스트에서도 읽을 수 있고, [`prefetch_read`] 의 `prefetcht0` 은 `x86_64`
    //! 타깃에서만 emit 된다(그 밖의 타깃에서는 no-op).

    /// arch 이름.
    pub const ARCH: &str = "x86_64";

    /// 전형적 L1 cache line 크기(bytes).  `x86_64` = 64.
    pub const CACHE_LINE: usize = 64;

    /// 한 cache line 에 담기는 `T` 개수(최소 1) — cache-friendly FIP batch 크기
    /// ([`y4_fip_core::compact`] 등 hot 루프의 배치 단위).  ZST 는 `usize::MAX`.
    #[must_use]
    pub const fn per_cache_line<T>() -> usize {
        match CACHE_LINE.checked_div(core::mem::size_of::<T>()) {
            // `T` 가 한 줄보다 크면 0 이 아니라 1 — batch 크기로 쓰여도
            // `chunks(0)` panic · 멈추지 않는 stride 루프가 생기지 않는다.
            Some(0) => 1,
            Some(n) => n,
            None => usize::MAX,
        }
    }

    /// read prefetch hint — hot in-place 루프에서 다음 원소를 미리 당긴다.
    /// `x86_64` 에서 `PREFETCHT0` 한 명령으로 인라인; 그 외 타깃에선 no-op.
    ///
    /// `core::arch::x86_64::_mm_prefetch` 를 쓰지 않는다 — 그 intrinsic 은
    /// `#[target_feature(enable = "sse")]` fn 이라, SSE 를 끈 커널 타깃
    /// (`x86_64-unknown-none`)에서는 인라인되지 못하고 힌트마다 out-of-line 호출이
    /// 된다(rustc 1.98 에서 실측).  `PREFETCHT0` 자체는 x86-64 기본 명령이라
    /// target feature 가 필요 없으므로 `asm!` 로 직접 낸다.
    // 한 명령짜리 힌트라 hot 루프 안에 반드시 인라인돼야 의미가 있다.
    #[allow(clippy::inline_always)]
    #[inline(always)]
    #[allow(unsafe_code)]
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

#[cfg(test)]
mod tests {
    use super::arch::{ARCH, CACHE_LINE, per_cache_line, prefetch_read};

    #[test]
    fn arch_name() {
        assert_eq!(ARCH, "x86_64");
    }

    #[test]
    fn per_cache_line_bounds() {
        assert_eq!(per_cache_line::<()>(), usize::MAX);
        assert_eq!(per_cache_line::<u8>(), CACHE_LINE);
        assert_eq!(per_cache_line::<[u8; 64]>(), CACHE_LINE / 64);
        assert_eq!(per_cache_line::<[u8; 65]>(), 1);
        assert_eq!(per_cache_line::<[u8; 128]>(), 1);
        assert_eq!(per_cache_line::<u64>(), CACHE_LINE / 8);
    }

    #[test]
    fn prefetch_is_a_hint() {
        let xs = [1u64, 2, 3];
        prefetch_read(xs.as_ptr());
        // 힌트는 fault 를 내지 않는다 — 매달린 포인터도 받는다.
        prefetch_read(core::ptr::null::<u64>());
        assert_eq!(xs, [1, 2, 3]);
    }
}
