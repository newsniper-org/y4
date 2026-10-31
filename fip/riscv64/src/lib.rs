// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-riscv64 — FIP 매크로, RISC-V `RV64` 특화
//!
//! [`y4_fip_core`] 의 FIP 계약을 **RISC-V `RV64`**(`RV64GC`) 용으로 특화한다
//! (compiler-guideline 튜닝).  seL4 `KernelSel4Arch=riscv64` 대응.  이 크레이트를
//! **`RV64` 를 타깃할 때** 사용한다.
//!
//! P3(`docs/hw_mechanism_abstraction.md` §2 「trait 뒤의 arch-neutral 로직」 · §1 규칙 1):
//! arch-neutral 조합자와 Y4-FIP 정의는 `y4-fip-core` 에 있고(재수출), 이 크레이트의
//! [`arch`] 모듈은 arch 별 튜닝 상수 · 힌트만 둔다.  세 arch 크레이트의 `arch`
//! 모듈은 같은 모양이지만 그 모양을 묶는 공통 trait 은 아직 없다(결정 대기 —
//! `.claude-notes/fip-isa-subcrates-open-items.md` §4-4).  설계:
//! `docs/design_decisions.md` §2.1.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

pub use y4_fip_core::*;

pub mod arch {
    //! RISC-V `RV64` compiler-guideline 튜닝 파라미터·helper.

    /// arch 이름.
    pub const ARCH: &str = "riscv64";

    /// 전형적 L1 cache line 크기(bytes).  RISC-V 는 impl-defined 이나 통상 64
    /// (플랫폼별 override 대상).
    pub const CACHE_LINE: usize = 64;

    /// 한 cache line 에 담기는 `T` 개수(최소 1) — cache-friendly FIP batch 크기.
    /// ZST 는 `usize::MAX`.
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

    /// read prefetch hint — 현재 **no-op**.  RISC-V `prefetch.r`(Zicbop 확장,
    /// 가용 시)을 `asm!` 로 emit 하는 실제 튜닝은 후속(audited unsafe).
    // 한 명령짜리 힌트라 hot 루프 안에 반드시 인라인돼야 의미가 있다.
    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn prefetch_read<T>(p: *const T) {
        let _ = p;
    }
}

#[cfg(test)]
mod tests {
    use super::arch::{ARCH, CACHE_LINE, per_cache_line, prefetch_read};

    #[test]
    fn arch_name() {
        assert_eq!(ARCH, "riscv64");
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
