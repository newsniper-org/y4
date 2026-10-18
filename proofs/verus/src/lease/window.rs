// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Global configuration window — `FAULT_STATUS` window-refusal verdict
//! (`docs/hiu_abi.md` §2.4 3 · §2.7 · §1.3, `docs/lease_capability.md` I4 2b · I7).
//!
//! 창 기입의 거절은 `[6]` 하나로 오지 않는다.  창 거절 집합 W 는 넷이다
//! (정본 `f6bc4df` 비트 이름 `e-defined.typ:519-521`):
//!
//! - `[5]` 「구성 — 소유 서로소 위반 (D2)」 — LNID V=1 기입의 서로소 위반
//!   (`01-isa-asm.typ:1303-1304` 「중복 소유를 만드는 사상표 MMIO
//!   쓰기(…)는 거절되고 loud fault 를 낸다.」 · `03-hiu.typ:1096` 「`0xF80` `[5]` 가 따로
//!   받고」).  원천이 Y4 기입뿐이고 엔트리에 파티션 태그가 없으므로
//!   (`03-hiu.typ:992-995`) 언제 섰든 Y4 결함이다.
//! - `[6]` 「구성 — 거절된 구성 기입」 — `0xC00` V=0 · `0xE10` · `0xE00` · LNID 의
//!   3.7.1 넷 · LBID 예약 비트 · FP_BIND 예약 비트(「예약 검사가 먼저다」
//!   `e-defined.typ:461`).  Y4 결함.
//! - `[7]` 「페이지 소유권 표 충돌 — 형상 · 점유에 따른 `0x208` OWN_SET 거절 (E.7 6항)」 — 원천은
//!   하나다: 「`[7]` 은 E.7 6항의 OWN_SET 거절 하나」(`e-defined.typ:527`).  「장치는 OWN_SET 을
//!   거절할 수 있다 — 그 프레임이 장치 소유로 앉을 수 있는 저장 칸이 전부 다른 장치 소유 페이지로
//!   차 있는 경우다」 · 「거절은 무효과다 — 어느 페이지의 상태도 바뀌지 않고, 대상은 호스트 소유로
//!   남으며, 아무것도 축출하지 않는다」 · 「어느 프레임이 거절되는가는 구현의 형상이 정하고 호스트
//!   계약이 아니다」(E.7 6항 `:1387`).  그래서 Y4 는 같은 창의 OWN_SET 되읽기 거절로 설명되는
//!   `[7]` 을 `OwnSetRefused` 로 받는다 — 적합 드라이버의 정상 순서에서도 선다(재구현은 64 칸 직접
//!   사상이다, RTL 사실 — 규범 아님: `rtl/…/hiu/OwnershipTable.scala:159` · `:318-329` ·
//!   `rtl/…/top/Wt64Top.scala:1550`).
//! - `[15]` 「step 프로그램 불량」 — 원천은 닫힌 넷이다(`e-defined.typ:464`, `lease::step`).  Y4 는
//!   (1)–(3) 을 기입 전에 피하므로 Y4 가 보는 `[15]` 는 (4) runaway 이거나 Y4 · 플랫폼 쪽의 의무
//!   위반이고, 「그 (4) 가 어느 결속 · 어느 걷기의 것인지는 v1.0 이 가르지 않는다」 · 「시점은
//!   판별의 근거가 아니다」(`:466`).  그래서 `[15]` 는 어느 임대에도 귀속하지 않는다 — 운영자 ·
//!   플랫폼 기록(`StepUnattributed`)이고 테넌트 보고도 정지도 아니다.  v1.0 에서 Y4 는 플랫폼이
//!   지정한 이미지만 싣는다(사용자 결정 U1) — (4) 는 플랫폼 · 도구 사슬의 결함이다.
//!
//! 창은 둘로 읽는다 (§2.4 3):
//!
//! - (가) `hiu_cfg` 를 쥔 뒤 첫 창 기입 전에 `0xF80` ∧ W 를 읽어 선 비트를 창 전
//!   사건으로 처분하고 해제한다 — `dispose_pre_window`.
//! - 창 동안(§2.2) 일반 fault 처리기는 W 를 락 보유자에게 넘기고 처분 · 해제하지
//!   않는다.  step 경로만 예외다 — 정지 확인 뒤의 꼬리 읽기와 FP 관문(`lease::step::StepGate`)은
//!   본 `[15]` 를 그 자리에서 미귀속으로 기록하고 해제한다.
//! - (나) 창의 마지막 기입(FP_BIND en = 1 이 있으면 그것) 뒤 `T_FLT`(플랫폼 레지스트리 — 기입
//!   거절 비트 가시 대기 상한)를 기다렸다가 `0xF80` ∧ W 를 읽고 `window_verdict` 로 처분한 뒤
//!   해제한다.
//!
//! 정지(`LeaseInvariant`)에 이르는 것은 `[5]` · `[6]` 뿐이다
//! (`window_halts_only_on_5_6` · `pre_window_halts_only_on_5_6`).  `[15]` 는 판정을 바꾸지 않고
//! 기록만 더한다(`fault15_only_records`).  이 확인이 보여 주는 것은 「장치가 거절로 알린 창 기입이
//! 없었다」까지다 — 겹치는 LBID 구간은 판정 없이 수락되고(`03-hiu.typ:1094`) 범위 밖 시작은
//! 처분이 정본에 없으므로(`b-gaps.typ:633-639` — 재구현은 수락, RTL 사실) `lease::lbid` 사전조건만이
//! 막는다.
//!
//! `Y4Defect` 는 `Y4Error::LeaseInvariant`(하이퍼바이저 정지)이고
//! `DeviceContradiction` 은 `Y4Error::DeviceFault`(장치 격리)다.  새 `Y4Error`
//! 변형은 두지 않는다.

