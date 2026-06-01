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
| **R6.7** dual backend 실행 | separate runs (Z3 단독 → OxiZ 단독 → diff) | clean baseline + side-effect 없음 |
| **R6.8** logicutils-driven verification | `freshcheck --method=hash` + `stamp record` + `lu-par --transaction` + `.lu-store/` artifact tarball | paper artifact §6.5 (vii) 정합 |
| **R6.9** 측정 결과 위치 | 본 tracker (summary) + `/home/ybi/y4-paper-artifact/microbench/` (raw data, Phase C 종반 신설) | tracker = compact summary, raw data = full |
| **R6.10** 측정 cycle | adsmt version bump 마다 자동 + paper artifact submission 시점 1 회 | rolling testing channel 정합 |

## 2. Baseline 추적 — adsmt testing channel HEAD 별

각 adsmt testing HEAD 시점의 51 verified invariant 의 z3 vs OxiZ 측정
결과 summary:

| Date | adsmt commit | Verus version | OxiZ version | 51 verified | Z3 평균 solve_time_us | OxiZ 평균 solve_time_us | result diff |
|---|---|---|---|---|---|---|---|
| (대기) | (첫 측정 시점에 row 추가) | — | — | — | — | — | — |

## 3. Per-invariant detail (sample size N=30, Tukey-trimmed)

`/home/ybi/y4-paper-artifact/microbench/per_invariant_<adsmt_commit>.jsonl`
에 raw JSON Lines.  본 tracker 의 summary 는 outlier (≥ p95 diff) 만:

| invariant | adsmt commit | Z3 vs OxiZ result | diff 사유 |
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
sample measurement command (Phase C 진입 후):
just verus-cross-validate
  → 1) freshcheck --method=hash unified-toolkit-pin.lock proofs/verus/src/
       (skip if unchanged)
  → 2) for env in [native, qemu-smoke, KernelDebugBuild=ON]:
         for backend in [z3, oxiz]:
             lu-par --transaction --jobs=N
               -- "verus --backend=$backend"
                  (capture metric → jsonl)
  → 3) stamp record --method=hash result.jsonl
  → 4) summary row append to this tracker
  → 5) raw data → ~/y4-paper-artifact/microbench/
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

- §2 baseline row: adsmt version bump 마다 (R6.10 auto)
- §3 per-invariant outlier: 본 sweep 의 outlier 발견 시
- §4 환경 별: critical milestone 마다
- 결과 mismatch (Z3 vs OxiZ result diff != 0) 발생 시:
  1. Y4 측 invariant 의 problem 분석 (specification 측 모호)
  2. OxiZ 측 issue file (Honey-Be/oxiz / cool-japan/oxiz)
  3. tracker §3 의 row 에 mismatch 본문 + GitHub issue link
