// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! LNID ownership-table write preconditions (`docs/hiu_abi.md` §1.2.1 (C) · §2.7 · §2.8).
//!
//! LNID 소유 표(`0x400`, 256 × 4 B)는 장치 전역 한 벌이고 엔트리에 파티션 태그가
//! 없다(정본 `f6bc4df` `03-hiu.typ:992-995` — `[31]` V · `[30:24]` 「예약 — 0 이어야
//! 한다」 · `[23:16]` 소유 지점 · `[15:0]` LNID).  기입 시점 거절은 3.7.1 의 넷
//! (「용량 밖 슬롯」 · 「없는 배달 지점」 · 「점유 엔트리의 유효 재기입」 · 「엔트리
//! 예약 비트의 비0」, `03-hiu.typ:1070-1077`, `[6]` 에 접힌다 `:1087`)과 1.4.5 서로소
//! 위반(`01-isa-asm.typ:1303-1304`, `[5]` 가 따로 받는다 `03-hiu.typ:1096`)이다.
//! 재사상은 「해제 후 재기입 두 조작」이다(`03-hiu.typ:1057-1058`) — V=1 은 나가는
//! 엔트리를 V=0 으로 지운 뒤에만 쓴다.
//!
//! Y4 는 `lnid_write_ok` 로 그 넷과 1.4.5 서로소 위반을 모두 미리 피하고 장치 거절에
//! 기대지 않는다.
//! 그래서 창의 `[5]` · `[6]` 은 Y4 결함이다(`lease::window`).  상한(등록 용량
//! `cap` · 배달 지점 수 `n_points`)은 알리는 장치 워드가 없으므로 플랫폼 레지스트리 값이다
//! (`hw_mechanism_abstraction.md` §2 — 장치 워드는 WaveTensor 에 보내는 C10 의 v1.x 후보).
//!
//! 「용량 밖 슬롯」은 「이 원인은 V = 1 등록에만 걸린다」 — 「용량 밖 슬롯에 대한 V = 0
//! 기입은 이 원인으로 거절되지 않고 효과도 fault 도 없다」(`03-hiu.typ:1072` — WaveTensor 에
//! 보낸 D3 의 답, 재구현도 같다: `rtl/…/hiu/MmioConfig.scala:398-430`).  `lnid_rejected6` 는 이
//! 명확화대로 「용량 밖 슬롯」을 V=1 에만 건다(`lnid_invalidate_never_rejected6`).
//! `lnid_write_ok` 는 보수적으로 모든 기입에 slot < `cap` 을 건다 — 장치 거절을 피하는 조건이
//! 아니라 Y4 정책이다.  용량 밖 슬롯은 등록이 거절되므로 유효일 수 없어, 부팅 sweep(§2.5 5)은
//! `cap` 아래로 좁혀 쓰고 장부의 초기 상태 「전 엔트리 무효」(`lnid_all_invalid_ok`)는 같다.
//! 「없는 배달 지점」도 V=1 에만 거는 것은 재구현 사실이고(RTL 사실 — 규범 아님:
//! `rtl/…/hiu/MmioConfig.scala:404-409`) 그 문면의 명확화는 WaveTensor 에 보내는 C7(비차단)이다
//! — Y4 의 V=0 기입은 지점 0 을 쓴다.
//! 필드 폭(지점 < 256 · LNID < 2^16)은 V 와 무관하게 건다 — 넘치면 `[30:24]` 예약에
//! 비트가 들고, 「예약 비트가 비0 이면 넷째가 그대로 받는다」(`03-hiu.typ:1072`).
//!
//! step 적재(`0xF00` FP_PROG · `0xF04` FP_BIND)의 사전조건과 적재 순서는 `lease::step` 이다.

use vstd::prelude::*;

