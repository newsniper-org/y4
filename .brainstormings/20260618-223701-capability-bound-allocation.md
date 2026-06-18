<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Capability-bound allocation — partition-keyed zone + lease-lifecycle-coupled memory
created: 2026-06-18T22:37:01+09:00   # KST (UTC+9)
status: brainstorming (partition CPU 모델 §2.8 결정 / 나머지 미결).  일부 ⏳ hiu_abi v1.0 차단
scope: front-end (SLUB/SLOB) 종류와 직교하는 cross-cutting capability
       layer.  Y4 고유 (기존 allocator 에 없음).  back-end 논외
refs:
  - docs/lease_capability.md (LeaseCap schema v0 — partition_id 0..3,
    I1~I6, atomic-rotate, lifecycle)
  - .brainstormings/20260618-212218-alloc-frontend-improvements.md §2.6/§2.7
  - .brainstormings/20260618-221518-pluggable-alloc-frontend-contract.md
    (AllocContext, contract C1~C4)
  - docs/hiu_abi.md (⏳ v1.0 frozen 대기 — 일부 차단)
  - CLAUDE.md §6.2 (capability-based isolation) / README §Why Y4
---

# Capability-bound allocation

## 0. 발제 (Claude 제안 2026-06-18)

앞선 3 발제 (SLUB / pluggable / SLOB) 는 모두 **"기존 allocator 의 Y4
변형"** — Y4 가 *남의 설계를 개선*.  본 발제는 방향을 틀어, **어떤 기존
allocator 에도 없는 Y4 고유의 층**: capability-bound allocation.

DragonFly / SLUB / SLOB / mimalloc / snmalloc 전부 single-tenant 또는
tenant-agnostic.  그러나 Y4 의 존재 이유 = **OS-level lease isolation**
(README §Why Y4) — "user-space bugs cannot leak across tenants".
allocator 가 이를 구현하려면 **capability-aware** 여야 한다.

front-end (SLUB/SLOB) 종류와 **직교** — 어느 front-end 위에든 얹히는
cross-cutting layer (AllocContext 가 매개).

## 1. 왜 — lease schema 정합

`docs/lease_capability.md` 에서:
- **tenant 경계 = `partition_id` (P0..P3, HIU `NUM_PARTITIONS=4` hard
  cap)** — 동시 활성 tenant ≤ 4 (I1 partition disjointness 따름정리).
  메모리 zone keying 은 `lease_id` 가 아니라 **`partition_id`** 가 정확
  (hard isolation boundary).
- **memory quota 는 §4 "policy layer, out-of-scope (mechanism 만)"** —
  즉 allocator 의 capability layer 가 이 policy 의 *메모리 측* 을 구현할
  자리.  lease schema 가 비워둔 칸.
- **I2 shadow slot disjointness** (≤16) — accelerator-visible memory 의
  partition.  improvements §2.7 (shadow-region-aligned) 직결.
- **I4 atomic-rotate / I6 expiry / revoke** — lease lifecycle 의 5-step
  직렬화.  메모리 allocation lifecycle 이 여기 coupling 되어야.
- **sealed type** (ChachaKey/Nonce, read-out 불가) — 그 메모리를 다루는
  allocation 의 zero-on-free 강제.

## 2. 설계

### 2.1 cross-cutting layer 위치 (front-end 직교)
- `AllocContext` (pluggable §2.1, improvements §2.6) 에 `partition_id`
  포함 — `{cpu, partition_id, numa}`.  AllocFrontend 의 alloc/free 가
  이 ctx 를 받음.
- capability layer = AllocContext 의 `partition_id` 를 zone keying +
  quota 검사에 쓰는 **thin cross-cutting 로직** — SLUB-style 이든 SLOB-
  style 이든 동일하게 얹힘.
- 즉 "어느 front-end + 어느 freelist + capability layer" 3-way 직교.

