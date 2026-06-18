<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Memory allocator front-end 개선 brainstorming
created: 2026-06-18T21:22:18+09:00   # KST (UTC+9)
revised: 2026-06-18T21:57:07+09:00   # frame 정정 (§0) + SLUB §2.8 + atomic-free 불가침 원칙 §0.5 (사용자 제약)
status: brainstorming (결정 X — 옵션 탐색)
scope: Y4/alloc/ front-end — "DragonFly SLAB algorithm port" 라는 최종
       구현 목표를 baseline 으로, 그것을 어떻게 더 개선/발전시킬지
refs:
  - README.md §reuse manifest ("DragonFlyBSD lock-free SLAB — alloc/src/slab.rs")
  - /home/ybi/y4-upstream-refs/dragonfly/sys/kern/{kern_slaballoc.c,kern_objcache.c}
  - docs/architecture.md §Memory allocator (DragonFly SLAB + scudo fusion)
  - CLAUDE.md §6 (TCB minimization / capability / formal-first)
---

# Memory allocator front-end 개선 brainstorming

## 0. Frame 정정 (revised 2026-06-18 21:43 KST)

초안은 **현재 scaffold (`slab.rs` subset) 의 gap-fill** 로 frame 을 잘못
잡았다 (jemalloc-style size class 재설계 등 *다른 allocator* 를 섞음).

올바른 frame (사용자 정정 2회):
1. baseline = **현재 코드가 아니라 최종 구현 목표** — README 가 못박은
   "DragonFlyBSD lock-free SLAB 의 algorithm port" 의 *완성형*.
2. 주제 = 그 DragonFly SLAB 을 **어떻게 더 개선/발전시킬지**.

→ 즉 DragonFly SLAB (2003년대 production kernel allocator) 을 그대로
베끼는 게 아니라, 그 골격을 **2026년 allocator 연구 (mimalloc / snmalloc
/ scudo / Hardened malloc) + Y4 특수 요구 (formal-first / capability /
deterministic latency / WaveTensor)** 로 넘어서는 것.

## 0.5 불가침 원칙 — atomic-operation 배제 (사용자 제약 2026-06-18 21:57)

> **모든 개선/차용은 이 원칙을 위배할 수 없다.**

DragonFly SLAB 의 핵심 철학 = **hot-path 에서 atomic RMW (CAS /
cmpxchg / fetch-add) 를 쓰지 않는다**.  per-CPU 데이터는 그 CPU 만
만지므로 (critical section = preemption/interrupt disable 로 보호)
atomic 이 원천적으로 불필요.  Y4 는 이 원칙을 **불가침**으로 유지한다.

**차용 규칙**:
- SLUB / mimalloc / snmalloc 에서 **데이터 구조**는 가져온다 (unqueued
  per-CPU active slab + partial list, metadata-in-page descriptor).
- 그들의 **atomic 동기화 메커니즘**은 **배제**한다 — SLUB 의
  `cmpxchg_double` fast path, cross-CPU atomic free list (DragonFly
  `z_RChunks` CAS 포함), mimalloc 의 atomic thread-free, snmalloc 의
  atomic message ring 모두 X.
- cross-CPU free 는 allocator *안*에서 atomic 으로 풀지 않는다 — 거부
  (caller 책임) 또는 Y4 IPC layer 위임 (§2.1).

**왜 불가침인가 — 3중 이득**:
1. **deterministic latency** (README 핵심 가치) — atomic RMW 의 cache-
   line contention / retry-loop jitter 가 0.  worst-case bound 가 명확.
2. **formal verification 극적 단순화** — atomic 이 없으면 lock-free
   linearizability / memory-ordering (acquire/release) 추론이 통째로
   사라짐.  per-CPU = **sequential reasoning** → **Verus 만으로 충분**
   (§2.4 의 Kani 의존이 사라짐).  이게 atomic-free ↔ formal-first 의
   시너지.
