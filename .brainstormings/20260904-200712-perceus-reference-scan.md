<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Perceus (Koka) reference scan — precise RC + reuse analysis 에서 Y4 가 참고/차용할 것 (allocator reuse fast-path + 보안 precision)
created: 2026-09-04T20:07:12+09:00   # KST (UTC+9)
status: brainstorming (reference scan — 외부 기법 차용가치 평가; research 제안 + 후속 연구 Frame-Limited Reuse·FP²/FIP·Counting Immutable Beans).  결정 방향 + 미결
scope: Koka 의 Perceus memory management 를 Y4 관점에서 스캔 — 무엇을 참고/차용/배제할지.
       allocator front-end(reuse fast-path) / 보안(zeroization precision) / atomic-free 경계 / FBIP 규율 / 검증
refs:
  - Perceus (Reinking·Xie·de Moura·Leijen, PLDI 2021) — "Garbage Free Reference Counting with Reuse"
  - Reference Counting with Frame-Limited Reuse (Lorenzen·Leijen, ICFP 2022) — reuse 정밀화·예측가능
  - FP²: Fully in-Place Functional Programming (Lorenzen·Leijen·Swierstra, ICFP 2023) — provable zero-alloc + constant stack
  - Counting Immutable Beans (de Moura·Ullrich, Lean 4, IFL 2019) — borrowing-based RC elision + reuse (관련 계보, verified-language)
  - .brainstormings/20260618-212218-alloc-frontend-improvements.md (§0.5 atomic 배제, front-end)
  - .brainstormings/20260618-221518-pluggable-alloc-frontend-contract.md (AllocFrontend/Freelist trait, measurement gate)
  - .brainstormings/20260618-223701-capability-bound-allocation.md (sealed zero-on-free)
  - .brainstormings/20260813-205326-key-management.md §5 (zeroization) · §2 (use-without-extraction)
  - .brainstormings/20260812-173424-ipc-layer-design.md §0.5 (atomic-free composition = P1)
  - docs/design_decisions.md P1(atomic-free) · P5(standard-first reuse) · P9(검증-모드) · §2.1(allocator)
  - .brainstormings/20260813-125742-asterinas-reference-scan.md (reference-scan 선례)
---

# Perceus reference scan — reuse analysis 를 atomic-free 로 차용

## §0 프레임

설계 탐색이 아니라 **외부 기법 차용가치 스캔**(Asterinas scan 선례).  **Perceus**
(Koka 의 메모리 관리)는 GC 없이 **precise reference counting + reuse analysis**
로 deterministic·garbage-free 메모리를 달성 — Y4 의 **allocator front-end** ·
**atomic-free 원칙(P1)** · **보안(zeroization precision)**과 실제 접점이 있다.
"그대로 채택"이 아니라 **부분별 참고/차용/배제**를 판정.

## §1 Perceus 요약 (fact, 조사 2026-09-04)

- **precise reference counting**: 컴파일 시점에 정확한 RC inc/dec 를 삽입 →
  cycle-free 프로그램은 **garbage-free**(live 참조만 유지, **last-use 즉시 해제**;
  scope-end 아님).
- **reuse analysis**: 마지막 참조가 drop 되어 해제될 객체의 메모리를 **같은
  shape 의 새 할당에 in-place 재사용**(free+malloc 왕복 회피) → **guaranteed
  in-place update**.
- **FBIP (Functional But In-Place)**: reuse analysis 로 순수 함수형 코드를 in-place
  변형 성능으로 실행(tail-call 이 loop 를 함수호출로 쓰게 하듯).
- **증명**: reference counting 을 **linear resource calculus** 로 형식화, Perceus
  가 **sound 하고 garbage-free** 임을 증명.
- ★ **공유(thread) = atomic RC**: 여러 스레드가 값을 공유하면 RC inc/dec 가
  **atomic 이어야 하고, 실무상 비싸다**(논문/저자 명시).
- Koka 에 구현, state-of-the-art collector 와 경쟁력.  PLDI 2021.

## §1.1 후속·관련 연구 (Perceus 계보 + Lean 4)

