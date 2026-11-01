// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-core — Y4-FIP 정의 + portable-fip facade
//!
//! Koka 의 Perceus/FP²(Lorenzen · Leijen · Swierstra, ICFP'23)에서 차용한
//! "fully in-place" 규율의 Y4 정의(**Y4-FIP**)와, 그 정의를 실현하는
//! [portable-fip](https://github.com/newsniper-org/portable-fip) 의 재수출이다
//! (사용자 결정 2026-11-01 — 장기적으로 portable-fip 은 Y4 안으로 들어오고 독립
//! 저장소는 그 추출판이 된다).  Y4 의 소비자는 이 크레이트만 문다 — portable-fip 을
//! Y4 안으로 들일 때 소비자 코드는 바뀌지 않는다.
//!
//! 설계 근거: `docs/design_decisions.md` §2.1 · `.claude-notes/fip-vs-portable-fip-comparison.md`.
//!
//! ## Y4-FIP 정의 (2026-10-31 사용자 결정)
//!
//! 함수 `f` 는 다음 다섯을 모두 만족할 때 **Y4-FIP** 다.
//!
//! - **F1 할당 0** — `f` 의 전이적 호출 그래프(넘겨받은 클로저 포함)가 할당
//!   원천을 부르지 않는다.  할당 원천 = Rust 전역 할당자, `y4-alloc` API,
//!   seL4 untyped retype.
//! - **F2 해제 0** — `f` 가 소비하는 owned 값의 drop 이 자원을 해제하지 않는다.
//! - **F3 정적 스택 상한** — 직접 · 간접 재귀가 없다(호출 그래프가 DAG).
//! - **F4 고유성** — 변형 대상은 `&mut` 이거나 by-value owned 다.
//! - **F5 고차 인자** — 넘겨받는 함수 · 클로저도 Y4-FIP 다.
//!
//! 완화 등급: **Y4-FBIP**(F2 · F3 해제) · **Y4-FIP(n)**(할당 n 회까지).
//!
//! ## portable-fip 대응과 강제 수준
//!
//! | Y4-FIP | portable-fip | 강제 |
//! |---|---|---|
//! | F1 | [`macro@fip`] | 매크로(본문 토큰의 할당 원천) · strict 인증서 · `fip-prelint`(MIR) |
//! | F2 | [`macro@fip`] (FBIP 아님) | `fip-prelint` 의 "may free" |
//! | F3 | [`macro@fip`], `#[fip(N)]` | 매크로(직접 재귀) · Verus(비순환) · `fip-prelint`(MIR 순환) |
//! | F4 | — | 검사 없음(`Cell`/`RefCell` 은 빠져나간다) |
//! | F5 | `param(g = K)` · [`fip_strict::FipFn`] · [`fip_fn!`] | strict 의 `FipBound` 컴파일 시 검사 · `fip-prelint` |
//! | Y4-FBIP | [`macro@fbip`] | 할당 원천 거부, 해제 · 재귀 허용 — 같은 뜻 |
//! | Y4-FIP(n) | `#[fip(N)]` | 상한 N |
//!
//! **알려진 차이 (F1 의 할당 원천):** portable-fip 은 Rust 전역 할당자만 할당
//! 원천으로 안다.  `y4-alloc` API 와 seL4 untyped retype 은 아직 표식할 수 없어서,
//! `y4-alloc` 경로는 portable-fip 의 검사로는 F1 위반으로 잡히지 않을 수 있다
//! (비교 문서 §6-1 — 사용자 선언 할당 원천은 portable-fip 쪽 과제 G).
//!
//! ## 재수출
//!
//! - 조합자: [`compact`] · [`compact_map`] · [`dedup_by`] · [`map_in_place`] ·
//!   [`swap`] · [`update`] · [`replace`] 와 [`strict`] 판(인증된 `FipFn` 만 받음).
//!   모든 공개 함수가 portable-fip 에서 Verus(`--no-cheating`)로 증명됐고, Y4 의
//!   proofs 게이트가 **핀한 rev 의 그 증명을 Y4 의 verus-fork(Z3 4.16.0)로 다시
//!   돌린다**(`just verus-portable-fip`).
//! - 자리값은 `Default` 가 아니라 호출자가 주는 `hole` 이다 — capability 토큰을
//!   자리값 때문에 만들어 낼 일이 없다.
//! - 매크로: [`macro@fip`] · [`macro@fbip`] · [`macro@fip_impl`] · [`fip_fn!`].
//!   **strict 모드**(`#[fip(strict = true, …)]`)가 만드는 코드는 `::fip_strict` 를
//!   직접 참조하므로, strict 를 쓰는 크레이트는 `fip-strict` 에도 의존해야 한다
//!   (`fip-strict.workspace = true`).
//! - [`fip_strict`] — strict 인증서 · `FipFn` · `FipBound` · `Certified`.
//!
//! ## 예
//!
//! ```
//! use y4_fip_core::{compact, fip};
//!
//! /// 짝수만 남긴다 — 할당 · 재귀 없음.
//! #[fip]
//! pub fn keep_even(xs: &mut [u32]) -> usize {
//!     compact(xs, |v: &u32| v % 2 == 0)
//! }
//!
//! let mut xs = [1, 2, 3, 4];
//! assert_eq!(keep_even(&mut xs), 2);
//! ```
//!
//! `#[fip]` 는 본문의 할당 원천을 거부한다(facade 를 거쳐도 같다):
//! ```compile_fail
//! extern crate alloc;
//! #[y4_fip_core::fip]
//! pub fn boxed(x: u32) -> alloc::boxed::Box<u32> {
//!     alloc::boxed::Box::new(x)
//! }
//! ```
//! 재귀도 거부한다(F3):
//! ```compile_fail
//! #[y4_fip_core::fip]
//! pub fn down(n: u32) -> u32 {
//!     if n == 0 { 0 } else { down(n - 1) }
//! }
//! ```
//!
//! ## 보장하지 않는 것
//!
//! - 조합자는 alloc-transparent 다: 넘겨받은 클로저가 할당하면 그 할당은 일어난다
//!   (strict 판은 상한 0 인 인증된 callable 만 받는다).
//! - core-only(`no_std` · `alloc` 미사용) 크레이트 경계는 F1 의 부분 필요조건일 뿐
//!   보장이 아니다.
//! - 조합자의 꼬리(버린 원소)는 순서가 섞인 채 살아 있는 owned 값으로 남는다 — drop
//!   되지 않는다.  `T` 가 capability 토큰이면 꼬리를 다시 쓰지 않도록 주의한다(이
//!   계약의 강제 방식은 결정 대기).