3. **TCB minimization** — 동기화를 allocator 가 떠안지 않고 critical
   section (이미 신뢰) + IPC (이미 검증) 에 위임 → allocator 의 trust
   surface 축소.

이 원칙이 아래 모든 §의 frame 을 재조정한다 (특히 §2.1 / §2.4 / §2.8).

---

## 1. DragonFly SLAB full algorithm (= 개선의 baseline)

upstream (`y4-upstream-refs/dragonfly/sys/kern/kern_slaballoc.c` ~1873L
+ `kern_objcache.c`) 의 완성형:

| 요소 | DragonFly 구현 |
|---|---|
| zone 구조 | **SLZone** in-band header (zone 메모리 시작에 배치), per-(CPU,class) |
| size class | 72~80개, 8-byte spacing (small) → 가변 간격 (8B~16KB) |
| local free | critical section 만 (lock-free), `z_LChunks` push/pop |
| **remote free** | atomic CAS 로 `z_RChunks` 링크 + `z_RSignal` 확인 후 **IPI** → owner 가 `kfree_remote()` 로 drain.  `z_RCount` 로 in-flight IPI 추적 (zone 파괴 race 방지) |
| **magazine/depot** | objcache 의 2-tier — per-CPU `loaded`+`previous` magazine + spinlock-protected `magazinedepot` (`full`/`empty` 리스트) |
| zone lifecycle | `VM_ALLOC` reserve → carve.  빈 zone → `FreeZones` → 주기적 poller 가 `ZONE_RELS_THRESH` 초과분 VM 반납 |
| metadata | **in-band** (zone 내부), `btokup()` 로 page→zone 역매핑 (kup map) |
| bigalloc | `ZoneLimit` 초과 → kmem 직접 (slab 우회) |

**DragonFly 의 설계 우선순위**: 처리량 (kernel hot-path) > 보안/검증.
2003년 단일-tenant kernel 가정.  이 가정들이 곧 개선 여지.

---

## 2. 개선 축 (DragonFly baseline → Y4 final target)

### 2.1 Remote-free: atomic 배제 → IPC 위임 / 거부 (★★★ deterministic)

> §0.5 불가침 원칙 적용 — allocator 안에서 atomic 으로 cross-CPU free
> 를 풀지 않는다.

**DragonFly**: remote free 가 **atomic CAS** (`z_RChunks`) + **IPI
signal**.  atomic + IPI 둘 다 — §0.5 위배.

**초안의 오류**: 내가 처음 제시한 "lock-free MPSC (atomic CAS push)"
도 atomic 이라 §0.5 위배.  폐기.

**§0.5-정합 옵션**:
- (a) **거부 + caller 라우팅** (현 slab.rs) — cross-CPU free = `InvalidArg`.
  caller 가 owner CPU context 에서 재시도.  allocator atomic 0, 가장
  단순, 검증 가장 쉬움.  단 caller 부담
- (b) **Y4 IPC msgport 위임** — free 요청을 owner CPU 의 msgport 로 send
  (`ipc/msgport.rs`).  동기화는 **IPC layer 의 책임** (이미 검증된 layer)
  → allocator 자체는 atomic-free 유지.  owner 가 자기 msgport drain 후
  local free (critical section).  Y4 의 IPC-first 구조 정합 + TCB 분리
- (c) **per-CPU deferred-out buffer** — remote 로 보낼 chunk 를 src CPU
  의 local 버퍼에 쌓아 두고, scheduler/IPC 가 batch 로 owner 에 전달.
  hot-path atomic 0

**Y4 정합 — (b) IPC 위임 확정 (사용자 선호 2026-06-18)**: cross-CPU free
의 동기화 복잡도를 allocator 밖 (검증된 IPC) 으로 밀어냄.  deterministic
latency (atomic contention 0) + TCB minimization + atomic-free 모두
만족.  IPI 의 jitter 도 IPC layer 의 정책 (batch / wave-aligned) 으로
흡수.  (a) 거부는 caller 부담 + cross-CPU 패턴 흔함 → (b) 채택.  설계
세부 (msgport verb / free-request 메시지 포맷 / owner drain timing) 는
후속 — `ipc/msgport.rs` 와 짝.

