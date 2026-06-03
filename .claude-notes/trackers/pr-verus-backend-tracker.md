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

## 1. Scope (R3.11 + R3.12 의 산출물, flag mechanism 갱신 2026-06-03)

> **2026-06-03 결정**: 새 `--backend=` flag 정의 X — Verus 의 기존 `-V
> <key>` extended-multi flag mechanism (`-V cvc5` 패턴) 안에 새 backend
> 옵션 추가.  upstream contribute-back path 의 자연성 ↑ + patch 분량
> ~300 LoC 절약.

### 1.1 `SmtSolver` enum 확장 + EXTENDED_KEYS 추가

**기존 (Verus upstream)**:
- `source/air/src/context.rs`:
  ```rust
  pub enum SmtSolver { Z3, Cvc5 }
  ```
- `source/rust_verify/src/config.rs` (`EXTENDED_KEYS`):
  ```rust
  const EXTENDED_CVC5: &str = "cvc5";
  // ...
  (EXTENDED_CVC5, "Use the cvc5 SMT solver, rather than the default (Z3)"),
  ```
- solver 선택 (`config.rs` line 827):
  ```rust
  solver: if extended.contains_key(EXTENDED_CVC5)
              { SmtSolver::Cvc5 } else { SmtSolver::Z3 },
  ```

**Y4 측 patch (R3.11 + R3.12)** — 위 3 위치에 확장:

```rust
// source/air/src/context.rs
pub enum SmtSolver { Z3, Cvc5, OxiZ, Adsmt }

// adsmt-abduce::rank::RankedCandidate 와 1:1 정합 (2026-06-03 schema 강화,
// adsmt v1.0.0-rc.7 의 native Candidate {hypotheses, explanations, sources}
// + Ranked{candidate, score} 정합)
pub struct AbductiveCandidate {
    pub rank: u32,                          // 1-based, top = 1
    pub score: f64,                          // adsmt native: smaller = stronger
    pub hypotheses: Vec<String>,             // adsmt Candidate.hypotheses, Display 직렬화
    pub explanations: Vec<Option<String>>,   // adsmt Candidate.explanations 정합
    pub sources: Vec<String>,                // adsmt Candidate.sources 정합
}

// 4번째 verdict variant (adsmt-only).  top-level `explain` field 폐기
// (per-hypothesis explanations 와 정합 강화, 2026-06-03)
pub enum SmtVerdict {
    Sat,
    Unsat,
    Unknown { reason: String },
    Abductive { candidates: Vec<AbductiveCandidate> },
}

// source/rust_verify/src/config.rs
const EXTENDED_OXIZ:                       &str = "oxiz";
const EXTENDED_ADSMT:                      &str = "adsmt";
const EXTENDED_REPORT_ABDUCTIVE_ON_UNKNOWN: &str = "report-abductive-on-unknown";
// ... in EXTENDED_KEYS:
(EXTENDED_OXIZ,  "Use the OxiZ SMT solver (pure-Rust Z3 reimplementation, Z3 protocol parity)"),
(EXTENDED_ADSMT, "Use the adsmt abductive-deductive HOL+HKT solver (4th verdict 'Abductive' available)"),
(EXTENDED_REPORT_ABDUCTIVE_ON_UNKNOWN,
    "When -V adsmt 의 verdict 가 Unknown 또는 Abductive 인 경우, ranked candidate JSON 을 jsonl 에 emit"),
```

`extended.contains_key(...)` 선택 로직 → match 형태:

```rust
solver: if extended.contains_key(EXTENDED_ADSMT) { SmtSolver::Adsmt }
        else if extended.contains_key(EXTENDED_OXIZ)  { SmtSolver::OxiZ }
        else if extended.contains_key(EXTENDED_CVC5)  { SmtSolver::Cvc5 }
        else                                          { SmtSolver::Z3 },
```

**분량 추정 (갱신)**: ~800 LoC → **~500 LoC** (mechanism 신설 부담 ↓):
- enum 확장 + EXTENDED keys ~50 LoC
- OxiZ backend impl ~200 LoC (adsmt 의 `external/oxiz/` oxiz-sat 호출)
- adsmt backend impl ~150 LoC (lu-smt 측 cert 파싱 + abductive verdict)
- Verdict mapping (Abductive variant) + jsonl reporter ~100 LoC

### 1.2 CLI 사용 형태 (`-V <key>` 패턴)

```sh
# 단일 backend
verus                              # default = Z3 (no flag)
verus -V cvc5                      # 기존 upstream
verus -V oxiz                      # 신규
verus -V adsmt                     # 신규
verus -V adsmt -V report-abductive-on-unknown
                                   # adsmt + abductive verdict reporter
```