#![no_std]
#![cfg_attr(fip_prelint, fip::export)]
#![forbid(unsafe_code)]

pub use fip_core::*;
pub use fip_macros::{fbip, fip, fip_fn, fip_impl};
pub use fip_strict;

/// Y4-FIP 경로에서 다룰 타입의 marker.
///
/// **선언용**이다 — 구현체 · bound · 매크로 연동이 없고, 이 트레이트를 구현해도
/// 컴파일러가 확인하는 의무는 없다.  portable-fip 의 [`fip_strict::FipBound`]
/// (호출 가능 값의 할당 상한)와는 뜻이 다르다.  형태(`unsafe trait` 로 F2 의무를
/// 걸지, 제거할지)는 결정 대기다(`.claude-notes/fip-isa-subcrates-open-items.md`
/// §4-2).
pub trait Fip {}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::vec;

    #[test]
    fn compact_keeps_in_order() {
        let mut xs = [1, 2, 3, 4, 5, 6];
        let n = crate::compact(&mut xs, |v: &i32| v % 2 == 0);
        assert_eq!(&xs[..n], &[2, 4, 6]);
    }

    #[test]
    fn dedup_by_consecutive_only() {
        let mut xs = [1, 1, 2, 1, 1];
        let n = crate::dedup_by(&mut xs, |a: &i32, b: &i32| a == b);
        assert_eq!(&xs[..n], &[1, 2, 1]);
    }

    #[test]
    fn update_takes_a_hole_not_default() {
        // `String`/`Vec` 처럼 `Default` 가 있는 타입이라도 자리값은 호출자가 준다.
        let mut v = vec![1u8, 2];
        let hole = crate::update(&mut v, vec![], |mut x| {
            x.push(3);
            x
        });
        assert_eq!(v, [1, 2, 3]);
        assert_eq!(hole, std::vec::Vec::<u8>::new());
    }

    #[test]
    fn map_in_place_returns_the_hole() {
        let mut xs = [1u32, 2, 3];
        let hole = crate::map_in_place(&mut xs, 0, |x| x * 10);
        assert_eq!((xs, hole), ([10, 20, 30], 0));
    }

    #[test]
    fn compact_map_transforms_and_filters() {
        let mut xs = [1u32, 2, 3, 4];
        let (n, hole) = crate::compact_map(&mut xs, 0, |x| (x + 100, x % 2 == 0));
        assert_eq!((&xs[..n], hole), (&[102, 104][..], 0));
    }
}