verus! {

/// One LNID entry as Y4 writes it: `[31]` V, `[30:24]` reserved,
/// `[23:16]` owning delivery point, `[15:0]` LNID.
pub struct LnidEntry {
    pub valid: bool,
    pub reserved: nat,
    pub point: nat,
    pub lnid: nat,
}

/// 비운 엔트리 — V=0 기입의 워드(예약 0).
pub open spec fn lnid_cleared() -> LnidEntry {
    LnidEntry { valid: false, reserved: 0, point: 0, lnid: 0 }
}

/// 엔트리 하나의 필드: 예약 0, 필드 폭 안(V 와 무관), 유효면 지점 < `n_points`.
pub open spec fn lnid_entry_ok(e: LnidEntry, n_points: nat) -> bool {
    &&& e.reserved == 0
    &&& e.point < 256
    &&& e.lnid < 0x1_0000
    &&& e.valid ==> e.point < n_points
}

/// 두 엔트리가 같은 LNID 를 소유한다(1.4.5 서로소 위반의 꼴).
pub open spec fn lnid_dup(a: LnidEntry, b: LnidEntry) -> bool {
    a.valid && b.valid && a.lnid == b.lnid
}

/// 표 불변식: 엔트리마다 필드가 맞고, 용량 밖 슬롯은 무효이며, 서로 다른 두
/// 유효 엔트리는 LNID 가 다르다.
pub open spec fn lnid_table_ok(t: Seq<LnidEntry>, cap: nat, n_points: nat) -> bool {
    &&& forall|i: int| 0 <= i < t.len() ==> #[trigger] lnid_entry_ok(t[i], n_points)
    &&& forall|i: int| 0 <= i < t.len() && i >= cap ==> !(#[trigger] t[i]).valid
    &&& forall|i: int, j: int|
        0 <= i < t.len() && 0 <= j < t.len() && i != j ==> !#[trigger] lnid_dup(t[i], t[j])
}

/// 대행 기입의 사전조건: slot < 등록 용량, 필드가 맞음(예약 0 · 지점 < 배달 지점
/// 수), V=1 은 빈 슬롯에만(V=0 뒤 V=1), 다른 어느 유효 엔트리와도 LNID 가 겹치지 않음.
pub open spec fn lnid_write_ok(
    t: Seq<LnidEntry>,
    idx: int,
    e: LnidEntry,
    cap: nat,
    n_points: nat,
) -> bool {
    &&& 0 <= idx < t.len()
    &&& idx < cap
    &&& lnid_entry_ok(e, n_points)
    &&& e.valid ==> !t[idx].valid
    &&& forall|j: int| 0 <= j < t.len() && j != idx ==> !#[trigger] lnid_dup(e, t[j])
}

/// 정본의 `[6]` 거절 넷(3.7.1, `03-hiu.typ:1070-1077`) — 장부가 장치 표를 비춘다는
/// 전제에서.  「용량 밖 슬롯」은 V=1 등록에만 걸린다(`03-hiu.typ:1072`).  「없는 배달
/// 지점」의 V=1 한정은 RTL 사실이다(규범 아님 — `rtl/…/hiu/MmioConfig.scala:404-409`,
/// 명확화는 C7).
pub open spec fn lnid_rejected6(
    t: Seq<LnidEntry>,
    idx: int,
    e: LnidEntry,
    cap: nat,
    n_points: nat,
) -> bool {
    ||| (e.valid && idx >= cap)
    ||| (e.valid && e.point >= n_points)
    ||| (e.valid && t[idx].valid)
    ||| e.reserved != 0
}

/// 정본의 `[5]` 서로소 위반(1.4.5) — 다른 슬롯이 같은 LNID 를 소유하고 있다.
pub open spec fn lnid_rejected5(t: Seq<LnidEntry>, idx: int, e: LnidEntry) -> bool {
    e.valid && exists|j: int|
        0 <= j < t.len() && j != idx && #[trigger] t[j].valid && t[j].lnid == e.lnid
}

/// `lnid_write_ok` 를 지킨 기입은 `[6]` 넷에도 `[5]` 에도 걸리지 않는다.
pub proof fn lnid_write_ok_avoids_rejection(
    t: Seq<LnidEntry>,
    idx: int,
    e: LnidEntry,
    cap: nat,
    n_points: nat,
)
    requires
        lnid_write_ok(t, idx, e, cap, n_points),
    ensures
        !lnid_rejected6(t, idx, e, cap, n_points),
        !lnid_rejected5(t, idx, e),
{
    if lnid_rejected5(t, idx, e) {
        let j = choose|j: int|
            0 <= j < t.len() && j != idx && #[trigger] t[j].valid && t[j].lnid == e.lnid;
        assert(lnid_dup(e, t[j]));
    }
}

/// 예약 0 인 V=0 기입은 슬롯이 용량 밖이어도 `[6]` 에도 `[5]` 에도 걸리지 않는다 — D3 의 답
/// 「용량 밖 슬롯에 대한 V = 0 기입은 이 원인으로 거절되지 않고 효과도 fault 도 없다」
/// (`03-hiu.typ:1072`).  그러니 `lnid_write_ok` 의 slot < `cap` 은 장치 거절을 피하는 조건이 아니라
/// Y4 의 보수 정책이다.
pub proof fn lnid_invalidate_never_rejected6(t: Seq<LnidEntry>, idx: int, cap: nat, n_points: nat)
    ensures
        !lnid_rejected6(t, idx, lnid_cleared(), cap, n_points),
        !lnid_rejected5(t, idx, lnid_cleared()),
{
}

/// V=1 등록은 용량 밖 슬롯이면 다른 필드와 무관하게 `[6]` 이다 — 「용량 밖 슬롯」은 등록에
/// 걸린다(`lnid_write_ok` 도 그 기입을 내지 않는다).
pub proof fn lnid_register_out_of_cap_rejected6(
    t: Seq<LnidEntry>,
    idx: int,
    e: LnidEntry,
    cap: nat,
    n_points: nat,
)
    requires
        e.valid,
        idx >= cap,
    ensures
        lnid_rejected6(t, idx, e, cap, n_points),
        !lnid_write_ok(t, idx, e, cap, n_points),
{
}

/// `lnid_write_ok` 를 통과한 기입은 표 불변식을 보존한다.
pub proof fn lnid_write_preserves(
    t: Seq<LnidEntry>,
    idx: int,
    e: LnidEntry,
    cap: nat,
    n_points: nat,
)
    requires
        lnid_table_ok(t, cap, n_points),
        lnid_write_ok(t, idx, e, cap, n_points),
    ensures
        lnid_table_ok(t.update(idx, e), cap, n_points),
{
    let u = t.update(idx, e);
    assert forall|i: int| 0 <= i < u.len() implies #[trigger] lnid_entry_ok(u[i], n_points) by {
        if i != idx {
            assert(u[i] == t[i]);
        }
    }
    assert forall|i: int| 0 <= i < u.len() && i >= cap implies !(#[trigger] u[i]).valid by {
        assert(i != idx);
        assert(u[i] == t[i]);
    }
    assert forall|i: int, j: int|
        0 <= i < u.len() && 0 <= j < u.len() && i != j implies !#[trigger] lnid_dup(u[i], u[j]) by {
        if i == idx {
            assert(u[j] == t[j]);
            assert(!lnid_dup(e, t[j]));
        } else if j == idx {
            assert(u[i] == t[i]);
            assert(!lnid_dup(e, t[i]));
        } else {
            assert(u[i] == t[i] && u[j] == t[j]);
        }
    }
}

