// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Step program load — `0xF00` FP_PROG · `0xF04` FP_BIND · `0xF08` FP_STATUS · `0xF0C`
//! FP_SHAPE (`docs/hiu_abi.md` §1.2.1 (A) 4항 · (C) · §1.3 · §1.4 · §2.5 5 · §2.7,
//! `docs/lease_capability.md` I4 2b · I7 3).
//!
//! 워드 배치(정본 `b16fc07` — 동결 sha, 정본 트리 `2dd78f80…`):
//!
//! - FP_PROG 「`[31:24]` 슬롯 · `[23:16]` step 색인 · `[15:0]` step 워드」 — 「이 MMIO 워드의
//!   필드에는 예약 비트가 없다 — step 워드 안의 예약 `[15:14]` 는 그 배치가 진다」
//!   (`e-defined.typ:459`).  step 워드의 배치는 3.10 이 동결했다 — `[0]` last · `[1]` unit ·
//!   `[5:2]` srcA · `[9:6]` srcB · `[13:10]` dst · 「`[15:14]` 예약 = 0 (엄격 — 비0 적재는 거절 +
//!   `[15]`」(`03-hiu.typ:1692`).  동결하는 것은 필드의 자리와 역할이고 「래치의 쓰임 · 데이터
//!   폭 같은 step 의 실행 의미도 이 항이 동결하지 않는다」(같은 줄).
//! - FP_BIND 「`[31]` en (1 = 결속 · 0 = 해제) · `[30:24]` 예약 = 0 (엄격) · `[23:16]` 슬롯 ·
//!   `[15:0]` 선택 열쇠」 — 「예약 검사가 먼저다」 · 「en = 0 은 슬롯 필드를 무시하고 그 열쇠의
//!   결속을 해제한다 (결속되지 않은 열쇠면 무연산)」 · 「이미 결속된 열쇠의 en = 1 은 그 결속을
//!   새 슬롯으로 바꾼다(재결속 — 옛 슬롯은 그 열쇠의 결속을 잃는다)」(`e-defined.typ:461`).
//! - FP_STATUS 「`[1]` 걷기 진행 중」, FP_SHAPE `[8:0]` P · 「`[17:9]` stepMax = 슬롯당 step 수
//!   상한 · `[23:18]` RO 예약 = 0 · `[31:24]` step 부호화 번호」(`e-defined.typ:463`).
//!
//! `[15]` 의 원천은 닫힌 넷이다 — 「그 밖의 원천은 `[15]` 에 접지 않는다.」
//! (`e-defined.typ:464`): (1) FP_PROG 의 용량 밖 슬롯 · step 색인, (2) step 워드 `[15:14]` 의
//! 비0, (3) FP_BIND en = 1 의 용량 밖 슬롯(예약 비0 은 `[6]` 만), (4) runaway.  앞의 셋은
//! 「기입 시점 거절이며 무효과다」(같은 줄).  Y4 는 (1) · (3) 을 `0xF0C` 의 P · stepMax 로
//! (`fp_prog_ok` · `fp_bind_ok`), (2) 를 step 워드의 내용 검사로(`step_word_ok` — E.1.1 「호스트
//! 해석 규범」 4항: 「배치가 동결돼 있어 기계적으로 판정되고, 남이 만든 이미지를 넘겨 싣는
//! 드라이버도 이 검사를 진다」 `e-defined.typ:208`) 기입 전에 피한다.  (4) 는 「같은 항의 결속
//! 전제(…)와 재적재 순서가 막는다」 — 걷기 순서가 동결됐으므로(「걷기 순서도 동결한 역할이다」
//! `03-hiu.typ:1692`, C5) 「실행 의미의 맞춤이 어긋난 이미지는 틀린 결과를 낼 수는 있어도 … (4) 를
//! 내지 않는다」(`:466`).  그래서 「그가 보는 `[15]` 는 자기 쪽(드라이버 · 플랫폼)의 적재 · 결속
//! 의무 위반이 낸 (4) 이거나 그 밖의 자기 쪽 의무 위반이다」(`:466`).  v1.0 에서 Y4 는 플랫폼이
//! 지정한 이미지만 싣고 테넌트(게스트 SDK)는 선택 열쇠로 고르기만 한다(사용자 결정 U1, 2026-10-18 —
//! 4항 「드라이버는 플랫폼(과 도구 사슬)이 그 구현에 맞춘 것으로 지정한 이미지만 결속한다」 `:208`,
//! C4).  테넌트 원인은 없으므로 `[15]` 는 `[19]` 선례로 귀속한다(사용자 결정 2026-10-22 —
//! `crate::error::step_fault_origin` · `lease::window::step_fault_verdict`): Y4 step 장부가
//! 어긋났으면 `LeaseInvariant`, 맞으면 귀속 불가 `DeviceFault`.  실행 의미가 어긋난 이미지는 장치
//! 그물 없이 조용히 틀린 결과를 내므로 그 방어는 플랫폼 지정(레지스트리)뿐이다.
//!
//! 적재 순서도 4항의 의무다(`e-defined.typ:208`): 「`0xF04` 결속(en = 1)은 프로그램이 마지막
//! 장치 reset 뒤에 다 실리고 종료 step 을 가진 슬롯에만 하고」 · 「결속된 슬롯에는 `0xF00` 를
//! 쓰지 않는다」 · 재적재는 「그 슬롯을 가리키던 결속을 모두 거둔 `0xF04` 기입(en = 0 해제이든,
//! 같은 열쇠를 다른 슬롯에 거는 en = 1 재결속이든 — 그 마지막 기입)이 장치 포트에서 수락된
//! 뒤에 수락된 `0xF08` 읽기가 `[1]` = 0 을 본 다음이다」 — 그리고 「v1.0 장치는 이 순서를
//! 판정하지 않고 수락하며 어긴 순서는 조용할 수 있다」.  「step 프로그램 저장의 reset 뒤
//! 내용은 정하지 않는다」 · 「reset 뒤 선택자 표가 비어 어느 슬롯도 결속돼 있지 않으며」
//! (`e-defined.typ:216`).  슬롯 장부 `SlotState` · `FpLedger` 가 그 의무를 상태기계로 진다 —
//! Unloaded → Loaded → Bound → Unbound → Quiesced.  장치 모델 `FpDevice` 는 선택자 표와 걷기
//! 중인 슬롯이고, 결속은 덮어쓰기 · 결속되지 않은 열쇠의 해제는 무연산 · 걷기는 조회 때 래치한
//! 슬롯을 계속 쓴다(RTL 사실 — 규범 아님: `rtl/…/fp/StepSequencer.scala:320-325` · `:389`).
//! 불변식 `fp_mirror` 아래에서 FP_PROG 대상 슬롯에는 결속도 걷기도 없다(`fp_prog_target_idle`).
//!
//! 해제 뒤 `[1]` = 0 의 관측이 `fp_quiesce` 다(대기 상한 `T_WALK` — 플랫폼 레지스트리).  무엇을
//! 덮는가는 정본이 정한다(C2, `e-defined.typ:463` FP_STATUS 행): 「그 읽기의 수락 사이클 앞에 그 열쇠를
//! 조회한 걷기가 — 그 기입의 수락 사이클까지의 조회든 그 뒤의 조회든, 옛 결속을 보았든 기입이 반영된
//! 표를 보았든 — 모두 끝났음을 뜻한다」 · 「그 읽기의 수락 사이클부터(같은 사이클 포함) 시작하는
//! 조회는 그 기입이 반영된 선택자 표를 본다」.  「반영 전이면 장치는 그 읽기의 수락을 반영 뒤로 미룰
//! 수 있다」 — 그 정지는 유계 장치 의무이고 `T_WALK` 폴링이 끊는 대기가 아니다(`hiu_abi.md` §2.7 4).
//! 「`[1]` 은 장치 전역의 순간값이라 다른 열쇠의 투입이 이어지면 내려오지 않을 수 있다 — 그때
//! 재적재는 기다린다」(`e-defined.typ:208`)이고 「v1.0 에서 일의 원천은 외부 제시라 투입을 멈출 수단이
//! 없다」(`e-defined.typ:332`) — 그래서 소진은 그 창의 적재 생략(능력 부재)이고, `K_WALK` 창 연속이면
//! step 기능을 끈다(`StepOff` — 장치 격리 아님, 사용자 결정 2026-10-22; `fp_quiesce_action`).
//! 재구현의 `[1]` 은 결속되지 않은 열쇠를 조회하는 사이클에도 1 이다(RTL 사실 — 규범 아님: `rtl/…/fp/StepSequencer.scala:337`).
//!
//! `0xF0C` 는 `0xFFC` → `0xFF8` → `0xF0C` 순서로 읽고 `0xFFC` major 가 1 일 때만 해석한다(§1.4).
//! 「RO 읽기는 정의된 비트로 마스크해서 해석한다」(`e-defined.typ:208` 2항)이므로 `[23:18]` 은 가린다
//! (`fp_shape_masks_reserved`).  「번호 0 은 배선 · 합의 전의 값뿐이다」 · 「1 이 아닌 비0 번호는
//! v1.0 배치가 아니다」(`:463`) — major 1 인데 번호 ≠ 1, P = 0(`02-fabric-alu.typ:489`),
//! P 또는 stepMax > 256(「9b 라서 슬롯 · 색인 필드(8b)가 가리킬 수 있는 최대 256 까지 담는다」
//! `e-defined.typ:463`), 플랫폼 레지스트리의 교차 확인값과의 불일치는 장치 모순(`DeviceFault`)이고,
//! stepMax = 0 은 「step 기능 없음」(적재 생략)이다(사용자 결정 U2 — 참조 RTL 은 stepMax ≥ 2 라
//! 그 갈래에 닿지 않는다, RTL 사실).  배선은 RTL 파도 W6(`efafb14`)가 착지했다 — 판 0 동안 읽기는
//! 워드 전체 0 이다(`e-defined.typ:463` FP_SHAPE 행).
//!
//! FP 관문 `StepGate`: 창의 FP_PROG 를 다 쓴 뒤 `T_FLT` 를 기다려 `0xF80[15]` 를 읽는다.  관문
//! 앞에서 결속을 모두 거두고 `[1]` = 0 을 보았으므로 이 구간에는 (3) 의 기입이 없고 새 걷기가
//! 시작할 수 없다 — 1 이면 적합 경로에서 날 수 없는 `[15]` 다.  다시 싣지 않고 결속하지 않으며
//! `[19]` 선례로 귀속한다(`step_gate_action`).  (3) 은 `fp_bind_ok` 가 미리 막고, 결속 뒤의 확인은
//! 창 기준선 (나)가 진다(FP_BIND en = 1 이 창의 마지막 기입이다).  step 경로가 `LeaseInvariant` 에
//! 이르는 것은 Y4 step 장부가 어긋났을 때뿐이다(`step_path_halts_only_on_ledger`).

