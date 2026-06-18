<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: soft real-time vs hard real-time — Y4 의 달성 방법
created: 2026-06-18T22:51:36+09:00   # KST (UTC+9)
status: brainstorming (§6.5 thermal 유예 부분 결정 / 나머지 미결).  일부 ⏳ hiu_abi / Phase C scheduler
scope: Y4 전체의 real-time 보장 (allocator 는 한 component).  scheduler /
       IPC / preemption / memory / WCET 의 cross-subsystem
refs:
  - README §Why Y4 (deterministic latency — wave-aligned, no jitter)
  - docs/amdv_safety.md §S4 (vmrun deadline, AV3 vmrun_terminates_within)
  - docs/lease_capability.md §4 (wave-aligned preemption — Phase C scheduler)
  - docs/power_safety.md (AV31 DVFS dwell / thermal throttle)
  - .brainstormings/20260618-212218-alloc-frontend-improvements.md §2.5
    (allocator deterministic latency) + §0.5 (atomic-free)
  - seL4 MCS (Mixed Criticality Support) kernel + WCET 분석
---

# soft real-time vs hard real-time — Y4 의 달성

## 0. 발제 (사용자 2026-06-18)

soft real-time 과 hard real-time 을 각각 어떻게 달성할까?

- **hard real-time**: deadline 을 **절대** 보장 (miss = 시스템 실패).
  worst-case 가 보장되어야 — WCET (worst-case execution time) 분석.
- **soft real-time**: 대부분 보장, 가끔 miss 허용 (best-effort + 통계적
  — p99/p999).  miss 시 graceful degradation (crash 아님).

Y4 의 README 핵심 가치 = **deterministic latency** (wave-aligned lease
scheduling, no cgroup/preempt jitter).  이걸 hard/soft 두 등급으로
어떻게 실현하나.

## 1. real-time 의 층위 + Y4 의 구조적 유리함

real-time 은 단일 component 가 아니라 **전체 path 의 worst-case** —
어느 한 층이 unbounded 면 전체가 무너짐:

| 층 | Y4 component | bounded? |
|---|---|---|
| scheduler | wave-aligned lease scheduling (Phase C kernel/scheduler) | wave 경계 = 자연 deadline |
| hypervisor | vmrun deadline (amdv S4 / AV3 `vmrun_terminates_within`) | ✅ 이미 invariant |
| allocator | bounded alloc + atomic-free (improvements §2.5/§0.5) | ✅ 설계됨 |
| IPC | scheme/msgport + cross-CPU free 위임 (§2.1) | bounded 必要 |
| preemption | seL4 preemption / MCS | seL4 WCET 분석됨 |
| memory | NPT / TLB / page fault | bounded 必要 (NPT pre-map?) |
| power | DVFS freq 변경 / thermal throttle (AV31) | ⚠️ WCET 흔듦 (§6) |

**Y4 가 hard real-time 에 드물게 유리한 6가지**:
1. **seL4 base** — 세계 유일 **WCET 분석된 verified kernel** (Blackham
   et al.).  Y4 의 hypervisor critical path 의 WCET 하한이 이미 확보.
   + seL4 **MCS** (Mixed Criticality Support) kernel = scheduling
   context + budget 으로 hard/soft 분리 네이티브 지원.
2. **Rust (no GC, no runtime)** — GC pause / runtime jitter 0.
3. **atomic-free allocator (§0.5)** — atomic RMW 의 cache contention /
   retry-loop jitter 0.  improvements §0.5 가 *사실 real-time 의 토대*
   였음 (소급).
4. **capability-based** — admission control 의 자연 메커니즘 (lease
   acquire 시 WCET budget 예약).
5. **wave-aligned** — accelerator 의 wave 경계 = **하드웨어 deadline**
   (자연 주기).  lease expiry (I6) = wave-aligned deadline.
6. **formal-first** — WCET bound 를 Verus invariant 로 (AV3 가 선례) →
   "formally-verified WCET" 가능.

→ Y4 는 "**formally-verified hard real-time hypervisor**" 가 될 드문
위치.  대부분 OS 는 hard real-time 을 측정/경험으로만 보장.

## 2. hard real-time 달성

