<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Pluggable memory allocator front-end + 규약(contract) 설계
created: 2026-06-18T22:15:18+09:00   # KST (UTC+9)
revised: 2026-06-18T22:24:10+09:00   # 사용자 결정 3개 (freelist 분리 / C2 lint+Verus / dyn 보류) 반영
status: brainstorming (일부 결정 — §2.1·2.4·2.6·§4 참조)
scope: Y4/alloc/ front-end 의 교체 가능(pluggable) 아키텍처 + 모든
       front-end 구현체가 지켜야 할 contract.  back-end (scudo/hardened)
       는 논외 (이미 PageBackend trait 로 분리됨)
refs:
  - .brainstormings/20260618-212218-alloc-frontend-improvements.md
    (reference impl = atomic-free SLUB-style SLAB)
  - alloc/src/page_backend.rs (PageBackend trait — back-end pluggability 선례)
  - alloc/src/integrated.rs (현 concrete IntegratedAllocator)
  - README.md §reuse manifest / CLAUDE.md §6 (formal-first / TCB)
  - Linux: SLAB / SLUB / SLOB (CONFIG_SL*B) + kmem_cache contract
---

# Pluggable memory allocator front-end + contract

## 0. 발제 (사용자 2026-06-18)

> Linux 커스텀 커널 패치 중 allocator front-end 교체 사례가 여럿이고,
> 업스트림부터 SLAB/SLUB/SLOB 를 고를 수 있게 해놨다.  Y4 도 front-end
> (back-end 논외) 를 **커스텀 교체 가능**하게 하면 어떨까?  단 Y4 는
> front-end 를 **Rust 로 구현**하니, 모든 구현체 (reference/custom 불문)
> 가 지켜야 할 **규약(contract)** 을 제대로 정하면 개발 편의 면에서
> Linux 대비 훨씬 유리.

**핵심 우위 (사용자 통찰)**: Linux 는 `kmem_cache_*` contract 가 **C
signature + 문서 + 관례로 암묵적** → custom allocator 가 미묘한 계약
위반해도 컴파일 통과, 런타임 버그.  Y4 는 **Rust trait + Verus spec**
으로 contract 를 *명시적·기계검증 가능* 하게 → 위반 시 (a) trait 불일치
= 컴파일 에러, (b) Verus refinement 실패 = merge 차단.  **contract-by-
construction**.

이미 Y4 는 **back-end** 를 `PageBackend` trait 으로 pluggable 하게 분리
(`alloc/src/page_backend.rs`).  본 발제 = 같은 패턴을 **front-end** 로.

---

## 1. Linux 의 현황 + 한계

### Linux allocator front-end 선택
- **SLAB** (Bonwick/Solaris 계열) — per-node queue, cache colouring,
  복잡한 메타.  NUMA 多 노드에서 O(N²) 메타
- **SLUB** (현 default) — unqueued, in-object freelist, cmpxchg_double
- **SLOB** (deprecated/제거됨) — tiny, K&R-style, embedded 용
- `CONFIG_SLAB` / `CONFIG_SLUB` / `CONFIG_SLOB` 으로 **compile-time** 택1
- custom kernel patch 로 front-end 교체 사례 다수 (보안 allocator,
  실험적 allocator 등)

### Linux 의 한계 (= Y4 기회)
- **contract 암묵적** — `kmem_cache_alloc` / `kmem_cache_free` /
  `kmalloc` 의 semantic 계약 (alignment 보장, NULL 반환 조건, GFP flag
  처리, per-cpu freelist 일관성, RCU-free 안전성 …) 이 **C signature +
  Documentation/ + 관례** 로만 존재.  컴파일러가 강제 X
- **위반이 런타임 버그** — custom allocator 가 alignment 미보장 / double-
  free 미검출 / freelist corruption 해도 빌드 통과 → 운영 중 crash/CVE