**Verification 영향 (§0.5 의 이득)**: cross-CPU free 가 allocator 의
atomic 이 아니라 **IPC message** → allocator 측은 per-CPU sequential
reasoning 만.  lock-free linearizability 증명 불필요 → **Verus
constructive 로 충분** (Kani 불필요).  IPC 의 정확성은 `ipc/` 의 기존
refinement proof 가 담당 (관심사 분리).

---

### 2.2 Metadata: in-band → out-of-band (★★★ 보안 + locality)

**DragonFly**: SLZone header 가 zone 메모리 **in-band**.  heap overflow
가 **metadata 오염** 가능 (freelist 포인터 조작 → arbitrary write).
2003년엔 kernel-only 라 위협 모델 밖.

**개선**: snmalloc / mimalloc-secure 식 **out-of-band metadata** —
slab page 는 데이터만, zone header/freelist 는 별도 metadata chunk
(page-aligned, 데이터와 분리).  heap overflow 가 metadata 못 건드림 +
cache pollution 감소 (metadata 가 hot data line 안 침범).

**Y4 정합**: TCB 안 allocator 의 robustness.  tenant 가 자기 heap
overflow 로 allocator metadata 오염 → cross-tenant 영향 = 차단해야 할
위협.  out-of-band 가 이 표면 제거.

**Verification 영향**: metadata ⊥ data 분리가 invariant 로 명확 (`∀
zone z, metadata(z) ∩ data_pages(z) == ∅`) → S1 류 disjointness 와 동족,
constructive 쉬움.

---

### 2.3 Hardening 융합: DragonFly 보안 0 → scudo-grade front (★★★)

**DragonFly**: SLAB front 는 hardening **0** (성능 위주).  Y4 는 backend
(scudo/hardened, B1~B6) 만 hardened, front (SLAB small path) 는 무방비 —
그런데 allocation 대부분이 small.

**개선** (scudo / Hardened-malloc / mimalloc-secure 기법을 front 에):
- **freelist pointer obfuscation** — free chunk 의 next 를 per-zone
  secret 과 XOR/rotate.  freelist corruption → arbitrary write 차단.
  overhead ≈ 0
- **zero-on-free** (small) — UAF 시 stale data 누출 0
- **double-free detection** — freelist 에 이미 있는 chunk 재-free 거부
  (in-band bitmap 또는 out-of-band, §2.2 와 묶임)
- **delayed reuse (zone-local quarantine)** — small UAF window 축소
  (hardened B6 의 small mirror)
- **slab guard page** — zone 경계 (page 단위, per-alloc 아님) overflow 잡음

**Y4 정합**: front+back **일관 hardening** — DragonFly 가 안 한 부분을
Y4 가 scudo 수준으로 끌어올림.  TCB 안 allocator 의 책임.

**Verification 영향**: obfuscation 은 자명 (`deobf(obf(p,s),s)==p`).
double-free / quarantine 은 B6 패턴 (refinement.rs) constructive.

---

### 2.4 Formally-verified lock-free SLAB (★★★ 세계 최초급 차별점)

**DragonFly**: C, 검증 **0** (race / UAF 는 코드 리뷰 + 운영 경험 의존).

**개선**: lock-free hot-path 까지 **formally verified** SLAB.  방금 land
한 R7.11 emit pipeline (Verus → adsmt cert → Isabelle/Rocq) 의 **첫 비-
amdv 적용 사례**.  S1/S2/S3 + remote-free (§2.1) + metadata 분리 (§2.2)
의 invariant 를 constructive proof 로.

