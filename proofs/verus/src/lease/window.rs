// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Global configuration window — `FAULT_STATUS` window-refusal verdict
//! (`docs/hiu_abi.md` §2.4 3 · §2.7 · §1.3, `docs/lease_capability.md` I4 2b · I7).
//!
//! 창 기입의 거절은 `[6]` 하나로 오지 않는다.  창 거절 집합 W 는 넷이다
//! (정본 `e88f6f4` 비트 이름 `e-defined.typ:519-521`):
//!
//! - `[5]` 「구성 — 소유 서로소 위반 (D2)」 — LNID V=1 기입의 서로소 위반
//!   (`01-isa-asm.typ:1303-1304` 「중복 소유를 만드는 사상표 MMIO 쓰기(…)는
//!   거절되고 loud fault 를 낸다.」 · `03-hiu.typ:1096` 「`0xF80` `[5]` 가 따로
//!   받고」).  원천이 Y4 기입뿐이고 엔트리에 파티션 태그가 없으므로
//!   (`03-hiu.typ:992-995`) 언제 섰든 Y4 결함이다.
//! - `[6]` 「구성 — 거절된 구성 기입」 — `0xC00` V=0 · `0xE10` · `0xE00` · LNID 의
//!   3.7.1 넷 · LBID 예약 비트 · FP_BIND 예약 비트.  Y4 결함.
//! - `[7]` 「페이지 소유권 표 충돌」 — 정본은 이름만 두고 원천을 적지 않는다
//!   (OWN_SET 은 「예약 비트가 없고 표현 불가 프레임도 없다」, `e-defined.typ:227`).
//!   재구현은 OWN_SET 의 직접 사상 충돌을 「쓰기 무효 + loud sticky fault」로
//!   낸다(RTL 사실 — 규범 아님: `rtl/…/hiu/OwnershipTable.scala:92-96` · `:159`
//!   `ownEntries: Int = 64` · `rtl/…/top/Wt64Top.scala:350` `B_OWN_COLLIDE = 7`).
//!   그래서 Y4 는 같은 창의 OWN_SET 되읽기 거절로 설명되는 `[7]` 을 용량 사건으로
//!   받는다.  원천 문면은 WaveTensor 에 돌려보낸 D1(sign-off 차단)이다.
//! - `[15]` 「step 프로그램 불량」 — 정본이 적은 원천은 FP_PROG 용량 밖 하나다
//!   (`e-defined.typ:459` 「용량 밖 슬롯·색인 기입은 거절 + loud fault (FAULT_STATUS
//!   `[15]`)」).  재구현은 step 워드 `[15:14]` 예약 · 용량 밖 FP_BIND · 실행 중
//!   runaway 도 여기에 접는다(RTL 사실 — 규범 아님: `rtl/…/fp/StepSequencer.scala:296`
//!   · `:309` · `:460-462`).  step 내용과 실행 사건은 테넌트 몫일 수 있으므로 `[15]`
//!   는 하이퍼바이저를 정지시키지 않는다.  원천 정렬은 D2(sign-off 차단)다.
//!
//! 창은 둘로 읽는다 (§2.4 3):
//!
//! - (가) `hiu_cfg` 를 쥔 뒤 첫 창 기입 전에 `0xF80` ∧ W 를 읽어 선 비트를 창 전
//!   사건으로 처분하고 해제한다 — `dispose_pre_window`.
//! - 창 동안(§2.2) 일반 fault 처리기는 W 를 락 보유자에게 넘기고 처분 · 해제하지
//!   않는다.  그래서 창 안에서 선 sticky W 비트는 (나)까지 남아 있어야 한다.
//! - (나) 창의 마지막 기입 뒤 `T_FLT`(플랫폼 레지스트리 — 창 거절 비트 가시 대기
//!   상한)를 기다렸다가 `0xF80` ∧ W 를 읽고 `window_verdict` 로 처분한 뒤 해제한다.
//!
//! 정지(`LeaseInvariant`)에 이르는 것은 `[5]` · `[6]` 뿐이다
//! (`window_halts_only_on_5_6` · `pre_window_halts_only_on_5_6`).  이 확인이 보여
//! 주는 것은 「장치가 거절로 알린 창 기입이 없었다」까지다 — 겹치는 LBID 구간은
//! 판정 없이 수락되고(`03-hiu.typ:1094`) 범위 밖 시작은 처분이 정본에 없으므로
//! (`b-gaps.typ:633-639` — 재구현은 수락, RTL 사실) `lease::lbid` 사전조건만이 막는다.  v1.0 한계: `T_FLT` 는 실행 사건 `[15]` 가 보이기까지의 상한이 아니므로,
//! 창 안에서 외부 원천이 일으킨 runaway 와, 창 전 사건(결속돼 있던 임대의 실행 사건
//! 포함)이 (가) 뒤에 늦게 보인 `[15]` 를 들어오는 임대의 FP 관문 적재 실패로 오인할 수
//! 있다(정지로는 가지 않는다).
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

