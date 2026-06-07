// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! AV1 — `intercept_floor_holds(vcpu)` (S2: 16-bit mandatory mask).
//!
//! R7.11 first emit milestone (2026-06-03).
//!
//! Spec source-of-truth: `docs/amdv_safety.md` §S2 + §5 의 AV1 row.
//!
//! 16 mandatory intercept bits (S2 본문 표):
//!   INTERCEPT_NMI / SMI / INIT / SHUTDOWN / VMRUN /
//!   VMLOAD / VMSAVE / CLGI / STGI / HLT / NPF /
//!   CPUID / INVD / WBINVD / MSR_PROT / VMMCALL
//!
//! 이 중 1 비트라도 0 이면 vmrun 거부 — `intercept_floor_holds(vcpu)`
//! 가 vmrun 직전 항상 true 이어야 한다.

use vstd::prelude::*;

verus! {

/// VMCB 의 intercept_words 의 type 약식 (u64).
///
/// 실제 ARCH-II' 의 `vmcb` capsule 의 layout 은 두 32-bit word
/// (INTERCEPT_CR_R/W + INTERCEPT_EXCEPTIONS + INTERCEPT_INSTRUCTIONS) —
/// 본 spec 의 단순화를 위해 single u64 로 표현.  P-redesign.3 후속 단계
/// 에서 vmcb capsule 의 실제 struct 매핑으로 sharpen.
pub type InterceptWords = u64;

/// S2 의 16-bit mandatory mask (S2 본문 표 정합).
///
/// 비트 위치는 ARCH-II' 의 vmcb_amd.bf (KernelSVM patch 의 산출물) 의
/// `intercept_floor.bf` 에 fixed.  현 spec 에서는 16 bit 의 임의 위치
/// 를 0..16 로 가정 (PR-Verus-Backend land 후 emit 시 실제 비트 위치
/// 와 정합 verification 추가).
pub spec const MANDATORY_INTERCEPT_MASK: InterceptWords = 0xFFFF;

/// AV1 — `intercept_floor_holds(vcpu)` 의 spec function.
///
/// vmcb 의 intercept_words 가 16 mandatory bit 를 모두 1 로 설정 시
/// true.  vmrun syscall (D1a 의 wrapper) 의 pre-condition.
pub open spec fn intercept_floor_holds(intercept_words: InterceptWords) -> bool {
    (intercept_words & MANDATORY_INTERCEPT_MASK) == MANDATORY_INTERCEPT_MASK
}

/// AV1 proof body — vmrun pre-condition 정합.
///
/// `intercept_floor_holds(words)` 가 만족 ⟹ `words` 의 16 mandatory
/// bit 모두 1.  자명한 bitwise identity — Z3 / OxiZ / adsmt 모두 즉시
/// discharge.
pub proof fn intercept_floor_implies_all_mandatory_bits_set(words: InterceptWords)
    requires intercept_floor_holds(words),
    ensures (words & MANDATORY_INTERCEPT_MASK) == MANDATORY_INTERCEPT_MASK,
{
}

/// AV1 의 converse — 모든 mandatory bit set ⟹ `intercept_floor_holds`.
pub proof fn all_mandatory_bits_set_implies_intercept_floor(words: InterceptWords)
    requires (words & MANDATORY_INTERCEPT_MASK) == MANDATORY_INTERCEPT_MASK,
    ensures intercept_floor_holds(words),
{
}

} // verus!
