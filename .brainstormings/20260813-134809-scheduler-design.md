<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Scheduler design — seL4 MCS·domain 메커니즘 위의 Y4 정책층 (여러 발제가 위임한 통합 지점: partition domain / SMT gang / RT / lease flush / restart budget)
created: 2026-08-13T13:48:09+09:00   # KST (UTC+9)
status: brainstorming (통합 발제 — 여러 선행 발제의 위임 회수).  결정 다수 + 미결 일부.  rev: §5.1 PikeOS/ARINC 653 계층적 feasibility 참고 추가
scope: Y4 scheduler 최종 목표.  seL4 mechanism(MCS SC + domain scheduler) 위의 policy 층.
       partition_id→domain / SMT gang / hard·soft RT / admission / restart budget /
       atomic-free 경계 / accelerator co-schedule / 검증
refs:
  - .brainstormings/20260813-124610-side-channel-isolation.md §4(lease boundary flush)·§6(SMT sibling=동일 partition)
  - .brainstormings/20260618-225136-soft-vs-hard-real-time.md (MCS·admission·criticality·WCET)
  - .brainstormings/20260812-174402-capsule-fault-isolation-restart.md §5·§5.1(restart budget bounded)
  - .brainstormings/20260618-223701-capability-bound-allocation.md (partition P0..3 = HIU partition_id)
  - .brainstormings/20260812-173424-ipc-layer-design.md §0.5(atomic-free, cross-CPU=IPC 위임)
  - MEMORY/wavetensor_terms.md (partitioned TLB 4×16 → 4 partition / lease / context_switch)
  - PikeOS(SYSGO, L4 계열) + ARINC 653(개방 표준) — 계층적 time-partition 스케줄링·feasibility 참고(§5.1, 코드 아닌 모델/분석)
  - CLAUDE.md §6 원칙 5(verified base 위 specialization)·6(formal-first)
---

# Scheduler design — seL4 메커니즘 위의 partition-aware 정책층

## §0 프레임

**최종 구현 목표 기준.**  scheduler 는 여러 발제가 명시적으로 **위임한 통합
지점**이다 — side-channel §4(lease boundary μarch flush) · §6(SMT gang) ·
real-time(MCS·admission·criticality) · capsule §5(restart budget bounded).
본 발제가 이들을 하나의 scheduler 정책으로 봉합.

★ Y4 는 scheduler 를 **바닥부터 쓰지 않는다**(원칙 5) — **seL4 의 verified
스케줄링 메커니즘**(MCS scheduling context + domain scheduler) 위에 WaveTensor
특화 **정책층**만 저작.

## §0.5 원칙 정합

- **atomic-free**(IPC §0.5) — ★ **경계 주의**: §0.5 는 **Y4-authored 층**에
  적용(policy 는 per-CPU + IPC 조율).  seL4 mechanism 내부 동기화(SMP big
  lock 등)는 **verified given(TCB)** — Y4 원칙 대상 아님(§7).
- **TCB 최소화**: scheduler policy 를 최소화, 결정은 seL4 verified 메커니즘에
  위임.
- **capability**: CPU 시간 = **seL4 SC capability**(budget/period), admission
  = capability feasibility 검사(§5).
- **partition**: partition_id(side-channel §2 master key) → seL4 domain(§2).

## §1 layered — seL4 mechanism vs Y4 policy

| 층 | 담당 | 내용 |
|---|---|---|
| **seL4 mechanism** (verified) | seL4 | **MCS SC**(budget/period capability) · priority · **domain scheduler**(static cyclic 고정 time-slice top-level 분할) |
| **Y4 policy** (본 발제) | Y4 | partition_id→domain · lease/tenant→SC · criticality→priority+admission · SMT gang→core 배정 |

원칙 5: 바닥부터 X — seL4 의 **domain scheduler**(정적 순환 시간 분할)와
**MCS SC**(시간 budget capability)를 그대로 쓰고, 그 위 매핑/정책만 저작.

## §2 (결정) partition_id → seL4 domain — 통합 지점 ★

seL4 **domain scheduler** = build-time 고정된 **static cyclic time-slice**
분할 → deterministic.  ★ 이것이 side-channel §4 가 요구한 바로 그것 —
**domain switch = 결정적 partition 경계 = μarch flush 지점**(seL4 time
protection 이 domain switch 마다 flush).