`just verus` recipe = default Z3.  cross-validation (`dual` / `triple`)
은 Verus 본체 flag X — Y4 측 `just verus-cross-validate` script 의
multi-invocation 로직 (script 가 internally `verus` / `verus -V oxiz` /
`verus -V adsmt` 3 회 호출 후 결과 diff).

### 1.3 Abductive verdict reporter (`-V report-abductive-on-unknown`)

- adsmt backend 의 `Verdict::Unknown` 또는 `Verdict::Abductive` 시점에
  ranked candidate JSON emit (smt-cross-validation-tracker §9 의 갱신된
  example JSON 정합)
- candidate JSON 의 source = adsmt-cli (lu-smt) 가 stdout 에 emit 하는
  single-line JSON.  Verus 측 reporter 는 그대로 forward — schema
  invariant 는 adsmt 측 (`adsmt-abduce::rank::RankedCandidate` +
  `Candidate {hypotheses, explanations, sources}`) 의 native shape
- z3/OxiZ/cvc5 backend 측은 본 flag 무효 (warning 출력 X — invariant
  관계없는 noise 회피)
- Y4 측 invariant 강화 candidate 의 input — av-proof-body-tracker §6 의
  per-cluster LoC actual 측정 시 hypothesis 활용

### 1.4 Verdict mapping

adsmt 의 4번째 verdict `Abductive` 의 Verus 측 표현:

- `SmtVerdict::Abductive { candidates: Vec<AbductiveCandidate> }`
  variant 정식 추가 (위 §1.1).  per-hypothesis level 의 explanation /
  source 는 candidate struct 안에 — top-level `explain` field X
- z3/OxiZ/cvc5 backend 측 시점에서 `Abductive` variant 는 unreachable
  (compile-time exhaustive match 강제)
- Verus reporter (jsonl 출력) 의 schema 갱신 — `verdict: "abductive"` +
  `abductive_candidates: [{rank, score, hypotheses[], explanations[],
  sources[]}, ...]` (smt-cross-validation-tracker §9 의 갱신된 example
  JSON 정합)

## 2. Y4 측 cross-ref (별 세션에서 읽을 dep)

| Y4 측 doc | 내용 |
|---|---|
| `.claude-notes/trackers/av-proof-body-tracker.md` §1 R3.11 | 본 patch 의 sub-PR scope 결정 (별도 sub-PR, Y4 P.3 cluster 작성과 분리) |
| `.claude-notes/trackers/av-proof-body-tracker.md` §1 R3.12 | opt-in 3-way 결정 (z3 + OxiZ default, adsmt 명시 시 + abductive verdict reporter) |
| `.claude-notes/trackers/av-proof-body-tracker.md` §7 | abductive verdict 활용 invariant 6 후보 (AV5/12/15/23/24/30) |
| `.claude-notes/trackers/smt-cross-validation-tracker.md` §9 | R3.12 의 abductive verdict JSON 예시 + 측정 cycle |
| `docs/verus_to_isabelle.md` §3.6 | unified-toolkit-pin.toml + `-V <key>` flag spec (2026-06-03 갱신, 기존 `--backend=` 명시 X) |
| `Y4/unified-toolkit-pin.toml` `[verus]` sub-table | Verus version range (min/max/recommended) |

## 3. 시작 조건 (별 세션이 진입 전)

> **status (2026-06-03)**: 1, 1.5 단계 완료 ✅ (사용자 보고 — `vargo
> build --release` 가 경고 0 으로 success).  2, 3, 3.5 는 별 세션 진입
> 직전 또는 진입 후 첫 step.

### 1. `~/verus-fork` clone ✅
사용자가 verus-lang/verus 를 fork (GitHub web UI) 후 로컬 clone:
```sh
git clone git@github.com:<user>/verus.git ~/verus-fork
```

### 1.5. Verus toolchain + Z3 + vargo build ✅
표준 `cargo build` 는 **작동 X** — Verus 는 자체 build wrapper `vargo`
(rustc internals + Z3 + workspace order 의존) 가 필수.

```fish
cd ~/verus-fork/source
./tools/get-z3.sh                           # Z3 4.12.5 binary (필수)
rustup toolchain install 1.95.0
rustup component add rustc-dev llvm-tools --toolchain 1.95.0
source ../tools/activate.fish               # vargo 자체 build + PATH/env setup
vargo build --release                       # Verus 전체 build (vstd 포함)
```

성공 시 `~/verus-fork/tools/vargo/target/release/vargo` 생성 + Verus
`rust_verify` binary + `builtin` / `builtin_macros` / `state_machines_
macros` / `vstd` 모두 build.

### 2. upstream remote 추가
```sh
cd ~/verus-fork
git remote add upstream https://github.com/verus-lang/verus.git
git fetch upstream
```