/// `0xF80[7]` 페이지 소유권 표 충돌.
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
/// `OWN_STATUS[0]`(1 = 장치 소유, `e-defined.typ:225`)을 읽는다.  `OWN_IDX` 는
/// 「전이에 불참한다」(`e-defined.typ:223`) — 조회 대상만 정한다.
#[derive(PartialEq, Eq, Structural)]
pub enum OwnReadback {
    /// OWN_SET 뒤 1 · OWN_CLR 뒤 0 — 전이가 확인됐다.  장부는 이때만 바뀐다.
    Confirmed,
    /// OWN_SET 뒤 0 — 장치가 이양을 받지 않았다.  프레임은 장부상 호스트
    /// 소유로 남고 사상하지 않는다.  그 창의 `own_refused` 에 든다.
    Refused,
    /// OWN_CLR 뒤 1 — 회수가 거절될 길은 정본에 없다.  `DeviceFault`.
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
/// 때뿐이다 — Y4 는 `OWN_STATUS` 로 확인된 프레임만 사상한다
/// (`error.rs` `nonowned_access_origin` 의 전제).
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

/// FP 관문: FP_PROG 를 모두 쓴 뒤 `T_FLT` 를 기다려 읽은 `0xF80[15]` 이 0 일 때만
/// FP_BIND 를 낸다.  1 이면 결속하지 않으므로 그 프로그램의 참조는 능력
/// 부재다(`02-fabric-alu.typ:493-496` 「상주하지 않는 프로그램의 참조는 능력
/// 부재다」) — 관문 앞에 FP_PROG 가 쓸 슬롯에 결속된 열쇠가 모두 FP_BIND en=0 으로 풀려
/// 있다는 전제에서 (전환 · 회전 창은 `invalidate_tables` 가, 강제 드레인 창은 그 슬롯에
/// 결속된 열쇠 전부의 해제가 먼저다 — `docs/hiu_abi.md` §2.7).  관문은 `[15]` 를 지우지 않는다 — 그 창의
/// `fp_gate_refused` 가 참이 되고 (나)의 `window_verdict` 가 보고 · 해제한다.
pub open spec fn fp_gate_binds(f15_after_tflt: bool) -> bool {
    !f15_after_tflt
}

// --- 판정 --------------------------------------------------------------------

/// 창 판정 (§2.4 3 (나) · §1.3 `[5]` `[6]` `[7]` `[15]` 행).
#[derive(PartialEq, Eq, Structural)]
pub enum WindowVerdict {
    /// W 가 비었고 확인도 모두 맞다.
    Clean,
    /// `[5]` 또는 `[6]` — 창 기입이 Y4 사전조건(`lnid_write_ok` · `lbid_write_ok` ·
    /// `fp_bind_ok` · 0xC00/0xE10/0xE00 꼴)을 어겼다.  `LeaseInvariant`.
    Y4Defect,
    /// `[7]` 이 같은 창의 OWN_SET 되읽기 거절로 설명된다 — 용량 사건.  거절된
    /// 프레임은 사상하지 않고, 그 임대의 풀 확장 실패로 보고한 뒤 해제한다.
    OwnCapacity,
    /// `[15]` 가 같은 창의 FP 관문에서 섰다 — 그 임대의 step 적재 실패.  FP_BIND
    /// 를 내지 않았고, 테넌트에 보고한 뒤 해제한다.
    FpLoadRefused,
    /// FP 관문이 보지 않은 `[15]` — 결속된 임대의 실행 사건으로 보고한다.
    FpRunEvent,
    /// 설명되지 않는 `[7]`, 되읽기는 거절인데 `[7]` 이 없음(조용한 거절), FP
    /// 관문이 본 `[15]` 가 (나)에서 사라짐(넘겨받은 sticky 의 소실) — 장치 모순.
    /// `DeviceFault`.
    DeviceContradiction,
}

/// 판정에 따른 Y4 의 동작.
#[derive(PartialEq, Eq, Structural)]
pub enum WindowAction {
    /// 계속한다(§2.4 3 의 `0xE14` 기입 · 강제 드레인 창의 탐침으로).
    Proceed,
    /// 보고하고 해당 W 비트를 해제한 뒤 계속한다 — 정지도 격리도 아니다.
    ReportAndClear,
    /// 이 오류로 멈춘다.
    Fail(Y4Error),
}

/// 판정을 동작으로.  `Y4Defect` = `Y4Error::LeaseInvariant`.
pub open spec fn verdict_action(v: WindowVerdict) -> WindowAction {
    match v {
        WindowVerdict::Clean => WindowAction::Proceed,
        WindowVerdict::Y4Defect => WindowAction::Fail(Y4Error::LeaseInvariant),
        WindowVerdict::OwnCapacity => WindowAction::ReportAndClear,
        WindowVerdict::FpLoadRefused => WindowAction::ReportAndClear,
        WindowVerdict::FpRunEvent => WindowAction::ReportAndClear,
        WindowVerdict::DeviceContradiction => WindowAction::Fail(Y4Error::DeviceFault),
    }
}

/// (나) 판정.  `f` = `T_FLT` 대기 뒤 `0xF80` 읽기, `own_refused` = 이 창의
/// OWN_SET 가운데 되읽기 `Refused` 가 하나라도 있었는가, `fp_gate_refused` = 이
/// 창의 FP 관문이 `[15]` 를 보았는가(관문이 없었으면 거짓).
///
/// 우선순위: Y4 결함 > 장치 모순 > 보고(소유권 용량 > step 적재 > 실행 사건).
/// 보고 갈래가 둘 이상 겹치면 Y4 는 `window_reports` 의 항을 모두 보고한다.
pub open spec fn window_verdict(f: u32, own_refused: bool, fp_gate_refused: bool) -> WindowVerdict {
    let b5 = has_bit(f, FAULT_DUP_OWN);
    let b6 = has_bit(f, FAULT_BAD_CFG_WRITE);
    let b7 = has_bit(f, FAULT_OWN_COLLIDE);
    let b15 = has_bit(f, FAULT_STEP_PROG);
    if b5 || b6 {
        WindowVerdict::Y4Defect
    } else if b7 != own_refused {
        WindowVerdict::DeviceContradiction
    } else if fp_gate_refused && !b15 {
        WindowVerdict::DeviceContradiction
    } else if b7 {
        WindowVerdict::OwnCapacity
    } else if b15 && fp_gate_refused {
        WindowVerdict::FpLoadRefused
    } else if b15 {
        WindowVerdict::FpRunEvent
    } else {
        WindowVerdict::Clean
    }
}

/// 보고 갈래의 항 전부 (판정이 `ReportAndClear` 일 때 Y4 가 낼 보고).
pub struct WindowReports {
    pub own_capacity: bool,
    pub fp_load_refused: bool,
    pub fp_run_event: bool,
}

pub open spec fn window_reports(f: u32, own_refused: bool, fp_gate_refused: bool) -> WindowReports {
    let b7 = has_bit(f, FAULT_OWN_COLLIDE);
    let b15 = has_bit(f, FAULT_STEP_PROG);
    WindowReports {
        own_capacity: b7 && own_refused,
        fp_load_refused: b15 && fp_gate_refused,
        fp_run_event: b15 && !fp_gate_refused,
    }
}

/// (가) 창 전 처분.  `f0` = `hiu_cfg` 를 쥔 뒤 첫 창 기입 전의 `0xF80` 읽기.
/// `[5]` · `[6]` 은 시점과 무관하게 Y4 결함, `[7]` 은 설명할 OWN_SET 이 아직 없으니
/// 장치 모순, `[15]` 는 지금 결속된 임대(cur)의 실행 사건으로 보고한다(정지 아님).
pub open spec fn dispose_pre_window(f0: u32) -> WindowVerdict {
    if has_bit(f0, FAULT_DUP_OWN) || has_bit(f0, FAULT_BAD_CFG_WRITE) {
        WindowVerdict::Y4Defect
    } else if has_bit(f0, FAULT_OWN_COLLIDE) {
        WindowVerdict::DeviceContradiction
    } else if has_bit(f0, FAULT_STEP_PROG) {
        WindowVerdict::FpRunEvent
    } else {
        WindowVerdict::Clean
    }
}

// --- 증명 --------------------------------------------------------------------

/// 망라: 여섯 판정은 각각 입력 술어와 동치이므로 서로 배타이고 모든 입력을 덮는다.
pub proof fn window_verdict_exhaustive(f: u32, own_refused: bool, fp_gate_refused: bool)
    ensures
        ({
            let b5 = has_bit(f, FAULT_DUP_OWN);
            let b6 = has_bit(f, FAULT_BAD_CFG_WRITE);
            let b7 = has_bit(f, FAULT_OWN_COLLIDE);
            let b15 = has_bit(f, FAULT_STEP_PROG);
            let v = window_verdict(f, own_refused, fp_gate_refused);
            &&& (v == WindowVerdict::Y4Defect) <==> (b5 || b6)
            &&& (v == WindowVerdict::DeviceContradiction) <==> (!(b5 || b6) && (b7 != own_refused
                || (fp_gate_refused && !b15)))
            &&& (v == WindowVerdict::OwnCapacity) <==> (!(b5 || b6) && b7 && own_refused && !(
            fp_gate_refused && !b15))
            &&& (v == WindowVerdict::FpLoadRefused) <==> (!(b5 || b6) && !b7 && !own_refused && b15
                && fp_gate_refused)
            &&& (v == WindowVerdict::FpRunEvent) <==> (!(b5 || b6) && !b7 && !own_refused && b15
                && !fp_gate_refused)
            &&& (v == WindowVerdict::Clean) <==> (!(b5 || b6) && !b7 && !own_refused && !b15
                && !fp_gate_refused)
        }),
{
}

/// 판정은 `0xF80` ∧ W 에만 걸린다 — W 밖 비트는 일반 처리기(§1.3)의 몫이다.
pub proof fn window_verdict_depends_only_on_w(f: u32, own_refused: bool, fp_gate_refused: bool)
    ensures
        window_verdict(f & W_MASK, own_refused, fp_gate_refused) == window_verdict(
            f,
            own_refused,
            fp_gate_refused,
        ),
        dispose_pre_window(f & W_MASK) == dispose_pre_window(f),
{
    w_bits_survive_mask(f);
}

/// `[5]` · `[6]` 은 판정 · 창 전 처분 어느 쪽에서도 Y4 결함(`LeaseInvariant`)이다.
pub proof fn fault5_or_6_is_y4_defect(f: u32, own_refused: bool, fp_gate_refused: bool)
    requires
        has_bit(f, FAULT_DUP_OWN) || has_bit(f, FAULT_BAD_CFG_WRITE),
    ensures
        window_verdict(f, own_refused, fp_gate_refused) == WindowVerdict::Y4Defect,
        dispose_pre_window(f) == WindowVerdict::Y4Defect,
        verdict_action(window_verdict(f, own_refused, fp_gate_refused)) == WindowAction::Fail(
            Y4Error::LeaseInvariant,
        ),
{
}

/// 정지에 이르는 것은 `[5]` · `[6]` 뿐이다 — `[7]` · `[15]` 는 무엇이 겹쳐도
/// `LeaseInvariant` 에 이르지 않는다.
pub proof fn window_halts_only_on_5_6(f: u32, own_refused: bool, fp_gate_refused: bool)
    ensures
        (verdict_action(window_verdict(f, own_refused, fp_gate_refused)) == WindowAction::Fail(
            Y4Error::LeaseInvariant,
        )) <==> (has_bit(f, FAULT_DUP_OWN) || has_bit(f, FAULT_BAD_CFG_WRITE)),
{
}

/// 창 전 처분에서도 정지는 `[5]` · `[6]` 뿐이다.  `[15]` 는 cur 에 보고한다.
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
        )) && has_bit(f0, FAULT_STEP_PROG) ==> dispose_pre_window(f0) == WindowVerdict::FpRunEvent
            && verdict_action(dispose_pre_window(f0)) == WindowAction::ReportAndClear,
{
}