### 2.2 partition-keyed zone (cross-tenant UAF 차단)
- zone key 에 `partition_id` 추가 — `(cpu, class, partition_id)` (SLUB)
  또는 `(cpu, partition_id)` (SLOB).  **최대 4 partition** 이라 zone 수
  폭증 없음 (improvements §2.1 의 lease_id 우려 해소 — partition 은 4
  hard cap).
- **cross-tenant UAF 차단**: partition P0 이 free 한 chunk 를 P1 이 재할당
  불가 (다른 partition zone) → UAF / 정보 누출이 partition 경계 안에
  갇힘.  I1 (partition disjointness) 의 **메모리 측 따름정리**.

### 2.3 partition memory quota (lease schema §4 의 빈 칸 채움)
- 각 partition 의 memory high-watermark.  lease_capability.md §4 가
  "policy layer" 로 남긴 quota 의 메모리 측 구현.
- **per-CPU budget 분배 (§2.8 결정 정합)**: `partition_quota[pid]` 를
  CPU 에 분배 → `per_cpu_budget[cpu][pid]`.  alloc hot-path 는 **per-CPU
  local budget 만 검사** (`per_cpu_used[cpu][pid] + size ≤ per_cpu_
  budget[cpu][pid]`) → atomic 0 (§0.5 유지).  초과 시 `Err(QuotaExceeded)`
  또는 lease lifecycle 에 재배분 요청 (드묾).
- 전체 partition 사용량 = per-CPU 값들의 **합산** → **§2.8 에 의해
  lease lifecycle 에서만** (hot-path 에서 절대 합산 X).
- quota 는 lease acquire (lease_capability §3.1) 시점에 lease attribute
  로 설정 가능 (HIU lease 가 메모리 quota 도 운반?) — ⏳ hiu_abi.

### 2.4 lease lifecycle ↔ allocation lifecycle (★ 핵심 coupling)
- **lease release / expiry(I6) / revoke → partition allocation bulk 회수**:
  lease_capability §3.3 의 atomic-rotate (I4) 가 발화될 때, 그 partition
  의 *모든 live allocation 을 일괄 free* (partition zone drop).
- **이점**: tenant 가 free 를 빠뜨려도 lease 종료 시 누수 0 (partition
  단위 garbage 회수).  bulk free = partition zone 통째 release →
  per-allocation free 불필요 (deterministic + 빠름).
- **atomic-rotate 정합**: I4 의 5-step 안에서 memory 회수가 어느 step?
  step 1 (`cap_table.invalidate`) 직후 partition 메모리 동결 → step 5
  완료 후 zone reclaim.  게스트는 I4 동안 HIU + 메모리 접근 차단 (I4
  invariant 와 동일 윈도).
- **sealed memory**: 회수 시 zero-on-free 강제 (ChachaKey 등 sealed
  data 잔존 0) — improvements §2.3 + lease sealed type.

### 2.5 shadow-region allocation (accelerator-visible, ⏳ hiu_abi)
- I2 (shadow slot disjointness, ≤16) — accelerator 가 직접 보는 메모리.
- shadow-region-aligned allocation (improvements §2.7) = shadow slot
  경계 정렬 + partitioned TLB (4×16) entry 와 1:1.
- DMA-pinned (migration 금지) + accelerator-NUMA-local (HBM).
- ⏳ hiu_abi v1.0 frozen 차단 — 지금은 §2.1 의 AllocContext 에 shadow
  hint 를 forward-compat hook 으로만.

### 2.6 sealed allocation (read-out 차단)
- lease 의 ChachaKey(256-bit)/Nonce(192-bit) = sealed (read-out 불가,
  lease_capability §1).  이를 담는 allocation:
  - zero-on-free 강제 (improvements §2.3, embedded 도 sealed 면 예외 X)
  - capability layer 가 sealed allocation 을 별도 marking → free 시 무조건
    zeroize (form-factor hardening 차등, SLOB §3.7, 의 예외 — sealed 는
    항상 zero)