**Y4 정합**: formal-first (CLAUDE.md §6.6) + verified seL4 base 위.
"formally-verified production-grade lock-free SLAB" = paper artifact
§6.1 의 강력한 차별점 (DragonFly/jemalloc/mimalloc 누구도 안 함).

**§0.5 의 이득 — Verus 만으로 완결**: atomic-free (§0.5) 덕분에 allocator
hot-path 가 **per-CPU sequential** → lock-free linearizability / memory-
ordering 추론이 *원천적으로 불필요*.  로드맵의 "Kani harness 필요"
(lock-free 영역) 항목이 **사라짐** — S1/S2/S3 + freelist disjointness +
obfusc round-trip + remote-via-IPC 경계 모두 Verus constructive 사정권.
이게 atomic-free ↔ formal-first 의 시너지 (DragonFly/SLUB 의 atomic
설계였으면 불가능).

**Verification 작업**: alloc 측 invariant 를 **AL catalog** (AL1~ALn,
4-cluster: slab / hardened / boundary / remote-via-IPC) 로 first-class
화 → amdv 의 AV catalog 패턴 정합, av-proof-body-tracker 에 통합 또는
al-proof-body-tracker 신설.  R7.11 emit pipeline (Verus → adsmt cert →
Isabelle/Rocq) 의 첫 비-amdv 적용 사례.

---

### 2.5 Deterministic latency: background drain → bounded worst-case (★★★)

**DragonFly**: zone drain 이 주기적 **poller thread** (background) +
magazine refill 이 hot-path 에서 일어날 수 있음 → **tail latency spike**.
throughput 최적이나 worst-case 비결정적.

**개선**:
- **bounded worst-case alloc** — refill 도 bounded step (amortize X,
  worst-case 보장).  Z_MAX 류 상한이 이미 그 방향
- **wave-aligned drain** — zone 회수 / magazine drain 을 lease dispatch
  중 금지, wave 경계의 idle slot 에만 (lease scheduler 와 협조)
- **no lazy refill on dispatch path** — accelerator dispatch 직전
  allocation 은 pre-warmed pool 에서만

**Y4 정합**: README 핵심 가치 = deterministic latency.  DragonFly 의
throughput-우선 drain 정책을 Y4 는 latency-우선으로 재조정 → 이게 Y4 의
allocator 가 DragonFly 와 갈라지는 지점.

**Verification 영향**: worst-case bound 를 spec 으로 (`alloc_terminates_
within(N_steps)`) — amdv 의 AV3 `vmrun_terminates_within` 과 동족 패턴.

---

### 2.6 Capability / tenant layer — DragonFly 에 없는 Y4 추가 (★★★)

**DragonFly**: single-tenant kernel allocator.  tenant 개념 없음.

**개선** (DragonFly 위에 Y4 가 얹는 층):
- **lease-keyed zone** — zone key 에 `lease_id` 추가.  같은 size class
  라도 tenant 분리 → UAF / 정보 누출이 tenant 경계 안에 갇힘
- **per-lease quota** — lease capability 가 memory high-watermark 보유,
  allocator 가 강제 (`docs/lease_capability.md` 와 짝)
- **AllocContext** — task 가 `{cpu, lease_id, numa}` capability 번들
  보유, ambient CPU/lease 주입 (명시적 capability 철학 정합)

**Y4 정합**: tenant isolation (§6.2) 의 allocator-level 구현.  "capability-
bound allocator" = DragonFly 에 없는 Y4 고유 발전.  단 front-end *자체*
의 개선이라기보다 그 위 layer — §2.1~2.5 와 직교.

**Verification 영향**: **AL-tenant** — `∀ chunk c (lease L), ∀ realloc,
owner ∈ {L} ∪ released(L)`.  S1 의 tenant 확장.

---

### 2.7 Accelerator-aware pool — WaveTensor 정합 (★★☆ ⏳ blocked)

**DragonFly**: NUMA-aware, 단 accelerator 개념 없음.