/// `[7]` · `[15]` 만 선 읽기(`[5]` · `[6]` 없음)는 정지하지 않는다 — 보고하거나
/// (설명될 때) 장치를 격리한다(설명되지 않을 때).
pub proof fn fault7_15_never_halt(f: u32, own_refused: bool, fp_gate_refused: bool)
    requires
        !has_bit(f, FAULT_DUP_OWN),
        !has_bit(f, FAULT_BAD_CFG_WRITE),
    ensures
        verdict_action(window_verdict(f, own_refused, fp_gate_refused)) != WindowAction::Fail(
            Y4Error::LeaseInvariant,
        ),
        window_verdict(f, own_refused, fp_gate_refused) != WindowVerdict::Y4Defect,
        dispose_pre_window(f) != WindowVerdict::Y4Defect,
{
}

/// 설명되지 않는 `[7]`(같은 창의 OWN_SET 거절이 없음)과 조용한 거절(되읽기는
/// 거절인데 `[7]` 이 없음)은 장치 모순(`DeviceFault`)이다.
pub proof fn unexplained_7_or_silent_refusal_is_contradiction(
    f: u32,
    own_refused: bool,
    fp_gate_refused: bool,
)
    requires
        !has_bit(f, FAULT_DUP_OWN),
        !has_bit(f, FAULT_BAD_CFG_WRITE),
        has_bit(f, FAULT_OWN_COLLIDE) != own_refused,
    ensures
        window_verdict(f, own_refused, fp_gate_refused) == WindowVerdict::DeviceContradiction,
        verdict_action(window_verdict(f, own_refused, fp_gate_refused)) == WindowAction::Fail(
            Y4Error::DeviceFault,
        ),
{
}

