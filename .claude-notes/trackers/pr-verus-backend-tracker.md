<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# PR-Verus-Backend tracker (2026-06-01)

> **목적:** Verus 본체 patch (`--backend=z3|oxiz|adsmt` + abductive
> verdict reporter) — P-redesign.3 의 R3.11 + R3.12 sign-off 의 산출물.
> 본 작업은 **별도 Claude Code 세션** 에서 진행 (로컬: `~/verus-fork`,
> Y4 의 sibling repo 패턴).  본 tracker = entry point + scope spec + Y4
> 측 cross-ref.
>
> **상태 (2026-06-01)**: 준비 단계.  `~/verus-fork` 디렉터리 신설 + 별
> 세션 진입 대기.  사용자가 `~/verus-fork` 를 verus-lang/verus 의 fork
> 로 clone 해두는 step 까지 완료 후 별 세션 진입.

## 1. Scope (R3.11 + R3.12 의 산출물)

### 1.1 z3 + OxiZ + adsmt 3 backend trait 통일

Verus 본체에 SMT backend trait 추가 — 현재 z3 가 hardcode 된 부분을
trait abstract:

```rust
// (예시 형식, 실제 Verus 본체 구조에 맞춰 patch 작성)
pub trait SmtBackend {
    fn solve(&self, query: SmtLibQuery) -> Verdict;
    fn name(&self) -> &'static str;
    fn supports_abductive(&self) -> bool { false }  // default
}

pub enum Backend { Z3, OxiZ, Adsmt }

pub enum Verdict {
    Sat,
    Unsat,
    Unknown { reason: String },
    Abductive {
        candidates: Vec<AbductiveCandidate>,
        explain:    String,
    },  // adsmt-only
}
```

분량 추정 ~800 LoC (trait 정의 ~150 + 3 backend impl ~450 + verdict
mapping ~100 + reporter ~100).

### 1.2 `--backend=` CLI flag

```sh
verus --backend=z3       # default
verus --backend=oxiz
verus --backend=adsmt
verus --backend=dual     # z3 + oxiz, separate runs, diff
verus --backend=triple   # z3 + oxiz + adsmt, separate runs, 3-way diff (R3.12 활성 시)
```

`just verus --backend=$BACKEND` recipe = Y4 측 `proofs/verus/justfile`
갱신 (P-redesign.2 §3.6 정합, 이미 spec frozen).

### 1.3 Abductive verdict reporter

`--report-abductive-on-unknown` flag (adsmt backend 만 활용):

- z3/OxiZ 가 `unknown` 시점에 adsmt 의 ranked hypothesis list 가 JSON 으로
  emit (Y4 측 `smt-cross-validation-tracker.md` §9 의 example JSON)
- Y4 측 invariant 강화 candidate 의 input

### 1.4 Verdict mapping

adsmt 의 4번째 verdict `Abductive` 를 Verus 의 기존 3-tuple 에 어떻게
표현?

- Verus 내부에서는 4번째 variant 정식 추가 (위 §1.1 의 `Verdict` enum)
- 단 z3/OxiZ 만 사용 시 `Abductive` variant 는 unreachable
- Verus reporter (jsonl 출력) 의 schema 갱신 — `verdict: "abductive"` +
  `abductive_candidates: [...]`

## 2. Y4 측 cross-ref (별 세션에서 읽을 dep)

| Y4 측 doc | 내용 |
|---|---|
| `.claude-notes/trackers/av-proof-body-tracker.md` §1 R3.11 | 본 patch 의 sub-PR scope 결정 (별도 sub-PR, Y4 P.3 cluster 작성과 분리) |
| `.claude-notes/trackers/av-proof-body-tracker.md` §1 R3.12 | opt-in 3-way 결정 (z3 + OxiZ default, adsmt 명시 시 + abductive verdict reporter) |
| `.claude-notes/trackers/av-proof-body-tracker.md` §7 | abductive verdict 활용 invariant 6 후보 (AV5/12/15/23/24/30) |
| `.claude-notes/trackers/smt-cross-validation-tracker.md` §9 | R3.12 의 abductive verdict JSON 예시 + 측정 cycle |
| `docs/verus_to_isabelle.md` §3.6 | unified-toolkit-pin.toml + `--backend=` flag spec |
| `Y4/unified-toolkit-pin.toml` `[verus]` sub-table | Verus version range (min/max/recommended) |