- **partition_id ↔ seL4 domain 매핑**.  WaveTensor 는 **4 partition**(partitioned
  TLB 4×16, P0..P3) → **4 partition domain + 1 Y4 system/background domain**.
  seL4 domain 이 build-time 고정 수라는 제약과 **정확히 부합**(partition 수가
  작고 고정 — Y4 의 bounded 스타일, MAX_CAPSULES=32 와 동류).
- 이로써 **temporal isolation(시간 분할) + side-channel flush 경계 + master-key
  (side-channel §2)** 가 **하나로 수렴**.
- domain **내부**: MCS SC + priority 로 tenant workload 스케줄.
- **spatial 4 slot × temporal multiplex**: 4 partition domain 은 동시 격리
  slot(하드웨어 한계).  더 많은 tenant 는 **lease 로 slot 에 시분할 배정** —
  domain switch(순환, 매 quantum, flush)는 빠른 경계, **lease 재배정**(tenant↔
  slot 교체)은 coarser 경계(full flush + accelerator `context_switch` nonce
  rebind).

## §3 (결정) SMT gang — domain 이 whole physical core 소유

side-channel §6 결정(SMT sibling=동일 partition)의 구체화:
- **domain 은 whole physical core(모든 SMT sibling) 단위로 점유** →
  cross-domain SMT co-location **0**.  또는 cross-tenant 구간 SMT off.
- core 배정이 **partition-aligned** — SMT sibling 이 partition 경계를
  가로지르지 못함.  side-channel §2 표의 "SMT group → partition_id" enforcer.

## §4 (결정) real-time — hard / soft (real-time 발제 통합)

- **hard-RT lease** → MCS **SC(budget/period 보장)** + **admission control**
  (§5) + **formal WCET**(real-time 발제, AV3 확장).
- **soft-RT** → reservation 내 best-effort + **background 분리**(별도 낮은
  priority) + graceful degradation.  background 는 hard-RT partition 의
  **미사용 slack 을 재활용**(PikeOS "time partition 0" 방식, §5.1) — 보장
  window 훼손 없이 slack 만 사용.
- **wave-aligned deadline** — accelerator lease period 에 host thread(가속기
  daemon/driver)를 **co-schedule**(위상 정렬, §8).
- lease **criticality** attr(공개 metadata) → priority + admission 등급.
  (side-channel §5 정합 — criticality 는 비밀 아님.)

## §5 (결정) admission control = capability feasibility 검사

- 새 lease 의 RT 요구는 **schedule feasibility**(Σ budget ≤ capacity 등
  스케줄링 조건) 만족 시에만 admit.
- **lease capability 가 budget 을 인코딩** → 발급 자체가 feasibility 보장
  (oversubscription 금지).  capability-bound 모델 정합 — CPU 시간도
  capability.
- feasibility 알고리즘은 §5.1(PikeOS/ARINC 653 계층적)로 결정.

## §5.1 (참고 + 결정) hard-RT feasibility — PikeOS/ARINC 653 계층적 스케줄링

**PikeOS**(SYSGO)는 **L4 계열**(seL4 와 공통 조상)의 상용 RTOS/hypervisor 로,
**time-and-space partitioning** 을 DO-178C / EN 50128 / ISO 26262 / IEC 61508
수준까지 인증한 성숙한 실사례.  그 스케줄러 구조가 Y4 §2(domain = time
partition)와 **정확히 동형** → 모델·분석을 참고(코드 아님, 아래 reuse).

- **모델 = ARINC 653(개방 표준) 2-level scheduler**:
  - top: **static cyclic time partition**(major time frame → partition
    window) = **seL4 domain scheduler = Y4 §2 partition domain**.
  - within: **fixed-priority preemptive** = domain 내 MCS SC/priority
    (seL4 도 fixed-priority — 정합).
- **결정 — feasibility = 계층적(compositional) schedulability**(plain global
  RM/EDF 아님):
  1. top-level cyclic schedule 이 각 partition 에 **주기적 resource supply**
     (major frame 당 보장 budget) 제공.
  2. within-partition **fixed-priority response-time / supply-bound function**
     로 그 supply 에 대해 task set 분석.
  §2(domain) + §4(within-domain priority) 구조에 **직접 대응** — admission(§5)이
  이 2-level 검사를 수행.  fixed-priority 로 확정(seL4 메커니즘 정합, EDF 배제).
- **slack reclamation**(PikeOS 특징): time partition 이 window 를 다 안 쓰면
  그 slack 을 **background/priority partition**(PikeOS "time partition 0")이
  사용 → **hard-RT 보장 window 훼손 없이** soft-RT/background 를 slack 에 태움.
  ★ §4 soft-RT background 분리의 성숙한 실현 — §4 에 반영.