use vstd::prelude::*;

use crate::error::Y4Error;

verus! {

/// `0xF80[5]` 구성 — 소유 서로소 위반 (D2).
pub const FAULT_DUP_OWN: u32 = 0x0000_0020;

/// `0xF80[6]` 구성 — 거절된 구성 기입.
pub const FAULT_BAD_CFG_WRITE: u32 = 0x0000_0040;

/// `0xF80[7]` 페이지 소유권 표 충돌 — 형상 · 점유에 따른 OWN_SET 거절 (E.7 6항).
pub const FAULT_OWN_COLLIDE: u32 = 0x0000_0080;

/// `0xF80[15]` step 프로그램 불량.
pub const FAULT_STEP_PROG: u32 = 0x0000_8000;

/// 창 거절 집합 W = `(1 << 5) | (1 << 6) | (1 << 7) | (1 << 15)`.
pub const W_MASK: u32 = 0x0000_80e0;

/// W 가 정확히 네 비트의 합이고 각 상수가 제 자리 비트라는 것.
pub proof fn w_mask_is_four_bits()
    ensures
        W_MASK == (FAULT_DUP_OWN | FAULT_BAD_CFG_WRITE | FAULT_OWN_COLLIDE | FAULT_STEP_PROG),
        FAULT_DUP_OWN == 1u32 << 5u32,
        FAULT_BAD_CFG_WRITE == 1u32 << 6u32,
        FAULT_OWN_COLLIDE == 1u32 << 7u32,
        FAULT_STEP_PROG == 1u32 << 15u32,
{
    assert(0x80e0u32 == (0x20u32 | 0x40u32 | 0x80u32 | 0x8000u32)) by (bit_vector);
    assert(0x20u32 == 1u32 << 5u32) by (bit_vector);
    assert(0x40u32 == 1u32 << 6u32) by (bit_vector);
    assert(0x80u32 == 1u32 << 7u32) by (bit_vector);
    assert(0x8000u32 == 1u32 << 15u32) by (bit_vector);
}

/// 한 `0xF80` 읽기에서 마스크 비트 `m` 이 섰는가.
pub open spec fn has_bit(f: u32, m: u32) -> bool {
    f & m != 0
}

/// W 의 네 비트를 W 로 가린 읽기도 똑같이 본다 — 판정은 `0xF80` ∧ W 에만 걸린다.
pub proof fn w_bits_survive_mask(f: u32)
    ensures
        has_bit(f & W_MASK, FAULT_DUP_OWN) == has_bit(f, FAULT_DUP_OWN),
        has_bit(f & W_MASK, FAULT_BAD_CFG_WRITE) == has_bit(f, FAULT_BAD_CFG_WRITE),
        has_bit(f & W_MASK, FAULT_OWN_COLLIDE) == has_bit(f, FAULT_OWN_COLLIDE),
        has_bit(f & W_MASK, FAULT_STEP_PROG) == has_bit(f, FAULT_STEP_PROG),
{
    assert(((f & 0x80e0u32) & 0x20u32 != 0) == (f & 0x20u32 != 0)) by (bit_vector);
    assert(((f & 0x80e0u32) & 0x40u32 != 0) == (f & 0x40u32 != 0)) by (bit_vector);
    assert(((f & 0x80e0u32) & 0x80u32 != 0) == (f & 0x80u32 != 0)) by (bit_vector);
    assert(((f & 0x80e0u32) & 0x8000u32 != 0) == (f & 0x8000u32 != 0)) by (bit_vector);
}

// --- 창 기입의 확인 (§2.7 「창 기입의 확인」) -------------------------------

/// OWN_SET / OWN_CLR 한 기입의 되읽기 분류.  기입마다 `OWN_IDX` ← f 뒤
/// `OWN_STATUS[0]`(1 = 장치 소유, `e-defined.typ:225`)을 읽는다 — 「그 사이에 f 의 다른 전이도
/// `0x200` OWN_IDX 의 다른 기입도 없으면」 그 읽기는 「그 전이 기입의 결과 — 커밋이든 거절이든 —
/// 를 본다」(E.7 7항 `e-defined.typ:1388`).  장치 포트에 그 순서로 닿게 하는 것은 플랫폼의
/// 몫이다 — 「끝-끝 순서는 플랫폼의 성질이다」(같은 줄; 쓰기 → 읽기 순서 보존은 플랫폼 레지스트리
/// 전제, `hw_mechanism_abstraction.md` §2).  장부는 커밋을 되읽어 확인한 뒤에만 장치 소유로
/// 셈한다(E.1.1 「호스트 해석 규범」 5항 `:208`).  `OWN_IDX` 는 「전이에 불참한다」(`:223`) — 조회
/// 대상만 정한다.
#[derive(PartialEq, Eq, Structural)]
pub enum OwnReadback {
    /// OWN_SET 뒤 1 · OWN_CLR 뒤 0 — 전이가 확인됐다.  장부는 이때만 바뀐다.
    Confirmed,
    /// OWN_SET 뒤 0 — 장치가 이양을 거절했다(E.7 6항 — 무효과, 대상은 호스트 소유로 남는다).
    /// 프레임은 장부상 호스트 소유로 남고 사상하지 않으며 그 창의 `own_refused` 에 든다.  할당기는
    /// 그 프레임을 호스트 풀로 돌리고 다른 프레임으로 다시 시도한다(상한 `K_OWN` — 플랫폼 레지스트리,
    /// 사용자 결정 U4) — 소유권 표의 형상은 할당기에 박지 않는다.
    Refused,
    /// OWN_CLR 뒤 1 — 「OWN_CLR 은 형상 · 점유로 거절되지 않는다 — 장치 소유인 페이지면 회수하고,
    /// 아니면 무연산이다」(E.7 6항 `e-defined.typ:1387` · `:229`).  `DeviceFault`.
    Contradiction,
}

/// `set` = OWN_SET(참) / OWN_CLR(거짓), `status0` = 되읽은 `OWN_STATUS[0]`.
pub open spec fn own_readback(set: bool, status0: bool) -> OwnReadback {
    if set {
        if status0 {
            OwnReadback::Confirmed
        } else {
            OwnReadback::Refused
        }
    } else if status0 {
        OwnReadback::Contradiction
    } else {
        OwnReadback::Confirmed
    }
}

/// 기입 뒤 장부상의 「장치 소유」: 확인된 전이만 장부를 바꾼다.
pub open spec fn own_ledger_after(set: bool, status0: bool, before: bool) -> bool {
    match own_readback(set, status0) {
        OwnReadback::Confirmed => set,
        _ => before,
    }
}

/// 되읽기 하나의 즉시 동작.  `Confirmed` · `Refused` 는 창을 계속하고(`Refused` 는 그
/// 창의 `own_refused` 에 모여 (나)의 `window_verdict` 가 처분한다), `Contradiction` 은
/// 그 자리에서 `DeviceFault` 다.
pub open spec fn own_readback_action(r: OwnReadback) -> WindowAction {
    match r {
        OwnReadback::Confirmed => WindowAction::Proceed,
        OwnReadback::Refused => WindowAction::Proceed,
        OwnReadback::Contradiction => WindowAction::Fail(Y4Error::DeviceFault),
    }
}

/// 한 창의 OWN_SET 되읽기 열(`(set, status0)` 쌍)에서 `window_verdict` 의
/// `own_refused` 인자: 되읽기 `Refused` 가 하나라도 있었는가.
pub open spec fn own_refused(rs: Seq<(bool, bool)>) -> bool {
    exists|i: int|
        0 <= i < rs.len() && own_readback((#[trigger] rs[i]).0, rs[i].1) == OwnReadback::Refused
}

/// OWN_CLR 뒤 1(`Contradiction`)은 `DeviceFault` 이고, 어느 되읽기도 정지
/// (`LeaseInvariant`)에 이르지 않는다.  OWN_CLR 되읽기는 `own_refused` 에 들지 않는다.
pub proof fn own_readback_never_halts(set: bool, status0: bool)
    ensures
        own_readback_action(own_readback(false, true)) == WindowAction::Fail(Y4Error::DeviceFault),
        own_readback_action(own_readback(set, status0)) != WindowAction::Fail(
            Y4Error::LeaseInvariant,
        ),
        own_readback(false, status0) != OwnReadback::Refused,
{
}

/// 호스트 소유 프레임이 장부에서 장치 소유가 되는 것은 OWN_SET 되읽기가 1 일
/// 때뿐이다 — Y4 는 `OWN_STATUS` 로 확인된 프레임만 사상한다(E.1.1 5항 ·
/// `error.rs` `nonowned_access_origin` 의 전제).
pub proof fn ledger_device_owned_only_if_confirmed(set: bool, status0: bool)
    ensures
        own_ledger_after(set, status0, false) ==> (set && status0),
        own_readback(set, status0) == OwnReadback::Refused ==> !own_ledger_after(
            set,
            status0,
            false,
        ),
{
}

// --- 판정 --------------------------------------------------------------------

/// 창 판정 (§2.4 3 (나) · §1.3 `[5]` `[6]` `[7]` `[15]` 행).
#[derive(PartialEq, Eq, Structural)]
pub enum WindowVerdict {
    /// W 가 비었고 확인도 모두 맞다.
    Clean,
    /// `[5]` 또는 `[6]` — 창 기입이 Y4 사전조건(`lnid_write_ok` · `lbid_write_ok` ·
    /// `lease::step::fp_bind_word` 의 예약 0 · 0xC00/0xE10/0xE00 꼴)을 어겼다.  `LeaseInvariant`.
    Y4Defect,
    /// `[7]` 이 같은 창의 OWN_SET 되읽기 거절로 설명된다(E.7 6항).  거절된 프레임은 사상하지
    /// 않았고, 다른 프레임 재시도(상한 `K_OWN`) 뒤에도 모자라면 그 임대에는 거친 「풀 확장 실패」만
    /// 돌려준다 — 프레임 · 시점은 알리지 않는다(U4).  `[7]` 은 해제한다.
    OwnSetRefused,
    /// `[15]` — 어느 임대에도 귀속하지 않는다.  운영자 · 플랫폼 기록 뒤 해제하고, 테넌트에는
    /// 알리지 않으며 정지하지 않는다.
    StepUnattributed,
    /// 설명되지 않는 `[7]`(같은 창에 거절된 OWN_SET 이 없음) 또는 되읽기는 거절인데 `[7]` 이
    /// 없음(조용한 거절) — 장치 모순.  `DeviceFault`.
    DeviceContradiction,
}

/// 판정에 따른 Y4 의 동작.
#[derive(PartialEq, Eq, Structural)]
pub enum WindowAction {
    /// 계속한다(§2.4 3 의 `0xE14` 기입 · 강제 드레인 창의 탐침으로).
    Proceed,
    /// `[7]` 을 해제하고 계속한다(`OwnSetRefused`) — 정지도 격리도 아니다.  테넌트 보고는 그 창의
    /// OWN_SET 이 `K_OWN` 재시도 뒤에도 모자랄 때(`pool_short`)만 거친 「풀 확장 실패」로 낸다
    /// (`tenant_pool_report`; `K_OWN` 은 모델 밖).  그 밖에는 운영자 기록이다(U4).
    ReportAndClear,
    /// 운영자 · 플랫폼 기록만 남기고 해당 W 비트를 해제한 뒤 계속한다(`StepUnattributed`) —
    /// 테넌트 보고가 아니다.
    RecordAndClear,
    /// 이 오류로 멈춘다.
    Fail(Y4Error),
}

/// 판정을 동작으로.  `Y4Defect` = `Y4Error::LeaseInvariant`.
pub open spec fn verdict_action(v: WindowVerdict) -> WindowAction {
    match v {
        WindowVerdict::Clean => WindowAction::Proceed,
        WindowVerdict::Y4Defect => WindowAction::Fail(Y4Error::LeaseInvariant),
        WindowVerdict::OwnSetRefused => WindowAction::ReportAndClear,
        WindowVerdict::StepUnattributed => WindowAction::RecordAndClear,
        WindowVerdict::DeviceContradiction => WindowAction::Fail(Y4Error::DeviceFault),
    }
}

/// (나) 판정.  `f` = `T_FLT` 대기 뒤 `0xF80` 읽기, `own_refused` = 이 창의 OWN_SET 가운데
/// 되읽기 `Refused` 가 하나라도 있었는가.
///
/// 우선순위: Y4 결함 > 장치 모순 > OWN_SET 거절 > step 기록.  둘 이상 겹치면 Y4 는
/// `window_reports` 의 항을 모두 처리한다.
pub open spec fn window_verdict(f: u32, own_refused: bool) -> WindowVerdict {
    let b5 = has_bit(f, FAULT_DUP_OWN);
    let b6 = has_bit(f, FAULT_BAD_CFG_WRITE);
    let b7 = has_bit(f, FAULT_OWN_COLLIDE);
    let b15 = has_bit(f, FAULT_STEP_PROG);
    if b5 || b6 {
        WindowVerdict::Y4Defect
    } else if b7 != own_refused {
        WindowVerdict::DeviceContradiction
    } else if b7 {
        WindowVerdict::OwnSetRefused
    } else if b15 {
        WindowVerdict::StepUnattributed
    } else {
        WindowVerdict::Clean
    }
}

/// 처리할 항 전부 — 운영자 · 플랫폼 기록과 테넌트 보고의 근거.
pub struct WindowReports {
    /// `[7]` 이 같은 창의 OWN_SET 거절로 설명된다 — `[7]` 해제 · 운영자 기록.  테넌트 보고는
    /// `pool_short` 일 때만이다(`tenant_pool_report` — `K_OWN` 은 모델 밖, U4).
    pub own_set_refused: bool,
    /// `[15]` — 운영자 · 플랫폼 기록(미귀속).
    pub step_unattributed: bool,
}

pub open spec fn window_reports(f: u32, own_refused: bool) -> WindowReports {
    WindowReports {
        own_set_refused: has_bit(f, FAULT_OWN_COLLIDE) && own_refused,
        step_unattributed: has_bit(f, FAULT_STEP_PROG),
    }
}

/// 테넌트 보고(U4): 설명된 `[7]` 이 있고 그 임대의 OWN_SET 이 `K_OWN` 재시도 뒤에도 모자랐을
/// 때(`pool_short` — lease_capability `L.pool_short`)만 그 임대에 거친 「풀 확장 실패」를 낸다.
/// 재시도가 성공한 창(`own_refused` 이지만 `!pool_short`)은 보고하지 않는다 — 창 단위 「거절이
/// 있었다」 신호가 다른 임대의 소유권 표 점유를 흘리지 않게 한다.
pub open spec fn tenant_pool_report(f: u32, own_refused: bool, pool_short: bool) -> bool {
    window_reports(f, own_refused).own_set_refused && pool_short
}

/// (가) 창 전 처분.  `f0` = `hiu_cfg` 를 쥔 뒤 첫 창 기입 전의 `0xF80` 읽기.
/// `[5]` · `[6]` 은 시점과 무관하게 Y4 결함, `[7]` 은 설명할 OWN_SET 이 아직 없으니
/// 장치 모순, `[15]` 는 미귀속 기록(정지 아님 — 결속된 임대에도 귀속하지 않는다).
pub open spec fn dispose_pre_window(f0: u32) -> WindowVerdict {
    if has_bit(f0, FAULT_DUP_OWN) || has_bit(f0, FAULT_BAD_CFG_WRITE) {
        WindowVerdict::Y4Defect
    } else if has_bit(f0, FAULT_OWN_COLLIDE) {
        WindowVerdict::DeviceContradiction
    } else if has_bit(f0, FAULT_STEP_PROG) {
        WindowVerdict::StepUnattributed
    } else {
        WindowVerdict::Clean
    }
}

// --- 증명 --------------------------------------------------------------------

/// 망라: 다섯 판정은 각각 입력 술어와 동치이므로 서로 배타이고 모든 입력을 덮는다.
pub proof fn window_verdict_exhaustive(f: u32, own_refused: bool)
    ensures
        ({
            let b5 = has_bit(f, FAULT_DUP_OWN);
            let b6 = has_bit(f, FAULT_BAD_CFG_WRITE);
            let b7 = has_bit(f, FAULT_OWN_COLLIDE);
            let b15 = has_bit(f, FAULT_STEP_PROG);
            let v = window_verdict(f, own_refused);
            &&& (v == WindowVerdict::Y4Defect) <==> (b5 || b6)
            &&& (v == WindowVerdict::DeviceContradiction) <==> (!(b5 || b6) && b7 != own_refused)
            &&& (v == WindowVerdict::OwnSetRefused) <==> (!(b5 || b6) && b7 && own_refused)
            &&& (v == WindowVerdict::StepUnattributed) <==> (!(b5 || b6) && !b7 && !own_refused
                && b15)
            &&& (v == WindowVerdict::Clean) <==> (!(b5 || b6) && !b7 && !own_refused && !b15)
        }),
{
}

/// 창 전 처분의 망라: 네 판정은 각각 입력 술어와 동치다.
pub proof fn pre_window_exhaustive(f0: u32)
    ensures
        ({
            let b5 = has_bit(f0, FAULT_DUP_OWN);
            let b6 = has_bit(f0, FAULT_BAD_CFG_WRITE);
            let b7 = has_bit(f0, FAULT_OWN_COLLIDE);
            let b15 = has_bit(f0, FAULT_STEP_PROG);
            let v = dispose_pre_window(f0);
            &&& (v == WindowVerdict::Y4Defect) <==> (b5 || b6)
            &&& (v == WindowVerdict::DeviceContradiction) <==> (!(b5 || b6) && b7)
            &&& (v == WindowVerdict::StepUnattributed) <==> (!(b5 || b6) && !b7 && b15)
            &&& (v == WindowVerdict::Clean) <==> (!(b5 || b6) && !b7 && !b15)
            &&& v != WindowVerdict::OwnSetRefused
        }),
{
}

/// 판정은 `0xF80` ∧ W 에만 걸린다 — W 밖 비트는 일반 처리기(§1.3)의 몫이다.
pub proof fn window_verdict_depends_only_on_w(f: u32, own_refused: bool)
    ensures
        window_verdict(f & W_MASK, own_refused) == window_verdict(f, own_refused),
        window_reports(f & W_MASK, own_refused) == window_reports(f, own_refused),
        dispose_pre_window(f & W_MASK) == dispose_pre_window(f),
{
    w_bits_survive_mask(f);
}

/// `[5]` · `[6]` 은 판정 · 창 전 처분 어느 쪽에서도 Y4 결함(`LeaseInvariant`)이다.
pub proof fn fault5_or_6_is_y4_defect(f: u32, own_refused: bool)
    requires
        has_bit(f, FAULT_DUP_OWN) || has_bit(f, FAULT_BAD_CFG_WRITE),
    ensures
        window_verdict(f, own_refused) == WindowVerdict::Y4Defect,
        dispose_pre_window(f) == WindowVerdict::Y4Defect,
        verdict_action(window_verdict(f, own_refused)) == WindowAction::Fail(
            Y4Error::LeaseInvariant,
        ),
{
}

/// 정지에 이르는 것은 `[5]` · `[6]` 뿐이다 — `[7]` · `[15]` 는 무엇이 겹쳐도
/// `LeaseInvariant` 에 이르지 않는다.
pub proof fn window_halts_only_on_5_6(f: u32, own_refused: bool)
    ensures
        (verdict_action(window_verdict(f, own_refused)) == WindowAction::Fail(
            Y4Error::LeaseInvariant,
        )) <==> (has_bit(f, FAULT_DUP_OWN) || has_bit(f, FAULT_BAD_CFG_WRITE)),
{
}

/// 창 전 처분에서도 정지는 `[5]` · `[6]` 뿐이다.  `[7]` 은 장치 모순, `[15]` 는 미귀속 기록이다.
pub proof fn pre_window_halts_only_on_5_6(f0: u32)
    ensures
        (verdict_action(dispose_pre_window(f0)) == WindowAction::Fail(Y4Error::LeaseInvariant))
            <==> (has_bit(f0, FAULT_DUP_OWN) || has_bit(f0, FAULT_BAD_CFG_WRITE)),
        !(has_bit(f0, FAULT_DUP_OWN) || has_bit(f0, FAULT_BAD_CFG_WRITE)) && has_bit(
            f0,
            FAULT_OWN_COLLIDE,
        ) ==> dispose_pre_window(f0) == WindowVerdict::DeviceContradiction,
        !(has_bit(f0, FAULT_DUP_OWN) || has_bit(f0, FAULT_BAD_CFG_WRITE) || has_bit(
            f0,
            FAULT_OWN_COLLIDE,
        )) && has_bit(f0, FAULT_STEP_PROG) ==> dispose_pre_window(f0)
            == WindowVerdict::StepUnattributed && verdict_action(dispose_pre_window(f0))
            == WindowAction::RecordAndClear,
{
}

/// `[7]` · `[15]` 만 선 읽기(`[5]` · `[6]` 없음)는 정지하지 않는다 — 기록 · 보고하거나
/// (설명될 때) 장치를 격리한다(설명되지 않을 때).
pub proof fn fault7_15_never_halt(f: u32, own_refused: bool)
    requires
        !has_bit(f, FAULT_DUP_OWN),
        !has_bit(f, FAULT_BAD_CFG_WRITE),
    ensures
        verdict_action(window_verdict(f, own_refused)) != WindowAction::Fail(
            Y4Error::LeaseInvariant,
        ),
        window_verdict(f, own_refused) != WindowVerdict::Y4Defect,
        dispose_pre_window(f) != WindowVerdict::Y4Defect,
{
}

/// 설명되지 않는 `[7]`(같은 창의 OWN_SET 거절이 없음)과 조용한 거절(되읽기는
/// 거절인데 `[7]` 이 없음)은 장치 모순(`DeviceFault`)이다.
pub proof fn unexplained_7_or_silent_refusal_is_contradiction(f: u32, own_refused: bool)
    requires
        !has_bit(f, FAULT_DUP_OWN),
        !has_bit(f, FAULT_BAD_CFG_WRITE),
        has_bit(f, FAULT_OWN_COLLIDE) != own_refused,
    ensures
        window_verdict(f, own_refused) == WindowVerdict::DeviceContradiction,
        verdict_action(window_verdict(f, own_refused)) == WindowAction::Fail(Y4Error::DeviceFault),
{
}

/// 같은 창의 OWN_SET 거절로 설명되는 `[7]` 은 `OwnSetRefused` 이고 해제할 항이다 — `[15]` 가
/// 겹쳐도 같다.  테넌트 보고 여부는 `tenant_pool_report` 가 따로 정한다.
pub proof fn explained_7_is_own_set_refused(f: u32)
    requires
        !has_bit(f, FAULT_DUP_OWN),
        !has_bit(f, FAULT_BAD_CFG_WRITE),
        has_bit(f, FAULT_OWN_COLLIDE),
    ensures
        window_verdict(f, true) == WindowVerdict::OwnSetRefused,
        verdict_action(window_verdict(f, true)) == WindowAction::ReportAndClear,
        window_reports(f, true).own_set_refused,
{
}

/// 판정과 처리할 항이 맞물린다: 테넌트 보고의 근거가 될 수 있는 동작(`ReportAndClear`)은 설명된
/// `[7]` 에서만 나오고(실제 보고는 `pool_short` 일 때만 — `tenant_pool_report_only_pool_short`),
/// 기록만 하는 동작(`RecordAndClear`)은 `[15]` 에서만 나오며 테넌트 항이 없다.
pub proof fn report_verdict_listed(f: u32, own_refused: bool)
    ensures
        ({
            let v = window_verdict(f, own_refused);
            let r = window_reports(f, own_refused);
            &&& v == WindowVerdict::OwnSetRefused ==> r.own_set_refused
            &&& v == WindowVerdict::StepUnattributed ==> (r.step_unattributed && !r.own_set_refused)
            &&& verdict_action(v) == WindowAction::ReportAndClear ==> r.own_set_refused
            &&& verdict_action(v) == WindowAction::RecordAndClear ==> (r.step_unattributed
                && !r.own_set_refused)
            &&& v == WindowVerdict::Clean ==> !(r.own_set_refused || r.step_unattributed)
        }),
{
}

/// `[15]` 는 기록만 더한다: 같은 읽기에서 `[15]` 를 세우든 지우든 테넌트 항 · 정지 · 격리는
/// 바뀌지 않고, 그 밖에 아무것도 없던 창만 `StepUnattributed` 가 된다 — `[15]` 는 테넌트 보고를
/// 낳지 않는다.  창 전 처분도 같다.
pub proof fn fault15_only_records(f: u32, own_refused: bool)
    ensures
        ({
            let f1 = f | FAULT_STEP_PROG;
            let f0 = f & !FAULT_STEP_PROG;
            &&& window_verdict(f0, own_refused) != WindowVerdict::Clean ==> window_verdict(
                f1,
                own_refused,
            ) == window_verdict(f0, own_refused)
            &&& window_verdict(f0, own_refused) == WindowVerdict::Clean ==> window_verdict(
                f1,
                own_refused,
            ) == WindowVerdict::StepUnattributed
            &&& window_reports(f1, own_refused).own_set_refused == window_reports(
                f0,
                own_refused,
            ).own_set_refused
            &&& window_reports(f1, own_refused).step_unattributed
            &&& !window_reports(f0, own_refused).step_unattributed
            &&& dispose_pre_window(f0) != WindowVerdict::Clean ==> dispose_pre_window(f1)
                == dispose_pre_window(f0)
            &&& dispose_pre_window(f0) == WindowVerdict::Clean ==> dispose_pre_window(f1)
                == WindowVerdict::StepUnattributed
        }),
{
    assert(((f | 0x8000u32) & 0x20u32 != 0) == ((f & !0x8000u32) & 0x20u32 != 0)) by (bit_vector);
    assert(((f | 0x8000u32) & 0x40u32 != 0) == ((f & !0x8000u32) & 0x40u32 != 0)) by (bit_vector);
    assert(((f | 0x8000u32) & 0x80u32 != 0) == ((f & !0x8000u32) & 0x80u32 != 0)) by (bit_vector);
    assert((f | 0x8000u32) & 0x8000u32 != 0) by (bit_vector);
    assert((f & !0x8000u32) & 0x8000u32 == 0) by (bit_vector);
}


/// 테넌트 보고는 `pool_short` 일 때만이고 설명된 `[7]` 을 전제한다.  `[15]` 는 그 보고를 바꾸지
/// 않는다.  재시도가 성공한 창(`!pool_short`)은 `[7]` 이 설명돼도 테넌트에 아무것도 내지 않는다.
pub proof fn tenant_pool_report_only_pool_short(f: u32, own_refused: bool, pool_short: bool)
    ensures
        tenant_pool_report(f, own_refused, pool_short) ==> (pool_short && own_refused && has_bit(
            f,
            FAULT_OWN_COLLIDE,
        )),
        !pool_short ==> !tenant_pool_report(f, own_refused, pool_short),
        tenant_pool_report(f | FAULT_STEP_PROG, own_refused, pool_short) == tenant_pool_report(
            f & !FAULT_STEP_PROG,
            own_refused,
            pool_short,
        ),
{
    fault15_only_records(f, own_refused);
}

} // verus!