**개선 방향** (⏳ `docs/hiu_abi.md` v1.0 frozen 대기):
- **DMA-pinned pool** — accelerator 직접 접근 buffer = migration 금지 +
  physically contiguous, NUMA-local to accelerator (HBM)
- **shadow-region-aligned** — shadow region (16개) 경계 정렬 → partitioned
  TLB (4×16) entry 와 1:1
- **masked allocation** — XChaCha20 nonce-bound (lease 192-bit nonce +
  256-bit key 와 짝)

**Y4 정합**: 존재 이유.  단 hiu_abi 차단 — 지금은 §2.6 (lease-aware) 를
hook 으로 설계해두면 forward-compat.

---

### 2.8 Linux SLAB → SLUB 차용 (★★★ — 사용자 제안 2026-06-18)

DragonFly SLAB(2003) 현대화의 **교과서적 사례**.  Linux 는 2007년
원조 SLAB(Bonwick/Solaris 계열) → SLUB(Christoph Lameter) 으로 전환,
정확히 "어떻게 더 개선" 의 답.  upstream Linux 소스는 y4-upstream-refs
부재 → 일반 지식 기준.

**SLAB → SLUB 전환의 핵심 교훈**:
1. **queue / 메타데이터 단순화** — SLAB 의 per-CPU `array_cache` + per-
   node `shared` / `alien` queue (NUMA 노드 N개 → O(N²) 메타) 제거.
   SLUB = per-CPU **active slab** 하나 + per-CPU/node **partial list**.
2. **in-object freelist** — free pointer 를 *free 된 객체 안에* 임베드
   (별도 freelist 배열/구조 X).  live 객체엔 영향 0 (데이터 영역 재사용).
3. **page-struct 메타데이터** — slab descriptor 를 `struct page` 필드에
   접어넣어 별도 관리구조 최소.
4. **`cmpxchg_double` lock-free fast path** — `cpu_slab` 의 `(freelist,
   tid)` 쌍을 한 atomic 으로 → lock 없는 alloc/free.
5. **slab merging** — 호환 size/flags cache 통합 (메타 + TLB 절감).
6. **`SLUB_DEBUG` runtime toggle** — redzone / poison / tracking 을
   build-time 고정이 아니라 **부팅 파라미터로 toggle**.

**Y4 (DragonFly baseline) 차용 매핑 — §0.5 (atomic 배제) 필터 적용**:

| SLUB 아이디어 | Y4 | §0.5 판정 |
|---|---|---|
| **unqueued** (per-CPU active slab + partial list) | ✅ **가져옴** — DragonFly objcache magazine/depot 2-tier 대신.  데이터 구조라 atomic 무관 | OK |
| **metadata-in-page** (descriptor 를 page/frame 메타에) | ✅ **가져옴** — §2.2 out-of-band descriptor 와 정합 (object data 와 분리) | OK |
| slab merging (size class 통합) | ✅ 가져옴 (데이터 구조) | OK |
| `SLUB_DEBUG` runtime toggle | ✅ 가져옴 — §2.3 hardening 을 form-factor 별 toggle (`boot/<ff>.rules`) | OK |
| **`cmpxchg_double` fast path** | ❌ **배제** — atomic RMW.  per-CPU exclusive + critical section 으로 대체 (DragonFly 원칙) | §0.5 위배 |
| **cross-CPU atomic free** | ❌ **배제** — §2.1 의 IPC 위임 (b) 로 | §0.5 위배 |

핵심: SLUB 의 **데이터 구조 (unqueued + metadata-in-page) 만 가져오고,
동기화 메커니즘 (cmpxchg/atomic) 은 배제**.  per-CPU active slab 은 그
CPU 만 만지므로 — SLUB 이 cmpxchg_double 을 쓰는 이유 (preemption +
remote 접근) 가 Y4 의 strict per-CPU exclusive + cross-CPU 배제 (§2.1)
하에선 *애초에 발생 안 함* → plain read/write (critical section) 로 충분.
즉 **SLUB 의 atomic 은 Y4 에선 불필요**, 데이터 구조 이득만 순수 취득.