### 2.1 WCET 분석 (end-to-end bound)
- seL4 의 WCET (verified) + Y4 의 각 bounded component 의 WCET 합성 →
  end-to-end worst-case.
- **모든 critical path component 가 bounded 여야**:
  - allocator: bounded-fit (SLOB §3.4) / bounded refill (improvements
    §2.5) / atomic-free (retry 0) / no background drain on dispatch
  - IPC: bounded message handling (scheme verb / msgport)
  - vmrun: AV3 `vmrun_terminates_within(deadline + slack)` (이미)
  - memory: page fault 회피 (critical region pre-fault / pinned, capability
    문서 §2.5 DMA-pinned 와 동족)

### 2.2 no unbounded — 금지 목록
- GC 없음 (Rust) / atomic retry loop 없음 (§0.5) / unbounded search 없음
  (bounded-fit) / background work on critical path 없음 (deferred to
  wave-idle, improvements §2.5) / lock contention 없음 (per-CPU exclusive)

### 2.3 admission control (capability 기반)
- hard real-time task = lease acquire 시 **WCET budget 예약** (capability
  문서 §2.3 의 quota 와 동족 — quota 의 *시간* 버전).
- 예약 불가 (budget 초과) → acquire 거부 (overload 방지).  seL4 MCS 의
  scheduling context budget 활용.

### 2.4 wave-aligned hard deadline
- accelerator dispatch = wave 경계 deadline.  wave 놓치면 다음 wave 까지
  (주기적 hard deadline).
- lease scheduler (Phase C) 가 wave 경계 전 dispatch 완료 보장 — vmrun
  deadline (AV3) + allocator bounded + IPC bounded 의 합이 wave period
  안.

### 2.5 formal WCET (Y4 차별점)
- AV3 (`vmrun_terminates_within`) 가 선례 — deadline bound 를 Verus
  invariant 로.
- 확장: **RT-WCET** invariant catalog — 각 critical path 의 WCET bound
  를 Verus spec (`<op>_terminates_within(N)`).  allocator (`alloc_
  terminates_within`), IPC, scheduler.
- end-to-end = 합성 정리 (component WCET 의 합 ≤ wave period).

## 3. soft real-time 달성

### 3.1 best-effort + 통계 (p99/p999)
- hard 의 worst-case 보장 대신 **통계적 보장** (p99 latency < target).
- 측정 기반 (P-redesign.6 microbench 인프라 재사용) — tail latency 추적.

### 3.2 background work 분리
- drain / coalescing (SLOB §3.5) / zone reclaim 을 **낮은 priority**
  background 로 — soft task 의 slack 에서.  hard task 의 critical path
  엔 절대 X (§2.2).

### 3.3 graceful degradation
- soft deadline miss → 품질 저하 (예: accelerator 결과 정밀도 ↓, frame
  drop) — crash 아님.
- reservation 초과 시 throttle (CBS — Constant Bandwidth Server 류).

### 3.4 reservation-based
- seL4 MCS 의 scheduling context budget = soft task 에 bandwidth 예약 →
  budget 소진 시 다음 period 까지 대기 (다른 task 보호).

## 4. hard ↔ soft 공존 — mixed-criticality (★)

- 같은 Y4 인스턴스에 hard (accelerator dispatch) + soft (telemetry,
  logging) + best-effort (background) task 공존.
- **seL4 MCS kernel 이 네이티브 지원** — scheduling context + budget +
  criticality.  Y4 가 이를 lease 에 매핑.
- **lease criticality attribute** (lease_capability.md 확장 후보):
  `LeaseCap` 에 `criticality: { Hard, Soft, BestEffort }` + `wcet_budget`
  / `period` 추가.  현 schema (§1) 엔 없음 → 추가 제안.
- hard lease = budget 보장 + admission control.  soft lease = slack
  활용 + CBS throttle.  best-effort = 남는 시간.
- **isolation**: criticality 간 간섭 차단 (capability — soft task 의
  overload 가 hard 의 WCET 못 흔듦).  capability 문서의 partition
  isolation 과 동족 (시간 축).

## 5. Y4 특수 — wave-aligned + lease + accelerator

