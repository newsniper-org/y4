<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Side-channel isolation — 공유 accelerator/host 에서 cross-tenant microarchitectural·timing 채널 방어 (partition_id = master key + seL4 time protection 상속)
created: 2026-08-13T12:46:10+09:00   # KST (UTC+9)
status: brainstorming (결정: §1 masking⊥side-channel / §2 partition_id master key / §4 seL4 time protection 상속+lease-boundary flush / §6 SMT sibling=동일 partition — 잔여 thermal/power 채널은 방향만).  capsule fault isolation 발제에서 이어짐
scope: multi-tenant accelerator hosting(server-farm form factor)의 cross-tenant side-channel.
       cache/TLB/SMT/DRAM/thermal/power/DVFS/interrupt 채널 분류·방어 + WaveTensor 하드웨어 partitioning + XChaCha20 masking 과의 관계
refs:
  - MEMORY/wavetensor_terms.md (partitioned TLB 4×16 / shadow 16 / XChaCha20 192-bit nonce·256-bit key / context_switch / lease / TRNG)
  - docs/power_safety.md (S15~S23 + AV21~AV40 — thermal/power 안전장치)
  - docs/glossary.md (HIU/lease/partitioned TLB 정전)
  - .brainstormings/20260812-174402-capsule-fault-isolation-restart.md §4.2 (partition_id = allocator zone·IOMMU domain 단일 key)
  - .brainstormings/20260618-223701-capability-bound-allocation.md (partition-keyed zone P0..3)
  - .brainstormings/20260618-225136-soft-vs-hard-real-time.md §6.5 (thermal throttle 선택적 유예)
  - .brainstormings/20260812-173424-ipc-layer-design.md §0.5 (atomic-free composition)
  - seL4 Time Protection (Ge/Yarom/Sligar/Heiser, EuroSys 2019) — 상속 기반
---

# Side-channel isolation — partition_id 를 microarchitecture 까지 확장

## §0 프레임

**최종 구현 목표 기준.**  server-farm form factor 는 다수 tenant 가 하나의
WaveTensor accelerator + host 를 공유한다 — **cross-tenant side-channel 이
이 배치의 핵심 보안 문제**.  값 격리(capability·partition·masking)가 완벽해도
**공유 하드웨어의 microarchitectural·timing·thermal 상태**를 통해 정보가
누출될 수 있다.

baseline 자산: WaveTensor 하드웨어 partitioning(partitioned TLB 4×16,
shadow 16), XChaCha20 masking, seL4 base, 그리고 방금 확정한 **partition
모델**(capsule 발제 §4.2 — partition_id 가 allocator zone·IOMMU domain 을
잇는 key).  side-channel 은 아직 brainstorming 미탐색.

## §0.5 원칙 정합

- **atomic-free composition**(IPC §0.5): temporal flush 는 per-CPU
  sequential(local μarch 상태 flush), spatial partition 은 애초에 공유를
  없애 contention·atomic 을 제거 — 시리즈 원칙과 정합.
- **TCB 최소화**: side-channel 은 TCB *밖* tenant 간 문제지만 공유 자원을
  통해 누출 → host 의 공유 표면을 최소화(TCB 최소화가 side-channel 표면도
  줄인다).
- **capability·partition isolation** 을 값에서 **microarchitecture** 로 확장.

## §1 (결정) masking 은 side-channel 방어가 아니다 — 직교성 ★

WaveTensor XChaCha20 masking(192-bit nonce / 256-bit key, `context_switch`
시 nonce bind)은 **content(값)** 를 가린다 — 데이터 직접 유출 차단.  그러나
side-channel 은 **access pattern·timing** 을 누출한다 — "어떤 cache line 을
언제 얼마나 오래" — **값과 무관**.

> masking 된 데이터라도 그것을 처리하는 **cache-line 접근 패턴·실행 시간**은
> 그대로 노출된다.  **masking 과 side-channel isolation 은 직교하며 둘 다
> 필요**하다.

**결정**: masking 에 기대어 side-channel 을 방치하지 않는다.  masking =
content 층, 본 문서의 방어 = pattern/timing 층 — 별개 계층으로 설계.

## §2 (결정) partition_id 가 side-channel domain 도 키한다 — master key ★

capsule §4.2: `partition_id`(P0..3, HIU 유래) = allocator zone(CPU memory)
+ IOMMU domain(device DMA)의 단일 key.  여기에 microarchitecture 차원을
추가한다:

| 차원 | enforcer | 매핑 |
|---|---|---|
| CPU memory | MMU + allocator zone | partition_id → zone (I1) |
| device DMA | IOMMU domain | partition_id → domain (§4.2, FA6) |
| **LLC (shared)** | **cache coloring** | **partition_id → page color set** |
| **host L1/TLB/BP** | **temporal flush** | **partition_id → flush domain** |
| **WaveTensor TLB** | **하드웨어 partition(4×16)** | **partition_id → TLB partition** |
| **SMT siblings** | **scheduler(gang)** | **partition_id → sibling group** |

> **결정 — partition_id = 모든 격리 차원의 master key.**  "하나의 partition
> 모델, N enforcer"(§4.2)를 microarchitecture 로 확장 — memory / device /
> μarch 가 하나의 key 로 일관 index.  일관성이 곧 검증 대상(§8).

## §3 방어 분류 — 채널별 spatial / temporal / masking / bound

| 채널 | 누출 예 | 방어 | 기반 |
|---|---|---|---|
| L1/L2 (core-local) | prime+probe | **temporal flush on switch** | seL4 time protection(§4) |
| LLC (shared) | flush+reload | **spatial: cache coloring**(color=partition) | seL4 coloring + partition_id |
| host TLB / BP | | **flush on switch** + BP 초기화 | seL4 time protection |
| WaveTensor TLB (4×16) | | **하드웨어 spatial partition (이미)** | WaveTensor |
| SMT / port contention | PortSmash | **sibling=동일 partition 강제** or cross-tenant SMT off | scheduler(§6) |
| DRAM row-buffer / bank | | **bank-aware partition 할당** | allocator + partition |
| thermal / power / DVFS | Hertzbleed | **§5 (partition·flush 불가 → bound)** | power_safety |
| interrupt timing | | **bounded/masked delivery** | IPC(§0.5) |

원칙: **partition 가능하면 spatial(공유 제거) → 아니면 temporal flush →
아니면 masking(직교) → 그마저 아니면 정량 bound.**  fail 순서는 강도 내림차순.

## §4 (결정) seL4 Time Protection 을 기반으로 상속 + lease boundary flush

Y4 는 side-channel 방어를 맨바닥에서 발명하지 않는다.  **seL4 Time
Protection**(Ge/Yarom/Sligar/Heiser, EuroSys 2019)의 mechanism 을 상속:
- (a) 공유 HW **spatial partition** — cache coloring 으로 LLC 를 domain 별
  분할.
- (b) partition 불가한 core-local 상태(L1/TLB/BP)는 domain switch 시
  **temporal flush + constant-time pad**.

**Y4 확장**:
- domain = **partition_id**(§2), switch 경계 = **lease boundary**.
- lease teardown(capability bulk reclaim §I4/I6)에 **μarch flush + pad**
  를 추가 — 한 tenant 의 lease 종료 → 다음 tenant lease 시작 사이에 L1/
  TLB/BP flush, switch latency 를 constant-time 로 pad(잔여 timing 채널
  차단).
- ★ **§0.5 정합**: flush 는 per-CPU local 연산, pad 는 deterministic —
  cross-CPU atomic 없음.  lease switch 는 이미 trusted teardown 경로.

**SC invariant(§8)**: "lease switch 후 cross-partition μarch 잔존 상태 0"
(flush 완전성) + "switch latency 가 이전 tenant 활동과 무관"(pad).

## §5 잔여 hard 채널 — thermal / power / DVFS (partition·flush 불가)

cache/TLB 는 partition/flush 가능하나 **온도·전력·주파수는 칩 전체가
물리적으로 공유** → spatial partition 불가, flush 불가.  Hertzbleed 류:
data-dependent power → DVFS → timing.  **완전 제거 불가, bound 만 가능**:

- **frequency/DVFS decoupling during sensitive lease** — DVFS 결정을 tenant
  computation 과 분리(민감 lease 구간 고정 주파수).  data-dependent
  frequency 채널 차단.
- **thermal throttle 를 tenant secret 과 decouple** — throttle 결정이 tenant
  **비밀 연산**에 의존하면 채널.  단 lease **criticality**(공개 metadata)에
  의존하는 건 허용(§6.5 thermal 유예 결정과 무모순 — criticality 는 비밀
  아님).  throttle 정책을 tenant-secret-agnostic 하게 power_safety AV 로.