**metadata-in-page ↔ §2.2 out-of-band 의 화해**: SLUB 의 metadata-in-
page (descriptor 를 struct page 에) 는 사실 *object data 와 분리* 라는
점에서 §2.2 out-of-band 와 같은 방향.  Y4 엔 struct page 가 없지만
(seL4 frame) → frame-keyed descriptor table (out-of-band) 로 동형.
heap overflow 가 descriptor 못 건드림.  ✅ 긴장 없음.

**남은 미결 — freelist (→ §2.9)**: SLUB 의 *in-object freelist* 만은
별도 판단 필요.  atomic 과 무관 (per-CPU exclusive 면 plain pointer 면
충분) 하나, (a) 보안 (UAF/overflow 가 freelist 조작 — Linux FREELIST_
HARDENED 가 막은 것) + (b) Verus 검증 난이도 (raw pointer aliasing) 의
두 축이 남음.  §2.9 에서 옵션 정리.

**§0.5 의 verification 이득 재확인**: cmpxchg/atomic 배제 →  §2.4 의
"Kani 필요" 가 사라짐.  남은 freelist 의 검증 난이도도 *lock-free* 가
아니라 *raw pointer aliasing* (sequential) 문제로 축소 → Verus 사정권
(§2.9 의 index-based 옵션이면 더 쉬움).

**결론**: SLUB 차용 = **데이터 구조 (unqueued + metadata-in-page) + 
runtime-toggle hardening 만 취하고 atomic 동기화는 전량 배제**.  이게
§2.1 (IPC 위임) + §2.2 (out-of-band) + §2.3 (hardening) + §2.5 (partial
list → refill 완화) 를 하나로 묶되, §0.5 (atomic-free) 를 지켜 §2.4
(Verus-only verification) 까지 보존하는 통합 frame.  유일한 미결 =
freelist 표현 (§2.9).

---

### 2.9 freelist 표현 — 미결 (사용자 2026-06-18 "생각해봐야")

> §0.5 (atomic 배제) + per-CPU exclusive 전제.  따라서 freelist 의
> *동시성* 문제는 없음 (그 CPU 만 만짐, critical section).  선택축은
> **(i) 보안 / (ii) Verus 친화 / (iii) double-free / (iv) locality·메모리**
> 4개로 재편 (lock-free 난이도 축은 §0.5 로 소멸).

| 옵션 | 딱지 | 구조 | (i) 보안 | (ii) Verus | (iii) double-free | (iv) loc/mem |
|---|---|---|---|---|---|---|
| A. in-object raw pointer (SLUB 원형) | ❌ **배제** | free 객체 안 next = 포인터 | ✗ overflow→포인터 조작 (obfusc 필수) | ✗ raw pointer aliasing/provenance 추론 난 | 별도 검사 | ◎ 최고 |
| B. in-object index | 후보 | free 객체 안 next = slab-relative idx (u16/32), addr=base+idx·stride | △ obfusc=idx^secret (간단) + bounds | ◎ idx=bounded int, aliasing 회피 | 별도 (or bitmap) | ○ locality 좋음, 메모리 절감 |
| C. out-of-band 배열/스택 (현 `heapless::Vec`) | 후보 | descriptor 옆 idx 배열 | ○ 객체 밖 (overflow 격리) | ◎ Vec spec 쉬움 | 별도 | ✗ Z_MAX 상한 + overhead |
| **D. out-of-band bitmap** (metadata-in-page 정합) | ⭐ **유력** | descriptor 의 slab당 bitmap (free=1) | ◎ 객체 밖 | ◎ bit-vector invariant 쉬움 | **◎ 공짜** (bit 이미 set=double-free) | ○ 1bit/slot, alloc 시 first-free scan |
| **E. hybrid B+D** | ⭐ **유력** | in-object idx list (속도) + bitmap (double-free guard) | ◎ | ○ | ◎ | ○ 메타 2벌 |

