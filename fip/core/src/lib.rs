// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-core — Y4-FIP 정의 + in-place 빌딩블록
//!
//! Koka 의 Perceus/FP²(Lorenzen · Leijen · Swierstra, ICFP'23)에서 차용한
//! "fully in-place" 규율을 Rust 로 옮긴 **Y4-FIP** 의 정의와, 그 위에서 쓰는
//! arch-neutral in-place 빌딩블록이다.
//!
//! 설계 근거: `docs/design_decisions.md` §2.1 (Perceus 차용 · Y4-FIP 정의).
//!
//! ## Y4-FIP 정의 (2026-10-31 사용자 결정)
//!
//! 함수 `f` 는 다음 다섯을 모두 만족할 때 **Y4-FIP** 다.
//!
//! - **F1 할당 0** — `f` 의 전이적 호출 그래프(넘겨받은 클로저 포함)가 할당
//!   원천을 부르지 않는다.  할당 원천 = Rust 전역 할당자, `y4-alloc` API,
//!   seL4 untyped retype.
//! - **F2 해제 0** — `f` 가 소비하는 owned 값의 drop 이 자원을 해제하지
//!   않는다(`!needs_drop::<T>()` 이거나, 값이 반환되어 재사용된다).
//! - **F3 정적 스택 상한** — 직접 · 간접 재귀가 없다(호출 그래프가 DAG).
//!   Rust 에는 꼬리 호출 보장이 없으므로 반복은 루프로 쓴다.
//! - **F4 고유성** — 변형 대상은 `&mut` 이거나 by-value owned 다.
//! - **F5 고차 인자** — 넘겨받는 함수 · 클로저도 Y4-FIP 다.
//!
//! 완화 등급:
//! - **Y4-FBIP** — F2 · F3 을 푼 등급(해제 · 유계가 아닌 스택 허용).  Rust 의
//!   affine(암묵 drop) 기본값이 여기에 가깝다.
//! - **Y4-FIP(n)** — F1 을 할당 n 회까지로 푼 등급.
//!
//! FP² 와의 차이: FP² 의 fip 는 owned 변수의 *선형* 사용과 생성자 reuse 로
//! 할당 0 · 해제 0 을 *증명*한다.  Y4-FIP 는 그 성질들을 함수 단위 계약으로
//! 옮긴 것이고, 지금은 **선언된 계약**이다 — 강제 장치(서명/capability 검사,
//! Verus 명세)는 로드맵에 있다.
//!
//! ## 이 크레이트가 보장하는 것과 보장하지 않는 것
//!
//! - 이 크레이트의 조합자 **본문**은 할당 · 해제를 하지 않는다: `#![no_std]`,
//!   `alloc` 을 끌어오지 않음, 보조 공간 O(1), 자기 프레임 O(1), 재귀 없음.
//! - 조합자는 **alloc-transparent** 다: 넘겨받은 클로저와 `T` 의 `Default` ·
//!   `PartialEq` · `Drop` 이 할당 · 해제하면 그 비용은 그대로 일어난다.  조합자
//!   호출이 Y4-FIP 인지는 넘긴 클로저와 `T` 가 F1 · F2 · F5 를 지키는지에 달렸다.
//! - core-only(`no_std` · `alloc` 미사용) 크레이트 경계는 F1 의 **부분
//!   필요조건**일 뿐 보장이 아니다 — core-only 크레이트도 다른 크레이트의
//!   함수를 통해 전이적으로 할당할 수 있고, Y4 의 할당기(`y4-alloc`) 자신이
//!   core-only API 다.
//! - in-place 변형은 `&mut` 로 표현하므로 safe Rust 다
//!   (`#![forbid(unsafe_code)]`).  drop-then-reuse fast-path 는 할당기의 일이고
//!   이 크레이트에는 없다.
//!
//! ## 제공 API
//! - [`Fip`] — Y4-FIP 경로용 타입 marker(형태는 결정 대기 — 아래).
//! - [`compact`] — `&mut [T]` in-place 압축(freelist / run-queue 용).
//! - [`compact_with`] — 압축 + 원소 in-place 변형을 1-pass 로.
//! - [`dedup`] — 연속 중복 제거(정렬된 freelist 등).
//! - [`fip_update!`] — place 의 in-place 값 변환.
//! - [`fip_map!`] — 슬라이스 place 의 각 원소를 value-변환.
//! - `#[fip]` 애노테이션은 `y4-fip-macros` 에 있다(v1 — 선언만, 강제 없음).
//!
//! ## 로드맵
//! - **F1 강제** — `#[fip]` 가 서명/capability 를 검사한다: 할당기 handle ·
//!   untyped capability · 전역 할당자 경로가 서명에 없으면 F1 후보.  `y4-alloc`
//!   에는 ambient `GlobalAlloc` 이 없고 handle 이 있어야 할당하므로 이 검사가
//!   구조적으로 가능하다.
//! - **Verus** — ⒜ `compact` · `dedup` 기능 명세(첫 소비자보다 먼저),
//!   ⒝ 할당 0 · 스택 상한 인코딩.
//! - **arch 튜닝** — `y4-fip-{x86_64,aarch64,riscv64}` 의 `arch` 모듈
//!   (aarch64 `PRFM` · riscv64 `prefetch.r` 은 아직 no-op).

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

