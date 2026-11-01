// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-arch — Y4-FIP arch 튜닝 facade
//!
//! portable-fip 의 `fip-arch` 재수출이다(사용자 결정 2026-11-01).  arch 는
//! `cfg(target_arch)` 로 고른다 — seL4 `KernelSel4Arch` 가 고른 Rust 타깃
//! (`x86_64-unknown-none` · `aarch64-unknown-none` · `riscv64gc-unknown-none-elf`)이
//! 곧 선택이다.  이전의 arch 별 크레이트 셋(`y4-fip-{x86_64,aarch64,riscv64}`)을
//! 대신한다.
//!
//! - [`ARCH`] — `"x86_64"` / `"aarch64"` / `"riscv64"` / `"other"`.
//! - [`CACHE_LINE`] — 기본 64 B(Apple aarch64 는 128 B).  플랫폼별로
//!   `--cfg fip_cache_line="32"|"64"|"128"` 로 덮어쓴다(다른 값은 컴파일 오류).
//! - [`per_cache_line`] — 한 줄에 담기는 `T` 개수, 최소 1(ZST 는 `usize::MAX`).
//! - [`prefetch_read`] — 읽기 prefetch 힌트: `x86_64` `prefetcht0`, `AArch64` `PRFM`,
//!   그 밖은 no-op.  힌트는 fault 를 내지 않으므로 어떤 포인터도 받는다.
//!
//! P3(`docs/hw_mechanism_abstraction.md` §2 「trait 뒤의 arch-neutral 로직」): arch
//! 차이는 이 크레이트의 상수 · 힌트에만 있고, 조합자는 `y4-fip-core` 에 있다.

#![no_std]
#![cfg_attr(fip_prelint, fip::export)]
#![forbid(unsafe_code)]

pub use fip_arch::*;

#[cfg(test)]
mod tests {
    use super::{ARCH, CACHE_LINE, per_cache_line, prefetch_read};

    #[test]
    fn arch_matches_target() {
        let want = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else if cfg!(target_arch = "riscv64") {
            "riscv64"
        } else {
            "other"
        };
        assert_eq!(ARCH, want);
    }

    #[test]
    fn per_cache_line_bounds() {
        assert_eq!(per_cache_line::<()>(), usize::MAX);
        assert_eq!(per_cache_line::<u8>(), CACHE_LINE);
        assert_eq!(per_cache_line::<[u8; 4096]>(), 1);
    }

    #[test]
    fn prefetch_is_a_hint() {
        let xs = [1u64, 2, 3];
        prefetch_read(xs.as_ptr());
        prefetch_read(core::ptr::null::<u64>());
        assert_eq!(xs, [1, 2, 3]);
    }
}