### 3. Branch 신설
```sh
git checkout -b backend-pluggable
```

### 3.5. VSCode setup (별 세션 IDE 사용 시 필수)
**원인**: `.vscode/` 에 `settings.json.template` / `launch.json.template`
/ `tasks.json.template` 만 있고 실제 파일 없음 → VSCode 가 default
`cargo build` 사용 → ~114 컴파일 error (Verus internal crate inter-
dependency + rustc-dev component 미인식).

**해결**:

```fish
cd ~/verus-fork
cp .vscode/settings.json.template .vscode/settings.json
cp .vscode/launch.json.template   .vscode/launch.json
cp .vscode/tasks.json.template    .vscode/tasks.json
```

`.vscode/settings.json` 의 platform 정합 수정:
- `runnables.command` 의 `[.exe]` 부분 삭제 (Linux/macOS):
  `"../tools/vargo/target/release/vargo[.exe]"` →
  `"../tools/vargo/target/release/vargo"`
- (권장) `linkedProjects` 추가 — VSCode workspace root = `~/verus-fork/`
  여도 cargo workspace 가 `source/` 안에 있음을 명시:
  `"rust-analyzer.linkedProjects": ["source/Cargo.toml"]`

**VSCode workspace 열기**:
```sh
code ~/verus-fork    # root = ~/verus-fork/, .vscode/ 위치와 정합
```

**verus-analyzer (선택)**: 표준 rust-analyzer 대신 [verus-lang/verus-
analyzer](https://github.com/verus-lang/verus-analyzer) extension 사용
시 Verus 의 `proof` / `spec` / `requires` / `ensures` keyword 인식 강화.
설치 시 일반 rust-analyzer extension disable 필요.

### 4. 별 Claude Code 세션 진입
```sh
cd ~/verus-fork && claude
```
첫 read 대상: 본 tracker §1 + §2 + §4 + Y4 측 cross-ref + 본 §3 의 3.5
VSCode setup.

## 4. 작업 phase

| Phase | 내용 | 분량 | 의존 |
|---|---|---|---|
| P-vb.1 | 현 cvc5 patch (`EXTENDED_CVC5` + `SmtSolver::Cvc5`) 의 위치 파악 — `air/src/context.rs` + `rust_verify/src/config.rs` + smt_process 측 backend dispatcher 모두 | (탐색) | 0 |
| P-vb.2 | `SmtSolver` enum 확장 (`+ OxiZ + Adsmt`) + `SmtVerdict` enum 신설 (Abductive variant 포함) | ~50 LoC | P-vb.1 |
| P-vb.3 | `EXTENDED_OXIZ` + `EXTENDED_ADSMT` + `EXTENDED_REPORT_ABDUCTIVE_ON_UNKNOWN` 추가 + solver 선택 로직 match 변환 | ~50 LoC | P-vb.2 |
| P-vb.4 | OxiZ backend impl (adsmt 의 `external/oxiz/` 측 oxiz-sat lib 호출) | ~200 LoC | P-vb.3 |
| P-vb.5 | adsmt backend impl (lu-smt 호출, abductive verdict 파싱) | ~150 LoC | P-vb.3 |
| P-vb.6 | Verdict mapping (Abductive variant) + jsonl reporter schema 갱신 | ~50 LoC | P-vb.4/5 |
| P-vb.7 | `-V report-abductive-on-unknown` flag 의 conditional emit | ~50 LoC | P-vb.6 |
| P-vb.8 | Test (Z3 default, `-V cvc5` 기존 회귀, `-V oxiz`, `-V adsmt`, `-V adsmt -V report-abductive-on-unknown` round-trip) | ~150 LoC | P-vb.7 |
| P-vb.9 | Upstream PR (verus-lang/verus, optional, post-Y4-cycle) | 0 | P-vb.8 |

합 **~700 LoC** (test 포함).  본체만 **~500 LoC** (2026-06-03 갱신,
기존 `-V <key>` mechanism 활용으로 새 flag 정의 부담 ~300 LoC 절약).

## 5. Y4 측 산출물 land 후 cross-validate trigger

별 세션이 P-vb.8 완료 후:

1. 사용자가 `cd ~/Y4 && git pull` → unified-toolkit-pin.lock 의 `[verus]`
   sub-table 갱신 (PR-Verus-Backend 의 commit sha)
2. Y4 측 `just verus` (default Z3, 회귀 확인) + `just verus -- -V oxiz`
   (OxiZ backend 정합 확인) + `just verus -- -V adsmt` (adsmt backend
   정합 확인)
3. Y4 측 `just verus-cross-validate` (`smt-cross-validation-tracker.md` §6
   의 measurement command — script 가 internally 3 invocation 처리) 활성
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