/// Y4-FIP 경로에서 다룰 타입의 marker.
///
/// **선언용**이다 — 지금은 구현체 · bound · `#[fip]` 연동이 없고, 이 트레이트를
/// 구현해도 컴파일러가 확인하는 의무는 없다.  형태(`unsafe trait` 로 F2 의무를
/// 걸지, 제거할지)는 결정 대기다(`.claude-notes/fip-isa-subcrates-open-items.md`
/// §4-2).
pub trait Fip {}

/// `&mut [T]` 를 **in-place 로 압축**한다 — `keep` 이 참인 원소만 앞으로 모으고
/// 유지된 원소 수를 돌려준다.  본문은 할당 · 해제 0, 보조 공간 O(1)(swap 기반)
/// 이고 유지 원소의 상대 순서를 보존한다.  `keep` 은 원소마다 원래 순서로 한 번
/// 불린다.
///
/// `Vec::retain`(alloc 필요) 대신 고정 `&mut [T]` 위에서 쓰는 빌딩블록이다
/// (할당기 freelist 정리, run-queue 압축 등).
///
/// **꼬리:** 반환한 길이 뒤에는 버린 원소들이 *순서가 섞인 채 살아 있는* owned
/// 값으로 남는다 — drop 되지 않고, 슬라이스의 주인이 drop 할 때 함께 drop 된다.
/// `T` 가 capability 토큰이면 꼬리를 다시 슬라이싱해 쓰지 않도록 주의한다.
///
/// **alloc-transparent:** `keep` 이 할당하면 그 할당은 일어난다(F1 · F5 는
/// 호출자 몫).
///
/// **panic:** `keep` 이 panic 해도 슬라이스는 원래 원소들의 순열로 남는다.
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

/// [`compact`] + **원소 in-place 변형을 1-pass 로**.  모든 원소에 `f(&mut x)` 를
/// 원래 순서로 한 번씩 호출하고(버릴 원소도 변형될 수 있다), `true` 를 돌려준
/// 원소만 앞으로 모아 유지 개수를 돌려준다.  본문은 할당 · 해제 0, 보조 공간
/// O(1).  꼬리 · alloc-transparent · panic 성질은 [`compact`] 와 같다.
///
/// # 예
/// ```
/// let mut xs = [1, 2, 3, 4];
/// // 짝수만 남기되, 남길 원소는 10 을 더한다.
/// let n = y4_fip_core::compact_with(&mut xs, |v| {
///     if *v % 2 == 0 { *v += 10; true } else { false }
/// });
/// assert_eq!(n, 2);
/// assert_eq!(&xs[..n], &[12, 14]);
/// ```
#[must_use = "the retained length must be used; elements past it are logically dead"]
pub fn compact_with<T>(xs: &mut [T], mut f: impl FnMut(&mut T) -> bool) -> usize {
    let mut w = 0;
    for r in 0..xs.len() {
        if f(&mut xs[r]) {
            xs.swap(r, w);
            w += 1;
        }
    }
    w
}

/// **연속 중복 제거** — 인접한 같은 원소를 하나만 남기고(앞으로 모음) 유지 개수를
/// 돌려준다.  본문은 할당 · 해제 0, 보조 공간 O(1).  **연속** 중복만 제거한다
/// (전역 아님).  `slice::partition_dedup`(nightly) / `Vec::dedup`(alloc)의
/// 고정 슬라이스 대응이다.  꼬리 · panic 성질은 [`compact`] 와 같고,
/// `PartialEq` 가 할당하면 그 할당은 일어난다.
///
/// # 예
/// ```
/// let mut xs = [1, 1, 2, 3, 3, 3, 4];
/// let n = y4_fip_core::dedup(&mut xs);
/// assert_eq!(n, 4);
/// assert_eq!(&xs[..n], &[1, 2, 3, 4]);
/// ```
#[must_use = "the deduplicated length must be used; elements past it are logically dead"]
pub fn dedup<T: PartialEq>(xs: &mut [T]) -> usize {
    if xs.is_empty() {
        return 0;
    }
    let mut w = 1;
    for r in 1..xs.len() {
        if xs[r] != xs[w - 1] {
            xs.swap(r, w);
            w += 1;
        }
    }
    w
}