- **reuse mode**: PikeOS 는 **proprietary(SYSGO, closed)** → **코드 X**.  참고
  대상 = **개방 표준(ARINC 653) + 학술 compositional RT 이론**(PikeOS 는 그
  L4-계열 exemplar).  idea/analysis-level 참고, clean-room 재구현(원칙 5 /
  contribute-back 정합).

## §6 (결정) restart budget (capsule §5 통합)

- crashing driver 의 restart(capsule §5.1 lifecycle)는 **bounded reservation**
  을 소모 → 초과 시 device dead(capsule §5.1 per-member limit).
- scheduler 가 restart 를 reservation 으로 강제 → **restart 가 다른 tenant
  deadline 을 못 깬다**(RT budget 격리).

## §7 (결정) atomic-free scheduling — ★ 경계 주의

- **Y4 policy 층** = **per-CPU runqueue**(각 core 가 자기 domain thread 를
  스케줄) + cross-CPU 조율(load balance / migration)은 **IPC(msgport)** 로 —
  공유 atomic runqueue 아님.  allocator 의 cross-CPU-free-via-IPC 패턴과 동형
  (§0.5).
- ★ **경계**: §0.5 atomic-free 는 **Y4-authored 층**에 적용.  seL4 mechanism
  의 내부 동기화(SMP big kernel lock 등)는 **verified given** — Y4 원칙의
  적용 대상이 아니라 신뢰하는 base(원칙 5).  즉 "atomic-free" 는 Y4 코드의
  약속이지 seL4 재작성 요구가 아님.

## §8 accelerator co-scheduling

- WaveTensor lease 를 쥔 tenant 의 host thread 를 그 **lease period 와
  co-schedule**(위상 정렬) — 가속기가 도는 동안 그 tenant 의 host 측
  daemon/driver 가 같은 domain 시간에.
- ⏳ `hiu_abi.md`(lease period / context_switch 타이밍 ABI)에 의존.

## §9 verification

scheduler **policy** 불변식(Verus):
- **admission feasibility** — oversubscription 0(Σ budget ≤ capacity).
- **partition-domain isolation** — thread 는 자기 partition 의 domain 시간에만
  실행.
- **SMT-gang** — cross-partition SMT co-location 0(§3).
- **flush-at-domain-switch** — domain 전환 시 μarch flush(side-channel §4 SC
  invariant)와 정합.

seL4 **domain scheduler correctness** 는 **inherited**(verified base) — Y4 는
policy 매핑만 증명, seL4 메커니즘은 재증명 X(§1, 원칙 5).

## §10 결정 / 미결 요약

**결정 방향(강)**:
- seL4 mechanism(MCS SC + domain scheduler) 위 **policy 층만 저작**(원칙 5)
- **partition_id → seL4 domain**(4 partition + system domain), temporal
  isolation·flush 경계·master-key 수렴(§2)
- **SMT gang = domain 이 whole physical core 소유**(§3)
- hard-RT = MCS SC + admission, soft = reservation + background 분리(§4)
- **admission = lease capability feasibility 검사**(oversubscription 금지, §5)
- **feasibility = 계층적(compositional) schedulability**(PikeOS/ARINC 653
  참고): top cyclic supply + within-partition **fixed-priority** response-time
  (EDF 배제, seL4 정합); slack reclamation 으로 background(§5.1)
- restart budget = bounded reservation(§6)
- Y4 policy 는 **per-CPU + IPC 조율**(atomic-free); seL4 mechanism 은 verified
  given(§7)

**미결(설계 필요) — 아래 항목은 본 발제 범위 밖, 별도 후속 논의로 이월**:
- domain 수 vs partition 수 정합 세부(seL4 domain build-time 고정 — 4+1 확정?
  system/background domain 분할?)
- supply-bound function 세부 파라미터(major frame 길이 / window 배치)
- IPC 기반 load-balance/migration 정책(빈도·trigger)
- wave-aligned co-schedule 구체(⏳ hiu_abi)

**⏳ 선행 의존**:
- `hiu_abi.md` v1.0 frozen — lease period / context_switch 타이밍(§8)
- seL4 MCS + domain_schedule build config(§1~2)
- side-channel μarch flush 구현(§2 domain switch 지점)

## §11 다음 발제 후보

- key management deep-dive — AK/session/XChaCha20 key lifecycle(attestation §10)
- reproducible build / supply-chain — RIM 공개 전제(attestation §9 유일 잔여 미결)
- boot chain deep-dive — Limine measured-boot vs DRTM 경로(attestation §2)
