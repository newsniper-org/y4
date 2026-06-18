<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Memory allocator front-end 개선 brainstorming
created: 2026-06-18T21:22:18+09:00   # KST (UTC+9)
status: brainstorming (결정 X — 옵션 탐색)
scope: Y4/alloc/ front-end (SLAB + integrated layer)
refs:
  - Y4/alloc/src/{slab,integrated,hardened,types,page_backend}.rs
  - Y4/proofs/verus/src/alloc/{state,slab,scudo,boundary,refinement}.rs
  - docs/architecture.md §Memory allocator
  - CLAUDE.md §6 (TCB minimization / capability-based / tenant isolation)
---

# Memory allocator front-end 개선 brainstorming

> verus-fork 측 작업 (R7.11 cross-check) 대기 중 진행하는 design 탐색.
> 결정 아님 — 후속 sign-off cycle 의 input.

## 0. 현재 baseline (개선 대상)

| 영역 | 현 구현 |
|---|---|
| 아키텍처 | DragonFly SLAB lock-free front + hardened/scudo backend 2-tier |
| size class | 9개 고정 power-of-2 (`[8,16,32,64,128,256,512,1024,2048]`), 2KB+ bypass (`slab.rs`) |
| caching | per-(CPU, class) zone, MAX_CPUS=32, Z_MAX=64.  per-thread/per-tenant cache 없음 |
| lock-free | hot-path exclusive ownership (CAS 없음, lock 없음); cross-CPU free = `InvalidArg` 거부 |
| hardening | guard page / quarantine / randomization = hardened backend (large) 에만.  SLAB small path 는 hardening 0 |
| API | `alloc(cpu, layout)` / `free(cpu, alloc)` 명시적 인자.  `GlobalAlloc` impl 없음 |
| verification | S1/S2/S3 (slab) + B1~B6 (hardened) + X1~X3 (boundary).  B3/B6/X3 constructive, 나머지 `assume()` at trusted boundary |

**핵심 gap 3개**: (1) small allocation 의 hardening 부재, (2) cross-CPU
free 미구현, (3) tenant isolation 이 per-CPU 입자 뿐 (같은 CPU 의 여러
tenant 간 분리 X).

---

## 1. Tenant-keyed zone — capability-bound allocation (★ Y4 차별점 후보)

### Motivation
Y4 의 핵심 가치 = tenant isolation (CLAUDE.md §6.2).  현 SLAB 는 per-CPU
zone 만 — **같은 CPU 에 두 tenant 가 같은 size class 를 쓰면 freelist 가
공유**됨.  tenant A 가 free 한 chunk 를 tenant B 가 재할당 → use-after-free
가 cross-tenant 정보 누출로 비화 가능.

### 옵션
- (a) **lease-keyed zone**: zone key = `(cpu, class, lease_id)`.  lease
  capability 가 메모리 pool 의 namespace.  같은 size class 라도 tenant
  분리 → UAF 가 tenant 경계 안에 갇힘
- (b) per-tenant **separate arena** (coarse): tenant 마다 독립 page pool,
  SLAB 는 그 위에 — 더 강한 격리, overhead ↑
- (c) zone 공유 유지 + free 시 **zeroing + freelist pointer obfuscation**
  으로 누출만 차단 (격리 X, 완화만)

### Tradeoff
- (a) 가 Y4 의 lease capability (`docs/lease_capability.md`) 와 자연 정합
  — "allocator 가 capability-aware" = 학술적 차별점 (paper artifact §6.1
  후보).  단 zone 수 폭증 (cpu × class × lease) → owner table capacity
  (현 `LinearMap` 256) 재설계 필요
- (b) 는 메모리 overhead 큼 (tenant 당 최소 arena)
- (c) 는 싸지만 isolation guarantee 약함

### Verification 영향
새 invariant 후보: **AL-tenant** — `∀ chunk c, alloc'd to lease L ⇒
∀ later realloc, owner(realloc) ∈ {L} ∪ released(L)`.  S1 (per-CPU
disjoint) 의 자연 확장.  refinement.rs 패턴으로 constructive 가능성.

### Y4 정합도: ★★★ (lease capability + tenant isolation 의 직접 구현)

---