- **leakage rate bound** — 제거 불가 채널은 **누출 대역폭을 정량 상한**
  (측정 기반, 증명 아님).  잔여 위험을 명시적으로 회계.

이 절은 power_safety(S15~S23/AV21~AV40)와 cross-cutting — 완전 해결은 open,
방향만 확정.

## §6 (결정) SMT / scheduling 제약

SMT sibling 은 L1·실행 port 를 공유 → **cross-tenant co-location 이 곧
채널**(PortSmash 류).

**결정**: **SMT sibling = 동일 partition 강제**(partition-aware gang
scheduling — 두 sibling thread 는 반드시 같은 partition_id) 또는 cross-tenant
구간 SMT off.  scheduler(real-time §)와 결합 — partition 이 SMT 경계를
가로지르지 못함.  §2 표의 "SMT siblings → partition_id sibling group"
enforcer 가 이것.

## §7 accelerator-internal (WaveTensor 고유) — ⏳ 일부

- **partitioned TLB 4×16** = 이미 하드웨어 spatial partition → cross-partition
  accelerator TLB 누출 차단(§3 표).  Y4 는 partition_id → TLB partition 매핑만
  보장.
- **shadow regions 16** = side-channel 표면 여부 검토 필요(shadow 접근 패턴이
  cross-partition 관측 가능한가) — ⏳ `hiu_abi.md` v1.0 frozen 후.
- **HIU context_switch** 가 masking nonce bind — 단 §1 대로 masking 은 content
  층.  context_switch 경계가 §4 의 μarch flush 경계와 정렬되는지 확인 필요.
- 상세는 HIU ABI frozen 후(현 차단) — 본 절은 표면 식별까지만.

## §8 verification

side-channel 은 대개 **hyperproperty**(non-interference / timing
indistinguishability) — 일반 Verus safety property 로 직접 표현하기 어렵다.
접근을 계층화:

- **(a) seL4 상속** — time protection 의 기존 모델·논증을 기반으로 사용
  (Y4 가 재증명 X, seL4 신뢰 경계).
- **(b) partition→enforcer 매핑 일관성 (Verus)** — FA6(§4.2)의 확장:
  `partition_id` 가 cache color / TLB partition / time-protection domain /
  SMT group 에 **일관 매핑**됨을 Verus 로.  이건 통상 safety property.
- **(c) flush 완전성 (checklist + 검증)** — lease switch 시 §4 에 나열된
  모든 μarch 상태가 flush 됨(SC invariant "잔존 0").  누락 = 채널.
- **(d) thermal/power 채널 = 정량 leakage bound (측정)** — 증명 대상 아님,
  대역폭 상한을 측정·회계(§5).

핵심 SC invariant: **"lease switch 후 cross-partition μarch 잔존 0"**(c) +
**"partition_id 매핑 일관성"**(b).

## §9 결정 / 미결 요약

**결정 방향(강)**:
- masking ⊥ side-channel — content 층과 pattern/timing 층 별개(§1)
- **partition_id = 모든 격리 차원의 master key**(memory·device·μarch, §2)
- 방어 우선순위 = spatial partition → temporal flush → masking → bound(§3)
- **seL4 Time Protection 상속 + lease boundary flush/pad**(§4, §0.5 정합)
- **SMT sibling = 동일 partition 강제**(gang, §6)

**미결(설계/측정 필요) — 아래 항목은 본 발제 범위 밖, 별도 후속 논의로 이월**:
- thermal/power/DVFS 잔여 채널 정량 bound 수치(§5, power_safety cross-cutting)
- cache coloring vs 하드웨어 CAT(Intel CAT / AMD 상당) 선택 — 하드웨어 의존
- accelerator-internal(shadow 16 등) side-channel 표면(§7, ⏳ hiu_abi)
- DRAM bank-aware 할당의 allocator 통합 구체(§3 표)

**⏳ 선행 의존**:
- `hiu_abi.md` v1.0 frozen — accelerator-internal 채널 상세(§7)
- power_safety AV(thermal/DVFS decoupling AV 신설, §5)
- scheduler 설계 — SMT gang + lease switch flush 경계(§6, §4)

## §10 다음 발제 후보

- **attestation / measured boot** — tenant 가 위 side-channel 방어(+TCB)가
  실제 활성임을 **암호학적으로 검증**.  Limine→seL4 chain of trust.
- **scheduler design** — SMT gang, lease switch, RT budget, μarch flush
  경계를 하나의 scheduler 로 통합.
