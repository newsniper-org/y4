// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! # y4-fip-core — Fully in-Place (FIP) 계약
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
//! ## core-only = 컴파일러가 강제하는 zero-alloc
//!
//! 이 크레이트는 `#![no_std]` 이며 **`alloc` 을 절대 끌어오지 않는다**.  따라서
//! 이 크레이트(및 이를 core-only 로 사용하는 hot 경로) 안에서는 **힙 할당이
//! 링크 수준에서 불가능**하다 — FP² 의 formal zero-alloc 을 Rust 에서 근사하는
//! 방법(§5.1 핵심 통찰).  in-place 변형은 `&mut` 로 표현하므로 **safe Rust**
//! (`#![forbid(unsafe_code)]`).  drop-then-reuse fast-path 는 allocator(perceus
//! scan §3)에 있고 이 크레이트에는 없다.
//!
//! ## per-ISA 구조 (P3 — 계약 공유, realization per-arch)
//!
//! 본 크레이트가 **계약**(marker trait + 매크로 표면)을 고정하고, seL4 가
//! 지원하는 64-bit ISA 별 subcrate(`y4-fip-x86_64` / `y4-fip-aarch64` /
//! `y4-fip-riscv64`)가 compiler-guideline 튜닝을 **realization** 으로 특화한다.
//!
//! ## 상태
//!
//! **scaffold** — 실제 FIP 매크로 API 는 설계 진행 중.  본 크레이트는 계약
//! 표면과 per-ISA subcrate 구조를 확립한다.

#![no_std]
#![forbid(unsafe_code)]

/// FIP(Fully in-Place)로 다룰 수 있는 타입의 marker.
///
/// 이 marker 를 구현한 타입은 `&mut` 를 통한 in-place 변형만으로(할당·해제 0)
/// 처리되도록 설계됨을 문서적으로 보증한다 — 향후 `fip!` 매크로가 이 계약을
/// 강제·검증(Verus)한다.
pub trait Fip {}

// TODO(design_decisions §2.1): `fip!` / `fip_fn!` 매크로.
//   hot 경로를 core-only(no `alloc`) + `&mut` in-place 로 구조화하고,
//   Verus 로 "이 경로 할당 0 + 스택 bounded" 불변식을 부착할 수 있게 한다.
//   (compiler-guideline 튜닝은 per-ISA subcrate 의 `arch` 모듈에서.)