- **back-end/front-end 경계 모호** — page allocator (buddy) 와 slab 의
  계약도 암묵
- **hardening 제각각** — `SLAB_FREELIST_HARDENED` 등은 SLUB 에만, custom
  은 알아서.  최소 보안 수준 강제 없음

---

## 2. Y4 pluggable front-end 설계

### 2.1 2-tier pluggable trait — `AllocFrontend` + `Freelist` (결정: 분리)

사용자 결정 (2026-06-18): **freelist 도 별도 pluggable trait 로 분리**.
front-end (size-class / zone / hardening 정책) 와 freelist (free slot
추적 전략, improvements §2.9 의 B/D/E) 가 **직교 pluggable** → 조합
가능 (예: SLUB-style front × bitmap freelist, SLOB-style front × index
freelist).

`page_backend.rs` 의 PageBackend (contract 를 doc + 구현 가드로 명시)
패턴을 두 trait 으로 mirror:

```rust
/// (tier 1) free slot 추적 전략 — improvements §2.9 의 B/D/E.
/// front-end 와 직교, slab/zone 내부에서만 동작 (atomic 0, §0.5).
pub trait Freelist {
    /// slab/zone 의 slot 수 + entropy seed (obfusc) 로 초기화.
    fn new(slots: usize, seed: u64) -> Self where Self: Sized;
    /// free slot 하나 획득 (없으면 None).
    fn pop(&mut self) -> Option<usize>;
    /// slot 을 free 로 반납.  이미 free 면 double-free → Err (C3).
    fn push(&mut self, slot: usize) -> Result<(), Y4Error>;
    /// (C3 guard) slot 이 현재 free 인지.
    fn is_free(&self, slot: usize) -> bool;
}
// 구현체: BitmapFreelist (D) / IndexFreelist (B) / HybridFreelist (E)
//        / ArrayFreelist (C).  A(raw pointer) 는 배제 (improvements §2.9).

/// (tier 2) allocator front-end — back-end(PageBackend) 위에 size-class
/// / zone / hardening 을 얹고, slot 추적은 Freelist 에 위임.
pub trait AllocFrontend {
    /// 얹히는 back-end (이미 pluggable).
    type Backend: PageBackend;
    /// 사용하는 freelist 전략 (직교 pluggable).
    type FL: Freelist;

    fn new(backend: Self::Backend, seed: u64) -> Result<Self, Y4Error>
    where Self: Sized;

    /// AllocContext (capability bundle: cpu + lease_id + numa) 로 할당.
    /// (improvements §2.6 AllocContext 정합)
    fn alloc(&mut self, ctx: AllocContext, layout: Layout)
        -> Result<Allocation, Y4Error>;

    /// 동일 ctx 로 해제.  cross-CPU 는 IPC 위임 (improvements §2.1).
    fn free(&mut self, ctx: AllocContext, alloc: Allocation)
        -> Result<(), Y4Error>;

    /// (introspection) layout → size class index, large bypass 면 None.
    fn size_class_of(&self, layout: Layout) -> Option<usize>;
}
```

**분리 이득**:
- freelist 선택 (improvements §2.9 의 측정 게이트) 이 front-end 와 독립
  → bitmap vs hybrid 를 front-end 골격 안 건드리고 교체/측정
- freelist contract (FL1~FL3, §2.3) 를 1회 증명하면 모든 front-end 가
  재사용 → 조합 폭발 (front × freelist) 을 곱셈 아닌 덧셈 검증
- embedded SLOB-style front 가 작은 slab + index freelist, server
  SLUB-style front 가 큰 slab + bitmap freelist 처럼 직교 조합

### 2.2 Contract — 4 계층 규약

모든 구현체 (reference + custom) 가 만족해야 할 contract.  PageBackend
의 doc-contract (reserve 는 page-aligned + multiple, release 는 double-
free→BadCap) 를 front-end 로 확장 + Verus spec 화 (§2.3):