use vstd::prelude::*;

use crate::error::Y4Error;

verus! {

// --- step 워드 (`0xF00` `[15:0]` — `03-hiu.typ:1692` 배치) -------------------

/// step 워드 `[15:14]` 예약 — 비0 적재는 거절 + `[15]`(원천 (2)).
pub const STEP_RSV: u32 = 0x0000_c000;

/// step 워드 `[0]` last — 「1 = 이 step 을 실행한 뒤 걷기를 끝내고 그 기록값이 결과다」
/// (`03-hiu.typ:1692`).
pub const STEP_LAST: u32 = 0x0000_0001;

/// 두 상수가 제 자리 비트라는 것.
pub proof fn step_consts()
    ensures
        STEP_RSV == (1u32 << 15u32) | (1u32 << 14u32),
        STEP_LAST == 1u32 << 0u32,
{
    assert(0xc000u32 == (1u32 << 15u32) | (1u32 << 14u32)) by (bit_vector);
    assert(0x1u32 == 1u32 << 0u32) by (bit_vector);
}

/// step 워드 하나의 Y4 내용 검사: 16 비트 안 · `[15:14]` = 0.  그 밖의 비트(실행 의미)는
/// 해석하지 않는다.
pub open spec fn step_word_ok(w: u32) -> bool {
    &&& w < 0x1_0000
    &&& w & STEP_RSV == 0
}

/// step 워드의 `[0]` last.
pub open spec fn step_word_last(w: u32) -> bool {
    w & STEP_LAST != 0
}

// --- `0xF00` FP_PROG ---------------------------------------------------------

/// Y4 가 쓰는 FP_PROG 워드 — `[31:24]` 슬롯 · `[23:16]` step 색인 · `[15:0]` step 워드.
pub open spec fn fp_prog_word(slot: u32, idx: u32, w: u32) -> u32 {
    (slot << 24u32) | (idx << 16u32) | w
}

/// 장치가 FP_PROG 워드에서 읽는 슬롯.
pub open spec fn fp_prog_slot(x: u32) -> u32 {
    x >> 24u32
}

/// 장치가 FP_PROG 워드에서 읽는 step 색인.
pub open spec fn fp_prog_idx(x: u32) -> u32 {
    (x >> 16u32) & 0xffu32
}

/// 장치가 FP_PROG 워드에서 읽는 step 워드.
pub open spec fn fp_prog_step(x: u32) -> u32 {
    x & 0xffffu32
}

/// 정본의 FP_PROG 거절 — 「`[15]` 의 원천」 (1) 용량 밖 슬롯 · step 색인, (2) step 워드
/// `[15:14]` 의 비0(`e-defined.typ:459` · `:464`).  `p` · `step_max` 는 그 기계의 용량이다.
pub open spec fn fp_prog_refused15(x: u32, p: nat, step_max: nat) -> bool {
    ||| fp_prog_slot(x) >= p
    ||| fp_prog_idx(x) >= step_max
    ||| fp_prog_step(x) & STEP_RSV != 0
}

/// FP_PROG 기입의 Y4 사전조건: 슬롯 < P · 색인 < stepMax(둘 다 `0xF0C` 의 값 —
/// `fp_shape_verdict` 가 `Usable`) · 필드 폭 안 · `step_word_ok`(4항의 내용 검사).
pub open spec fn fp_prog_ok(slot: u32, idx: u32, w: u32, p: nat, step_max: nat) -> bool {
    &&& slot < p
    &&& idx < step_max
    &&& slot < 256
    &&& idx < 256
    &&& step_word_ok(w)
}

/// 필드 폭 안의 슬롯 · 색인 · step 워드로 만든 FP_PROG 워드는 장치가 같은 필드로 읽는다 —
/// 필드가 서로의 자리로 넘치지 않는다.
pub proof fn fp_prog_word_fields(slot: u32, idx: u32, w: u32)
    requires
        slot < 256,
        idx < 256,
        w < 0x1_0000,
    ensures
        fp_prog_slot(fp_prog_word(slot, idx, w)) == slot,
        fp_prog_idx(fp_prog_word(slot, idx, w)) == idx,
        fp_prog_step(fp_prog_word(slot, idx, w)) == w,
{
    assert(((slot << 24u32) | (idx << 16u32) | w) >> 24u32 == slot) by (bit_vector)
        requires
            slot < 256u32,
            idx < 256u32,
            w < 0x1_0000u32,
    ;
    assert((((slot << 24u32) | (idx << 16u32) | w) >> 16u32) & 0xffu32 == idx) by (bit_vector)
        requires
            slot < 256u32,
            idx < 256u32,
            w < 0x1_0000u32,
    ;
    assert(((slot << 24u32) | (idx << 16u32) | w) & 0xffffu32 == w) by (bit_vector)
        requires
            slot < 256u32,
            idx < 256u32,
            w < 0x1_0000u32,
    ;
}

/// `fp_prog_ok` 를 지킨 FP_PROG 는 (1) · (2) 어느 것에도 걸리지 않는다.
pub proof fn fp_prog_ok_avoids_15(slot: u32, idx: u32, w: u32, p: nat, step_max: nat)
    requires
        fp_prog_ok(slot, idx, w, p, step_max),
    ensures
        !fp_prog_refused15(fp_prog_word(slot, idx, w), p, step_max),
{
    fp_prog_word_fields(slot, idx, w);
}

/// `[15:14]` 가 비0 인 step 워드는 용량과 무관하게 거절된다 — (2) 는 용량 계산이 아니라
/// `step_word_ok` 만이 막는다(`fp_prog_ok` 의 내용 검사는 공허하지 않다).
pub proof fn fp_prog_reserved_refused(slot: u32, idx: u32, w: u32, p: nat, step_max: nat)
    requires
        slot < 256,
        idx < 256,
        w < 0x1_0000,
        w & STEP_RSV != 0,
    ensures
        !step_word_ok(w),
        fp_prog_refused15(fp_prog_word(slot, idx, w), p, step_max),
{
    fp_prog_word_fields(slot, idx, w);
}

// --- `0xF04` FP_BIND ---------------------------------------------------------

/// FP_BIND `[31]` en.
pub const FP_BIND_EN: u32 = 0x8000_0000;

/// FP_BIND `[30:24]` 예약(엄격).
pub const FP_BIND_RSV: u32 = 0x7f00_0000;

/// Y4 가 쓰는 FP_BIND 워드 — `[31]` en · `[30:24]` 0 · `[23:16]` 슬롯 · `[15:0]` 선택 열쇠
/// (`[15:8]` 연산 종류 · `[7:0]` 정밀도 — 자리와 폭만 동결, `03-hiu.typ:1692`).
pub open spec fn fp_bind_word(en: bool, slot: u32, key: u32) -> u32 {
    (if en {
        FP_BIND_EN
    } else {
        0u32
    }) | (slot << 16u32) | key
}

/// 장치가 FP_BIND 워드에서 읽는 en.
pub open spec fn fp_bind_en(x: u32) -> bool {
    x & FP_BIND_EN != 0
}

/// 장치가 FP_BIND 워드에서 읽는 `[30:24]`.
pub open spec fn fp_bind_rsv(x: u32) -> u32 {
    x & FP_BIND_RSV
}

/// 장치가 FP_BIND 워드에서 읽는 슬롯.
pub open spec fn fp_bind_slot(x: u32) -> u32 {
    (x >> 16u32) & 0xffu32
}

/// 장치가 FP_BIND 워드에서 읽는 선택 열쇠.
pub open spec fn fp_bind_key(x: u32) -> u32 {
    x & 0xffffu32
}

/// 정본의 FP_BIND `[6]` 거절 — `[30:24]` 비0.  「예약 검사가 먼저다」 — 이 기입에는 `[15]` 가
/// 서지 않는다(`e-defined.typ:461`).
pub open spec fn fp_bind_refused6(x: u32) -> bool {
    fp_bind_rsv(x) != 0
}

/// 정본의 FP_BIND `[15]` 거절 — 원천 (3): 예약이 0 인 en = 1 의 용량 밖 슬롯.  「en = 0 은 슬롯
/// 필드를 무시하므로 이 원천이 아니다」(`e-defined.typ:464`).
pub open spec fn fp_bind_refused15(x: u32, p: nat) -> bool {
    &&& !fp_bind_refused6(x)
    &&& fp_bind_en(x)
    &&& fp_bind_slot(x) >= p
}

/// en = 1 결속의 Y4 사전조건: 슬롯 < P(`0xF0C`) · 필드 폭 안.  장부 쪽 조건(이번 세대의 관문을
/// 통과한 슬롯 · 장부상 결속되지 않은 열쇠)은 `fp_key_bindable` 이다.
pub open spec fn fp_bind_ok(slot: u32, key: u32, p: nat) -> bool {
    &&& slot < p
    &&& slot < 256
    &&& key < 0x1_0000
}

/// en = 0 해제의 Y4 사전조건: 열쇠가 16 비트 안.  슬롯 필드는 무시되므로 슬롯 조건이 없다 —
/// Y4 는 결정성을 위해 0 을 쓴다.
pub open spec fn fp_unbind_ok(key: u32) -> bool {
    key < 0x1_0000
}

/// 필드 폭 안의 슬롯 · 열쇠로 만든 FP_BIND 워드는 예약이 0 이고 장치가 같은 en · 슬롯 · 열쇠로
/// 읽는다.
pub proof fn fp_bind_word_fields(en: bool, slot: u32, key: u32)
    requires
        slot < 256,
        key < 0x1_0000,
    ensures
        fp_bind_en(fp_bind_word(en, slot, key)) == en,
        fp_bind_rsv(fp_bind_word(en, slot, key)) == 0,
        fp_bind_slot(fp_bind_word(en, slot, key)) == slot,
        fp_bind_key(fp_bind_word(en, slot, key)) == key,
{
    if en {
        assert((0x8000_0000u32 | (slot << 16u32) | key) & 0x8000_0000u32 != 0) by (bit_vector);
        assert((0x8000_0000u32 | (slot << 16u32) | key) & 0x7f00_0000u32 == 0) by (bit_vector)
            requires
                slot < 256u32,
                key < 0x1_0000u32,
        ;
        assert(((0x8000_0000u32 | (slot << 16u32) | key) >> 16u32) & 0xffu32 == slot)
            by (bit_vector)
            requires
                slot < 256u32,
                key < 0x1_0000u32,
        ;
        assert((0x8000_0000u32 | (slot << 16u32) | key) & 0xffffu32 == key) by (bit_vector)
            requires
                slot < 256u32,
                key < 0x1_0000u32,
        ;
    } else {
        assert((0u32 | (slot << 16u32) | key) & 0x8000_0000u32 == 0) by (bit_vector)
            requires
                slot < 256u32,
                key < 0x1_0000u32,
        ;
        assert((0u32 | (slot << 16u32) | key) & 0x7f00_0000u32 == 0) by (bit_vector)
            requires
                slot < 256u32,
                key < 0x1_0000u32,
        ;
        assert(((0u32 | (slot << 16u32) | key) >> 16u32) & 0xffu32 == slot) by (bit_vector)
            requires
                slot < 256u32,
                key < 0x1_0000u32,
        ;
        assert((0u32 | (slot << 16u32) | key) & 0xffffu32 == key) by (bit_vector)
            requires
                slot < 256u32,
                key < 0x1_0000u32,
        ;
    }
}

/// `fp_bind_ok` 를 지킨 en = 1 결속은 `[6]` 에도 (3) 에도 걸리지 않는다.
pub proof fn fp_bind_ok_avoids_refusal(slot: u32, key: u32, p: nat)
    requires
        fp_bind_ok(slot, key, p),
    ensures
        !fp_bind_refused6(fp_bind_word(true, slot, key)),
        !fp_bind_refused15(fp_bind_word(true, slot, key), p),
{
    fp_bind_word_fields(true, slot, key);
}

/// 예약이 0 인 en = 1 의 용량 밖 슬롯은 (3) 으로 거절된다 — `fp_bind_ok` 의 슬롯 < P 는
/// 공허하지 않다.
pub proof fn fp_bind_out_of_capacity_refused(slot: u32, key: u32, p: nat)
    requires
        slot < 256,
        key < 0x1_0000,
        slot >= p,
    ensures
        !fp_bind_ok(slot, key, p),
        fp_bind_refused15(fp_bind_word(true, slot, key), p),
{
    fp_bind_word_fields(true, slot, key);
}

/// en = 0 해제는 슬롯 필드와 P 에 무관하게(P = 0 이어도) `[6]` 에도 `[15]` 에도 걸리지 않는다
/// — 「en = 0 은 슬롯 필드를 무시하고」(`e-defined.typ:461`).
pub proof fn fp_unbind_never_refused(slot: u32, key: u32, p: nat)
    requires
        slot < 256,
        fp_unbind_ok(key),
    ensures
        !fp_bind_refused6(fp_bind_word(false, slot, key)),
        !fp_bind_refused15(fp_bind_word(false, slot, key), p),
{
    fp_bind_word_fields(false, slot, key);
}

/// 결속한 열쇠는 해제의 사전조건도 만족한다 — 나가는 임대의 해제는 언제나 쓸 수 있다.
pub proof fn fp_unbind_ok_after_bind(slot: u32, key: u32, p: nat)
    requires
        fp_bind_ok(slot, key, p),
    ensures
        fp_unbind_ok(key),
{
}

/// 부팅의 선택 열쇠 전 범위 해제(en = 0 · 슬롯 0 · 열쇠 < 2^16)는 어느 P 에서도 거절되지 않는다
/// (`docs/hiu_abi.md` §2.5 5 — P ≥ 1 전제가 필요 없다).
pub proof fn fp_unbind_all_ok(key: u32, p: nat)
    requires
        key < 0x1_0000,
    ensures
        fp_unbind_ok(key),
        !fp_bind_refused6(fp_bind_word(false, 0, key)),
        !fp_bind_refused15(fp_bind_word(false, 0, key), p),
{
    fp_unbind_never_refused(0, key, p);
}

// --- step 경로의 동작 ------------------------------------------------------

/// step 경로의 동작.
#[derive(PartialEq, Eq, Structural)]
pub enum StepAction {
    /// 계속한다 — 적재 · 결속으로.
    Proceed,
    /// 이 창의 step 적재를 생략한다 — 그 선택 열쇠의 참조는 능력 부재로 남는다(「상주하지 않는
    /// 프로그램의 참조는 능력 부재다」 `02-fabric-alu.typ:493-494`).
    SkipLoad,
    /// 이 장치의 step 기능을 끈다 — 이후 창의 step 적재를 모두 생략하고(그 선택 열쇠들의 참조는
    /// 능력 부재) 운영자에 보고한다.  장치 격리가 아니다(사용자 결정 2026-10-22).
    StepOff,
    /// 이 오류로 멈춘다.
    Fail(Y4Error),
}

// --- `0xF0C` FP_SHAPE --------------------------------------------------------

/// FP_SHAPE 의 정의된 비트 — `[31:24]` 번호 · `[17:9]` stepMax · `[8:0]` P(`[23:18]` RO 예약은
/// 가린다).
pub const FP_SHAPE_DEFINED: u32 = 0xff03_ffff;

/// 재구현 형상(P = 10 · stepMax = 32)의 판 값 뒤 읽기 — 「`0xFFC` 가 0 이 아니게 된 뒤의 읽기는
/// `0x0100_400A` 다」(`e-defined.typ:738`, RTL 사실 — 규범 아님).
pub const FP_SHAPE_RTL_REFERENCE: u32 = 0x0100_400a;

/// `[8:0]` P = 프로그램 슬롯 수(개수 그대로).
pub open spec fn fp_shape_p(w: u32) -> u32 {
    w & 0x1ffu32
}

/// `[17:9]` stepMax = 슬롯당 step 수 상한(개수 그대로).
pub open spec fn fp_shape_step_max(w: u32) -> u32 {
    (w >> 9u32) & 0x1ffu32
}

/// `[31:24]` step 부호화 번호(1 = 3.10 의 v1.0 배치).
pub open spec fn fp_shape_enc(w: u32) -> u32 {
    w >> 24u32
}

/// `0xF0C` 판정 (사용자 결정 U2).
#[derive(PartialEq, Eq, Structural)]
pub enum FpShapeVerdict {
    /// 번호 1 · 1 ≤ P ≤ 256 · 1 ≤ stepMax ≤ 256 · 레지스트리와 어긋나지 않음 — 그 P · stepMax 로
    /// `fp_prog_ok` · `fp_bind_ok` 를 건다.
    Usable,
    /// stepMax = 0 — step 기능 없음.  적재를 생략하고, 선택 열쇠의 참조는 능력 부재로 남는다.
    NoStep,
    /// major 1 인데 번호 ≠ 1, P = 0, P 또는 stepMax > 256, 레지스트리 불일치 — 장치 모순.
    /// `DeviceFault`.
    Contradiction,
}

/// 플랫폼 레지스트리의 (P, stepMax) 가 있고 장치 값과 다르다 — 그 레지스트리는 다른 장치를
/// 기술하므로 장치 워드가 없는 레지스트리 값(LBID 슬롯 수 · LNID 등록 용량 · 배달 지점 수)도
/// 믿을 수 없다.
pub open spec fn fp_shape_registry_mismatch(
    p: u32,
    step_max: u32,
    reg: Option<(nat, nat)>,
) -> bool {
    match reg {
        Some(r) => r.0 != p || r.1 != step_max,
        None => false,
    }
}

/// `0xF0C` 판정.  `0xFFC` major 가 1 일 때만 부른다 — 판 0 의 장치에서 Y4 는 진행하지 않고,
/// 워드 전체가 0 인 동안의 장치 밖 형상 지식(레지스트리)은 mock · 반입 시험만 쓴다.  `reg` =
/// 플랫폼 레지스트리의 교차 확인값(없으면 `None`).
pub open spec fn fp_shape_verdict(w: u32, reg: Option<(nat, nat)>) -> FpShapeVerdict {
    let p = fp_shape_p(w);
    let sm = fp_shape_step_max(w);
    if fp_shape_enc(w) != 1 || p == 0 || p > 256 || sm > 256 {
        FpShapeVerdict::Contradiction
    } else if fp_shape_registry_mismatch(p, sm, reg) {
        FpShapeVerdict::Contradiction
    } else if sm == 0 {
        FpShapeVerdict::NoStep
    } else {
        FpShapeVerdict::Usable
    }
}

/// `0xF0C` 판정의 동작: `Usable` → 진행, `NoStep` → 적재 생략, `Contradiction` → `DeviceFault`.
pub open spec fn fp_shape_action(v: FpShapeVerdict) -> StepAction {
    match v {
        FpShapeVerdict::Usable => StepAction::Proceed,
        FpShapeVerdict::NoStep => StepAction::SkipLoad,
        FpShapeVerdict::Contradiction => StepAction::Fail(Y4Error::DeviceFault),
    }
}

/// 예약 `[23:18]` 을 가려도 해독과 판정이 같다 — `[23:18]` 의 비0 을 결함으로 읽지 않는다.
pub proof fn fp_shape_masks_reserved(w: u32, reg: Option<(nat, nat)>)
    ensures
        fp_shape_p(w & FP_SHAPE_DEFINED) == fp_shape_p(w),
        fp_shape_step_max(w & FP_SHAPE_DEFINED) == fp_shape_step_max(w),
        fp_shape_enc(w & FP_SHAPE_DEFINED) == fp_shape_enc(w),
        fp_shape_verdict(w & FP_SHAPE_DEFINED, reg) == fp_shape_verdict(w, reg),
{
    assert((w & 0xff03_ffffu32) & 0x1ffu32 == w & 0x1ffu32) by (bit_vector);
    assert(((w & 0xff03_ffffu32) >> 9u32) & 0x1ffu32 == (w >> 9u32) & 0x1ffu32) by (bit_vector);
    assert((w & 0xff03_ffffu32) >> 24u32 == w >> 24u32) by (bit_vector);
}

/// 재구현 형상의 참조값 `0x0100_400A` 은 (P, stepMax, 번호) = (10, 32, 1) 이다.  레지스트리가
/// 없으면 `Usable` 이고, 레지스트리가 있으면 (10, 32) 일 때만 `Usable` · 아니면 `Contradiction`
/// 이다.
pub proof fn fp_shape_reference(rp: nat, rsm: nat)
    ensures
        fp_shape_p(FP_SHAPE_RTL_REFERENCE) == 10,
        fp_shape_step_max(FP_SHAPE_RTL_REFERENCE) == 32,
        fp_shape_enc(FP_SHAPE_RTL_REFERENCE) == 1,
        fp_shape_verdict(FP_SHAPE_RTL_REFERENCE, None) == FpShapeVerdict::Usable,
        (fp_shape_verdict(FP_SHAPE_RTL_REFERENCE, Some((rp, rsm))) == FpShapeVerdict::Usable) <==> (
        rp == 10 && rsm == 32),
        (fp_shape_verdict(FP_SHAPE_RTL_REFERENCE, Some((rp, rsm))) == FpShapeVerdict::Contradiction)
            <==> !(rp == 10 && rsm == 32),
{
    assert(0x0100_400au32 & 0x1ffu32 == 10u32) by (bit_vector);
    assert((0x0100_400au32 >> 9u32) & 0x1ffu32 == 32u32) by (bit_vector);
    assert(0x0100_400au32 >> 24u32 == 1u32) by (bit_vector);
}

/// 워드 전체 0(번호 0)은 major 1 의 장치에서는 모순이다 — 「번호 0 은 배선 · 합의 전의
/// 값뿐이다」(`e-defined.typ:463`).
pub proof fn fp_shape_zero_contradicts(reg: Option<(nat, nat)>)
    ensures
        fp_shape_verdict(0, reg) == FpShapeVerdict::Contradiction,
{
    assert(0u32 >> 24u32 == 0u32) by (bit_vector);
}

/// 망라: 세 판정은 각각 입력 술어와 동치이고, `Usable` 의 P · stepMax 는 8 비트 슬롯 · 색인
/// 필드가 가리킬 수 있는 1–256 안이다.
pub proof fn fp_shape_verdict_exhaustive(w: u32, reg: Option<(nat, nat)>)
    ensures
        ({
            let p = fp_shape_p(w);
            let sm = fp_shape_step_max(w);
            let bad = fp_shape_enc(w) != 1 || p == 0 || p > 256 || sm > 256
                || fp_shape_registry_mismatch(p, sm, reg);
            let v = fp_shape_verdict(w, reg);
            &&& (v == FpShapeVerdict::Contradiction) <==> bad
            &&& (v == FpShapeVerdict::NoStep) <==> (!bad && sm == 0)
            &&& (v == FpShapeVerdict::Usable) <==> (!bad && sm != 0)
            &&& v == FpShapeVerdict::Usable ==> (fp_shape_enc(w) == 1 && 1 <= p <= 256 && 1 <= sm
                <= 256)
            &&& (fp_shape_action(v) == StepAction::Fail(Y4Error::DeviceFault)) <==> bad
            &&& fp_shape_action(v) != StepAction::Fail(Y4Error::LeaseInvariant)
        }),
{
}

// --- 이미지 (Y4 정책) --------------------------------------------------------

/// Y4 가 한 슬롯에 싣는 이미지: 색인 0 부터 연속 n 워드(1 ≤ n ≤ stepMax), 모두 `step_word_ok`,
/// 마지막 워드의 `[0]` last = 1.  정본은 「종료 step 을 가진」 슬롯까지만 요구하므로
/// (`e-defined.typ:208`) 마지막 워드의 last 는 그보다 보수적인 Y4 정책이다.  이미지는 플랫폼이
/// 지정한다(U1).
pub open spec fn fp_image_ok(img: Seq<u32>, step_max: nat) -> bool {
    &&& 1 <= img.len() <= step_max
    &&& forall|i: int| 0 <= i < img.len() ==> step_word_ok(#[trigger] img[i])
    &&& step_word_last(img[img.len() - 1])
}

/// 이미지의 i 번째 FP_PROG 워드 — 색인 i 에 i 번째 step 워드.
pub open spec fn fp_image_word(slot: u32, img: Seq<u32>, i: int) -> u32 {
    fp_prog_word(slot, i as u32, img[i])
}

/// `fp_image_ok` 이미지를 용량 안 슬롯에 실으면 모든 기입이 `fp_prog_ok` 이고 (1) · (2) 에 걸리지
/// 않는다.
pub proof fn fp_image_ok_prog_ok(slot: u32, img: Seq<u32>, p: nat, step_max: nat)
    requires
        fp_image_ok(img, step_max),
        slot < p,
        p <= 256,
        step_max <= 256,
    ensures
        forall|i: int|
            0 <= i < img.len() ==> fp_prog_ok(slot, i as u32, #[trigger] img[i], p, step_max),
        forall|i: int|
            0 <= i < img.len() ==> !fp_prog_refused15(
                #[trigger] fp_image_word(slot, img, i),
                p,
                step_max,
            ),
{
    assert forall|i: int| 0 <= i < img.len() implies fp_prog_ok(
        slot,
        i as u32,
        #[trigger] img[i],
        p,
        step_max,
    ) by {
        assert(step_word_ok(img[i]));
    }
    assert forall|i: int| 0 <= i < img.len() implies !fp_prog_refused15(
        #[trigger] fp_image_word(slot, img, i),
        p,
        step_max,
    ) by {
        assert(step_word_ok(img[i]));
        fp_prog_ok_avoids_15(slot, i as u32, img[i], p, step_max);
    }
}

/// 걷기가 runaway((4))에 이르는 슬롯 내용 — 걷기 순서는 동결된 역할이다: 「걷기는 결속된 슬롯의
/// 색인 0 의 step 에서 시작해 색인이 1 씩 느는 순서로 step 을 실행하고, 그 순서에서 처음 만난 last
/// = 1 의 step 을 실행한 뒤 끝난다 (그것을 만나지 못한 채 stepMax 개를 실행하면 … (4))」
/// (정본 `b16fc07` `03-hiu.typ:1692` — C5).  그래서 색인 0 부터 stepMax 개 가운데 last 가 없을
/// 때다(재구현도 같다 — RTL 사실: `rtl/…/fp/StepSequencer.scala:391` · `:440-453` · `:460`).
pub open spec fn fp_runaway(content: Seq<u32>, step_max: nat) -> bool {
    forall|k: int| 0 <= k < step_max && k < content.len() ==> !step_word_last(#[trigger] content[k])
}

/// 슬롯의 색인 0 부터가 `fp_image_ok` 이미지면, 그 뒤에 무엇이 남아 있든 걷기는 runaway 에
/// 이르지 않는다 — 정본의 귀결과 같다: 「이 전제와 아래 재적재 순서를 지킨 결속의 걷기는
/// 「`[15]` 의 원천」 (4) 에 이르지 않는다」(`e-defined.typ:208`).  실행 의미의 맞춤이 어긋난
/// 이미지는 틀린 결과를 낼 수는 있어도 (4) 를 내지 않는다(`:466`) — 그 방어는 플랫폼 지정(U1)뿐이다.
pub proof fn fp_image_ok_no_runaway(img: Seq<u32>, content: Seq<u32>, step_max: nat)
    requires
        fp_image_ok(img, step_max),
        img.len() <= content.len(),
        forall|i: int| 0 <= i < img.len() ==> #[trigger] content[i] == img[i],
    ensures
        !fp_runaway(content, step_max),
{
    let k = img.len() - 1;
    assert(content[k] == img[k]);
    assert(step_word_last(content[k]));
}

// --- 슬롯 장부 · 장치 모델 ----------------------------------------------------

/// 한 슬롯의 장부 상태.  쓸 수 있음(FP_PROG)과 결속할 수 있음(en = 1):
///
/// | 상태 | FP_PROG | en = 1 |
/// |---|---|---|
/// | `Unloaded { walk_free: true }` | 예 | 아니오 |
/// | `Unloaded { walk_free: false }` | 아니오 | 아니오 |
/// | `Loaded { gated: false }` | 예 | 아니오 |
/// | `Loaded { gated: true }` | 예 | 예 |
/// | `Bound` | 아니오 | 예 |
/// | `Unbound` | 아니오 | 예 |
/// | `Quiesced` | 예 | 예 |
#[derive(PartialEq, Eq, Structural)]
pub enum SlotState {
    /// 이번 reset 세대에 다 실린 이미지가 없다 — 부팅 · 장치 reset 뒤(내용 미정).  `walk_free` =
    /// 결속이 없고 그 뒤 `[1]` = 0 을 보았다(걷기가 없음을 장부가 안다).
    Unloaded { walk_free: bool },
    /// 이번 세대에 `fp_image_ok` 이미지를 색인 0 부터 다 실었다(완결 · last 보유).  `gated` = 그 창의
    /// FP 관문을 통과했다.
    Loaded { gated: bool },
    /// 장부상 결속된 열쇠가 있다.
    Bound,
    /// 결속을 모두 거뒀으나 그 뒤 `[1]` = 0 을 아직 보지 못했다 — 걷기가 남아 있을 수 있다.
    Unbound,
    /// 마지막 해제 뒤 `[1]` = 0 을 보았다 — 다시 쓸 수 있고(재적재), 이미지가 그대로이므로 다시
    /// 결속할 수 있다.
    Quiesced,
}

/// Y4 의 step 장부: 슬롯 P 개의 상태와 장부상 결속(선택 열쇠 → 슬롯).
pub struct FpLedger {
    pub slots: Seq<SlotState>,
    pub keys: Map<nat, nat>,
}

/// 장치 모델: 선택자 표(결속된 열쇠 → 슬롯)와 걷기가 진행 중인 슬롯(조회 때 래치한 슬롯).
pub struct FpDevice {
    pub sel: Map<nat, nat>,
    pub walking: Set<nat>,
}

/// 걷기가 없음을 장부가 아는 상태 — FP_PROG 를 쓸 수 있는 상태다.
pub open spec fn slot_walk_free(st: SlotState) -> bool {
    match st {
        SlotState::Unloaded { walk_free } => walk_free,
        SlotState::Loaded { .. } => true,
        SlotState::Bound => false,
        SlotState::Unbound => false,
        SlotState::Quiesced => true,
    }
}

/// 이번 세대의 관문을 통과한 완결 이미지가 그대로인 상태 — 결속할 수 있는 상태다.
pub open spec fn slot_image_ready(st: SlotState) -> bool {
    match st {
        SlotState::Unloaded { .. } => false,
        SlotState::Loaded { gated } => gated,
        SlotState::Bound => true,
        SlotState::Unbound => true,
        SlotState::Quiesced => true,
    }
}

/// FP_PROG 를 그 슬롯에 써도 된다 — 결속이 없고 걷기가 없음을 장부가 안다.
pub open spec fn fp_slot_writable(l: FpLedger, s: nat) -> bool {
    s < l.slots.len() && slot_walk_free(l.slots[s as int])
}

/// 그 슬롯에 결속(en = 1)해도 된다 — 이번 세대에 다 실리고 관문을 통과한 이미지가 그대로다.
pub open spec fn fp_slot_bindable(l: FpLedger, s: nat) -> bool {
    s < l.slots.len() && slot_image_ready(l.slots[s as int])
}

/// 장부상 그 슬롯에 결속된 열쇠가 있다.
pub open spec fn fp_key_bound_to(keys: Map<nat, nat>, s: nat) -> bool {
    exists|k: nat| #[trigger] keys.contains_key(k) && keys[k] == s
}

/// en = 1 의 장부 조건: 열쇠가 16 비트 안 · 장부상 결속되지 않은 열쇠(재결속은 쓰지 않는다) ·
/// 결속할 수 있는 슬롯.
pub open spec fn fp_key_bindable(l: FpLedger, k: nat, s: nat) -> bool {
    &&& k < 0x1_0000
    &&& !l.keys.contains_key(k)
    &&& fp_slot_bindable(l, s)
}

/// 이미지 적재의 사전조건: 쓸 수 있는 슬롯 · 장부 슬롯 수 = P · P · stepMax ≤ 256 ·
/// `fp_image_ok`.
pub open spec fn fp_load_ok(l: FpLedger, s: nat, img: Seq<u32>, p: nat, step_max: nat) -> bool {
    &&& fp_slot_writable(l, s)
    &&& l.slots.len() == p
    &&& p <= 256
    &&& step_max <= 256
    &&& fp_image_ok(img, step_max)
}

/// 장부의 꼴: 결속은 16 비트 열쇠에서 실재 슬롯으로 가고, `Bound` 는 결속된 열쇠가 있는 슬롯과
/// 정확히 같다.
pub open spec fn fp_ledger_wf(l: FpLedger) -> bool {
    &&& forall|k: nat| #[trigger]
        l.keys.contains_key(k) ==> k < 0x1_0000 && l.keys[k] < l.slots.len()
    &&& forall|s: int|
        0 <= s < l.slots.len() ==> ((#[trigger] l.slots[s] == SlotState::Bound)
            <==> fp_key_bound_to(l.keys, s as nat))
}

/// 장부가 장치를 비춘다: 장부의 꼴이 맞고, 장치의 결속은 모두 장부의 결속이며(결속이 거절 ·
/// 유실돼 장치 쪽이 적은 것은 안전하다 — 능력 부재), 걷기가 없다고 장부가 아는 슬롯에는 걷기가
/// 없다.  장치가 Y4 의 FP_BIND 를 모두 받으면 결속은 같다.
pub open spec fn fp_mirror(l: FpLedger, d: FpDevice) -> bool {
    &&& fp_ledger_wf(l)
    &&& forall|k: nat| #[trigger]
        d.sel.contains_key(k) ==> l.keys.contains_key(k) && d.sel[k] == l.keys[k]
    &&& forall|s: int|
        0 <= s < l.slots.len() && slot_walk_free(#[trigger] l.slots[s]) ==> !d.walking.contains(
            s as nat,
        )
}

/// 부팅(선택 열쇠 전 범위 en = 0 · `fp_quiesce`)과 장치 reset(「reset 뒤 선택자 표가 비어」
/// `e-defined.typ:216`) 뒤의 장부: 결속 없음 · 전 슬롯 `Unloaded`.  `quiesced` = 그 뒤 `[1]` = 0 을
/// 보았다.
pub open spec fn fp_ledger_init(p: nat, quiesced: bool) -> FpLedger {
    FpLedger {
        slots: Seq::new(p, |i: int| SlotState::Unloaded { walk_free: quiesced }),
        keys: Map::empty(),
    }
}

/// 슬롯 s 에 이미지 `img` 를 색인 0 부터 다 실었다(관문 전).  `Loaded` 로 가는 전이는 이것뿐이고,
/// 그 보존 · 관문 보조정리(`fp_mirror_load` · `fp_load_then_gate`)는 `fp_load_ok` — 곧
/// `fp_image_ok`(완결 · last) — 를 요구한다.  그래서 `Loaded` 의 「완결 · last 보유」는 이름의
/// 관례가 아니라 증명된 전이의 사전조건이다.
pub open spec fn fp_ledger_load(
    l: FpLedger,
    s: nat,
    img: Seq<u32>,
    p: nat,
    step_max: nat,
) -> FpLedger
    recommends
        fp_load_ok(l, s, img, p, step_max),
{
    FpLedger { slots: l.slots.update(s as int, SlotState::Loaded { gated: false }), keys: l.keys }
}

/// 관문 통과 — 그 창에 실린 슬롯이 결속할 수 있게 된다.
pub open spec fn slot_gate_pass(st: SlotState) -> SlotState {
    match st {
        SlotState::Loaded { .. } => SlotState::Loaded { gated: true },
        _ => st,
    }
}

pub open spec fn fp_ledger_gate_pass(l: FpLedger) -> FpLedger {
    FpLedger { slots: Seq::new(l.slots.len(), |i: int| slot_gate_pass(l.slots[i])), keys: l.keys }
}

/// en = 1 — 열쇠 k 를 슬롯 s 에 결속한다.
pub open spec fn fp_ledger_bind(l: FpLedger, k: nat, s: nat) -> FpLedger {
    FpLedger { slots: l.slots.update(s as int, SlotState::Bound), keys: l.keys.insert(k, s) }
}

/// en = 0 — 열쇠 k 의 결속을 거둔다.  그 슬롯의 마지막 결속이었으면 `Unbound` 다.
pub open spec fn fp_ledger_unbind(l: FpLedger, k: nat) -> FpLedger {
    let s = l.keys[k];
    let keys = l.keys.remove(k);
    FpLedger {
        slots: l.slots.update(
            s as int,
            if fp_key_bound_to(keys, s) {
                SlotState::Bound
            } else {
                SlotState::Unbound
            },
        ),
        keys,
    }
}

/// 장부상 결속을 모두 거둔다(`fp_unbind_all` — 나가는 임대가 결속한 열쇠 전부, 부팅은 2^16 전
/// 범위).
pub open spec fn slot_unbind_all(st: SlotState) -> SlotState {
    match st {
        SlotState::Bound => SlotState::Unbound,
        _ => st,
    }
}

pub open spec fn fp_ledger_unbind_all(l: FpLedger) -> FpLedger {
    FpLedger {
        slots: Seq::new(l.slots.len(), |i: int| slot_unbind_all(l.slots[i])),
        keys: Map::empty(),
    }
}

/// `[1]` = 0 을 보았다 — 거둔 슬롯은 `Quiesced`, 걷기 미상이던 `Unloaded` 는 걷기 없음이 된다.
pub open spec fn slot_quiesced(st: SlotState) -> SlotState {
    match st {
        SlotState::Unbound => SlotState::Quiesced,
        SlotState::Unloaded { .. } => SlotState::Unloaded { walk_free: true },
        _ => st,
    }
}

pub open spec fn fp_ledger_quiesced(l: FpLedger) -> FpLedger {
    FpLedger { slots: Seq::new(l.slots.len(), |i: int| slot_quiesced(l.slots[i])), keys: l.keys }
}

/// 장치: en = 1 은 덮어쓰기다(재결속이면 옛 슬롯이 그 열쇠의 결속을 잃는다).
pub open spec fn fp_dev_bind(d: FpDevice, k: nat, s: nat) -> FpDevice {
    FpDevice { sel: d.sel.insert(k, s), walking: d.walking }
}

/// 장치: en = 0 은 그 열쇠의 결속을 지운다(결속되지 않은 열쇠면 무연산).
pub open spec fn fp_dev_unbind(d: FpDevice, k: nat) -> FpDevice {
    FpDevice { sel: d.sel.remove(k), walking: d.walking }
}

/// 장치: 선택 열쇠 전 범위(< 2^16)에 en = 0 — 부팅 5b 의 sweep.
pub open spec fn fp_dev_unbind_all(d: FpDevice) -> FpDevice {
    FpDevice { sel: d.sel.remove_keys(Set::new(|k: nat| k < 0x1_0000)), walking: d.walking }
}

/// 장치: 열쇠 집합 `ks` 에만 en = 0 — 창의 `fp_unbind_all`(장부의 열쇠 `fp.keys` 전부)이 내는 기입 열.
pub open spec fn fp_dev_unbind_keys(d: FpDevice, ks: Set<nat>) -> FpDevice {
    FpDevice { sel: d.sel.remove_keys(ks), walking: d.walking }
}

/// 장치(외부 제시): 결속된 열쇠의 요청이 걷기를 시작한다 — 조회한 슬롯을 래치한다.  v1.0 은
/// 이것을 멈출 수단이 없다.
pub open spec fn fp_dev_walk_start(d: FpDevice, k: nat) -> FpDevice
    recommends
        d.sel.contains_key(k),
{
    FpDevice { sel: d.sel, walking: d.walking.insert(d.sel[k]) }
}

/// 장치: 걷기가 끝난다(last · runaway).
pub open spec fn fp_dev_walk_end(d: FpDevice, s: nat) -> FpDevice {
    FpDevice { sel: d.sel, walking: d.walking.remove(s) }
}

// --- 장부 불변식의 보존 --------------------------------------------------------

/// 결속이 비고 (`quiesced` 면) 걷기도 없는 장치에서 부팅 · reset 장부는 장치를 비춘다.
pub proof fn fp_mirror_init(p: nat, quiesced: bool, d: FpDevice)
    requires
        d.sel.dom() =~= Set::<nat>::empty(),
        quiesced ==> d.walking =~= Set::<nat>::empty(),
    ensures
        fp_mirror(fp_ledger_init(p, quiesced), d),
{
    let l = fp_ledger_init(p, quiesced);
    assert forall|s: int| 0 <= s < l.slots.len() implies ((#[trigger] l.slots[s]
        == SlotState::Bound) <==> fp_key_bound_to(l.keys, s as nat)) by {
        assert(l.slots[s] == SlotState::Unloaded { walk_free: quiesced });
        if fp_key_bound_to(l.keys, s as nat) {
            let k = choose|k: nat| #[trigger] l.keys.contains_key(k) && l.keys[k] == s as nat;
            assert(l.keys.dom().contains(k));
        }
    }
    assert forall|k: nat| #[trigger] d.sel.contains_key(k) implies l.keys.contains_key(k)
        && d.sel[k] == l.keys[k] by {
        assert(d.sel.dom().contains(k));
    }
    assert forall|s: int|
        0 <= s < l.slots.len() && slot_walk_free(#[trigger] l.slots[s]) implies !d.walking.contains(
        s as nat,
    ) by {
        assert(l.slots[s] == SlotState::Unloaded { walk_free: quiesced });
    }
}

/// 장치의 결속이 모두 16 비트 열쇠면 전 범위 en = 0 뒤 선택자 표는 빈다.
pub proof fn fp_dev_unbind_all_clears(d: FpDevice)
    requires
        forall|k: nat| #[trigger] d.sel.contains_key(k) ==> k < 0x1_0000,
    ensures
        fp_dev_unbind_all(d).sel.dom() =~= Set::<nat>::empty(),
{
}

/// FP_PROG 대상 슬롯에는 결속도 걷기도 없다 — 「결속된 슬롯에는 `0xF00` 를 쓰지 않는다」와
/// 재적재 순서(E.1.1 4항, `e-defined.typ:208`)의 이행.
pub proof fn fp_prog_target_idle(l: FpLedger, d: FpDevice, s: nat)
    requires
        fp_mirror(l, d),
        fp_slot_writable(l, s),
    ensures
        !d.walking.contains(s),
        !fp_key_bound_to(l.keys, s),
        forall|k: nat| #[trigger] d.sel.contains_key(k) ==> d.sel[k] != s,
{
    assert(slot_walk_free(l.slots[s as int]));
    assert(l.slots[s as int] != SlotState::Bound);
    assert forall|k: nat| #[trigger] d.sel.contains_key(k) implies d.sel[k] != s by {
        if d.sel[k] == s {
            assert(l.keys.contains_key(k) && l.keys[k] == s);
            assert(fp_key_bound_to(l.keys, s));
        }
    }
}

/// 결속된 슬롯은 쓸 수 없다.
pub proof fn fp_bound_not_writable(l: FpLedger, s: nat)
    requires
        fp_ledger_wf(l),
        s < l.slots.len(),
        fp_key_bound_to(l.keys, s),
    ensures
        !fp_slot_writable(l, s),
        l.slots[s as int] == SlotState::Bound,
{
}

/// 이미지 적재는 불변식을 지킨다(장치의 선택자 표 · 걷기는 FP_PROG 로 바뀌지 않는다).
pub proof fn fp_mirror_load(l: FpLedger, d: FpDevice, s: nat, img: Seq<u32>, p: nat, step_max: nat)
    requires
        fp_mirror(l, d),
        fp_load_ok(l, s, img, p, step_max),
    ensures
        fp_mirror(fp_ledger_load(l, s, img, p, step_max), d),
{
    let l2 = fp_ledger_load(l, s, img, p, step_max);
    fp_prog_target_idle(l, d, s);
    assert forall|i: int| 0 <= i < l2.slots.len() implies ((#[trigger] l2.slots[i]
        == SlotState::Bound) <==> fp_key_bound_to(l2.keys, i as nat)) by {
        if i != s as int {
            assert(l2.slots[i] == l.slots[i]);
        }
    }
    assert forall|i: int|
        0 <= i < l2.slots.len() && slot_walk_free(
            #[trigger] l2.slots[i],
        ) implies !d.walking.contains(i as nat) by {
        if i != s as int {
            assert(l2.slots[i] == l.slots[i]);
        }
    }
}

/// 이미지 적재의 기입은 모두 `fp_prog_ok` 이고 (1) · (2) 에 걸리지 않는다.
pub proof fn fp_load_ok_words(l: FpLedger, s: nat, img: Seq<u32>, p: nat, step_max: nat)
    requires
        fp_load_ok(l, s, img, p, step_max),
    ensures
        (s as u32) as nat == s,
        forall|i: int|
            0 <= i < img.len() ==> fp_prog_ok(s as u32, i as u32, #[trigger] img[i], p, step_max),
        forall|i: int|
            0 <= i < img.len() ==> !fp_prog_refused15(
                #[trigger] fp_image_word(s as u32, img, i),
                p,
                step_max,
            ),
{
    fp_image_ok_prog_ok(s as u32, img, p, step_max);
}

/// 관문 통과는 불변식을 지킨다.
pub proof fn fp_mirror_gate_pass(l: FpLedger, d: FpDevice)
    requires
        fp_mirror(l, d),
    ensures
        fp_mirror(fp_ledger_gate_pass(l), d),
{
    let l2 = fp_ledger_gate_pass(l);
    assert forall|i: int| 0 <= i < l2.slots.len() implies ((#[trigger] l2.slots[i]
        == SlotState::Bound) <==> fp_key_bound_to(l2.keys, i as nat)) by {
        assert(l2.slots[i] == slot_gate_pass(l.slots[i]));
    }
    assert forall|i: int|
        0 <= i < l2.slots.len() && slot_walk_free(
            #[trigger] l2.slots[i],
        ) implies !d.walking.contains(i as nat) by {
        assert(l2.slots[i] == slot_gate_pass(l.slots[i]));
    }
}

/// 적재만으로는 결속할 수 없고, 관문을 통과해야 결속할 수 있다.  적재는 `fp_load_ok`(완결 · last
/// 이미지 — `fp_image_ok`) 아래에서만 하므로, 결속할 수 있게 된 슬롯의 이미지는 그 꼴이다.
pub proof fn fp_load_then_gate(l: FpLedger, s: nat, img: Seq<u32>, p: nat, step_max: nat)
    requires
        fp_load_ok(l, s, img, p, step_max),
    ensures
        fp_image_ok(img, step_max),
        !fp_slot_bindable(fp_ledger_load(l, s, img, p, step_max), s),
        fp_slot_bindable(fp_ledger_gate_pass(fp_ledger_load(l, s, img, p, step_max)), s),
        fp_slot_writable(fp_ledger_load(l, s, img, p, step_max), s),
{
    let l1 = fp_ledger_load(l, s, img, p, step_max);
    assert(l1.slots[s as int] == SlotState::Loaded { gated: false });
    assert(fp_ledger_gate_pass(l1).slots[s as int] == SlotState::Loaded { gated: true });
}

/// 부팅 · reset 뒤에는 어느 슬롯도 결속할 수 없다 — 결속하려면 이번 세대에 다시 싣는다.
pub proof fn fp_init_nothing_bindable(p: nat, quiesced: bool, s: nat)
    ensures
        !fp_slot_bindable(fp_ledger_init(p, quiesced), s),
        s < p ==> (fp_slot_writable(fp_ledger_init(p, quiesced), s) <==> quiesced),
{
    if s < p {
        assert(fp_ledger_init(p, quiesced).slots[s as int] == SlotState::Unloaded {
            walk_free: quiesced,
        });
    }
}

/// en = 1 결속은 불변식을 지킨다(장치가 받든, (3) 등으로 받지 않든).
pub proof fn fp_mirror_bind(l: FpLedger, d: FpDevice, k: nat, s: nat)
    requires
        fp_mirror(l, d),
        fp_key_bindable(l, k, s),
    ensures
        fp_mirror(fp_ledger_bind(l, k, s), fp_dev_bind(d, k, s)),
        fp_mirror(fp_ledger_bind(l, k, s), d),
{
    let l2 = fp_ledger_bind(l, k, s);
    let d2 = fp_dev_bind(d, k, s);
    assert forall|k2: nat| #[trigger] l2.keys.contains_key(k2) implies k2 < 0x1_0000 && l2.keys[k2]
        < l2.slots.len() by {
        if k2 != k {
            assert(l.keys.contains_key(k2));
        }
    }
    assert forall|i: int| 0 <= i < l2.slots.len() implies ((#[trigger] l2.slots[i]
        == SlotState::Bound) <==> fp_key_bound_to(l2.keys, i as nat)) by {
        if i == s as int {
            assert(l2.keys.contains_key(k) && l2.keys[k] == s);
        } else {
            assert(l2.slots[i] == l.slots[i]);
            if fp_key_bound_to(l.keys, i as nat) {
                let k2 = choose|k2: nat| #[trigger]
                    l.keys.contains_key(k2) && l.keys[k2] == i as nat;
                assert(k2 != k);
                assert(l2.keys.contains_key(k2) && l2.keys[k2] == i as nat);
            }
            if fp_key_bound_to(l2.keys, i as nat) {
                let k2 = choose|k2: nat| #[trigger]
                    l2.keys.contains_key(k2) && l2.keys[k2] == i as nat;
                assert(k2 != k);
                assert(l.keys.contains_key(k2) && l.keys[k2] == i as nat);
            }
        }
    }
    assert forall|k2: nat| #[trigger] d2.sel.contains_key(k2) implies l2.keys.contains_key(k2)
        && d2.sel[k2] == l2.keys[k2] by {
        if k2 != k {
            assert(d.sel.contains_key(k2));
        }
    }
    assert forall|k2: nat| #[trigger] d.sel.contains_key(k2) implies l2.keys.contains_key(k2)
        && d.sel[k2] == l2.keys[k2] by {
        assert(l.keys.contains_key(k2));
        assert(k2 != k);
    }
    assert forall|i: int|
        0 <= i < l2.slots.len() && slot_walk_free(
            #[trigger] l2.slots[i],
        ) implies !d2.walking.contains(i as nat) by {
        if i != s as int {
            assert(l2.slots[i] == l.slots[i]);
        }
    }
}

/// 장부상 결속된 열쇠에 en = 1 을 다시 쓰면(재결속) 옛 슬롯이 결속을 잃는데 장부는 그것을
/// `Bound` 로 남긴다 — 장부의 꼴이 깨지고, 옛 슬롯이 `Unbound` 를 거치지 않아 재적재 전의
/// `[1]` = 0 확인이 빠질 수 있다.  그래서 en = 1 은 장부상 결속되지 않은 열쇠에만 낸다
/// (`fp_key_bindable`) — 열쇠를 다른 슬롯으로 옮기려면 en = 0 으로 먼저 거둔다.
pub proof fn fp_rebind_breaks_wf(l: FpLedger, k: nat, s: nat)
    requires
        fp_ledger_wf(l),
        l.keys.contains_key(k),
        l.keys[k] != s,
        s < l.slots.len(),
        forall|k2: nat| #[trigger] l.keys.contains_key(k2) && k2 != k ==> l.keys[k2] != l.keys[k],
    ensures
        !fp_ledger_wf(fp_ledger_bind(l, k, s)),
{
    let s0 = l.keys[k];
    let l2 = fp_ledger_bind(l, k, s);
    assert(fp_key_bound_to(l.keys, s0));
    assert(l.slots[s0 as int] == SlotState::Bound);
    assert(l2.slots[s0 as int] == SlotState::Bound);
    if fp_key_bound_to(l2.keys, s0) {
        let k2 = choose|k2: nat| #[trigger] l2.keys.contains_key(k2) && l2.keys[k2] == s0;
        assert(k2 != k);
        assert(l.keys.contains_key(k2) && l.keys[k2] == s0);
    }
}

/// en = 0 해제는 불변식을 지킨다.
pub proof fn fp_mirror_unbind(l: FpLedger, d: FpDevice, k: nat)
    requires
        fp_mirror(l, d),
        l.keys.contains_key(k),
    ensures
        fp_mirror(fp_ledger_unbind(l, k), fp_dev_unbind(d, k)),
{
    let s = l.keys[k];
    let l2 = fp_ledger_unbind(l, k);
    let d2 = fp_dev_unbind(d, k);
    assert(l2.keys == l.keys.remove(k));
    assert forall|k2: nat| #[trigger] l2.keys.contains_key(k2) implies k2 < 0x1_0000 && l2.keys[k2]
        < l2.slots.len() by {
        assert(l.keys.contains_key(k2));
    }
    assert forall|i: int| 0 <= i < l2.slots.len() implies ((#[trigger] l2.slots[i]
        == SlotState::Bound) <==> fp_key_bound_to(l2.keys, i as nat)) by {
        if i != s as int {
            assert(l2.slots[i] == l.slots[i]);
            if fp_key_bound_to(l.keys, i as nat) {
                let k2 = choose|k2: nat| #[trigger]
                    l.keys.contains_key(k2) && l.keys[k2] == i as nat;
                assert(k2 != k);
                assert(l2.keys.contains_key(k2) && l2.keys[k2] == i as nat);
            }
            if fp_key_bound_to(l2.keys, i as nat) {
                let k2 = choose|k2: nat| #[trigger]
                    l2.keys.contains_key(k2) && l2.keys[k2] == i as nat;
                assert(l.keys.contains_key(k2) && l.keys[k2] == i as nat);
            }
        }
    }
    assert forall|k2: nat| #[trigger] d2.sel.contains_key(k2) implies l2.keys.contains_key(k2)
        && d2.sel[k2] == l2.keys[k2] by {
        assert(d.sel.contains_key(k2));
    }
    assert forall|i: int|
        0 <= i < l2.slots.len() && slot_walk_free(
            #[trigger] l2.slots[i],
        ) implies !d2.walking.contains(i as nat) by {
        if i != s as int {
            assert(l2.slots[i] == l.slots[i]);
        }
    }
}

/// 결속을 모두 거두는 것은 불변식을 지키고, 장치의 선택자 표를 비운다.
pub proof fn fp_mirror_unbind_all(l: FpLedger, d: FpDevice)
    requires
        fp_mirror(l, d),
    ensures
        fp_mirror(fp_ledger_unbind_all(l), fp_dev_unbind_all(d)),
        fp_dev_unbind_all(d).sel.dom() =~= Set::<nat>::empty(),
{
    let l2 = fp_ledger_unbind_all(l);
    let d2 = fp_dev_unbind_all(d);
    assert forall|k: nat| #[trigger] d.sel.contains_key(k) implies k < 0x1_0000 by {
        assert(l.keys.contains_key(k));
    }
    fp_dev_unbind_all_clears(d);
    assert forall|i: int| 0 <= i < l2.slots.len() implies ((#[trigger] l2.slots[i]
        == SlotState::Bound) <==> fp_key_bound_to(l2.keys, i as nat)) by {
        assert(l2.slots[i] == slot_unbind_all(l.slots[i]));
        if fp_key_bound_to(l2.keys, i as nat) {
            let k = choose|k: nat| #[trigger] l2.keys.contains_key(k) && l2.keys[k] == i as nat;
            assert(l2.keys.dom().contains(k));
        }
    }
    assert forall|k: nat| #[trigger] d2.sel.contains_key(k) implies l2.keys.contains_key(k)
        && d2.sel[k] == l2.keys[k] by {
        assert(d2.sel.dom().contains(k));
    }
    assert forall|i: int|
        0 <= i < l2.slots.len() && slot_walk_free(
            #[trigger] l2.slots[i],
        ) implies !d2.walking.contains(i as nat) by {
        assert(l2.slots[i] == slot_unbind_all(l.slots[i]));
    }
}

/// 창의 `fp_unbind_all` — 장부의 열쇠에만 en = 0 을 쓰는 실제 기입 열 — 도 불변식을 지키고 장치의
/// 선택자 표를 비운다(`fp_mirror` 아래 장치의 결속은 장부 열쇠의 부분집합이다).
/// `fp_mirror_unbind_all` 은 부팅 5b 의 전 범위 sweep 용이다.
pub proof fn fp_mirror_unbind_ledger_keys(l: FpLedger, d: FpDevice)
    requires
        fp_mirror(l, d),
    ensures
        fp_mirror(fp_ledger_unbind_all(l), fp_dev_unbind_keys(d, l.keys.dom())),
        fp_dev_unbind_keys(d, l.keys.dom()).sel.dom() =~= Set::<nat>::empty(),
{
    let l2 = fp_ledger_unbind_all(l);
    let d2 = fp_dev_unbind_keys(d, l.keys.dom());
    assert forall|k: nat| !(#[trigger] d2.sel.dom().contains(k)) by {
        if d.sel.contains_key(k) {
            assert(l.keys.contains_key(k));
        }
    }
    assert forall|i: int| 0 <= i < l2.slots.len() implies ((#[trigger] l2.slots[i]
        == SlotState::Bound) <==> fp_key_bound_to(l2.keys, i as nat)) by {
        assert(l2.slots[i] == slot_unbind_all(l.slots[i]));
        if fp_key_bound_to(l2.keys, i as nat) {
            let k = choose|k: nat| #[trigger] l2.keys.contains_key(k) && l2.keys[k] == i as nat;
            assert(l2.keys.dom().contains(k));
        }
    }
    assert forall|k: nat| #[trigger] d2.sel.contains_key(k) implies l2.keys.contains_key(k)
        && d2.sel[k] == l2.keys[k] by {
        assert(d2.sel.dom().contains(k));
    }
    assert forall|i: int|
        0 <= i < l2.slots.len() && slot_walk_free(
            #[trigger] l2.slots[i],
        ) implies !d2.walking.contains(i as nat) by {
        assert(l2.slots[i] == slot_unbind_all(l.slots[i]));
    }
}

/// `[1]` = 0 의 관측(그때 걷기가 없다)은 불변식을 지킨다.
pub proof fn fp_mirror_quiesced(l: FpLedger, d: FpDevice)
    requires
        fp_mirror(l, d),
        d.walking =~= Set::<nat>::empty(),
    ensures
        fp_mirror(fp_ledger_quiesced(l), d),
{
    let l2 = fp_ledger_quiesced(l);
    assert forall|i: int| 0 <= i < l2.slots.len() implies ((#[trigger] l2.slots[i]
        == SlotState::Bound) <==> fp_key_bound_to(l2.keys, i as nat)) by {
        assert(l2.slots[i] == slot_quiesced(l.slots[i]));
    }
}

/// 외부 제시가 결속된 열쇠로 걷기를 시작해도 불변식이 선다 — 그 슬롯은 장부상 `Bound` 다.
pub proof fn fp_mirror_walk_start(l: FpLedger, d: FpDevice, k: nat)
    requires
        fp_mirror(l, d),
        d.sel.contains_key(k),
    ensures
        fp_mirror(l, fp_dev_walk_start(d, k)),
{
    let s = d.sel[k];
    assert(l.keys.contains_key(k) && l.keys[k] == s);
    assert(fp_key_bound_to(l.keys, s));
    assert(l.slots[s as int] == SlotState::Bound);
}

/// 걷기의 끝은 불변식을 지킨다.
pub proof fn fp_mirror_walk_end(l: FpLedger, d: FpDevice, s: nat)
    requires
        fp_mirror(l, d),
    ensures
        fp_mirror(l, fp_dev_walk_end(d, s)),
{
}

/// 재적재 순서: 슬롯의 마지막 결속을 거둔 직후에는 쓸 수 없고(`Unbound` — 걷기가 남아 있을 수
/// 있다), 그 뒤 `[1]` = 0 을 보아야 쓸 수 있다.  이미지는 그대로이므로 두 상태 모두 다시 결속할
/// 수 있다.
pub proof fn fp_reload_needs_quiesce(l: FpLedger, k: nat)
    requires
        fp_ledger_wf(l),
        l.keys.contains_key(k),
        forall|k2: nat| #[trigger] l.keys.contains_key(k2) && k2 != k ==> l.keys[k2] != l.keys[k],
    ensures
        ({
            let s = l.keys[k];
            let u = fp_ledger_unbind(l, k);
            &&& u.slots[s as int] == SlotState::Unbound
            &&& !fp_slot_writable(u, s)
            &&& fp_slot_bindable(u, s)
            &&& fp_slot_writable(fp_ledger_quiesced(u), s)
            &&& fp_slot_bindable(fp_ledger_quiesced(u), s)
        }),
{
    let s = l.keys[k];
    let u = fp_ledger_unbind(l, k);
    if fp_key_bound_to(u.keys, s) {
        let k2 = choose|k2: nat| #[trigger] u.keys.contains_key(k2) && u.keys[k2] == s;
        assert(l.keys.contains_key(k2) && k2 != k);
    }
    assert(u.slots[s as int] == SlotState::Unbound);
    assert(fp_ledger_quiesced(u).slots[s as int] == SlotState::Quiesced);
}

/// 결속을 모두 거두고 `[1]` = 0 을 보면 모든 슬롯을 다시 쓸 수 있다 — 창이 cur 의 열쇠를
/// 전부 푸는 까닭이다(일부만 풀면 남은 결속의 투입이 `[1]` 을 내리지 않을 수 있다).
pub proof fn fp_quiesced_all_writable(l: FpLedger, d: FpDevice, s: nat)
    requires
        fp_mirror(l, d),
        d.walking =~= Set::<nat>::empty(),
        s < l.slots.len(),
    ensures
        fp_slot_writable(fp_ledger_quiesced(fp_ledger_unbind_all(l)), s),
        fp_mirror(fp_ledger_quiesced(fp_ledger_unbind_all(l)), fp_dev_unbind_all(d)),
{
    fp_mirror_unbind_all(l, d);
    fp_mirror_quiesced(fp_ledger_unbind_all(l), fp_dev_unbind_all(d));
    let u = fp_ledger_unbind_all(l);
    assert(u.slots[s as int] == slot_unbind_all(l.slots[s as int]));
    assert(fp_ledger_quiesced(u).slots[s as int] == slot_quiesced(u.slots[s as int]));
}

/// 장부 조건을 지킨 en = 1 은 FP_BIND 워드로도 `fp_bind_ok` 이고 거절되지 않는다.
pub proof fn fp_key_bindable_word_ok(l: FpLedger, k: nat, s: nat, p: nat)
    requires
        fp_key_bindable(l, k, s),
        l.slots.len() == p,
        p <= 256,
    ensures
        (s as u32) as nat == s,
        (k as u32) as nat == k,
        fp_bind_ok(s as u32, k as u32, p),
        !fp_bind_refused6(fp_bind_word(true, s as u32, k as u32)),
        !fp_bind_refused15(fp_bind_word(true, s as u32, k as u32), p),
{
    fp_bind_ok_avoids_refusal(s as u32, k as u32, p);
}

// --- 정지 확인 (`fp_quiesce`) · FP 관문 ---------------------------------------

/// `fp_quiesce` 의 처분.  `saw_idle` = 마지막 결속 해제 기입 뒤에 수락된 `0xF08` 읽기가 `T_WALK`
/// 안에 `[1]` = 0 을 보았다, `misses` = 그 앞에 연이어 소진된 창의 수, `k` = 연속 소진 상한 `K_WALK`
/// (플랫폼 레지스트리, ≥ 1).  관측이면 진행, 소진이면 이 창의 적재를 생략하고, `K_WALK` 창 연속
/// 소진이면 step 기능을 끈다(`StepOff` — 사용자 결정 2026-10-22).  적합 장치도 외부 투입이 `[1]`
/// 을 계속 세우면 여기에 들 수 있으므로(「그때 재적재는 기다린다」 `e-defined.typ:208` · v1.0 은
/// 투입을 멈출 수단이 없다 `:332`) 장치 격리로 번지지 않게 한다.
pub open spec fn fp_quiesce_action(saw_idle: bool, misses: nat, k: nat) -> StepAction {
    if saw_idle {
        StepAction::Proceed
    } else if misses + 1 >= k {
        StepAction::StepOff
    } else {
        StepAction::SkipLoad
    }
}

/// 다음 창으로 넘기는 연속 소진 수 — 관측하면 0 으로 되돌린다.
pub open spec fn fp_quiesce_misses(saw_idle: bool, misses: nat) -> nat {
    if saw_idle {
        0
    } else {
        misses + 1
    }
}

/// 망라: 세 처분은 각각 입력 술어와 동치이고 정지(`LeaseInvariant`)에도 장치 격리(`DeviceFault`)
/// 에도 이르지 않는다.
pub proof fn fp_quiesce_action_exhaustive(saw_idle: bool, misses: nat, k: nat)
    requires
        k >= 1,
    ensures
        (fp_quiesce_action(saw_idle, misses, k) == StepAction::Proceed) <==> saw_idle,
        (fp_quiesce_action(saw_idle, misses, k) == StepAction::SkipLoad) <==> (!saw_idle && misses
            + 1 < k),
        (fp_quiesce_action(saw_idle, misses, k) == StepAction::StepOff) <==> (!saw_idle && misses
            + 1 >= k),
        fp_quiesce_action(saw_idle, misses, k) != StepAction::Fail(Y4Error::LeaseInvariant),
        fp_quiesce_action(saw_idle, misses, k) != StepAction::Fail(Y4Error::DeviceFault),
        saw_idle ==> fp_quiesce_misses(saw_idle, misses) == 0,
{
}

/// 0 에서 시작해 연이어 소진되면 처음 `K_WALK` − 1 창은 적재를 생략하고 `K_WALK` 번째 창에서 step
/// 기능을 끈다.
pub proof fn fp_quiesce_k_consecutive(k: nat)
    requires
        k >= 1,
    ensures
        forall|j: nat|
            j + 1 < k ==> #[trigger] fp_quiesce_action(false, j, k) == StepAction::SkipLoad,
        fp_quiesce_action(false, (k - 1) as nat, k) == StepAction::StepOff,
        forall|j: nat| #[trigger] fp_quiesce_misses(false, j) == j + 1,
{
}

/// FP 관문 결과.  `g` = 창의 FP_PROG 를 다 쓴 뒤 `T_FLT` 를 기다려 읽은 `0xF80[15]`.
/// Y4 는 (1) · (2) 를 기입 전에 피하고 관문 앞에서 결속을 모두 거둬 `[1]` = 0 을 보았으므로, 1 이면
/// 적합 경로에서 날 수 없는 `[15]` 다 — 다시 싣지 않고(사용자 결정 2026-10-22 — rc3 의 1회 재적재를
/// 지웠다) 결속하지 않으며 `[19]` 선례로 귀속한다(`step_gate_action`).
#[derive(PartialEq, Eq, Structural)]
pub enum StepGate {
    /// 0 — 결속한다.
    Clean,
    /// 1 — 결속하지 않는다.  귀속은 `crate::error::step_fault_origin`.
    Refused,
}

pub open spec fn step_gate(g: bool) -> StepGate {
    if g {
        StepGate::Refused
    } else {
        StepGate::Clean
    }
}

/// 관문을 지나 결속하는가 — 관문을 통과한 슬롯만 `fp_ledger_gate_pass` 로 결속할 수 있게 된다.
pub open spec fn step_gate_binds(g: StepGate) -> bool {
    g == StepGate::Clean
}

/// 관문의 처분.  `step_ledger_ok` = `lease::window::step_fault_verdict` 와 같은 Y4 자기 점검 —
/// 장부가 맞으면 귀속 불가(`DeviceFault`), 어긋나면 Y4 의도 결함(`LeaseInvariant`).
pub open spec fn step_gate_action(g: StepGate, step_ledger_ok: bool) -> StepAction {
    match g {
        StepGate::Clean => StepAction::Proceed,
        StepGate::Refused => StepAction::Fail(
            crate::error::fault_disposition(crate::error::step_fault_origin(step_ledger_ok)),
        ),
    }
}

/// 망라: 관문은 `[15]` 를 볼 때만 결속하지 않고, 그때 장부가 맞으면 `DeviceFault`, 어긋나면
/// `LeaseInvariant` 다.
pub proof fn step_gate_exhaustive(g: bool, step_ledger_ok: bool)
    ensures
        (step_gate(g) == StepGate::Clean) <==> !g,
        (step_gate(g) == StepGate::Refused) <==> g,
        step_gate_binds(step_gate(g)) <==> !g,
        (step_gate_action(step_gate(g), step_ledger_ok) == StepAction::Fail(Y4Error::DeviceFault))
            <==> (g && step_ledger_ok),
        (step_gate_action(step_gate(g), step_ledger_ok) == StepAction::Fail(
            Y4Error::LeaseInvariant,
        )) <==> (g && !step_ledger_ok),
{
}

/// step 경로의 처분(`0xF0C` 판정 · 정지 확인 · FP 관문)이 `LeaseInvariant` 에 이르는 것은 FP
/// 관문이 `[15]` 를 보았고 Y4 step 장부가 어긋났을 때뿐이다 — 그 밖의 결과는 진행 · 적재 생략 ·
/// step 기능 끔 · `DeviceFault` 다.  정지 확인은 장치 격리에도 이르지 않는다.
pub proof fn step_path_halts_only_on_ledger(
    w: u32,
    reg: Option<(nat, nat)>,
    saw_idle: bool,
    misses: nat,
    k: nat,
    g: bool,
    step_ledger_ok: bool,
)
    ensures
        fp_shape_action(fp_shape_verdict(w, reg)) != StepAction::Fail(Y4Error::LeaseInvariant),
        fp_quiesce_action(saw_idle, misses, k) != StepAction::Fail(Y4Error::LeaseInvariant),
        fp_quiesce_action(saw_idle, misses, k) != StepAction::Fail(Y4Error::DeviceFault),
        (step_gate_action(step_gate(g), step_ledger_ok) == StepAction::Fail(
            Y4Error::LeaseInvariant,
        )) <==> (g && !step_ledger_ok),
{
}

} // verus!
