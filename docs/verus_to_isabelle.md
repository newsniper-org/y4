<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Verus → Isabelle/HOL 번역기 — 설계 사양

Y4 의 contribute-back 경로 (`docs/amdv_safety.md` §6, `docs/sel4_fork_policy.md`
§4) 의 일부. seL4 mainline 의 verification 트랙은 Isabelle/HOL 이고
Y4 의 정리는 Verus (Z3 SMT). 두 쪽 사이의 다리.

목적: **seL4 개발진이 Y4 의 정리를 자기 도구로 받아 사용**할 수 있도록
번역.  본 도구가 없으면 seL4 팀은 Y4 의 Verus 증명을 처음부터 Isabelle
로 재작성해야 하므로 contribute-back 진입 장벽 큼.

## 0. 범위 한정 (scope clamp)

**General-purpose Verus → Isabelle/HOL translator 는 본 도구의 목표가
아님.**  본 도구는 **Y4 의 Verus 코드를 Isabelle/HOL 의 proper subset
언어로 번역하는 것만을 목표** — Y4 가 사용하는 Verus 기능 부분집합만
지원.

귀결:
- Y4 가 쓰지 않는 Verus 기능 (예: `state_machines_macros` DSL, 일부
  trait 제약, closure / fn-ptr 형 ghost 등) 은 지원 X — 만나면 명확한
  에러 + Y4 측 회피 권고
- Isabelle 측 출력은 HOL 의 *proper subset* — locale / class /
  axiomatization 같은 광범위 feature 는 Y4 invariant 를 표현하기에
  필요한 범위만
- 외부 사용자가 일반 Verus 프로젝트에 본 도구를 적용하려 시도하면
  Y4-specific 가정 (특정 trait 묶음 / 특정 ghost 패턴 / lease cap 같은
  Y4 도메인 type) 으로 인해 실패 가능 — 본 도구는 그 경우 책임 X
- 본 scope 한정 덕에 §4 의 ~1500 LoC 분량이 가능 — general-purpose
  translator 라면 분량 폭증

---

## 1. 범위 — (T-i) + (T-ii) hybrid

### (T-i) Statement-only 번역

Verus 의 `spec fn` / `proof fn` / `exec fn ... ensures` 의 **시그니처
+ requires + ensures + decreases** 만 추출하여 Isabelle/HOL 의
`definition` / `lemma ... sorry` 로 변환.  증명 본문은 `sorry` —
seL4 팀이 Isabelle Isar 로 자체 증명.

장점: 분량 적음, 의미 보존 신뢰.  seL4 팀이 자기 도구로 검증.
단점: seL4 팀의 작업량이 그대로 남음 (도구는 boilerplate 만 줄임).

### (T-ii) `axiom` 옵트인

Verus 측 lemma 에 attribute 를 붙이면 Isabelle `axiomatization` 으로
변환:

```rust
#[verus_to_isabelle::axiom]
proof fn intercept_floor_holds(vcpu: VCPU)
    ensures s2_holds(vcpu)
{
    // ... Verus / Z3 증명 ...
}
```

→ Isabelle:

```isabelle
axiomatization where
  intercept_floor_holds:
    "S2_holds vcpu"
```

장점: seL4 팀이 Verus 증명을 axiom 으로 신뢰하면 즉시 사용 가능.
단점: trust boundary 가 Verus + Z3 로 이동 — Isabelle 측 보증의 "내부
일관성" 만 유지, "Z3 가 옳다" 는 외부 가정.

### Hybrid 사용 패턴

기본은 (T-i) `sorry` — seL4 팀이 자체 증명 권장.  명시적으로
`#[verus_to_isabelle::axiom]` 표시한 정리만 axiom 으로 import.

도구 내부 옵션:
- `--all-sorry` — 모든 lemma 를 `sorry` (default)
- `--respect-attrs` — attribute 가 있으면 axiom (T-ii 활성)
- `--all-axiom` — 모든 lemma 를 axiom (논의용 / 제안 단계)

---

## 2. 입력 → 출력 매핑

| Verus | Isabelle/HOL |
|---|---|
| `spec fn f(x: T) -> R { body }` | `definition f :: "T ⇒ R" where "f x = body"` |
| `proof fn lemma() requires P, ensures Q` | `lemma name: "P ⟹ Q" sorry` (또는 axiom) |
| `pub struct S { a: T, b: U }` | `record S = a :: T b :: U` (또는 `datatype`) |
| `pub enum E { A, B(T) }` | `datatype E = A | B T` |
| `Set<T>` | `'a set` (HOL.Set) |
| `Seq<T>` | `'a list` |
| `Map<K, V>` | `K ⇀ V` (option-valued partial) |
| `nat`, `int` | `nat`, `int` |
| `forall\|x: T\| P(x)` | `∀x::T. P x` |
| `exists\|x: T\| P(x)` | `∃x::T. P x` |
| `==>` | `⟶` |
| `&&`, `\|\|` | `∧`, `∨` |
| `decreases m` | termination via `function ... by (relation "measure m")` |
| `recommends` | (무시 — Verus 의 hint 만, 의미 변경 X) |
| `assume(P)` | `axiomatization where ... : P` (위험 — 명시적 표시 권장) |