**AllocFrontend contract (C1~C4)**:

| 계층 | 규약 | 강제 수단 |
|---|---|---|
| **C1 functional** | (a) no-overlap: 동시 live allocation 끼리 range disjoint.  (b) alignment: `alloc(l).start` 가 `l.align` 정렬.  (c) round-trip: alloc 한 것만 free 가능 + free 후 그 range 재-vend 안전 | trait signature + **Verus spec** (B1/S3 류) |
| **C2 safety (§0.5)** | atomic-free: hot-path 에 atomic RMW (CAS/cmpxchg/fetch-add) 0.  per-CPU exclusive + critical section | **lint + Verus 둘 다** (결정) — §2.6 |
| **C3 hardening (최소 수준)** | (a) free pointer/index obfuscation, (b) double-free detection, (c) zero-on-free (small).  *최소* 보장 — custom 이 더 강화는 OK, 약화 X.  (a)(b) 는 대개 Freelist (FL) 가 담당 | **Verus spec** (obfusc round-trip, double-free→error) + contract const |
| **C4 boundary** | back-end (PageBackend) 와의 X1 (vend된 page 안에서만 carve) / X2 (error 전파) / X3 (composed no-overlap) | **Verus spec** (boundary.rs 류) |

**Freelist contract (FL1~FL3)** — 별도 trait 이므로 독립 contract:

| 계층 | 규약 | 강제 수단 |
|---|---|---|
| **FL1 round-trip** | `push(s)` 후 그 slot 이 `pop` 가능 + `pop()` 한 slot 은 다시 pop 안 됨 (다음 push 까지).  live slot ∩ free slot = ∅ | **Verus spec** |
| **FL2 double-free** | 이미 free 인 slot 을 `push` → `Err` (`is_free(s)` 가 guard).  improvements §2.9 의 D(bitmap) 면 구조적 공짜 | **Verus spec** (push 시 is_free 검사) |
| **FL3 obfusc** (C3 의 freelist 측) | free slot 표현 (index/pointer) 이 secret 으로 obfuscated, round-trip 항등 | **Verus spec** (`deobf(obf(s,k),k)==s`) |

FL contract 1회 증명 → 모든 AllocFrontend 가 그 Freelist 를 재사용 (조합
검증을 곱셈 → 덧셈).

### 2.3 Verus refinement contract — 기계검증 (★ Linux 대비 결정적 우위)