/// place 의 **in-place 값 변환**: `place = f(take(place))`.
///
/// `&mut` 뒤의 값을 `Clone` 없이 `FnOnce(T) -> T` 로 소비-변환하는 관용구다.
///
/// - **place 만 받는다** — 변수 · `*r` · 인덱스 · 필드처럼 대입할 수 있는 식이어야
///   하고, const 항목 · 함수 호출 · 임시값은 컴파일 오류다(그런 식은 조용히
///   아무것도 바꾸지 않기 때문이다).
/// - `f` 식은 place 를 비우기 **전에** 한 번 평가된다.
/// - `mem::take` 를 쓰므로 `T: Default` 가 필요하다.  `Default` 가 할당하면 그
///   할당은 일어난다(alloc-transparent) — 할당하는 `Default` 를 가진 타입에서는
///   항등 변환도 할당 · 해제를 낸다.
/// - **panic:** `f` 가 panic 하면 place 에는 `Default` 값이 남는다.
///
/// # 예
/// ```
/// let mut x = 5i32;
/// y4_fip_core::fip_update!(x, |v| v * 2);
/// assert_eq!(x, 10);
/// ```
///
/// place 가 아닌 식은 거부된다:
/// ```compile_fail
/// const K: i32 = 5;
/// y4_fip_core::fip_update!(K, |v| v * 2);
/// ```
/// ```compile_fail
/// let x = 5i32;
/// y4_fip_core::fip_update!(x.clone(), |v| v * 2);
/// ```
#[macro_export]
macro_rules! fip_update {
    ($place:expr, $f:expr) => {{
        // place 검사 — 실행되지 않는 분기에서 `addr_of_mut!` 가 const 항목 ·
        // 임시값을 컴파일 오류(E0745)로 거부한다.  런타임 비용 0.
        if false {
            let _ = ::core::ptr::addr_of_mut!($place);
        }
        let __fip_f = $f;
        let __fip_place = &mut $place;
        let __fip_old = ::core::mem::take(__fip_place);
        *__fip_place = (__fip_f)(__fip_old);
    }};
}

