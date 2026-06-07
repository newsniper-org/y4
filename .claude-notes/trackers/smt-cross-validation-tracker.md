<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# SMT cross-validation tracker (P-redesign.6, 2026-06-01)

> **목적:** Verus 의 z3 ↔ OxiZ dual backend (P-redesign.2 §3.6) 의
> cross-validation 실험 plan + 측정 결과 record.  paper artifact (vmm_arch
> §6.5 / power_arch §6.5) 의 Reproducible 자격 evidence + 학술적 차별점
> §6.1.8 (Y4 가 unified verification toolkit 의 첫 industrial adoption)
> 의 quantitative support.

> **상태:** P-redesign.6 sign-off 2026-06-01.  baseline 측정은 첫 Verus
> dual backend 작업 (P-redesign.2 본격 구현) 이 본격 진입 후 — 본
> tracker 는 plan + schema + cycle policy 만 sign-off, 실측 row 는 후속.

## 1. 실험 design 결정 (R6.1~R6.10 sign-off)

| Item | 결정 | 비고 |
|---|---|---|
| **R6.1** invariant 범위 | 51 verified 전체 (alloc 18 + ipc 18 + capsules 11 + amdv 1 + error 1 + top-level 2) | 현 시점 verifiable 모두.  AV catalog (statement-only frozen) 는 proof body 채움 (P-redesign.3 후) 추가 |
| **R6.2** 측정 metric | 6 metric: `solve_time_us` / `proof_cert_size_bytes` / `peak_memory_mib` / `parallelism` (single vs N-thread) / `cache_miss_rate` / `result` (sat/unsat/timeout) | 통계 + 성능 + reproducibility evidence 동시 |
| **R6.3** sample size | N=30 per invariant, Tukey-trimmed mean + p95 + std-dev | 통계적 유의성 |
| **R6.4** 산출물 형식 | JSON Lines (jsonl) + Apache Arrow IPC | streaming + columnar query |
| **R6.5** Visualization | typst (paper artifact 측 typst 통합) | logicutils tutorial 의 typst 패턴 정합 |
| **R6.6** 실행 환경 | 3-tier: native + qemu-smoke + KernelDebugBuild=ON | 3 환경 cross-comparison |
| **R6.7** dual backend 실행 | separate runs (Z3 단독 → OxiZ 단독 → adsmt 단독 → 3-way diff) — **R3.12 sign-off (2026-06-01) 후 3-way 확장** | clean baseline + side-effect 없음.  adsmt third backend 는 R3.12 opt-in (6 invariant 후보: AV5/12/15/23/24/30, `av-proof-body-tracker.md` §7) |
| **R6.8** logicutils-driven verification | `freshcheck --method=hash` + `stamp record` + `lu-par --transaction` + `.lu-store/` artifact tarball | paper artifact §6.5 (vii) 정합 |
| **R6.9** 측정 결과 위치 | 본 tracker (summary) + `/home/ybi/y4-paper-artifact/microbench/` (raw data, Phase C 종반 신설) | tracker = compact summary, raw data = full |
| **R6.10** 측정 cycle | adsmt version bump 마다 자동 + paper artifact submission 시점 1 회 | rolling testing channel 정합 |

## 2. Baseline 추적 — adsmt testing channel HEAD 별

각 adsmt testing HEAD 시점의 51 verified invariant 의 z3 vs OxiZ 측정
결과 summary:

| Date | adsmt commit | Verus version | OxiZ version | 51 verified | Z3 평균 solve_time_us | OxiZ 평균 solve_time_us | adsmt 평균 solve_time_us (opt-in 6 inv) | result diff |
|---|---|---|---|---|---|---|---|---|
| (대기) | (첫 측정 시점에 row 추가) | — | — | — | — | — | — | — |

## 3. Per-invariant detail (sample size N=30, Tukey-trimmed)

`/home/ybi/y4-paper-artifact/microbench/per_invariant_<adsmt_commit>.jsonl`
에 raw JSON Lines.  본 tracker 의 summary 는 outlier (≥ p95 diff) 만:

| invariant | adsmt commit | Z3 vs OxiZ vs adsmt result | diff 사유 |
|---|---|---|---|
| (대기) | (outlier 발견 시 row 추가) | — | — |

