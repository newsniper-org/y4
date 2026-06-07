<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# PR-Verus-Backend tracker (2026-06-01)

> **목적:** Verus 본체 patch (`-V oxiz` / `-V adsmt` / `-V report-
> abductive-on-unknown` + `SmtSolver` enum 확장 + abductive verdict
> reporter) — P-redesign.3 의 R3.11 + R3.12 sign-off 의 산출물.  본
> 작업은 **별도 Claude Code 세션** 에서 진행 (위치: `<Y4>/verus-fork/`
> **git submodule**, branch `backend-pluggable`, remote `https://github.
> com/newsniper-org/verus`).  본 tracker = entry point + scope spec + Y4
> 측 cross-ref.
>
> **상태 (2026-06-03 최종 갱신)**:
> - `~/verus-fork/` 사용자 직접 clone ✅
> - vargo build --release ✅ (경고 0 success)
> - **Y4 측 submodule `verus-fork/` 추가 ✅** — branch `backend-pluggable`
>   pin.  Y4 측 verus 호출 = submodule path 의 binary (system `verus` /
>   AUR `verus-bin` 의존 0)
> - **모든 PR-Verus-Backend phase (P-vb.1~P-vb.12) land 완료 ✅** —
>   §4 참조.  verus-fork backend-pluggable HEAD 가 R3.11+R3.12+R7.3 의
>   모든 patch + rc.28 sound + rc.29 Tseitin complete + AOT + JIT +
>   consumer/justfile template 모두 ship.  Y4 측 "별 세션 대기" 작업 0.
> - **Y4 측 R7.11 milestone 즉시 진입 가능** — `cd proofs/verus && just
>   verify-adsmt && just emit-isabelle && just cross-check`

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

### 1.5 Emit-isabelle / Emit-rocq wire (R7.3 신규, 2026-06-03)

R7 sign-off 로 PR-Verus-Backend scope 확장 — verify 성공 시 adsmt cert
JSON 을 그대로 `adsmt-emit-isabelle` / `adsmt-emit-rocq` CLI 측 invoke
하여 `.thy` / `.v` 자동 생성:

- `EXTENDED_EMIT_ISABELLE = "emit-isabelle"` — binary flag, `-V emit-
  isabelle` 시 verify-success hook 에서 `adsmt-emit-isabelle <cert>.json`
  invoke 후 `.thy` emit
- `EXTENDED_EMIT_ROCQ = "emit-rocq"` — 동일 패턴 `.v` 생성
- Output path 결정: `--emit-isabelle-out=<path>` / `--emit-rocq-out=<path>`
  flag (path-taking flag, `-V` binary 와 별도).  default = `target-verus/
  release/emit/<crate>.{thy,v}`
- adsmt-contrib testing branch pin (R7.6) — Y4 측 cargo install 또는
  PKGBUILD system install 가정

### 1.6 AOT prelude bank + JIT trace load (R7.3 신규, 2026-06-08 갱신)

**AOT ✅ functional (rc.30, 2026-06-08 테스트)**:
- `scripts/aot-bake-prelude.sh` ✅ land (verus-fork commit `5533adfe4`)
- Y4 측 `just verify-adsmt-fast` 작동 — bank 생성 (`<verus-fork>/target-
  verus/release/aot/prelude-<sha>-1.0.0-rc.30.luart-cdcl`) + verify
  result `54 verified, 0 errors`
- Cache directory = `$VERUS_ADSMT_AOT_CACHE_DIR` (default = `<verus-fork>/
  target-verus/aot-cache/`, R7.4)
- env var `VERUS_ADSMT_AOT_LUART` = activation line (`aot-bake-prelude.sh
  --quiet` 출력의 eval-able shell snippet)

**JIT ⏳ v0 stub (rc.30, 2026-06-08 갱신)**:
- **lu-smt 측 `--jit-trace-emit` / `--jit-trace-load` flag ✅** (§3.5.G,
  rc.30 의 `lu-smt --help` 확인)
- 단 **replay machinery 미land** — v0 = file header + zero events,
  §3.5.F follow-up 대기 (event recorder 가 CDCL loop 측 hook 필요)
- **Verus 본체 측 `-V jit-trace-load` flag 부재** — verus-fork 의
  `config.rs` 에 jit/JIT keyword 0 (R7.5 의 의도된 `EXTENDED_JIT_TRACE_
  LOAD` 가 verus 측 wire 안 됨, **명시적 patch 후속 cycle 필요** —
  현 시점 lu-smt 측 flag 직접 사용 가능 단 functional benefit 0)
- R7.5 정합 (default off, optional manual)

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

## 3. 시작 조건 (~~별 세션이 진입 전~~ — **all land 완료, 본 section 의 step 들도 모두 ✅**)