- **wave 경계 = 하드웨어 주기 deadline** — accelerator 의 자연 real-time
  clock.  Y4 의 hard real-time 은 이 wave period 에 정렬 (artificial
  timer 가 아니라 hardware-driven).
- **lease expiry (I6) = wave-aligned deadline** — lease lifecycle 이
  이미 real-time deadline 개념 내장.
- **atomic-rotate (I4) = bounded context switch** — lease hand-off 의
  5-step 직렬화가 bounded → context switch WCET 보장.
- accelerator dispatch 의 hard deadline = wave 경계 전 (vmrun + alloc +
  IPC 의 WCET 합 < wave period).

## 6. subsystem 연결 + 충돌

### 정합 (이미 real-time 친화)
- allocator: §0.5 atomic-free (jitter 0) + §2.5 deterministic + bounded
  alloc → real-time component
- amdv: AV3 vmrun deadline (hard real-time vmrun)
- IPC: bounded message (cross-CPU free 위임도 bounded 여야)
- capability: partition isolation = 시간 isolation 의 공간 짝

### 충돌 (★ power management ↔ real-time)
- **DVFS frequency 변경** (power_safety AV31 dwell time) — CPU freq 가
  바뀌면 WCET 가 바뀜!  hard real-time 중 DVFS 가 WCET 가정을 깸.
  → hard task active 중 **DVFS 동결** 또는 WCET 를 최저 freq 기준으로
  (보수적).
- **thermal throttle** (power_safety) — 과열 시 강제 freq ↓ → hard
  deadline 위협.  hard real-time vs thermal safety 의 긴장 (둘 다 안전
  속성).  → **§6.5 에서 논의: soft throttle (`_PSV`) 만 Hard lease 가
  bounded 유예, hardlimit (AV27) 불변**.
- **C-state / idle** — deep sleep 진입/복귀 latency 가 wake deadline
  위협 (power AV28 wake).  hard task 는 shallow C-state 만?

## 6.5 thermal throttle 선택적 유예 (사용자 제안 2026-06-18)

제안: hard real-time deadline 을 위해 thermal throttle 을 **선택적으로
끌** 수 있게.

**안전 분석 — thermal 3단계 구분 필수** (power_safety §S21):
power_safety 가 이미 thermal 을 단계화:
- **S21.5 `_PSV` (passive trip)** = soft throttle — passive cooling,
  freq ↓.  headroom 있음
- **S21.4 `_TCC` (thermal control circuit)** = 중간, **host-only**
  (이미 capability-gated)
- **S21.8 hardlimit emergency** (`thermal_hardlimit_c`, **AV27**
  `thermal_hardlimit_emergency`: `t ≥ HARDLIMIT_C ⟹ all_vcpus_paused ∧
  lease_throttled`) = hard — 칩 보호 최후

→ **끌 수 있는 건 (a) soft throttle (`_PSV` passive) 뿐.  (b) S21.8
hardlimit (AV27) 은 절대 불가** — 하드웨어 손상 / 화재 방지.  "deadline
miss < 칩 파괴" 라 하드웨어 안전이 우선.

**기존 메커니즘 재사용** — 새로 만들 필요 적음:
- **S21.9 thermal force-toggle** (`y4.pm.thermal_force`, default
  `policy`) — *이미* thermal 유예 toggle 존재 (force 5개 중 하나, AV34
  `force_toggle_masks_mode_signal`).  real-time 이 이걸 활용.
- **S21.4 `_TCC` host-only** — 이미 capability-gated (host operator).
- **S18.5 host operator only thermal threshold** — threshold 변경도
  host-only.

**안전한 설계 (soft throttle 만, 기존 S21.x 위에)**:
- **capability-gated** — S21.4/S18.5 의 host-only 정합 + real-time §4 의
  Hard criticality lease 만 유예 요청.  아무나 못 끔
- **bounded** — 유예 budget: (i) 온도 budget (hardlimit 까지 ΔT 여유
  안), (ii) 시간 budget (N ms / wave 경계까지).  budget 소진 → 강제
  throttle 복귀
- **hardlimit 불변** — 유예 중에도 `t ≥ HARDLIMIT_C` 도달 시 AV27 무조건
  발화 (real-time 무시, all_vcpus_paused).  soft 유예가 hard limit 을
  *미루지 못함*