## 4. 실행 환경 별 비교 (R6.6)

3-tier env 별 측정 (per-adsmt-commit):

| env | 의미 | 측정 frequency |
|---|---|---|
| native | host CPU, no virtualization | 매 측정 |
| qemu-smoke | Y4 capsule cluster simulated (`just qemu-smoke`) | 매 측정 |
| KernelDebugBuild=ON | sel4 측 timing trace (sel4_fork_policy §3.6 G6 와 짝) | adsmt version bump 마다 + paper artifact submission |

## 5. 측정 trigger

- **adsmt version bump auto**: adsmt testing channel HEAD 변경 시 (rolling)
  → CI 가 자동 측정 + 본 tracker §2 에 row 추가
- **Manual cycle**: paper artifact submission 직전 1 회 — 모든 env +
  invariant 의 full sweep
- **Critical milestone**: adsmt v1.0.0 stable release 도달 시 1 회
  comprehensive — paper artifact 의 reference baseline

## 6. Reproducibility 보장 (R6.8)

```
sample measurement command (R7 sign-off 2026-06-03 갱신 — consumer pattern):

# Y4 측 proofs/verus/justfile (R7.2) 의 cross-check + AOT recipe 활용.
# Verus fork (R3.11+R3.12+R7.3) + adsmt rc.28+ + rc.29+ 도달 후 작동.

cd proofs/verus

# Single-backend verify (per-env / per-backend)
just verify                  # Z3 default
just verify-adsmt            # -V adsmt (sound rc.28+, complete rc.29+)
just verify-oxiz             # -V oxiz
just verify-adsmt-fast       # -V adsmt + AOT prelude bank (R7.4)

# Differential audit (z3 vs adsmt — R7.9, cluster 별 batch R3.6)
just cross-check             # 출력 = "z3: <z3-res>" / "adsmt: <ad-res>"
                             # diff 0 시 ✓ backends agree, mismatch 시 ✗ DIVERGENCE

# R3.12 opt-in (6 invariant — AV5/12/15/23/24/30) — abductive verdict
just verify-adsmt -- -V report-abductive-on-unknown
                             # adsmt 측 Unknown 시 ranked hypothesis JSON emit

# Per-env (R6.6 — native / qemu-smoke / KernelDebugBuild=ON) 는 wrapper
# script 가 위 recipe 를 env 별로 invoke.  본 tracker §2 의 row 추가는
# wrapper script 의 후속 step.
```

## 7. 학술 paper §6.1.8 evidence link

본 tracker 의 측정 data 가 `power_arch.md` §6.1.8 의 차별점 8 (logicutils
+ adsmt + OxiZ 의 unified verification toolkit 첫 industrial adoption)
의 quantitative support.  paper artifact §6.5 (vii) logicutils-driven
artifact verification 의 input.

특히:
- **OxiZ ↔ Z3 result diff = 0** 이 차별점 "100% Z3 parity" claim 의
  reproducibility evidence
- solve_time_us 비교 (Z3 = Microsoft binary, OxiZ = pure-Rust) 가 산업
  차별점 §6.2 의 supply chain integrity 의 quantitative 보강

## 8. 갱신 path

- §2 baseline row: adsmt version bump 마다 (R6.10 auto) + cluster 완료 시
  (P-redesign.3 R3.6, `av-proof-body-tracker.md` §9)
- §3 per-invariant outlier: 본 sweep 의 outlier 발견 시
- §4 환경 별: critical milestone 마다
- 결과 mismatch (Z3 vs OxiZ vs adsmt result diff != 0) 발생 시:
  1. Y4 측 invariant 의 problem 분석 (specification 측 모호)
  2. mismatch 위치 별 issue file:
     - z3 vs OxiZ diff → OxiZ 측 issue (Honey-Be/oxiz / cool-japan/oxiz)
     - z3/OxiZ vs adsmt diff → adsmt 측 issue (newsniper-org/adsmt) + Verus
       본체 patch (R3.11) 의 verdict mapping 측 review
  3. tracker §3 의 row 에 mismatch 본문 + GitHub issue link

## 9. R3.12 — adsmt opt-in third backend (2026-06-01, flag 형식 갱신 2026-06-03)