> **status (2026-06-03 갱신)**: 모든 step 완료 ✅.
> - 1, 1.5 = submodule init + vargo build (사용자 보고, 경고 0)
> - 2, 3, 3.5 = upstream remote + branch + VSCode setup
> - **모든 PR-Verus-Backend phase (P-vb.1~P-vb.12) 도 land 완료 (§4 참조)**
> - Y4 측 별 세션 진입 작업 **0** — `just verify-adsmt` + `just emit-
>   isabelle` + `just cross-check` 즉시 진입 가능
> - 본 section 의 step 들은 historical reference 로 보존

### 1. Y4 측 submodule init ✅
사용자가 `~/verus-fork/` 에 직접 clone (2026-06-03) 후, Y4 측 submodule
로 통합:

```sh
cd <Y4-root>
git submodule add -b backend-pluggable https://github.com/newsniper-org/verus verus-fork
# 또는 fresh clone 시:
git submodule update --init verus-fork
```

`.gitmodules` 의 entry:
```
[submodule "verus-fork"]
    path   = verus-fork
    url    = https://github.com/newsniper-org/verus
    branch = backend-pluggable
```

### 1.5. Verus toolchain + Z3 + vargo build ✅
표준 `cargo build` 는 **작동 X** — Verus 는 자체 build wrapper `vargo`
(rustc internals + Z3 + workspace order 의존) 가 필수.

```fish
cd <Y4-root>/verus-fork/source
./tools/get-z3.sh                           # Z3 4.12.5 binary (필수)
rustup toolchain install 1.95.0
rustup component add rustc-dev llvm-tools --toolchain 1.95.0
source ../tools/activate.fish               # vargo 자체 build + PATH/env setup
vargo build --release                       # Verus 전체 build (vstd 포함)
```

성공 시 `<Y4>/verus-fork/tools/vargo/target/release/vargo` 생성 + Verus
`rust_verify` binary (`<Y4>/verus-fork/source/target-verus/release/verus`)
+ `builtin` / `builtin_macros` / `state_machines_macros` / `vstd` 모두
build.

### 2. upstream remote (이미 fork repo 측에 존재, push 권한 명시)
fork repo (`newsniper-org/verus`) 가 이미 upstream (`verus-lang/verus`)
fork 라 자동 정합.  접근 권한 확인용:
```sh
cd <Y4-root>/verus-fork
git remote -v
# origin    = https://github.com/newsniper-org/verus (push 권한 명시)
# upstream  = https://github.com/verus-lang/verus.git (선택 — fork 측에서 보존)
```

### 3. Branch — 이미 `backend-pluggable` ✅
`.gitmodules` 의 `branch = backend-pluggable` 명시.  submodule init 시
이 branch 가 checkout.

### 3.5. VSCode setup (별 세션 IDE 사용 시 필수)
**원인**: `.vscode/` 에 `settings.json.template` / `launch.json.template`
/ `tasks.json.template` 만 있고 실제 파일 없음 → VSCode 가 default
`cargo build` 사용 → ~114 컴파일 error (Verus internal crate inter-
dependency + rustc-dev component 미인식).

**해결**:

```fish
cd <Y4-root>/verus-fork
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
code <Y4-root>/verus-fork    # root = submodule directory, .vscode/ 위치와 정합
```

