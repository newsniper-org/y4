---
name: Y4 ipc/alloc 사전점검 결정
description: Phase B step 3 의 ipc/ 와 alloc/ Verus 명세 시작 전 결정 사항. C1–C4, I-P1–3, A-P1–3.
type: project
originSessionId: 78ff80c3-5421-425a-9e23-3da166ef2bb9
---
2026-05-04 사용자 결정 (`Y4/.claude-memos/ipc-and-alloc.md`).

## 확정 사항

**C2 — 동시성 모델:** **SMP-first**. 명세도 SMP, 구현도 SMP.
**Why:** LWKT 의 핵심 가치 (per-CPU lock-free) 가 SMP 에서만 의미.
명세를 UP-only 로 시작하면 Phase D 진입 시 재작성 비용이 크다.
**How to apply:** 모든 ipc/alloc Verus 명세는 multi-core trace 위에서
race-free 를 증명. seL4 cmake config 의 `KernelMaxNumNodes` 도 함께
승급 (현 1 → 4 또는 형상별 분기).

**C3 — 에러 enum:** **공통 `Y4Error`** (`kernel/error.rs`).
**Why:** 두 서브시스템의 error path 가 결국 같은 capability 검증 실패
도메인. 변환 boilerplate 회피.
**How to apply:** `Y4Error::{NoMemory, BadCap, Timeout, ...}` 단일
enum. 새 variant 추가는 두 서브시스템 영향 모두 고려.

**C4 — seL4 trusted 호출 명단:** 표 그대로 채택 (`Send/Recv/Call/Reply`,
`Signal/Wait`, `Untyped_Retype`, `CNode_Copy/Move/Mint`, `Page_Map/Unmap`,
`TCB_*`). 본 표가 Verus 의 `#[verifier::trusted]` 함수 명단의 superset.
**Why:** 명세 boundary 를 미리 좁혀 두지 않으면 seL4 surface 가 흐려진다.
**How to apply:** 위 호출 외 새 seL4 함수 사용은 PR description 에서
명시 + 표 갱신 동반.

**A-P3 — alloc 명세 v0:** **모두 in** — use-after-free 검출, 주소
randomization, guard page 정합.
**Why:** OpenBSD malloc 의 보안 가치 셋. 명세에서 빠지면 OpenBSD
참조 의의가 사라진다.
**How to apply:** alloc Verus spec v0 의 invariant 카탈로그에 세 항목
모두 포함. 분량 증가는 감수.

## 추가 확정 사항 (2026-05-04, 조사 결과 + 사용자 채택)

**C1 — 의존 방향:** **(d) ipc/alloc 독립**. 둘 다 seL4 만 직접 의존,
cross-call 없음.
**Why:** DragonFly LWKT 코드 검증 결과, LWKT 자체는 메시지를 할당하지
않음 (`lwkt_sendmsg(port, msg)` 의 `msg` 는 caller-owned, intrusive
TAILQ_ENTRY). `M_LWKTMSG` malloc tag 는 `lwkt_msgport.c` 에 *선언*
되지만 *사용*은 `sys/kern/uipc_msg.c` (소켓 레이어 — caller). 즉 LWKT
는 dispatch-only.
**How to apply:** ipc/ 명세는 alloc/ 를 import 하지 않음. 메시지 객체는
caller (root task / scheme client) 가 alloc/ 에서 받아 ipc/ 에 넘김.
PR 머지 순서는 둘이 자유 (병렬). Verus crate 둘 사이 import 금지.

**I-P2 — IPC 외부 API 경계:** **(c) hybrid** — scheme 은 제어평면
표준, raw LWKT msgport 는 데이터평면 옵션. 둘 다 외부 API.
**Why:** Redox scheme dispatch overhead ~60–100 cycle vs LWKT msgport
~10–30 cycle = 2–10×. 제어평면 (lease 발급, capability mint, scheme
verb) 은 overhead OK; 데이터평면 (HIU MMIO 디스패치, accel-d↔tenant)
은 overhead NG. 단일 API 선택은 둘 중 한 쪽을 손해 봄.
**How to apply:** ipc/ 명세는 두 layer 각각의 invariant 를 진다 +
"scheme 으로 한 일을 raw msgport 로도 동등하게 할 수 있다" 일관성
정리 1개. 두 API 가 서로의 race 를 만들지 않음을 증명.