P-redesign.3 R3.12 sign-off 로 adsmt 가 Verus fork 측 third backend 로
추가됨 (`av-proof-body-tracker.md` §1 R3.12).  **2026-06-03 갱신**: flag
형식이 새 `--backend=` 가 아닌 기존 `-V <key>` extended-multi pattern
정합 (Verus 의 `-V cvc5` 패턴 mirror) — `pr-verus-backend-tracker.md`
§1.2 참조.

**Opt-in invariant 6 (abductive verdict 활용 후보)**:

| AV | cluster | 활용 사유 |
|---|---|---|
| AV5 `parent_thread_group_pinned` | amdv upper | S6.5 (cspace ∧ vspace) higher-order |
| AV12 `audit_per_cpu_order` | amdv lower | ∀ entry ∃ read view alternating quantifier |
| AV15 `orphan_frame_absent` | amdv upper | ∀ frame ∃ cap alternating quantifier + revoke chain |
| AV23 `sub_mode_transition_atomicity` | power lower | AEAD integrity + lease suspended conjunction |
| AV24 `mode_invariant_holds` | power upper | named sub-mode polymorphic quantifier |
| AV30 `smt_pair_power_state_sync` | power lower | SMT pair pairwise quantifier |

**Verus 본체 patch dep (R3.11)**: `-V adsmt` + `-V report-abductive-on-
unknown` flag 추가 land 후 작동 (기존 `-V <key>` extended-multi mechanism
활용, `SmtSolver` enum 에 `Adsmt` variant 추가).  z3 (default, no flag) /
OxiZ (`-V oxiz`) default 측 cross-validation 은 patch 의 OxiZ variant +
EXTENDED_OXIZ key 만 land 되면 우선 시작 가능.

**Abductive verdict 처리** (2026-06-03 schema 정합 강화 — adsmt
v1.0.0-rc.7 의 native ranking layer 정합): adsmt 의 `Verdict::Unknown`
또는 `Verdict::Abductive` 시점에 Verus 측 reporter 가 ranked
candidate JSON 을 emit.

```json
{
  "invariant":  "AV15_orphan_frame_absent",
  "backend":    "adsmt",
  "verdict":    "unknown",
  "abductive_candidates": [
    {
      "rank":         1,
      "score":        1.025,
      "hypotheses":   ["∀ c, revoked(c) ⟹ ¬ alive(owner(c).frame)"],
      "explanations": [null],
      "sources":      ["abducible-frame-revoke-chain"]
    },
    {
      "rank":         2,
      "score":        2.013,
      "hypotheses":   ["frame_alloc.linear_chain(host_memory)"],
      "explanations": [null],
      "sources":      ["abducible-linear-chain"]
    }
  ]
}
```

**Per-candidate schema** (adsmt-abduce v1.0 정합):

| field | type | 의미 |
|---|---|---|
| `rank` | `u32` | 1-based ranking (1 = top) |
| `score` | `f64` | `adsmt-abduce::rank::rank_candidates` 의 score — **smaller = stronger** (v0.1: `hypotheses.len() + 0.001 * depth()`; future: domain `RankPolicy` 통한 vocabulary preference / salience weights — Q17 sec 20) |
| `hypotheses` | `[String]` | candidate 의 hypothesis list (1 candidate = N hypothesis conjunction).  adsmt 의 `Candidate.hypotheses: Vec<Term>` 를 `Display` impl 로 serialize |
| `explanations` | `[String \| null]` | per-hypothesis explanation, optional.  adsmt 의 `Candidate.explanations: Vec<Option<String>>` 정합 |
| `sources` | `[String]` | per-hypothesis source identifier (`abducible-...` 등).  adsmt 의 `Candidate.sources: Vec<String>` 정합 |

`hypotheses` / `explanations` / `sources` 는 동일 길이 (lock-step) — 같은 index 가 같은 hypothesis 의 (term, optional explanation, source) triple.

candidates list 가 Y4 측 invariant 강화 candidate 의 input — `av-proof-body-tracker.md`
§6 의 actual LoC 가 expected 보다 클 경우 candidate 추가 후 LoC 감소
가능성.

**Paper artifact §6.5 (vii) self-application evidence**: adsmt 가 자체
Verus backend 의 *third option* 으로 활용 = unified toolkit 의 self-host
조립.