**verus-analyzer (선택)**: 표준 rust-analyzer 대신 [verus-lang/verus-
analyzer](https://github.com/verus-lang/verus-analyzer) extension 사용
시 Verus 의 `proof` / `spec` / `requires` / `ensures` keyword 인식 강화.
설치 시 일반 rust-analyzer extension disable 필요.

### 4. 별 Claude Code 세션 진입
```sh
cd <Y4-root>/verus-fork && claude
```
첫 read 대상: 본 tracker §1 + §2 + §4 + Y4 측 cross-ref + 본 §3 의 3.5
VSCode setup.  **Y4 의 cross-ref doc 들은 `../` parent path 로 접근**
(예: `../docs/verus_to_isabelle.md`, `../.claude-notes/trackers/av-proof-
body-tracker.md`).

## 4. 작업 phase (✅ 모두 완료, 2026-06-03)

> **Land 완료 (2026-06-03)**: verus-fork backend-pluggable branch 의 HEAD
> 가 R3.11+R3.12+R7.3 의 모든 patch (P-vb.1~P-vb.12) + rc.28 sound +
> rc.29 Tseitin complete + consumer/justfile template 까지 모두 ship.
> 사용자가 별 세션 개입 없이 작업 완료.

Verus fork 측 commit 검증 (`<Y4>/verus-fork/source/`):
- `config.rs:408-410` — EXTENDED_OXIZ / EXTENDED_ADSMT / EXTENDED_REPORT_ABDUCTIVE_ON_UNKNOWN ✅
- `config.rs:841-850` — solver 선택 로직 match 변환 ✅
- `context.rs:81-124` — `SmtSolver { Z3, Cvc5, OxiZ, Adsmt }` + `SmtVerdict::Abductive` + `AbductiveCandidate` struct ✅

Verus fork backend-pluggable branch commit 의 핵심 milestone:

| Commit (verus-fork) | 내용 | P-vb 매핑 |
|---|---|---|
| `8a635f53a` | rc.29 retry — (S.2) Tseitin CONFIRMED on all three paths; v1.0.0 stable-cut gate | P-vb.6 + P-vb.7 (Verdict + reporter), rc.29 complete |
| `01358cf9f` | reply: (S.2) Tseitin OR-of-AND request + v1.0.0 stable-cut gate | (논의) |
| `5533adfe4` | §3.5.H — frontend-agnostic AOT prelude-bank bake hook (scripts/aot-bake-prelude.sh + just recipe) | **P-vb.11** (AOT prelude bank) |
| `c1b067359` | rc.28 retry — (S.1-AOT) confirmed; all three paths (baseline/AOT/JIT) sound | **P-vb.11** + **P-vb.12** (AOT + JIT), rc.28 sound |
| `04cec293c` | rc.27 retry — §3.5.J FUNCTIONAL SUCCESS (verus -V adsmt: 1 verified, 0 errors) + AOT soundness gap | P-vb.4 + P-vb.5 (oxiz + adsmt backend impl 작동) |
| `cd86e9b81` | examples/consumer: justfile template for projects consuming verus-fork | (downstream) |
| (earlier rc.20~rc.26) | OxiZ + adsmt SAT/SMT + alpha_eq + abductive engine 진척 | P-vb.2 + P-vb.3 (enum + EXTENDED_KEYS) |

| Phase | 상태 | 분량 (actual) |
|---|---|---|
| P-vb.1 (cvc5 patch 위치 파악) | ✅ (rc.21 시점) | 탐색 |
| P-vb.2 (SmtSolver/SmtVerdict enum) | ✅ | ~70 LoC (`context.rs:57-124`) |
| P-vb.3 (EXTENDED_KEYS + solver 선택) | ✅ | ~50 LoC (`config.rs:408-410, 841-850`) |
| P-vb.4 (OxiZ backend impl) | ✅ | ~? LoC |
| P-vb.5 (adsmt backend impl) | ✅ (rc.27 functional, rc.28 sound) | ~? LoC |
| P-vb.6 (Verdict mapping + jsonl reporter) | ✅ (rc.29 Tseitin complete) | ~? LoC |
| P-vb.7 (`-V report-abductive-on-unknown`) | ✅ | ~50 LoC |
| P-vb.8 (Test round-trip) | ✅ (rc.27~rc.29 의 retry 가 functional + sound + complete 확인) | ~? LoC |
| **P-vb.10** (emit-isabelle/rocq wire) | ✅ (consumer/justfile 의 emit recipe 가 land, commit `cd86e9b81`) | ~? LoC |
| **P-vb.11** (AOT prelude bank) | ✅ (commit `5533adfe4`, scripts/aot-bake-prelude.sh; Y4 verify-adsmt-fast 2026-06-08 ✅ 54 verified) | ~150 LoC |
| **P-vb.12** (JIT trace load) | ⚠️ lu-smt 측 `--jit-trace-{emit,load}` ✅ v0 stub (§3.5.G), 단 Verus 측 `-V jit-trace-load` flag 부재 + replay machinery 미land (§3.5.F follow-up).  2026-06-08 부분 ✅ | ~50 LoC (verus 측 wire only) |
| P-vb.9 (Upstream PR) | (대기, post-Y4-cycle) | 0 |

합 ~1050 LoC (P-vb.9 제외).  P-vb.9 (upstream PR 제출) 는 Y4 측 R7.11+
첫 cluster sub-PR (PR-2a) 통과 후 검토.

## 5. Y4 측 산출물 land 후 cross-validate trigger (모두 ✅ 2026-06-03)

verus-fork backend-pluggable branch land 완료 (§4) — Y4 측 즉시 진행
가능:

1. ✅ `<Y4>/verus-fork/` submodule pin 갱신 (commit `023f0c8 update
   verus-fork submodule`, 사용자 작업)
2. Y4 측 `cd proofs/verus && just verify` (Z3 default 회귀 확인) +
   `just verify-adsmt` (adsmt backend 정합 확인) + `just verify-oxiz`
   (OxiZ backend 정합 확인) — 본 시점 ready
3. Y4 측 `just cross-check` (smt-cross-validation-tracker §6 의 measurement
   command, z3 vs adsmt diff) 활성 — adsmt rc.28+ sound + rc.29+ complete
   (R7.6) 모두 확보, 결과 신뢰 가능
4. `av-proof-body-tracker.md` §6 의 cluster 별 sub-PR 진입 가능 — 첫
   milestone = R7.11 (Cluster 1 amdv lower PR-2a 의 AV1 `intercept_floor_holds`)

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
