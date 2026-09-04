// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-core — Fully in-Place (FIP) 계약 + 빌딩블록
//!
//! Y4 FIP(Fully in-Place) 매크로 계층의 **arch-neutral 계약**.  Koka 의
//! Perceus/FP²(ICFP'23)에서 차용한 "provable zero-alloc + in-place" 규율을
//! Rust 로 realize 한다.
//!
//! 설계 근거:
//! - `docs/design_decisions.md` §2.1 (allocator / Perceus 차용)
//! - `.brainstormings/20260904-200712-perceus-reference-scan.md` §5(FIP 규율)·
//!   §5.1(`alloc::Rc` vs FIP-macro 비교 → FIP-macro 우선)
//!
//! ## core-only = 컴파일러가 강제하는 zero-alloc (핵심)
//!
//! 이 크레이트는 `#![no_std]` 이며 **`alloc` 을 절대 끌어오지 않는다**.  따라서
//! 이 크레이트(및 이를 core-only 로 사용하는 hot 경로) 안에서는 **힙 할당이
//! 링크 수준에서 불가능**하다 — FP² 의 formal zero-alloc 을 Rust 에서 realize
//! 하는 방법(§5.1 핵심 통찰).  강제는 **매크로가 아니라 core-only 크레이트
//! 경계**가 제공하며, 본 크레이트는 그 위에서 쓰는 **in-place 빌딩블록 +
//! 매크로**를 제공한다.  in-place 변형은 `&mut` 로 표현하므로 **safe Rust**
//! (`#![forbid(unsafe_code)]`); drop-then-reuse fast-path 는 allocator(perceus
//! scan §3)에 있고 이 크레이트에는 없다.
//!
//! ## 제공 API (v1)
//! - [`Fip`] — FIP-able 타입 marker.
//! - [`compact`] — `&mut [T]` in-place 압축(freelist / run-queue 용, no-alloc).
//! - [`fip_update!`] — `&mut` place 의 in-place 값 변환(no-alloc).
//!
//! ## 로드맵 (다음 단계 — 의존성 대기)
//! - **`#[fip]` proc-macro** (Koka `fip fun` 대응 애노테이션): 함수를 FIP 로
//!   표시 + (debug) no-alloc guard 주입.  런타임 no-alloc guard 는 y4-alloc 의
//!   global allocator hook(per-CPU forbid flag) 필요 → allocator 통합 후.
//! - **Verus**: "이 경로 할당 0 + 스택 bounded" 불변식 → verus-fork 안정화 후.
//! - **per-ISA compiler-guideline 튜닝**: `y4-fip-{x86_64,aarch64,riscv64}` 의
//!   `arch` 모듈.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

/// FIP(Fully in-Place)로 다룰 수 있는 타입의 marker.
///
/// 이 marker 를 구현한 타입은 `&mut` 를 통한 in-place 변형만으로(할당·해제 0)
/// 처리되도록 설계됨을 문서적으로 보증한다 — 향후 `#[fip]`(로드맵)가 이 계약을
/// 강제·검증(Verus)한다.
pub trait Fip {}

/// `&mut [T]` 를 **in-place 로 압축**한다 — `keep` 이 참인 원소만 앞으로 모으고,
/// 유지된 원소 수를 반환.  **힙 할당 0, 상수 추가 공간**(swap 기반), 유지 원소의
/// 상대 순서 보존.
///
/// Y4 의 no-alloc FIP 빌딩블록 — allocator freelist 정리, scheduler run-queue
/// 압축 등에서 `Vec::retain`(alloc 필요) 대신 고정 `&mut [T]` 위에서 쓴다.
/// 반환한 길이 이후의 원소들은 미지정 상태(제자리 유지되나 논리적으로 dead).
///
/// # 예
/// ```
/// let mut xs = [1, 2, 3, 4, 5, 6];
/// let n = y4_fip_core::compact(&mut xs, |v| v % 2 == 0);
/// assert_eq!(n, 3);
/// assert_eq!(&xs[..n], &[2, 4, 6]);
/// ```
#[must_use = "the retained length must be used; elements past it are logically dead"]
pub fn compact<T>(xs: &mut [T], mut keep: impl FnMut(&T) -> bool) -> usize {
    let mut w = 0;
    for r in 0..xs.len() {
        if keep(&xs[r]) {
            xs.swap(r, w);
            w += 1;
        }
    }
    w
}

/// `&mut` place 의 **in-place 값 변환**: `*place = f(take(place))`.
///
/// 힙 할당 없이(core-only) 값을 소비-변환한다.  `mem::take` 를 쓰므로
/// `T: Default` 필요(교체용 placeholder).  `&mut T` 뒤의 값을 `Clone`/alloc
/// 없이 `FnOnce(T) -> T` 로 변환하는 FIP 관용구.
///
/// # 예
/// ```
/// let mut x = 5i32;
/// y4_fip_core::fip_update!(x, |v| v * 2);
/// assert_eq!(x, 10);
/// ```
#[macro_export]
macro_rules! fip_update {
    ($place:expr, $f:expr) => {{
        let __fip_place = &mut $place;
        let __fip_old = ::core::mem::take(__fip_place);
        *__fip_place = ($f)(__fip_old);
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_keeps_even_in_order() {
        let mut xs = [1, 2, 3, 4, 5, 6];
        let n = compact(&mut xs, |v| v % 2 == 0);
        assert_eq!(n, 3);
        assert_eq!(&xs[..n], &[2, 4, 6]);
    }

    #[test]
    fn compact_all_or_none() {
        let mut a = [1, 2, 3];
        assert_eq!(compact(&mut a, |_| true), 3);
        assert_eq!(&a, &[1, 2, 3]);
        let mut b = [1, 2, 3];
        assert_eq!(compact(&mut b, |_| false), 0);
    }

    #[test]
    fn compact_empty() {
        let mut e: [i32; 0] = [];
        assert_eq!(compact(&mut e, |_| true), 0);
    }

    #[test]
    fn fip_update_value() {
        let mut x = 5i32;
        fip_update!(x, |v| v * 2);
        assert_eq!(x, 10);
    }

    #[test]
    fn fip_update_through_slice_ref() {
        let mut arr = [1, 2, 3];
        for slot in &mut arr {
            fip_update!(*slot, |v| v + 10);
        }
        assert_eq!(arr, [11, 12, 13]);
    }
}