**I-P3 zero-copy:** **v0 OUT**, 별도 "shared-frame capability" primitive
로 Phase C 즈음 분리.
**Why:** Y4 IPC 는 통상 작은 메시지 (cap mint, lease 발급, scheme verb;
< 64B). 큰 데이터는 IPC 가 아니라 HIU shared region / seL4 frame share
로 전달. zero-copy 는 IPC 의 속성이 아니라 별개 primitive.
**How to apply:** ipc/ Verus 명세는 send-by-copy 모델. 큰 페이로드의
shared-frame primitive 는 hiu/ 또는 신규 frames/ 서브트리에서.

**I-P3 priority-inversion:** **v0 IN**.
**Why:** SMP-first (C2) 환경에서 우선순위 전도는 wave-aligned scheduling
의 결정성을 깨뜨릴 수 있음. 명세에서 회피 invariant 를 못 박지 않으면
Phase C 에서 lease scheduler 와 충돌 시 추적이 어려움.
**How to apply:** ipc/ 명세에 "고우선 thread 가 저우선 thread 의 msgport
hold 으로 인해 무한 대기하지 않음" invariant 포함. PI (priority
inheritance) 또는 ceiling 중 메커니즘 선택은 구현 PR 시점.

**A-P1 — alloc 컴포넌트 조합:** **(β) DragonFly SLAB + scudo (NUMA +
보안 모두)**.
**Why:** scudo 의 LLVM hardened 스택이 OpenBSD malloc 의 보안 가치
(UAF / randomization / guard pages) 를 모두 제공 + Apache-2.0 라이선스.
DragonFly SLAB 의 lock-free per-CPU magazine 은 hot-path 객체 캐시를
담당. SLUB 의 NUMA-aware partial-list 역할도 scudo 가 수행. 명세
부담은 (α) 자체 NUMA 구현보다 작음.
**How to apply:** alloc/ 의 컴포넌트 = (DragonFly SLAB front-end) +
(scudo backend). OpenBSD malloc 은 reference 에서 제거 (scudo 가 흡수).
Verus 명세는 두 컴포넌트의 boundary contract + scudo 의 보안 invariant
(use-after-free 검출, randomization, guard page) 를 명시.

**A-P2 — seL4 VA reservation:** **(a) bare kernel API**
(`seL4_X86_Page_Map / Unmap`, `seL4_X86_PageTable_Map`).
**Why:** `seL4_VSpaceObject_Map_Reservation()` 은 seL4 15.0.0 kernel
API 에 부재 — `libsel4vspace` (`seL4_libs` 별도 upstream) 의
`vspace_reserve_range()` 가 사용자가 떠올린 함수의 실체. C4 결정상
trusted boundary 가 `seL4_Page_Map / Unmap` 으로 좁혀져 있으므로
bare kernel API 채택이 자연.
**How to apply:** alloc/ 가 VA reservation 알고리즘을 자체 구현. 추가
seL4 libs 의존 없음. third_party 에 seL4_libs 추가 안 함.

## 외부 reference 위치

- DragonFly LWKT 소스: `/home/ybi/y4-upstream-refs/dragonfly/sys/kern/lwkt_*.c`
  + `/home/ybi/y4-upstream-refs/dragonfly/sys/sys/{thread,msgport}.h` (sparse-checkout)
- Redox scheme 소스: `/home/ybi/y4-upstream-refs/redox-kernel/src/scheme/`
- 둘 다 read-only — Y4 트리에는 들어가지 않음. fork 가 아니라 참조용.

scudo 와 OpenBSD malloc 추가 reference 가 필요해질 시점 (alloc/ 첫 PR)
에 동일 패턴으로 `~/y4-upstream-refs/{scudo,openbsd-libc}/` 추가.

## 외부 reference 위치

- DragonFly LWKT 소스: `/home/ybi/y4-upstream-refs/dragonfly/sys/kern/lwkt_*.c`
  + `/home/ybi/y4-upstream-refs/dragonfly/sys/sys/{thread,msgport}.h` (sparse-checkout)
- Redox scheme 소스: `/home/ybi/y4-upstream-refs/redox-kernel/src/scheme/`
- 둘 다 read-only — Y4 트리에는 들어가지 않음. fork 가 아니라 참조용.