- **audit** — 유예 결정 + 온도 궤적이 power_safety audit trail 기록

**두 안전 속성의 우선순위 (결정)**:
```
S21.8 hardlimit (AV27)  >  hard real-time deadline  >  soft throttle (_PSV, default)
```
- soft throttle (`_PSV`) 영역: real-time 이 thermal headroom 빌림 (유예 OK)
- hardlimit 영역: 하드웨어 안전 절대 우선 (real-time 양보)

**새 invariant 후보**:
- **RT-thermal**: `throttle_deferred(lease) ⟹ Hard(lease) ∧ t <
  HARDLIMIT_C ∧ within_budget ∧ host_authorized` — 유예는 hard limit
  미만 + budget 안 + Hard lease + host 승인.
- **AV27 불변 보존**: 유예가 AV27 (hardlimit emergency) 를 *약화 X* —
  오직 S21.5 `_PSV` soft throttle 만 대상.  Verus: `deferred ⇒ AV27 still
  holds at HARDLIMIT_C`.

**결론**: 사용자 제안은 power_safety 의 기존 S21.9 force-toggle + S21.4
host-only + S21.8 hardlimit 구조에 **자연 정합** — "soft throttle (`_PSV`)
만 Hard lease 가 bounded + audited 로 유예, hardlimit (AV27) 은 불변".
새 메커니즘 최소, 기존 thermal force 위에 real-time criticality gate 추가.

## 7. 결정 / 미결

### 방향 (잠정)
- hard = WCET 분석 (seL4 verified + Y4 bounded components) + admission
  control (capability budget) + formal WCET invariant (AV3 확장) +
  wave-aligned deadline
- soft = 통계 (p99) + background 분리 + graceful degradation + MCS
  reservation
- 공존 = seL4 MCS + lease criticality attribute

### 부분 결정 (사용자 2026-06-18)
- **thermal throttle 선택적 유예 (§6.5)** — soft throttle (`_PSV`) 만
  Hard criticality lease 가 bounded (온도/시간 budget) + capability-gated
  (host-only, S21.4 정합) + audited 로 유예.  **S21.8 hardlimit (AV27)
  은 절대 불가** (하드웨어 안전 > deadline).  우선순위 = hardlimit >
  hard RT > soft throttle.  기존 S21.9 force-toggle 재사용

### 미결
- **lease criticality attribute** — lease_capability.md §1 에 `criticality`
  + `wcet_budget` + `period` 추가?  (schema v0 미동결이라 가능)
- **formal WCET 범위** — 어디까지 Verus invariant (verified WCET), 어디
  부터 측정 (soft).  RT-WCET catalog 의 scope
- **DVFS ↔ real-time** (§6 나머지) — DVFS freq 변경이 WCET 흔듦 → hard
  task 중 DVFS 동결 vs 최저-freq 보수적 WCET (thermal 은 §6.5 로 해소,
  DVFS 는 미결)
- **RT-thermal invariant 정식화** (§6.5) — `throttle_deferred ⟹ Hard ∧
  t<HARDLIMIT ∧ budget ∧ host` + AV27 불변 보존 증명.  power_safety §S21
  과의 spec 통합
- **seL4 MCS 활용 범위** — Y4 가 MCS kernel config 채택 여부 (현
  third_party/sel4 가 MCS build 인가?)
- **wave period ↔ WCET 합성** — vmrun + alloc + IPC 의 WCET 합이 wave
  period 안인지의 구체 budget (⏳ hiu_abi 의 wave timing)

### 후속 / 연결
- 본 발제가 allocator brainstorming 들 (§0.5 atomic-free / §2.5
  deterministic) 에 **real-time 의 의미 부여** — allocator 설계가 사실
  Y4 의 hard real-time 토대였음.
- AV3 (vmrun deadline) → RT-WCET invariant catalog 의 선례 → al-proof
  catalog (capability/allocator) + rt-proof catalog 로 확장 가능
- ⏳ hiu_abi (wave timing) + Phase C kernel/scheduler (lease scheduling)
  unblock 후 구체화.  지금은 아키텍처 레벨 + bounded component 설계까지