## 2. Small-path hardening — SLAB front 의 보안 강화

### Motivation
대부분 allocation 이 small (< 2KB → SLAB).  현재 SLAB 는 hardening 0 —
보안 표면의 대부분이 무방비.  hardened backend 는 large 만 cover.

### 옵션 (overhead 오름차순)
- (a) **freelist pointer obfuscation**: free chunk 의 next-pointer 를
  per-zone secret 과 XOR.  freelist corruption (heap overflow 로 다음
  포인터 조작) 방어.  overhead ≈ 0 (XOR 1회), no size class 변경
- (b) **delayed reuse (zone-local quarantine)**: free 한 chunk 를 즉시
  freelist 복귀 X, 작은 ring (예: 8칸) 통과 후 복귀.  UAF window 축소.
  hardened B6 의 small 버전.  overhead = ring 크기
- (c) **per-chunk canary / redzone**: chunk 끝에 magic word.  overflow
  detection.  단 size class 마다 overhead (8B class 에 8B canary = 2배)
  → power-of-2 class 와 충돌, fine-grained class 필요 (§3 와 묶임)
- (d) **zone guard page**: per-allocation 아닌 zone 경계 (page 단위).
  zone overflow 만 잡음, intra-zone 은 못 잡음.  overhead = page/zone

### Tradeoff
(a) + (b) 가 cost/benefit sweet spot — overhead 거의 0 인데 freelist
공격 + UAF window 둘 다 완화.  (c) 는 §3 (fine-grained class) 선행 필요.
(d) 는 약하지만 거의 공짜.

### Verification 영향
(a) obfuscation → **AL-obfusc**: `deobfusc(obfusc(ptr, secret), secret)
== ptr` (자명, 즉시 discharge).  (b) → B6 quarantine spec 의 small
mirror.

### Y4 정합도: ★★★ (TCB 안의 allocator 가 hardening 책임 — 표면 대부분 cover)

---

## 3. Size class 전략 — fragmentation vs verification 부담

### Motivation
현 9개 power-of-2 class → internal fragmentation 최악 ~2배 (예: 1025B
요청 → 2048 class, 998B 낭비).  실제 워크로드의 평균 fragmentation 50%
근처.

### 옵션
- (a) **현행 유지** (9 power-of-2): verification 부담 최소, fragmentation
  감수
- (b) **jemalloc-style sub-power-of-2 spacing**: 각 power-of-2 구간을
  4분할 (예: 16-spacing for small → 16/32/48/64, then 1.25× growth).
  fragmentation 최악 ~25% 로 감소.  class 수 ~30개
- (c) **tcmalloc-style 테이블** (~88 class): fragmentation 최소, 단
  spec 증명 + owner table 부담 큼
- (d) **2-tier**: small (< 256B) 은 fine-grained (b), medium 은 power-of-2
  (a) — 작은 객체가 많으니 거기만 촘촘히

### Tradeoff
Y4 는 **formal-first** (CLAUDE.md §6.6) — class 수 ↑ 는 S2/S3 spec 의
case 폭증 + owner table capacity.  단 S2 (Z_MAX) / S3 (aligned) 는
class 수에 parametric 하게 작성 가능 (배열 순회 invariant) → class 수가
증명 *구조* 를 바꾸진 않음, *상수* 만 바뀜.  (d) 가 실용적 타협.

### Verification 영향
S3 (`class_size ≥ align`) 는 class 테이블이 sorted + align-respecting
이면 parametric.  class 테이블을 spec const 로 두고 invariant 를 `∀ i`
형태로 → class 수 무관.

### Y4 정합도: ★★ (실용 이득 크나 Y4 특수성과 직결 X)

---

## 4. Cross-CPU free — 현재 거부 → async reclaim

### Motivation
현 cross-CPU free = `InvalidArg`.  caller 가 IPI 라우팅 책임 (미구현).
실제로는 흔한 패턴 (producer CPU alloc, consumer CPU free) → 현 설계는
caller 에 부담 전가.

### 옵션
- (a) **per-CPU MPSC remote-free inbox**: 각 CPU 가 lock-free MPSC queue.
  remote free = owner CPU 의 inbox 에 push (lock-free).  owner CPU 가
  alloc hot-path 또는 주기적 reaper 에서 drain.  DragonFly magazine/depot
  패턴