/// 용량 안 슬롯의 V=0 기입은 표 불변식을 깨지 않는다 — 전환 · 회전 재적재가 나가는
/// 임대의 엔트리를 이렇게 지운다(§2.8 규칙 5).
pub proof fn lnid_invalidate_preserves(t: Seq<LnidEntry>, idx: int, cap: nat, n_points: nat)
    requires
        lnid_table_ok(t, cap, n_points),
        0 <= idx < t.len(),
        idx < cap,
    ensures
        lnid_table_ok(t.update(idx, lnid_cleared()), cap, n_points),
{
    assert(lnid_write_ok(t, idx, lnid_cleared(), cap, n_points));
    lnid_write_preserves(t, idx, lnid_cleared(), cap, n_points);
}

/// 점유 슬롯에 바로 V=1 을 쓰는 것은 사전조건 위반이다(`[6]` 「점유 엔트리의 유효
/// 재기입」).  다른 슬롯이 같은 LNID 를 쥐고 있는 동안의 등록도 위반이다(`[5]`).
/// 그래서 같은 이름의 재등록은 옛 엔트리의 V=0 이 먼저다(I7 · I4 의 `invalidate_tables`).
pub proof fn lnid_reregister_needs_invalidate(
    t: Seq<LnidEntry>,
    idx: int,
    k: int,
    e: LnidEntry,
    cap: nat,
    n_points: nat,
)
    requires
        0 <= idx < t.len(),
        0 <= k < t.len(),
        e.valid,
        t[k].valid,
        t[k].lnid == e.lnid,
    ensures
        !lnid_write_ok(t, idx, e, cap, n_points),
{
    if idx != k {
        assert(lnid_dup(e, t[k]));
    }
}