contract 를 Verus spec (`AllocFrontendSpec`) 으로 1회 작성 → **각 구현체
가 그 spec 을 refine 증명**.  CLAUDE.md §6.6 ("privileged path 는 proof
동반, 없으면 merge X") 정합:

```
proofs/verus/src/alloc/frontend_contract.rs   (AllocFrontend spec — 1회)
  ├─ spec fn c1_no_overlap(state) -> bool
  ├─ spec fn c1_aligned(alloc, layout) -> bool
  └─ spec fn c4_boundary_*(...) -> bool

proofs/verus/src/alloc/freelist_contract.rs    (Freelist spec — 1회)
  ├─ spec fn fl1_roundtrip(...) -> bool
  ├─ spec fn fl2_double_free_rejected(...) -> bool
  └─ spec fn fl3_obfusc_roundtrip(...) -> bool

proofs/verus/src/alloc/freelist/<fl_name>.rs   (per-freelist 증명)
  proof fn <fl>_satisfies_contract() ensures FL1~FL3 { ... }
    # BitmapFreelist / IndexFreelist / HybridFreelist / ArrayFreelist

proofs/verus/src/alloc/frontend/<impl_name>.rs (per-front-end 증명)
  proof fn <impl>_satisfies_contract() ensures C1~C4 { ... }
    # FL contract 는 type FL: Freelist 의 증명을 재사용 (덧셈 검증)
```

- **reference impl** (atomic-free SLUB-style, improvements.md) 가 첫 증명
- **custom impl** 은 같은 `<impl>_satisfies_contract()` 증명 없으면
  merge gate 에서 차단
- → "front-end 를 바꿔도 contract 만족이 *기계검증* 되므로 back-end
  (scudo) 와 안전 조합 보장"

### 2.4 선택 메커니즘 — compile-time monomorphization

| 옵션 | 의미 | 판정 |
|---|---|---|
| **(a) generic param** `Kernel<F: AllocFrontend>` | compile-time 택1, monomorphized, vtable 0 | ✅ **우선** — no_std + deterministic + Verus per-impl 정합 |
| (b) feature flag `--features slub-style\|slob-style` | compile-time, cargo feature | ✅ (a) 의 cargo 표면, form-factor build matrix (`boot/<ff>.rules`) 정합 |
| (c) runtime `dyn AllocFrontend` | vtable dispatch, 런타임 교체 | ⚠️ **통째 배제는 보류** — generic 으로 표현 불가능한 경우에만 |

**채택 방향 (결정 2026-06-18)**: **"generic 으로도 표현 가능한 경우엔
generic"** 원칙.
- (a) generic + (b) feature 를 **기본** — compile-time 으로 표현 가능한
  모든 경우 generic 강제 (런타임 비용 0 + Verus per-impl + deterministic).
- (c) runtime dyn 의 **통째 배제는 섣부르므로 보류** — generic 으로
  표현 *불가능* 한 경우 (예: 부팅 시 form-factor probe 결과로 allocator
  를 고르는데 그게 compile-time 에 안 정해지는 시나리오) 에 한해 dyn 을
  열어둘 여지.  단 그런 경우에도 (a) 가 가능하면 (a) 우선.
- 즉 **generic-first, dyn-only-when-必要** — Linux `CONFIG_SL*B` 의
  compile-time 결을 기본으로 하되 runtime 가능성을 못박지 않음.

### 2.5 Reference + custom impl (form-factor 별)

- **reference** = atomic-free SLUB-style SLAB (improvements.md 의 결과)
  — 모든 form-factor 의 default
- **custom 예시**:
  - **embedded SoC/SoM** = SLOB-style (tiny, 단순, 메모리 최소) custom
    — slot 수 적으니 freelist D(bitmap) scan 비용 무시
  - **server-farm host** = reference SLUB-style (throughput + hardening)
  - **handheld** = reference + 전력 인지 (power_arch 정합)
- 각 custom 이 §2.3 의 `<impl>_satisfies_contract()` 증명 동반 → 안전
  교체 보장

### 2.6 Cross-cutting 원칙을 contract 에 강제하는 법

§0.5 (atomic-free) 같은 원칙은 trait type 으로 직접 강제 어려움 (Rust
type system 이 "atomic 안 씀" 을 표현 못함).  **결정 (2026-06-18):
lint + Verus 간접 증거 둘 다** (단일 수단에 의존 X):
- **lint** (1차 방어) — `clippy.toml` / custom lint 로 `core::sync::
  atomic` import 를 front-end + freelist crate 에서 deny (improvements
  §2.4 의 unsafe+proof lint 패턴 정합).  컴파일 시점 즉시 차단
- **Verus 간접 증거** (2차/심층) — atomic 을 쓰면 sequential refinement
  증명이 성립 불가 → C1~C4 / FL1~FL3 증명이 통과한다는 사실 자체가
  atomic-free 의 구조적 증거.  lint 가 놓친 우회 (예: inline asm 의
  `lock` prefix) 도 증명 단계에서 sequential 모델과 불일치로 노출
- 둘 다: lint = 빠른 1차 grep-level 차단, Verus = 의미 수준 보증.
  중첩 방어 (lint 통과해도 증명 실패 가능, 역도 성립) → robust
- **contract const** (보조) — C3 최소 수준을 type-level 상수로 (hardening
  약화 방지)

---

## 3. Linux 대비 Y4 우위 (contract-by-construction)

| 측면 | Linux (SLAB/SLUB/SLOB) | Y4 (AllocFrontend trait) |
|---|---|---|
| 선택 | `CONFIG_SL*B` compile-time | generic-first (param + feature), dyn 보류 |
| **trait 입자** | front-end 단일 (slab) | **2-tier** (AllocFrontend + Freelist 직교 pluggable) |
| 런타임 비용 | 0 (compile-time) | 0 (monomorphization) |
| **contract 표현** | C signature + 문서 + 관례 (암묵) | **trait + Verus spec (명시)** |
| **위반 검출 시점** | 런타임 (crash/CVE) | **컴파일 (trait) + 증명 (Verus), merge 전** |
| custom 안전성 | 개발자 책임 (계약 위반 가능) | **contract-by-construction** (증명 없으면 merge X) |
| hardening 보장 | 구현마다 제각각 | **C3 최소 수준 강제** |
| back/front 경계 | 암묵 | PageBackend / AllocFrontend trait 로 명시 + C4 Verus |
| atomic-free 보장 | 없음 (구현 자유) | §0.5 lint + Verus 간접 증거 |

**요지**: Linux 의 pluggability(좋음) + Rust trait safety + Verus formal
contract = "**바꿔도 안전이 기계검증되는** front-end pluggability".
개발 편의 (사용자 발제) = custom 작성자가 contract spec 만 refine 하면
나머지 (back-end 조합 안전성, hardening 최소선, no-overlap) 가 자동 보장
→ Linux 처럼 미묘한 계약을 일일이 손으로 지킬 필요 X.

---

## 4. 결정 / 미결

### 확정 (사용자 2026-06-18)
- **2-tier pluggable**: `AllocFrontend` (tier 2) + `Freelist` (tier 1)
  별도 trait 로 분리.  직교 조합 (front × freelist), FL contract 재사용
  으로 조합 검증을 덧셈으로
- **C2 atomic-free 강제 = lint + Verus 간접 증거 둘 다** (중첩 방어)
- **선택 = generic-first** — compile-time 으로 표현 가능하면 generic
  (a)+(b) 강제.  runtime dyn (c) **통째 배제는 보류** (generic 불가능한
  경우에만 열어둠)
- contract 위반 = merge gate (CLAUDE.md §6.6)
- reference (atomic-free SLUB-style, improvements.md) + form-factor custom

### 미결
- **C3 hardening 최소 수준의 정확한 set** — obfusc + double-free +
  zero-on-free 가 minimum?  form-factor 별 차등 허용 범위
- **AllocContext 통합** — improvements §2.6 의 AllocContext 가 trait
  signature 에 들어가는 형태 확정
- **PageBackend / Freelist 결합 방식** — associated type (`type
  Backend` / `type FL`) vs generic param.  associated type 이 Verus
  per-impl 증명엔 깔끔하나, 한 front-end 가 여러 freelist 와 조합되려면
  generic 이 유연 — 측정 게이트 (improvements §2.9) 가 조합 비교를
  원하면 generic param 이 나을 수도
- **dyn 이 정말 필요한 경계 사례** — generic 불가능한 부팅-시-probe
  시나리오가 실재하는지 (Phase C form-factor 작업에서 판명)

### 후속 행동
- improvements.md 의 reference impl 설계 = `AllocFrontend` 첫 구현체 +
  improvements §2.9 의 freelist (D/E) = `Freelist` 첫 구현체 → 두
  brainstorming 이 한 design memo 로 수렴 ("**pluggable (2-tier),
  atomic-free, Verus-verified allocator front-end + freelist**")
- R7.11 emit pipeline 으로 contract spec (C1~C4 + FL1~FL3) 의 Isabelle/
  Rocq emit (seL4 inbound) — front-end/freelist 교체해도 contract 고정
  이라 seL4 측 1회 검토로 충분 (pluggability 가 검증 비용 ↑ X)