- Verus: `sealed(alloc) ⇒ ∀ free, zeroized(alloc.range)` invariant.

### 2.7 형식화 — AL-partition invariant
- **AL-partition** (I1 의 메모리 refinement): `∀ chunk c allocated to
  partition P, ∀ later realloc r, owner(r) ∈ {P} ∪ released(P)` —
  cross-tenant 재할당 불가.
- **AL-quota**: `∀ partition P, Σ live(P).size ≤ quota(P)`.
- **AL-sealed**: `sealed(c) ⇒ free(c) ⟹ zeroized(c)`.
- **AL-bulk**: `lease_release(P) ⟹ ∀ c ∈ live(P), freed(c) ∧ zeroized
  if sealed`.
- pluggable contract 의 **C5 (capability)** 신규 계층 후보 (§3).

### 2.8 partition CPU 모델 — aggregation 은 lease lifecycle 에서만 (결정 2026-06-18)

> 사용자 결정.  §6 미결 1번 (partition CPU 모델 ↔ §0.5 긴장) 해소.

**결정**: partition 별 상태 (`partition_used`, quota budget 등) 는
**per-CPU 로 분할** 보관하고, **모든 aggregation (per-CPU 값들의 합산
포함) 은 반드시 lease lifecycle event 에서만** 수행.

- **hot-path (alloc/free)**: per-CPU local 상태만 read/write — `per_cpu_
  used[cpu][pid]` 갱신.  **atomic 0** (§0.5 불가침 유지).  cross-CPU
  공유 상태 접근 0.
- **aggregation (합산 / 재배분 / 전체 사용량 조회)**: lease lifecycle
  event (acquire §3.1 / release·expiry·revoke §3.3 / atomic-rotate I4)
  에서만.  이 event 들은 **드물고** (per-allocation 아님) + 이미 직렬화
  지점 (I4 atomic-rotate 의 single capability invocation) → 합산이
  atomic 없이 안전 (lifecycle 이 이미 quiescent window).
- **함의**: quota 의 전체 정확도는 lifecycle 시점에만 보장 (hot-path 는
  per-CPU budget 근사).  per-CPU budget 초과 시 → lifecycle 재배분 요청
  (드문 slow-path).  이 근사가 deterministic latency + atomic-free 의
  대가 — tenant quota 는 hard real-time boundary 가 아니라 lifecycle-
  granular 이므로 수용 가능.
- **§0.5 정합 확인**: hot-path atomic 0 → Verus sequential per-CPU
  reasoning 유지 (Kani 불필요).  aggregation 은 lifecycle 의 직렬화
  window 안 → 그것도 sequential.  capability layer 전체가 atomic-free.

이로써 capability layer 가 §0.5 (atomic 배제) 와 충돌 없이 성립 — "모든
aggregation 은 lease lifecycle 에서만" 이 그 화해의 핵심 규칙.

---

## 3. front-end 직교성 + pluggable contract 관계

- capability layer 는 front-end (C1~C4) 위 **별도 계층** — front-end 가
  partition-agnostic 하게 동작하고, capability layer 가 partition_id 로
  zone 선택 + quota gate + bulk reclaim 을 감쌈.
- **옵션**:
  - (a) **pluggable contract 에 C5 (capability) 추가** — AllocFrontend
    가 C5 (AL-partition / AL-quota / AL-sealed / AL-bulk) 도 만족.  단
    front-end 마다 재증명
  - (b) **별도 wrapper layer** `CapabilityAlloc<F: AllocFrontend>` —
    capability 로직을 generic wrapper 로, front-end 는 partition 모름.
    C5 증명 1회 (wrapper) → 모든 front-end 재사용 (덧셈 검증, freelist
    분리와 같은 이득)
- **잠정 (b)**: wrapper 가 깔끔 — capability layer 가 front-end 직교라는
  본질을 type 으로 표현 (`CapabilityAlloc<SlubStyle>` / `CapabilityAlloc
  <SlobStyle>`).  C5 증명 1회 + front-end 의 C1~C4 와 합성.