- (b) **Y4 IPC msgport 경유**: free 요청을 owner CPU 의 msgport 로 (ipc/
  msgport.rs 와 짝).  Y4 의 기존 IPC 인프라 재사용 — 새 동기화 primitive
  0.  단 IPC overhead (msgport 가 free 1건에 무거울 수 있음)
- (c) **seL4 IPI cap** (roadmap 원안): IPI 로 owner CPU 깨움.  무거움
- (d) **현행 유지** (거부): caller 책임, allocator 단순

### Tradeoff
(a) 가 hot-path 영향 최소 + self-contained.  (b) 는 Y4 IPC 와 정합 좋으나
free-heavy 워크로드에서 msgport 압박.  **(a) lock-free inbox + batch
drain** 추천 — DragonFly 검증된 패턴 + Y4 IPC 의존 0.

### Verification 영향
새 invariant: **AL-remote** — `∀ chunk in inbox(cpu), owner(chunk) ==
cpu ∧ drain 후 freelist 복귀`.  MPSC queue 의 lock-free 정확성 (Verus
로 어려움 — Kani harness 후보, roadmap 정합).

### Y4 정합도: ★★☆ ((a) 는 generic, (b) 는 Y4 IPC 직결)

---

## 5. Caller ergonomics — ambient CPU + alloc context

### Motivation
현 `alloc(cpu, layout)` 명시 인자 — caller 가 매번 CPU 지정.  kernel/
에서 per-task alloc context 로 wrap 예정 (roadmap).

### 옵션
- (a) **`current_cpu()` ambient** (seL4 측 read): thin wrapper 가 CPU
  자동 주입 → `alloc(layout)`.  단 ambient state = verification 측 추적
  대상 ↑
- (b) **per-task AllocContext struct**: task 가 `{cpu, lease_id, numa}`
  컨텍스트 보유, 그걸 통해 alloc.  §1 (tenant-keyed) 와 자연 묶임
- (c) **`GlobalAlloc` impl**: `#[global_allocator]` 로 Rust `alloc`
  crate 호환.  단 bare-metal no_std 에서 ambient CPU + lease 주입이 trait
  signature (`alloc(Layout)`) 에 안 맞음 → thread_local 또는 per-CPU
  static 필요, Y4 의 명시적 capability 철학과 긴장

### Tradeoff
(b) AllocContext 가 Y4 의 capability 철학 정합 — context = capability
번들.  (c) GlobalAlloc 은 편의 크나 ambient 의존이 TCB/capability 원칙과
충돌 (암묵 전역 상태).  → (b) 채택 + GlobalAlloc 은 신뢰 task 한정 opt-in.

### Verification 영향
(b) AllocContext 가 lease_id 보유 → §1 의 AL-tenant 와 통합 spec.

### Y4 정합도: ★★★ ((b) 가 capability-bundle 철학 직접 구현)

---

## 6. WaveTensor 정합 — accelerator-aware allocation (장기)

### Motivation
Y4 의 존재 이유 = WaveTensor accelerator 호스팅.  accelerator 는
partitioned TLB (4×16) + shadow region (16) + XChaCha20 masking + HBM 을
가짐 (memory: wavetensor_terms).  일반 host allocation 과 accelerator-
visible allocation 은 요구가 다름.

### 옵션 / 방향
- (a) **DMA-safe / pinned pool**: accelerator 가 직접 접근하는 buffer 는
  migration 금지 + physically contiguous.  별도 pool (NUMA-local to
  accelerator)
- (b) **shadow-region-aligned allocation**: shadow region (16개) 경계에
  정렬된 allocation → accelerator 의 partitioned TLB entry 와 1:1
- (c) **masked allocation**: XChaCha20 nonce-bound buffer (lease 의
  192-bit nonce + 256-bit key 와 짝) — allocator 가 마스킹 메타데이터
  보유
- (d) **lease quota integration**: lease capability 가 memory quota
  포함 → allocator 가 lease 별 high-watermark 강제 (§1 tenant-keyed 의
  확장)