**관찰**:
- **A (SLUB 원형) 는 배제 확정** — §2.4 (Verus-only) 의 가장 큰 적이
  raw pointer aliasing.  atomic 은 §0.5 로 없앴는데 pointer provenance
  로 다시 어려워지면 본말전도.
- **D (bitmap) ⭐ 유력** — metadata-in-page (§2.8) 와 가장 정합 +
  **double-free 가 구조적으로 공짜** + Verus 가장 쉬움.  단 alloc 시
  first-free scan (small slab=1 word, 무시 가능).
- **E (hybrid) ⭐ 유력** — D 의 double-free·검증 이득 + in-object index
  list (B) 의 속도.  scan cost 가 문제일 때의 대안.  메타 2벌 비용.
- B / C 는 후보 잔류 (측정 결과가 D/E 를 뒤집을 경우).

**결정 = 측정 게이트 (사용자 2026-06-18)**: 최종 선택은 **size class
별 slot 수 → scan cost 측정 후**.  bitmap first-free scan 이 큰 slab
(슬롯 多) 에서 부담이면 D → E (in-object list 앞세움) 또는 B 로 이동.
측정 전까지 **D / E 가 유력 후보**, A 배제, B / C 보류.

> **측정 protocol (후속)**: size class 별 (slot 수, slab 크기) 조합에서
> bitmap first-free scan 의 worst-case word 수 + alloc/free latency 를
> microbench (P-redesign.6 의 smt-cross-validation 인프라 / `just`
> bench recipe 재사용).  scan worst-case 가 deterministic latency
> (§2.5) bound 안인지가 D vs E 의 분기점.

---

## 3. 묶음 + 우선순위 (brainstorming 결론)

### 방향성 2축
- **축 A — DragonFly SLAB 의 SLUB-style 현대화** (front-end core): §2.8
  (SLUB 차용) 이 §2.1 (lock-free cmpxchg) + §2.2 (out-of-band descriptor)
  + §2.3 (in-object obfuscated freelist hardening) + §2.5 (partial list
  → refill spike 완화) 를 **하나의 통합 설계**로 묶음.  "DragonFly objcache
  골격 → SLUB 식 단순화 + scudo-grade hardening" = 2003 → 2026 현대화.
- **축 B — Y4 고유 layer** (core 위): §2.4 (formal verification) + §2.6
  (capability/tenant) + §2.7 (accelerator).  DragonFly / SLUB / mimalloc
  누구도 안 한 Y4 차별점.

### 추천 진입 순서
| 순위 | 항목 | 근거 |
|---|---|---|
| 1 | §2.3 hardening (obfusc + zero-on-free + double-free) | overhead ≈ 0, small 표면 cover, 증명 쉬움, hiu 무관 |
| 2 | §2.8 SLUB 데이터 구조 (unqueued + metadata-in-page) + §2.2 out-of-band descriptor | core 골격.  atomic 동기화 배제 (§0.5), 데이터 구조만 |
| 3 | **§2.9 freelist** — D(bitmap) / E(hybrid) ⭐유력, A 배제, B/C 보류.  **측정 게이트** (size class별 slot 수 → scan cost) | core 핵심 미결.  D 면 §2.3 double-free 공짜.  측정 후 확정 |
| 4 | §2.4 formal verification (AL catalog, **Verus-only**) | R7.11 첫 비-amdv 적용.  §0.5 덕에 Kani 불필요 |
| 5 | §2.1 cross-CPU free = **IPC 위임 (b)** (사용자 확정) | atomic-free, `ipc/msgport.rs` 정합, TCB 분리 |
| 6 | §2.5 deterministic latency | lease scheduler (Phase C) 협조 — 일부 ⏳ |
| 7 | §2.6 capability/tenant | Y4 차별점, §2.7 forward-compat hook |
| 8 | §2.7 accelerator pool | ⏳ hiu_abi v1.0 frozen |

