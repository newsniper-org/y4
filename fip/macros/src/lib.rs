// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-macros — FIP proc-macros
//!
//! [`macro@fip`] — 함수를 **Fully in-Place** 로 표시하는 애노테이션 (Koka 의
//! `fip fun` 대응).  설계: `docs/design_decisions.md` §2.1,
//! `.brainstormings/20260904-200712-perceus-reference-scan.md` §5.
//!
//! **build-time only** — proc-macro 는 host 에서 컴파일 시점에 실행되며 런타임
//! TCB 가 아니다.  외부 의존성 없이 raw `proc_macro::TokenStream` 만 사용.
//!
//! ## v1 (착수)
//! `#[fip]` 는 (1) 함수(`fn`)에 적용됐는지 검증하고, (2) FIP 계약 문서를 주입한
//! 뒤 item 을 재방출한다 — **애노테이션 + 문서 hook** 을 확립하는 단계.
//!
//! ## 로드맵 (의존성 대기)
//! - **no-alloc guard 주입**: (debug) 본문을 per-CPU forbid-alloc guard scope 로
//!   감싸 할당 시 abort — y4-alloc 이 global allocator hook 을 제공한 후.
//! - **Verus 연동**: `#[fip]` 함수에 "할당 0 + 스택 bounded" 불변식 부착.

use proc_macro::{TokenStream, TokenTree};

/// 함수를 **Fully in-Place (FIP)** 로 표시한다 — 힙 할당 없이 `&mut` in-place
/// 갱신만 하는 함수 (Koka `fip fun` 대응).
///
/// v1: `fn` 적용 검증 + FIP 계약 문서 주입 후 item 재방출.  강제(no-alloc
/// guard / Verus)는 crate 문서의 로드맵 참조.
///
/// # 사용
/// ```
/// use y4_fip_macros::fip;
///
/// #[fip]
/// fn double(x: &mut i32) {
///     *x *= 2; // in-place, 힙 할당 없음
/// }
///
/// fn main() {
///     let mut v = 3;
///     double(&mut v);
///     assert_eq!(v, 6);
/// }
/// ```
#[proc_macro_attribute]
pub fn fip(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // 최상위 토큰에 `fn` 키워드(Ident)가 있는지로 함수 여부 검증.
    let is_fn = item
        .clone()
        .into_iter()
        .any(|tt| matches!(&tt, TokenTree::Ident(id) if *id.to_string() == *"fn"));
    if !is_fn {
        return format!(
            "::core::compile_error!({:?});",
            "#[fip] can only be applied to a function"
        )
        .parse()
        .expect("y4-fip-macros: compile_error tokens must be valid");
    }

    const NOTE: &str = " **FIP** (Fully in-Place): 힙 할당 없음(no-alloc), \
                        in-place `&mut` 갱신. 설계 design_decisions §2.1.";
    let doc: TokenStream = format!("#[doc = {NOTE:?}]")
        .parse()
        .expect("y4-fip-macros: generated doc attribute must be valid tokens");
    let mut out = doc;
    out.extend(item);
    out
}