## 3. 시작 조건 (별 세션이 진입 전)

1. **`~/verus-fork` clone**: 사용자가 verus-lang/verus 를 fork (GitHub
   web UI) 후 로컬 clone — `git clone git@github.com:<user>/verus.git ~/verus-fork`
2. **upstream remote**: `cd ~/verus-fork && git remote add upstream
   https://github.com/verus-lang/verus.git && git fetch upstream`
3. **branch 신설**: `git checkout -b backend-pluggable`
4. **별 Claude Code 세션**: `cd ~/verus-fork && claude` → 본 tracker 의
   §1 + §2 + §4 + Y4 측 cross-ref 읽기 → 작업 시작

## 4. 작업 phase

| Phase | 내용 | 분량 | 의존 |
|---|---|---|---|
| P-vb.1 | Verus 본체 source tree 탐색 + 현 SMT backend (z3) hardcode 위치 파악 | (탐색) | 0 |
| P-vb.2 | `SmtBackend` trait 정의 + `Backend` enum + `Verdict` enum (Abductive variant 포함) | ~150 LoC | P-vb.1 |
| P-vb.3 | z3 backend impl (기존 코드의 trait 채택) | ~150 LoC | P-vb.2 |
| P-vb.4 | OxiZ backend impl (adsmt 의 `external/oxiz/` 측 oxiz-sat lib 호출) | ~200 LoC | P-vb.2 |
| P-vb.5 | adsmt backend impl (lu-smt 호출, abductive verdict 파싱) | ~200 LoC | P-vb.2 |
| P-vb.6 | `--backend=` CLI flag + verdict mapping + jsonl reporter 갱신 | ~100 LoC | P-vb.3/4/5 |
| P-vb.7 | `--report-abductive-on-unknown` reporter flag | ~100 LoC | P-vb.6 |
| P-vb.8 | Test (z3 single, OxiZ single, dual diff, triple diff, abductive verdict round-trip) | ~200 LoC | P-vb.7 |
| P-vb.9 | Upstream PR (verus-lang/verus, optional, post-Y4-cycle) | 0 | P-vb.8 |

합 ~1100 LoC (test 포함).  본체만 ~800 LoC.

## 5. Y4 측 산출물 land 후 cross-validate trigger

별 세션이 P-vb.8 완료 후:

1. 사용자가 `cd ~/Y4 && git pull` → unified-toolkit-pin.lock 의 `[verus]`
   sub-table 갱신 (PR-Verus-Backend 의 commit sha)
2. Y4 측 `just verus --backend=z3` (default 정합 확인)
3. Y4 측 `just verus-cross-validate` (`smt-cross-validation-tracker.md` §6
   의 measurement command) 활성
4. `av-proof-body-tracker.md` §9 의 cluster 별 cross-validate 시작 가능

## 6. License (별 세션 진입 시 주의)

- Verus = MIT 또는 Apache-2.0 dual (`verus-lang/verus/LICENSE`)
- Y4 측 contribution = Apache-2.0
- 본 patch = MIT 또는 Apache-2.0 dual (Verus 측 upstream 정책 정합)
- DCO sign-off (`-s`) 의무 — Y4 정책 (CONTRIBUTING.md §1) 동일하게 적용

## 7. 갱신 path

- P-vb.X 의 완료 시 §4 표 갱신
- 본체 patch 가 land 된 후 §5 의 cross-validate trigger 활성
- Upstream PR 제출 시 §4 P-vb.9 행 갱신 + URL 기록
- Y4 측 `av-proof-body-tracker.md` §1 R3.11 의 status 도 본 tracker 의 §4
  진행 상태 반영