### Tradeoff
이건 HIU ABI (`docs/hiu_abi.md` v1.0 frozen) 차단 영역 — Phase B 에서는
mock 뒤 격리 (CLAUDE.md §2).  **brainstorming 만, 구현은 hiu/ unblock
후**.  단 §1 (tenant-keyed) + §5 (AllocContext) 를 지금 lease-aware 하게
설계해두면 (d) quota integration 이 자연 확장됨 — **forward-compat hook**.

### Verification 영향
HIU ABI frozen 후.  현 시점은 placeholder invariant 만.

### Y4 정합도: ★★★ (존재 이유) 단 ⏳ blocked (hiu_abi v1.0)

---

## 7. Verification 강화 — assume() → constructive (adsmt pipeline 활용)

### Motivation
S1/S2/S3/B1/B2/B4/B5 가 여전히 `assume()` at trusted boundary.  방금
land 한 R7.11 emit pipeline (Verus → adsmt cert → Isabelle/Rocq) +
AOT/cross-check 인프라를 alloc 측에도 적용 가능.

### 옵션
- (a) **SLAB lift to Verus** (roadmap stage 2): S1/S2/S3 를 executable
  state model + inductive proof 로 (refinement.rs 의 B3/B6/X3 패턴 복제)
- (b) **Kani harness** (roadmap line 82): lock-free / MPSC 측은 Verus
  가 약함 → Kani model-checking (bounded)
- (c) **AV-style invariant catalog**: alloc 측도 AL1~ALn 카탈로그화 +
  4-cluster (slab / hardened / boundary / tenant) → amdv/power 와 동일
  패턴 → av-proof-body-tracker 에 통합 또는 al-proof-body-tracker 신설
- (d) **cross-check (z3 vs adsmt)** 를 alloc spec 에도 — 방금 land 한
  smt-cross-validation 인프라 재사용

### Tradeoff
(a) + (c) 가 Y4 의 formal-first + AV catalog 패턴 정합.  alloc 의
invariant 를 AV catalog 처럼 first-class 화 → paper artifact 의
self-application evidence 강화.

### Verification 영향
이게 곧 verification 작업 자체.  R7.11 pipeline 의 첫 non-amdv 적용
사례가 될 수 있음.

### Y4 정합도: ★★★ (formal-first + 방금 land 한 emit pipeline 활용)

---

## 8. 우선순위 제안 (brainstorming 결론)

### 즉시 진입 가능 (hiu_abi 무관, verification 인프라 ready)
| 순위 | 항목 | 근거 |
|---|---|---|
| 1 | **§2(a) freelist obfuscation + §2(b) zone quarantine** | overhead ≈ 0, 보안 표면 대부분 (small) cover, 증명 쉬움 |
| 2 | **§1(a) lease-keyed zone** + **§5(b) AllocContext** | Y4 차별점 (capability allocator), §6(d) quota 의 forward-compat hook |
| 3 | **§7(a)+(c) SLAB lift + AL catalog** | formal-first, R7.11 pipeline 첫 비-amdv 적용 |
| 4 | **§4(a) per-CPU MPSC remote-free inbox** | cross-CPU free 실구현, DragonFly 검증 패턴 |

### 후속 / 차단
| 항목 | 차단 사유 |
|---|---|
| §3 size class 재설계 | 실용 이득 크나 Y4 특수성 X — 여유 시 (d) 2-tier |
| §6 WaveTensor 정합 | ⏳ hiu_abi v1.0 frozen 대기 (단 §1+§5 를 lease-aware 설계로 hook) |

### 묶음 관찰
§1 (tenant-keyed) + §5 (AllocContext) + §6(d) (lease quota) + §7(c)
(AL-tenant invariant) 가 **하나의 일관 방향** — "capability-bound,
verification-first allocator".  이게 Y4 allocator 의 정체성이 될 후보.
§2 (hardening) 는 직교 (어느 방향이든 독립 적용).

---

## 다음 단계 (사용자 결정 대기)
- 위 우선순위 중 어느 묶음을 sign-off cycle 로 진입할지
- §1+§5+§7(c) 의 "capability allocator" 방향을 정식 design memo 로 승격할지
- verus-fork R7.11 cross-check 완료 후 §7 을 첫 적용 사례로 삼을지