### 핵심 관찰
- **§0.5 (atomic 배제) 가 전체 frame 의 척추** — SLUB/mimalloc/snmalloc
  에서 **데이터 구조만** 취하고 **atomic 동기화는 전량 배제**.  이게
  세 가지를 동시에 확보: deterministic latency / Verus-only verification
  (Kani 불필요) / TCB minimization.
- **§2.8 (SLUB) = 축 A 의 통합 frame, 단 §0.5 필터링됨** — unqueued +
  metadata-in-page (데이터 구조) ✅, cmpxchg_double + cross-CPU atomic
  free ❌.  Linux SLAB→SLUB 가 검증한 *구조* 를 따르되, Y4 는 (a)
  FREELIST_HARDENED 처음부터, (b) descriptor out-of-band split, (c)
  atomic-free + formal verify — Linux 가 못/안 한 3가지.
- §2.4 (Verus-only) + §2.8 (SLUB 구조) + §0.5 (atomic-free) 묶음 =
  **"atomic-free, formally-verified (Verus-only), hardened, SLUB-style
  SLAB"** = Y4 allocator 정체성 + paper §6.1 차별점 (누구도 production
  SLAB 을 atomic-free + Verus-verify + scudo-harden 안 함).
- §2.5 (deterministic) 가 DragonFly/Linux 와 Y4 가 *철학적으로* 갈라지는
  지점 (throughput-우선 → latency-우선).  §0.5 (atomic-free) 가 이를
  강제 — atomic contention jitter 가 원천 차단.
- **남은 핵심 결정 = §2.9 freelist 표현** — atomic 을 §0.5 로 없애고
  cross-CPU 를 IPC 로 위임 (§2.1) 하고 나니, trade-off 가 "SLUB↔Kani"
  에서 **"freelist 를 B(index)/D(bitmap)/E(hybrid) 중 무엇으로"** 로
  좁혀짐.  A(raw pointer) 만 배제 (Verus 적대적).  D(bitmap) 이 double-
  free 공짜 + 검증 최易 + metadata-in-page 정합으로 유력.

---

## 4. 확정 / 미결 정리

### 확정 (사용자, 2026-06-18)
- **§0.5 atomic-operation 배제 = 불가침** — SLUB 등에서 데이터 구조만
  차용, atomic 동기화 (cmpxchg_double / cross-CPU atomic free) 전량 배제
- **§2.1 cross-CPU free = IPC layer 위임 (b)** (거부 아님)
- **§2.8 SLUB 차용 = unqueued + metadata-in-page 데이터 구조만**

### 미결 — 측정 게이트 (사용자 2026-06-18 결정)
- **§2.9 freelist 표현** — **D(bitmap) / E(hybrid) ⭐ 유력 딱지**, A
  배제, B/C 보류.  **size class 별 slot 수 (scan cost) 측정 후 최종 선택**
  (측정 protocol = §2.9 말미).  측정이 P-redesign.6 microbench 인프라
  재사용 → R7.11 cross-check 와 같은 bench harness 에 얹을 수 있음
- §2.1 의 IPC 위임 세부 (msgport verb / free-request 포맷 / owner drain
  timing)
- §2.5 deterministic 의 lease scheduler 협조 (Phase C)
- §2.6 capability/tenant 진입 시점

### 후속 행동
- §2.4 (formally-verified SLAB) 를 R7.11 cross-check 완료 후 첫 비-amdv
  적용 사례로
- "**atomic-free, Verus-verified, hardened, SLUB-style SLAB**" 묶음
  (§0.5 + §2.8 + §2.4 + §2.9-결정) 을 정식 design memo (`docs/`) 로
  승격할지 — Y4 allocator 정체성 후보