- **Perceus 계보** (Koka, Microsoft Research):
  - Perceus(PLDI'21) → **Reference Counting with Frame-Limited Reuse**
    (Lorenzen·Leijen, ICFP'22): reuse 를 **frame 단위로 제한**해 더 예측가능·강한
    reuse 보장 — Y4 의 deterministic/RT 성향과 정합(§3 refine).
  - → **FP²: Fully in-Place Functional Programming** (Lorenzen·Leijen·Swierstra,
    ICFP'23): 순수 함수형이 **인자가 다른 곳에 공유되지 않으면 provably
    (de)allocation 0 + constant stack** 으로 실행됨을 **linear FIP calculus** 로
    증명; splay/finger tree·merge/quick sort·generic map 을 heap·stack 0 로.
    ★ **provable zero-alloc + bounded stack = 하이퍼바이저 hot/RT 경로의 이상**
    → §5 로 승격(tier 1).
- **관련 계보 — Lean 4 "Counting Immutable Beans"** (de Moura·Ullrich, IFL'19):
  Perceus 의 sibling; **borrowing-based 분석으로 RC update elision** + reuse;
  **정리증명기 Lean 4 에서 production**.  ★ Y4/Rust 접점: **Rust 의 정적 borrow
  checker 가, Lean 이 (동적으로) 추론하는 borrowing 을 컴파일-시점에 이미 수행**
  → "Rust static > runtime RC"(§2) 논지 강화; Lean 4 가 **verified-language** 라
  Y4 형식검증 지향과 정합.  (Lean 은 cycle 생성 불가 → RC 의 cycle 비판 무효;
  Y4 도 동일, §6.)
- **borrowing 최적화** (Lorenzen, thesis "Optimizing Reference Counting with
  Borrowing"): Perceus RC 를 borrowing 으로 최적화 — Rust 는 borrowing 을 이미
  언어 차원에서 가짐(위 접점 재확인).

## §2 Y4 와의 근본 차이 — 왜 통째로 못 가져오나

- Y4 = Rust **static ownership**(owned value 는 **runtime RC 0**).  Perceus =
  컴파일-삽입 **precise RC**(runtime inc/dec, 단 최소화+reuse).  ⟹ common case
  에선 **Rust 정적 ownership 이 Perceus 보다 이미 "더 정적"** → Y4 는 **RC-
  everywhere 기질 채택 X**.
- ★ **atomic RC 충돌**: Perceus 의 공유-객체 RC = **atomic** → Y4 **§0.5
  atomic-free(P1) 위반**.  Y4 의 cross-CPU 공유는 **IPC**(P1 atomic-free
  composition)로 하지 shared atomic-RC 객체로 하지 않는다.  ⟹ Perceus 의 **atomic
  RC 부분은 명시 배제**(§6), **단일-소유 / per-CPU 부분(reuse analysis)만 차용**.

## §3 (차용 tier 1) reuse analysis → allocator reuse fast-path ★★★

Perceus reuse 의 alloc 대응: **drop-then-reuse fast-path**.
- 방금 free 된 size-class S 의 slot 을 **freelist 왕복 없이 즉시 다음 alloc(S)에
  재배정** — per-CPU **"hot slot" / reuse cache**(직전 해제 slot 캡처).
- ★ **atomic-free 정합(P1)**: per-CPU 라 §0.5 준수.  **deterministic**(freelist
  traversal·scan 회피 → WCET 개선, real-time 정합).  **Frame-Limited Reuse
  (ICFP'22)**가 reuse 를 frame 단위로 bound → **예측가능한 reuse**(RT 정합 강화).
- ★ **Verus 정합(formal-first)**: reuse fast-path 가 allocator 불변식(conservation,
  no double-use) 보존을 증명 — Perceus 가 이미 **linear calculus 로 증명**했으니
  참고 모델.
- alloc front-end 브레인스토밍(pluggable `AllocFrontend`/`Freelist`, capability-
  bound)에 이 fast-path 를 **옵션**으로.  단 alloc 시리즈의 **measurement gate**
  (size-class 별 이득 측정 후 채택)와 정합 — reuse cache 이득은 size-class·워크로드
  의존.

## §4 (차용 tier 1) precision(free-at-last-use) → 보안 ★

Perceus 는 garbage-free — 객체를 **last-use 즉시** 해제(scope-end 아님).  Y4 보안
접점:
- key-mgmt §5 **zeroization** + capability-bound **sealed zero-on-free**: 민감
  데이터(key 자료 등)를 **죽는 즉시** 해제·zero → **민감 데이터 잔존 window 최소화**.
  Perceus 의 precision 이 이 window 를 좁힌다.
- 단 Rust drop 은 scope-end(또는 explicit `drop`) — Perceus 만큼 정밀하진 않음.
  Y4 는 민감 자료에 **explicit early-drop + zeroize**(`Zeroizing<T>` 패턴)로
  근사하고, **"last-use 즉시 해제"를 보안 목표**로 삼는다(key-mgmt §9 zeroization
  불변식 강화).

## §5 (차용 tier 1) FIP 규율 — provable zero-alloc + constant stack ★

**FP²/FIP**(ICFP'23)는 순수 함수형이 인자 미공유 시 **provably (de)allocation 0
+ constant stack** 으로 실행됨을 증명 — FBIP 의 "in-place 가능"을 **형식적 판정
기준**으로 끌어올린 것.
- ★ **하이퍼바이저 hot/RT 경로에 이상적**: provable zero-alloc + bounded stack =
  **WCET(real-time) · fail-closed(P10) · no-hidden-cost** 정합.  Y4 의 **IPC fast
  path · scheduler policy · allocator metadata · capsule 상태기계** 등 무-할당 hot
  경로를 **FIP 스타일**로 작성 → 그 경로가 **할당·해제 0 + 상수 스택** 보장.
  (kernel 의 bounded stack 요구와도 정합.)
- **Rust 근사**: Rust 는 함수형-first 아니나 FIP 의 판정(무엇이 in-place 실현
  가능한가)을 **"hot 경로 무할당 + `&mut` in-place 변형 + §3 reuse"** 규율로 근사.
  FP² 의 FIP calculus 를 **"이 경로가 무할당으로 실현 가능한가"의 판정 도구**로
  참고.
- Verus 결합: "hot 경로 할당 0 + 스택 bounded" 를 불변식으로(§8).

## §5.1 (비교분석) `alloc::Rc` vs 하이퍼바이저 FIP inline-macro

**질문**: Perceus/FIP 를 Y4 에 들일 때 (A) `alloc` crate 의 **`Rc` 활용** vs
(B) 컴파일러 가이드라인에 최적화된 **하이퍼바이저 전용 FIP inline-macro 구조**
작성 — 어느 쪽?

| 축 | A: `alloc::Rc` | B: Y4 FIP inline-macro |
|---|---|---|
| 해결 문제 | **공유 소유**(다중 owner, RC 로 해제 시점 결정) | **고유 소유 + in-place + zero-alloc** |
| ★ Perceus 가치 포착 | ✗ — RC 는 Perceus 가 *줄이려는* 오버헤드 그 자체 | ✅ — Perceus/FP² 의 핵심(uniqueness→in-place, no RC) |
| atomic-free(P1) | non-atomic(§0.5 OK)·`!Send`(per-CPU 한정); 런타임 inc/dec | RC 0(`&mut`/core-only) → 무비용 |
| 할당/WCET | `Rc::new`=heap; `make_mut`=비고유 시 **숨은 clone 할당**(WCET 저해) | **provable zero-alloc** → WCET·no-hidden-cost |
| 검증(P9) | std trusted(unsafe 내부, Verus X) | Y4-authored → **Verus 검증가능**(단 저작·검증 비용) |
| unsafe | std 내부 캡슐화 | pure in-place(`&mut`)는 **safe**; reuse(§3)만 unsafe/allocator |
| 유지보수(P5) | std, 무유지보수 ✅ | Y4 유지, 복잡 ↑ |
| Rust 실현 | 즉시 | ★ **core-only(no `alloc` import) → 컴파일러가 zero-alloc 강제** + `&mut` in-place; FP² 의 *형식 증명*은 Koka 백엔드 전용이라 Rust 는 **근사** |
| 적합 | 단일-CPU 내 **불가피한 동적 공유(비-hot)** | **hot/RT 무할당 경로**(IPC fast path·scheduler·capsule 상태기계) |

**결론**:
- 둘은 **다른 문제**를 푼다(경쟁 아닌 상보).  그러나 **Perceus 가치 포착이
  목표라면 B(FIP-macro)가 결정적** — A(`Rc`)를 쓰는 건 Perceus 를 "차용"하는 게
  아니라 그것이 최소화하려는 **RC baseline 을 쓰는 것**.
- ★ **핵심 실현 통찰**: Rust 에서 FP² 의 zero-alloc 을 *형식 증명*까진 못 가져와도
  (Koka 전용 백엔드), **hot 경로를 `core`-only(no `alloc`)로 작성하면 컴파일러가
  할당 불가능을 강제** → 실질 zero-alloc 보장.  FIP-macro 는 이 **core-only +
  `&mut` in-place + §3 reuse** 를 캡슐화·강제(+ Verus 로 "할당 0" 불변식).
- **권고 — 계층적**:
  - **hot/RT/고유 경로 = B(FIP-macro) 우선**(core-only 강제 + Verus).
  - **드문 진짜 동적 공유(비-hot, 단일-CPU) = A(`std Rc`)** — P5(표준 재사용),
    `Rc::get_mut`/`make_mut` 로 unique-fast-path.  ★ **`Arc` 는 hot 경로 금지**
    (atomic, §0.5/P1).
  - 하나만 고른다면 Perceus 목표상 **B**.

## §6 (배제) 명시적 non-adoption

- **runtime RC-everywhere 기질** — Rust 정적 ownership 이 common case 에서 우월
  (runtime RC 0).  Y4 는 RC 를 일반 메모리에 도입 X(Rust `Rc/Arc` 는 기존대로
  opt-in, 그나마 `Arc` 의 atomic 은 §0.5 상 hot 경로 회피).
- ★ **atomic RC for 공유** — §0.5(P1) 위반; cross-CPU 공유는 IPC(§2).
- **effect-handler 기계장치** — Koka 의 algebraic effects 는 Rust 무관.
- **cycle collection** — Perceus 도 기본 미지원; Y4 는 ownership/value semantics
  로 cycle 회피(별도 필요 X).

## §7 license / reuse mode

- **기법/논문 차용**(무료 — 아이디어·알고리즘).  Koka 런타임 코드를 직접 import
  하는 게 아니라 **reuse-analysis 기법을 Y4 Rust allocator 로 재구현** → license
  무관(clean-room, 원칙 5 / contribute-back 정합).
- (만약 Koka repo 의 런타임 코드를 *직접* 참고할 경우에만 그 license 확인 — 본
  스캔은 paper-level 기법 차용이라 해당 없음.)

## §8 verification 접점

- Perceus 는 **linear resource calculus 로 soundness/garbage-free 증명**(paper)
  → Y4 의 reuse fast-path(§3) Verus 증명의 **참고 모델**.
- Y4 는 Rust ownership + Verus 로 **정적** 보장 → Perceus 의 runtime-RC 증명보다
  강한 정적 보장 가능; reuse fast-path 의 **런타임 동작(직전 slot 재배정)**만 Verus
  로 불변식 보존 증명(검증-모드 P9).

## §9 결정 / 미결 요약

**결정 방향(강)**:
- reuse analysis → **allocator drop-then-reuse fast-path**(per-CPU, atomic-free
  P1, Verus, measurement gate; **Frame-Limited Reuse(ICFP'22)** 로 예측가능 reuse)
  (§3) — 가장 강한 차용
- precision → **민감 자료 early-drop + zeroize** 로 잔존 window 최소화(§4)
- **FIP 규율(FP², ICFP'23)** — hot/RT 경로 **provable zero-alloc + constant
  stack**(§5, tier 1 승격)
- **배제**: RC-everywhere · atomic RC · effect-handler · cycle collection(§6)
- 기법 차용 = **clean-room**(paper-level, license 무관, §7)

**미결(설계 필요)**:
- reuse fast-path 를 front-end trait(`AllocFrontend`/`Freelist`)에 어떻게 노출할지
- reuse cache 크기·교체 정책 · size-class 별 measurement gate 결과
- explicit early-drop(§4)을 민감 자료에 어디까지 강제할지(lint? 타입?)

**⏳ 선행 의존**:
- allocator 실제 구현(front-end)
- verus-fork 안정화 — reuse fast-path Verus(§3/§8)

**연결**: 본 스캔의 차용은 design_decisions §2.1(allocator) 로 수렴 — 채택 시
alloc front-end 브레인스토밍에 reuse fast-path 절 추가.

## §10 다음 발제 후보

- observability / telemetry in minimal TCB
- (사용자 지시 대기)