/// V=0 뒤에는 같은 슬롯에 같은 이름을 다시 등록할 수 있다 — 다른 슬롯이 그
/// 이름을 쥐지 않고 필드가 맞으면.
pub proof fn lnid_reregister_after_invalidate(
    t: Seq<LnidEntry>,
    idx: int,
    e: LnidEntry,
    cap: nat,
    n_points: nat,
)
    requires
        lnid_table_ok(t, cap, n_points),
        0 <= idx < t.len(),
        idx < cap,
        e.valid,
        lnid_entry_ok(e, n_points),
        t[idx].valid,
        t[idx].lnid == e.lnid,
    ensures
        lnid_write_ok(t.update(idx, lnid_cleared()), idx, e, cap, n_points),
{
    let u = t.update(idx, lnid_cleared());
    assert forall|j: int| 0 <= j < u.len() && j != idx implies !#[trigger] lnid_dup(e, u[j]) by {
        assert(u[j] == t[j]);
        assert(!lnid_dup(t[idx], t[j]));
    }
}

/// Boot state: every entry invalid (`docs/hiu_abi.md` §2.5 step 5).
pub open spec fn lnid_all_invalid(t: Seq<LnidEntry>) -> bool {
    forall|i: int|
        0 <= i < t.len() ==> !(#[trigger] t[i]).valid && t[i].reserved == 0 && t[i].point < 256
            && t[i].lnid < 0x1_0000
}

/// 전 엔트리 무효인 부팅 장부는 표 불변식을 만족한다.
pub proof fn lnid_all_invalid_ok(t: Seq<LnidEntry>, cap: nat, n_points: nat)
    requires
        lnid_all_invalid(t),
    ensures
        lnid_table_ok(t, cap, n_points),
{
    assert forall|i: int| 0 <= i < t.len() implies #[trigger] lnid_entry_ok(t[i], n_points) by {
        assert(!t[i].valid && t[i].reserved == 0 && t[i].point < 256 && t[i].lnid < 0x1_0000);
    }
    assert forall|i: int| 0 <= i < t.len() && i >= cap implies !(#[trigger] t[i]).valid by {
        assert(!t[i].valid);
    }
    assert forall|i: int, j: int|
        0 <= i < t.len() && 0 <= j < t.len() && i != j implies !#[trigger] lnid_dup(t[i], t[j]) by {
        assert(!t[i].valid);
    }
}

} // verus!