미지원 (Phase 1 범위 외):
- Verus 의 `Tracked<T>` / `Ghost<T>` (linear ghost state) — Isabelle 에 직접 대응 X
- `verifier::trusted` — 직접 axiom 매핑
- closure / function pointer
- Verus 의 `state_machines_macros` DSL

---

## 3. 도구 형상

### 옵션 A — 별도 repo `y4-verus2isabelle`

- 자체 Rust crate, Cargo workspace
- Verus 소스를 입력, `.thy` 파일 출력
- Y4 가 client (proofs/verus → docs/isabelle 변환), seL4 팀도 별도 활용

### 옵션 B — Y4 워크스페이스 멤버 `tools/v2i/`

- Y4 cargo workspace 안의 멤버
- 시작 비용 작음, 재사용성 약간 낮음

### 옵션 C — Verus 측 backend plugin

- Verus 자체에 PR — Verus 가 출력 backend 로 Isabelle 추가
- 가장 깨끗하지만 Verus 팀과 협의 필요

**권고: A** — y4-drivers / y4-hypercall 와 동일 패턴, Y4 외부 도구.
seL4 팀에 도구 자체를 contribute 가능 (Apache-2.0).

---

## 4. 구현 분량 추정

| 단계 | 분량 |
|---|---|
| Verus AST parser (`syn` crate + Verus attribute 파싱) | ~300 LoC |
| 타입 매핑 + 기본 식 번역 | ~400 LoC |
| `quantifier` / `decreases` / pattern match | ~300 LoC |
| `Set` / `Seq` / `Map` API 매핑 | ~200 LoC |
| Isabelle text emit + pretty-print | ~200 LoC |
| Test fixtures (proofs/verus/src/ 의 50+ invariant 를 round-trip) | ~300 LoC |
| **합계 (Phase 1 — `sorry` only)** | **~1500 LoC Rust** |

attribute opt-in (T-ii) 추가: +~100 LoC.

---

## 5. 차단 의존

도구 자체는 외부 의존 적음:
- `syn` crate — Rust 파싱
- `quote` crate — Rust → Rust 출력 (Isabelle 출력은 자체 emitter)
- Isabelle 2024+ 설치 (테스트만)

차단 없이 즉시 진입 가능 — D1d / fork policy / y4-hypercall 작업과
**병렬 가능**.

---

## 6. seL4 측 채택 시나리오

1. Y4 가 `y4-verus2isabelle` 발표 + 사용 가이드 + 자체 50+ invariant
   round-trip 검증
2. seL4 팀이 도구 평가 — sorry 기반 statement skeleton 의 정확성
3. Y4 의 D1d 안전장치 13 개 (S1–S13) 을 Isabelle skeleton 으로 export
4. seL4 팀이 자체 Isar 증명 작성 (또는 일부 axiom 으로 수용)
5. seL4 mainline 의 AMD-V 패치가 verification 트랙도 포함된 형태로 머지

도구가 없을 때 vs 있을 때:
- 도구 없음: seL4 팀은 Y4 의 Rust 코드 + 영문 description 만 받음.
  Isabelle 작성을 처음부터.
- 도구 있음: Isabelle skeleton 을 자동 받음. lemma statement / 종속성
  / 보조 정의 모두 정렬됨. 본 작업이 "재작성" 이 아니라 "증명 채움".

---

## 7. 동결 정책

본 문서는 v0 spec.  `v1.0 frozen` 마킹 조건:
- §1 hybrid scope 사용자 sign-off
- §2 매핑 표 사용자 sign-off
- §3 repo 형상 결정

frozen 후 도구 구현 PR 진입.  도구 구현은 D1d 의 seL4 측 패치와
**병렬** 가능 — 차단 없음.

---

## 8. 미해결 / 추가 결정 필요

1. Verus 의 `Tracked<T>` / `Ghost<T>` 의 Isabelle 대응 — 필요한가? 본
   도구 v1.0 에서는 미지원, 사용 시 도구가 명확한 에러
2. **Verus 의 trigger annotation** (`#![trigger ...]`) — Isabelle 에선
   `[trigger]` attribute 가 다른 의미.  무시해도 되는지 검토
3. **Isabelle 의 locale / class** 활용 — Verus 의 trait bound 를 어떻게
   매핑할지
4. **Round-trip test 의 ground truth** — Y4 의 50+ invariant 를 도구
   v1.0 검증의 fixture 로 사용. 새로 추가되는 invariant 도 자동 포함

---

## 9. fork-policy 와의 정합성

본 도구의 산출물 (`*.thy` 파일) 이 seL4 fork 의 어디에 들어가는지:

| 위치 | 의미 |
|---|---|
| (i) seL4 fork 의 `proof/Y4/` 디렉터리 | additive — fork-policy §1 contract 만족. 기존 proof/ 변경 0. |
| (ii) seL4 fork 의 기존 `proof/sel4-{arch}/` 와 통합 | 기존 정리 변경 위험 — fork-policy 의 forbidden 항목과 충돌 가능. 비추천 |
| (iii) Y4 측 `proofs/isabelle/` 별도 트리 | seL4 fork 와 분리. PR 로 mainline 보낼 때 (i) 로 이동 |

**권고: (iii) 시작, mainline contribute 시 (i) 로 이동.**  fork-policy
의 Strictly Additive 원칙 자연 만족.