## 4. hot-path 비용

- partition_id 가 이미 AllocContext 에 있음 → zone 선택 시 key 에
  partition_id 포함 (추가 lookup 0).
- quota gate = 비교 1회 (`used + size ≤ quota`).
- bulk reclaim = lease lifecycle (드묾) 에서만, hot-path 무관.
- sealed marking = allocation flag 1bit.
- → capability layer 의 hot-path overhead ≈ 0 (partition keying +
  per-CPU budget 비교 1회).  **atomic-free (§0.5) 확정** — §2.8 결정
  (per-CPU 분할 + aggregation 은 lease lifecycle 에서만) 으로 hot-path
  cross-CPU 공유 접근 0.

## 5. 기존 대비 — capability-aware allocator 부재

| allocator | tenant 모델 |
|---|---|
| DragonFly SLAB / Linux SLUB·SLOB | single-tenant kernel (tenant 개념 0) |
| mimalloc / snmalloc | per-thread (tenant-agnostic, 보안 격리 X) |
| scudo / Hardened malloc | hardening O, tenant 격리 X |
| **Y4 capability-bound** | **partition (HIU lease) 별 hard 격리 + quota + lifecycle coupling** |

→ "lease capability 가 메모리 allocation 의 namespace + quota + lifecycle
owner" = 어떤 기존 allocator 에도 없음.  paper §6.1 최강 차별점 후보.
README §Why Y4 의 "OS-level lease isolation" 의 allocator 측 실체.

## 6. 결정 / 미결

### 확정 (사용자 2026-06-18)
- **partition CPU 모델 (§2.8)** — per-CPU 분할 + **모든 aggregation
  (합산 포함) 은 lease lifecycle event 에서만**.  hot-path atomic 0 →
  §0.5 정합 + Verus sequential.  capability layer ↔ atomic-free 화해의
  핵심 규칙

### 방향 (잠정)
- capability layer = front-end 직교 cross-cutting (`CapabilityAlloc<F>`
  wrapper, §3 (b))
- partition-keyed zone (P0..3, I1 메모리 따름정리) + partition quota
  (lease §4 빈 칸, per-CPU budget) + lease-lifecycle bulk reclaim
  (I4/I6) + sealed zero-on-free
- C5 capability contract (AL-partition / AL-quota / AL-sealed / AL-bulk)

### 미결
- **quota 의 출처** — HIU lease 가 memory quota 운반 (⏳ hiu_abi) vs Y4
  policy layer 가 독립 설정.  lease_capability §4 가 policy 로 남긴 부분
- **C5 = contract 계층 (a) vs wrapper (b)** — wrapper 유력하나 C5 증명이
  front-end 의 C1~C4 와 어떻게 합성되는지 (Verus 합성 증명)
- **shadow-region allocation (§2.5)** — ⏳ hiu_abi v1.0 frozen 전면 차단.
  AllocContext shadow hint 만 forward-compat
- **bulk reclaim ↔ atomic-rotate step 정합** (§2.4) — I4 의 어느 step 에
  메모리 동결/회수.  lease runtime (kernel/lease/) 미구현이라 ⏳

### 후속 / 연결
- 본 발제 = improvements §2.6(capability)+§2.7(accelerator) 의 심화 +
  lease_capability.md 와 직접 정합.
- 4 brainstorming (improvements / pluggable / SLOB / capability) 수렴
  시: capability = "front-end 직교 cross-cutting 층 + lease 정합" 장.
- ⏳ hiu_abi 차단 부분 (§2.5 shadow, §2.3 quota 출처, §2.4 atomic-rotate
  정합) 은 hiu/ unblock 후 — 지금은 partition-keyed zone + bulk reclaim
  + sealed (hiu 무관 부분) 까지 설계 가능.