/// 슬라이스 place 의 **각 원소를 value-변환**한다: 원소마다 `x = f(take(x))`.
///
/// - `$slice` 는 place 다 — 배열 변수, `*s`(`s: &mut [T]`), `v[..]` 등.
///   임시값(`v.clone()` 등)은 컴파일 오류다.
/// - `f` 식은 루프 **전에 한 번** 평가되고, 같은 `FnMut(T) -> T` 값이 원소마다
///   원래 순서로 불린다(클로저 상태가 원소 사이에 이어진다).
/// - `T: Default`, alloc-transparent 는 [`fip_update!`] 와 같다.
/// - **panic:** `f` 가 k 번째 원소에서 panic 하면 앞의 원소들은 변환된 채,
///   k 번째는 `Default` 값으로, 나머지는 그대로 남는다.
///
/// # 예
/// ```
/// let mut xs = [1, 2, 3];
/// y4_fip_core::fip_map!(xs, |v| v * 2);
/// assert_eq!(xs, [2, 4, 6]);
///
/// // 클로저 상태가 원소 사이에 이어진다.
/// let mut ys = [0, 0, 0];
/// let mut c = 10;
/// y4_fip_core::fip_map!(ys, move |_v: i32| { c += 1; c });
/// assert_eq!(ys, [11, 12, 13]);
/// ```
///
/// 임시값은 거부된다:
/// ```compile_fail
/// let xs = [1, 2, 3];
/// y4_fip_core::fip_map!(xs.clone(), |v| v * 2);
/// ```
#[macro_export]
macro_rules! fip_map {
    ($slice:expr, $f:expr) => {{
        if false {
            let _ = ::core::ptr::addr_of_mut!($slice);
        }
        let mut __fip_f = $f;
        for __fip_elem in ($slice).iter_mut() {
            let __fip_old = ::core::mem::take(__fip_elem);
            *__fip_elem = (__fip_f)(__fip_old);
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::{compact, compact_with, dedup};
    use std::cell::Cell;
    use std::panic::{AssertUnwindSafe, catch_unwind};

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
    fn compact_tail_is_a_permutation_of_removed() {
        let mut xs = [1, 2, 3, 4, 5, 6, 7];
        let n = compact(&mut xs, |v| v % 3 == 0);
        assert_eq!(&xs[..n], &[3, 6]);
        let mut tail = xs[n..].to_vec();
        tail.sort_unstable();
        assert_eq!(tail, [1, 2, 4, 5, 7]);
    }

    #[test]
    fn compact_panic_leaves_a_permutation() {
        let mut xs = [1, 2, 3, 4, 5, 6];
        let r = catch_unwind(AssertUnwindSafe(|| {
            compact(&mut xs, |v| {
                assert!(*v != 4, "boom");
                v % 2 == 1
            })
        }));
        assert!(r.is_err());
        let mut all = xs.to_vec();
        all.sort_unstable();
        assert_eq!(all, [1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn compact_drops_each_element_exactly_once() {
        struct D<'a>(u32, &'a Cell<u32>);
        impl Drop for D<'_> {
            fn drop(&mut self) {
                self.1.set(self.1.get() + 1);
            }
        }
        let drops = Cell::new(0);
        {
            let mut xs = [D(1, &drops), D(2, &drops), D(3, &drops), D(4, &drops)];
            let n = compact(&mut xs, |d| d.0 % 2 == 0);
            assert_eq!(n, 2);
            assert_eq!(drops.get(), 0, "compact itself drops nothing");
        }
        assert_eq!(drops.get(), 4);
    }

    #[test]
    fn compact_with_filters_and_transforms() {
        let mut xs = [1, 2, 3, 4, 5, 6];
        let n = compact_with(&mut xs, |v| {
            if *v % 2 == 0 {
                *v += 100;
                true
            } else {
                false
            }
        });
        assert_eq!(n, 3);
        assert_eq!(&xs[..n], &[102, 104, 106]);
    }

    #[test]
    fn dedup_consecutive() {
        let mut xs = [1, 1, 2, 3, 3, 3, 4, 4];
        let n = dedup(&mut xs);
        assert_eq!(n, 4);
        assert_eq!(&xs[..n], &[1, 2, 3, 4]);
    }

    #[test]
    fn dedup_keeps_nonconsecutive_repeats() {
        // 전역이 아니라 연속만 제거 — 1,2,1 은 그대로.
        let mut xs = [1, 2, 1];
        let n = dedup(&mut xs);
        assert_eq!(n, 3);
        assert_eq!(&xs[..n], &[1, 2, 1]);
    }

    #[test]
    fn dedup_empty_and_single() {
        let mut e: [i32; 0] = [];
        assert_eq!(dedup(&mut e), 0);
        let mut s = [7];
        assert_eq!(dedup(&mut s), 1);
    }

    #[test]
    fn fip_update_value() {
        let mut x = 5i32;
        crate::fip_update!(x, |v| v * 2);
        assert_eq!(x, 10);
    }

    #[test]
    fn fip_update_through_slice_ref() {
        let mut arr = [1, 2, 3];
        for slot in &mut arr {
            crate::fip_update!(*slot, |v| v + 10);
        }
        assert_eq!(arr, [11, 12, 13]);
    }

    #[test]
    fn fip_update_evaluates_f_once_before_take() {
        let mut x = 5i32;
        let mut built = 0;
        crate::fip_update!(x, {
            built += 1;
            |v| v + 1
        });
        assert_eq!((x, built), (6, 1));
    }

    #[test]
    fn fip_update_panic_leaves_default() {
        let mut x = 5i32;
        let r = catch_unwind(AssertUnwindSafe(|| {
            crate::fip_update!(x, |v: i32| -> i32 { panic!("boom {v}") });
        }));
        assert!(r.is_err());
        assert_eq!(x, 0);
    }

    #[test]
    fn fip_map_doubles_slice() {
        let mut xs = [1, 2, 3];
        crate::fip_map!(xs, |v| v * 2);
        assert_eq!(xs, [2, 4, 6]);
    }

    #[test]
    fn fip_map_threads_closure_state() {
        let mut xs = [0, 0, 0];
        let mut c = 10;
        crate::fip_map!(xs, move |_v: i32| {
            c += 1;
            c
        });
        assert_eq!(xs, [11, 12, 13]);
    }

    #[test]
    fn fip_map_evaluates_f_once() {
        let mut xs = [1, 2, 3];
        let mut built = 0;
        crate::fip_map!(xs, {
            built += 1;
            |v| v + 1
        });
        assert_eq!((xs, built), ([2, 3, 4], 1));
    }

    #[test]
    fn fip_map_through_mut_slice_ref() {
        let mut v = [1, 2, 3, 4];
        let s: &mut [i32] = &mut v[1..3];
        crate::fip_map!(*s, |x| x * 10);
        assert_eq!(v, [1, 20, 30, 4]);
    }
}