/// 같은 창의 OWN_SET 거절로 설명되는 `[7]` 은 용량 사건이고 보고 갈래에 든다.
pub proof fn explained_7_is_capacity(f: u32, fp_gate_refused: bool)
    requires
        !has_bit(f, FAULT_DUP_OWN),
        !has_bit(f, FAULT_BAD_CFG_WRITE),
        has_bit(f, FAULT_OWN_COLLIDE),
        !(fp_gate_refused && !has_bit(f, FAULT_STEP_PROG)),
    ensures
        window_verdict(f, true, fp_gate_refused) == WindowVerdict::OwnCapacity,
        verdict_action(window_verdict(f, true, fp_gate_refused)) == WindowAction::ReportAndClear,
        window_reports(f, true, fp_gate_refused).own_capacity,
{
}

/// 보고 갈래의 판정은 제 항을 `window_reports` 에 갖고, 보고 갈래 밖의 판정은
/// (정지 · 격리 · 계속) 어느 것도 보고 항 없이 `ReportAndClear` 가 되지 않는다.
pub proof fn report_verdict_listed(f: u32, own_refused: bool, fp_gate_refused: bool)
    ensures
        ({
            let v = window_verdict(f, own_refused, fp_gate_refused);
            let r = window_reports(f, own_refused, fp_gate_refused);
            &&& v == WindowVerdict::OwnCapacity ==> r.own_capacity
            &&& v == WindowVerdict::FpLoadRefused ==> r.fp_load_refused
            &&& v == WindowVerdict::FpRunEvent ==> r.fp_run_event
            &&& verdict_action(v) == WindowAction::ReportAndClear ==> (r.own_capacity
                || r.fp_load_refused || r.fp_run_event)
            &&& v == WindowVerdict::Clean ==> !(r.own_capacity || r.fp_load_refused
                || r.fp_run_event)
        }),
{
}

/// FP 관문이 `[15]` 를 보면 결속하지 않는다.  관문이 본 `[15]` 가 (나)까지
/// 남아 있으면(넘겨받은 sticky) 판정은 step 적재 실패다.
pub proof fn fp_gate_refusal_never_binds(f: u32)
    requires
        !has_bit(f, FAULT_DUP_OWN),
        !has_bit(f, FAULT_BAD_CFG_WRITE),
        !has_bit(f, FAULT_OWN_COLLIDE),
        has_bit(f, FAULT_STEP_PROG),
    ensures
        !fp_gate_binds(true),
        window_verdict(f, false, true) == WindowVerdict::FpLoadRefused,
        verdict_action(window_verdict(f, false, true)) == WindowAction::ReportAndClear,
{
}

} // verus!
