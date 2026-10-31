// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-macros — FIP proc-macros
//!
//! [`macro@fip`] — 함수를 **Y4-FIP** 로 *선언*하는 애노테이션 (Koka 의
//! `fip fun` 대응).  Y4-FIP 의 정의(F1–F5 · Y4-FBIP · Y4-FIP(n))는
//! `y4-fip-core` crate 문서와 `docs/design_decisions.md` §2.1 에 있다.
//!
//! proc-macro 는 host 에서 컴파일 시점에 실행되므로 **런타임 TCB 가 아니다** —
//! 다만 빌드 TCB 에는 든다(P7).  외부 의존성 없이 raw
//! `proc_macro::TokenStream` 만 사용한다.
//!
//! ## v1 — 선언만
//! `#[fip]` 는 (1) 함수 정의(몸체 있는 `fn`)에 적용됐는지, (2) 속성 인자가
//! 없는지 검사하고, (3) 사용자 문서 *뒤에* 「Y4-FIP (선언)」 절을 붙여 item 을
//! 재방출한다.  **F1–F5 는 아직 강제하지 않는다.**
//!
//! ## 로드맵
//! - **F1 강제** — 서명/capability 검사: 할당기 handle · untyped capability ·
//!   전역 할당자 경로가 서명에 없으면 F1 후보(`y4-fip-core` crate 문서).
//! - **Verus 연동** — 「할당 0 + 스택 상한」 명세 부착.

#![forbid(unsafe_code)]

use proc_macro::{Delimiter, TokenStream, TokenTree};

/// 함수를 **Y4-FIP** 로 *선언*한다 (Koka `fip fun` 대응).
///
/// v1 은 선언만 한다 — F1–F5 를 강제하지 않는다.  검사하는 것은 둘이다.
/// - 대상이 몸체 있는 함수 정의인가: 바깥 속성 · 가시성 · 한정자
///   (`const` · `async` · `unsafe` · `extern "ABI"` · `default`) 다음에 `fn` 이
///   오고 마지막 토큰이 `{ … }` 몸체여야 한다.  `type F = fn();` · `static` ·
///   `const C: fn() = …` · `impl … for fn()` · 몸체 없는 선언은 거부한다.
/// - 속성 인자가 없는가: `#[fip(…)]` 는 거부한다(인자는 로드맵에서 정한다).
///
/// # 사용
/// ```
/// use y4_fip_macros::fip;
///
/// /// 값을 두 배로 한다.
/// #[fip]
/// fn double(x: &mut i32) {
///     *x *= 2; // in-place
/// }
///
/// fn main() {
///     let mut v = 3;
///     double(&mut v);
///     assert_eq!(v, 6);
/// }
/// ```
///
/// 함수 정의가 아니면 거부된다:
/// ```compile_fail
/// #[y4_fip_macros::fip]
/// type F = fn();
/// ```
/// ```compile_fail
/// #[y4_fip_macros::fip]
/// static S: fn() = || {};
/// ```
/// ```compile_fail
/// #[y4_fip_macros::fip]
/// const C: fn() = || {};
/// ```
/// ```compile_fail
/// trait T {
///     #[y4_fip_macros::fip]
///     fn f(&self);
/// }
/// ```
///
/// # Panics
///
/// 이 매크로가 만드는 토큰이 문법상 틀릴 때만 panic 한다(내부 불변식 — 사용자
/// 입력으로는 일어나지 않는다).
///
/// 속성 인자도 거부된다:
/// ```compile_fail
/// #[y4_fip_macros::fip(no_alloc = false)]
/// fn f() {}
/// ```
#[proc_macro_attribute]
pub fn fip(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return compile_error("#[fip] takes no arguments");
    }
    let tokens: Vec<TokenTree> = item.into_iter().collect();
    let Some(fn_at) = fn_keyword_position(&tokens) else {
        return compile_error("#[fip] can only be applied to a function definition");
    };
    let has_body = matches!(
        tokens.last(),
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace
    );
    if !has_body {
        return compile_error("#[fip] can only be applied to a function definition with a body");
    }

    // 사용자 문서 뒤, 함수 머리 앞에 「Y4-FIP (선언)」 절을 붙인다 — 빈 줄로
    // 문단을 끊어 rustdoc 요약 줄을 가로채지 않는다.
    let head_at = item_head_position(&tokens, fn_at);
    let note: TokenStream = concat!(
        "#[doc = \"\"] ",
        "#[doc = \"# Y4-FIP (선언)\"] ",
        "#[doc = \"\"] ",
        "#[doc = \"`#[fip]` 로 Y4-FIP 라고 선언된 함수다 — v1 은 F1–F5 를 강제하지 않는다 ",
        "(정의: `y4-fip-core` crate 문서).\"]",
    )
    .parse()
    .expect("y4-fip-macros: generated doc attributes must be valid tokens");

    let mut out = TokenStream::new();
    out.extend(tokens[..head_at].iter().cloned());
    out.extend(note);
    out.extend(tokens[head_at..].iter().cloned());
    out
}

/// `compile_error!` 를 내는 토큰.
fn compile_error(msg: &str) -> TokenStream {
    format!("::core::compile_error!({msg:?});")
        .parse()
        .expect("y4-fip-macros: compile_error tokens must be valid")
}

/// 바깥 속성 · 가시성 · 한정자를 건너뛴 위치에 `fn` 이 있으면 그 색인.
fn fn_keyword_position(tokens: &[TokenTree]) -> Option<usize> {
    let mut i = skip_outer_attributes(tokens, 0);
    // 가시성: `pub`, `pub(crate)`, `pub(in path)` …
    if is_ident(tokens.get(i), "pub") {
        i += 1;
        if matches!(tokens.get(i), Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis)
        {
            i += 1;
        }
    }
    // 한정자: const · async · unsafe · default · extern ["ABI"]
    loop {
        match tokens.get(i) {
            Some(TokenTree::Ident(id))
                if matches!(
                    id.to_string().as_str(),
                    "const" | "async" | "unsafe" | "default"
                ) =>
            {
                i += 1;
            }
            Some(TokenTree::Ident(id)) if id.to_string() == "extern" => {
                i += 1;
                if matches!(tokens.get(i), Some(TokenTree::Literal(_))) {
                    i += 1;
                }
            }
            _ => break,
        }
    }
    is_ident(tokens.get(i), "fn").then_some(i)
}

/// 바깥 속성(`#[…]`) 다음 위치 — 문서 절을 끼울 자리.
fn item_head_position(tokens: &[TokenTree], fn_at: usize) -> usize {
    skip_outer_attributes(tokens, 0).min(fn_at)
}

/// `start` 부터 연속한 바깥 속성 `#` `[…]` 을 건너뛴 위치.
fn skip_outer_attributes(tokens: &[TokenTree], start: usize) -> usize {
    let mut i = start;
    while matches!(tokens.get(i), Some(TokenTree::Punct(p)) if p.as_char() == '#')
        && matches!(tokens.get(i + 1), Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket)
    {
        i += 2;
    }
    i
}

fn is_ident(tt: Option<&TokenTree>, want: &str) -> bool {
    matches!(tt, Some(TokenTree::Ident(id)) if id.to_string() == want)
}
