<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# AV proof body 작성 tracker (P-redesign.3, 2026-06-01)

> **목적:** AV1~AV40 (amdv 21 + power 16 active + power 4 reserved) 의
> proof body 작성 sub-cluster 별 plan + 진행 상태 record.  4 cluster =
> amdv lower (12) / amdv upper (9) / power upper (11) / power lower (5).
>
> **상태:** P-redesign.3 sign-off 2026-06-01 (R3.1~R3.12, 12 결정).
> 실제 body 작성은 PR-2 (amdv) + PR-5d (power) 안에서 cluster 별 sub-PR
> 형태로 land.  본 tracker = plan + 진행 상태 + per-cluster sub-PR 매핑
> + cross-validation timing.

## 1. 12 결정 ledger (R3.1~R3.12 sign-off)

| Item | 결정 | 비고 |
|---|---|---|
| **R3.1** 4 cluster boundary | (a) `amdv_safety.md` §5 + `power_safety.md` §4 의 Upper/Lower 분류 그대로 채택 | frozen 된 v1.0 catalog 의 classification 변경 X |
| **R3.2** Cluster 작성 순서 | (a) amdv lower → amdv upper → power upper → power lower | capsule cluster build 가용성 (PR-2 amdv → PR-5 power) 자연 순서 정합 |
| **R3.3** 분량 추정 | Upper ~300 LoC/inv (cross-tenant 복잡), Lower ~200 LoC/inv (within-cluster 단순), 합 ~9400 LoC (amdv lower 2400 + amdv upper 2700 + power upper 3300 + power lower 1000) | actual vs estimate row 추가 |
| **R3.4** File layout | (b) `amdv_safety.md` §5 표 "위치" 열의 자연 grouping (AV9+AV10 → `upper/bitmap_immut.rs`, AV11+AV12+AV13 → `lower/audit.rs`, AV14+AV15 → `upper/lifetime.rs`) | frozen 의 일부 |
| **R3.5** 의존 graph 처리 | (a) topological order — AV6 → AV1 / AV2-D → AV2 / AV23 → AV22 / AV30 → AV4 의 의존 가능한 invariant 먼저 | `lu-par` DAG-aware (`verus_to_isabelle.md` line 138 정합) |
| **R3.6** cross-validate timing | (b) cluster 완료 시 batch (smt-cross-validation-tracker §2 에 row 1 회 / cluster) — Verus 본체 patch land 후 작동 | overhead vs evidence quality 정합 |
| **R3.7** P-redesign.4 / .5 wrapper 통합 | (c) cluster 별 parallel sub-cycle (각 cluster PR 안에 Rocq + Isabelle emission 통합) | rolling X (per-AV cost ↑), sequential X (timeline ↑) |
| **R3.8** Tracker 위치 | (a) `.claude-notes/trackers/av-proof-body-tracker.md` 신설 (본 file) | long-running active tracker |
| **R3.9** PR-N scope 정합 | (b) PR-2 scope = P-redesign.3 의 amdv 2 cluster, PR-5d scope = P-redesign.3 의 power 2 cluster | PR-N 와 cluster 1:1 정합 |
| **R3.10** Lean4 backend retrofit | (a) AV proof body 의 `.lean.rs` 자동 emission, adsmt v1.1.x 도달 시 retrofit (P-redesign.4/.5 의 Rocq + Isabelle 와 짝) | `cpu_virt_compat.md` §8 (4) Lean4 watch row 의 trigger 활용 |
| **R3.11** Verus 본체 patch 위치 | (b) Verus 본체 patch 는 별도 sub-PR (Y4 의 P.3 cluster 작성과 분리, upstream contribute-back path 분리).  **(2026-06-03 갱신)** 새 `--backend=` flag 정의 X — 기존 `-V <key>` extended-multi flag mechanism (`-V cvc5` 패턴) 안에 `-V oxiz` + `-V adsmt` + `-V report-abductive-on-unknown` 추가.  `SmtSolver` enum 확장 (`Z3` / `Cvc5` + 신규 `OxiZ` + `Adsmt`).  Patch 분량 추정 ~800 → **~500 LoC** (mechanism 신설 부담 ↓, ~300 LoC 절약).  cross-validation 의 `dual` / `triple` 은 Verus 본체 flag X — Y4 측 `just verus-cross-validate` script 의 multi-invocation 로직 | upstream contribute-back path 자연성 ↑ (기존 flag pattern 정합) |
| **R3.12** Verus fork 측 adsmt third backend | (b) **opt-in 3-way** — z3 (default, no flag) + OxiZ (`-V oxiz`) default, adsmt (`-V adsmt`) 명시 시 + abductive verdict reporter (`-V report-abductive-on-unknown`).  **(2026-06-03 갱신)** flag 형식이 `-V <key>` 패턴으로 갱신 (R3.11 정합) | abductive verdict 이 가치 있는 invariant (AV5 / AV12 / AV15 / AV23 / AV24 / AV30 6 후보) 한정 |

## 2. 4 cluster boundary + file layout

### Cluster 1 — amdv lower (12 invariant, ~2400 LoC)

| AV | file | source S |
|---|---|---|
| AV1 | `lower/intercept_floor.rs` | S2 |
| AV3 | `lower/deadline.rs` | S4 |
| AV6 | `lower/gif.rs` (microkernel 측 본체) | S7 |
| AV7 | `lower/tsc.rs` | S8 |
| AV8 | `lower/nested.rs` | S9 |
| AV11 + AV12 + AV13 | `lower/audit.rs` (shared) | S12 / S12.8 / S12.5 |
| AV16 | `lower/vmcb_whitelist.rs` | §4 |
| AV18 | `lower/cluster_dep.rs` | §2.4 / §8.2 |
| AV19 | `lower/boundary.rs` | §8.3 |
| AV20 | `lower/dispatch.rs` | §8.1 |

### Cluster 2 — amdv upper (9 invariant, ~2700 LoC)

| AV | file | source S |
|---|---|---|
| AV2 + AV2-D | `upper/npt.rs` (AV2-D = Phase D placeholder) | S3.1/3.2/3.3 + S3.4 |
| AV4 | `upper/cpu_pin.rs` | S5 |
| AV5 | `upper/thread_group.rs` | S6 |
| AV9 + AV10 | `upper/bitmap_immut.rs` (shared) | S10 + S11 |
| AV14 + AV15 | `upper/lifetime.rs` (shared) | S13 + S13.6 |
| AV17 | `upper/firmware.rs` | S14 |

### Cluster 3 — power upper (11 invariant, ~3300 LoC)

| AV | file | source S/M |
|---|---|---|
| AV22 | `upper/sub_mode.rs` | M11 / §2.1 |
| AV24 | `upper/mode_invariants.rs` | M11 / M13 / §2.6 |
| AV25 | `upper/voltage_range.rs` | S23.4 |
| AV26 | `upper/magic_packet.rs` | S22.6 |
| AV27 | `upper/thermal_hardlimit.rs` | S21.8 |
| AV28 | `upper/wake_whitelist.rs` | S22.2 / S22.4 |
| AV28-D | `upper/wake_iommu.rs` (Phase D placeholder) | S22.4 + Phase D |
| AV29 | `upper/rapl_budget.rs` | S17.8 |
| AV32 | `upper/acpi_integrity.rs` | S18.8 |
| AV33 | `upper/wake_epoch.rs` | S20.4.2 |
| AV35 | `upper/boot_fix.rs` | §2.5 / M10 |

### Cluster 4 — power lower (5 invariant, ~1000 LoC)

| AV | file | source S/M |
|---|---|---|
| AV21 | `lower/tpm_consistency.rs` | S20.2.8 |
| AV23 | `lower/sub_mode_transition.rs` | M11 / §2.4 / S20.3 |
| AV30 | `lower/smt_sync.rs` | S19.2 / S19.7 |
| AV31 | `lower/dvfs_dwell.rs` | S15.5 |
| AV34 | `lower/force_mask.rs` | M5 / S19.6.4 |

### Reserved (v1.x patch)

AV36 / AV37 / AV38 / AV39 / AV40 — `power_safety.md` §1.4 위협 ledger 발견 시 채움.

## 3. 의존 graph (R3.5)

```
Cluster 1 (amdv lower)
  AV1  ─────────┐
                ▼
  AV6 (microkernel)
  AV11 ─┬─→ AV12 ─→ AV13
  (audit ordering / key-destruction chain)

Cluster 2 (amdv upper)
  AV2 ───→ AV2-D (Phase D placeholder)

Cluster 3 (power upper)
  AV22 ─→ AV23 (lower cluster 4)
  AV4 (amdv upper, cluster 2) ─→ AV30 (lower cluster 4)

Cluster 4 (power lower)
  AV21 (standalone, S20.2.8 TPM)
  AV23 (cluster 3 의 AV22 의존)
  AV30 (cluster 2 의 AV4 의존)
  AV31 (standalone, S15.5 DVFS dwell)
  AV34 (M5 force 5 개 정합, 다른 power AV 와 cross-ref)
```

cross-cluster 의존: AV23 / AV30 의 upper-side AV (AV22 / AV4) 가 먼저
land 되어있어야 lower-side body 의 AEAD integrity / SMT pair 정합 본문
작성 가능.  R3.2 의 순서 (cluster 1 → 2 → 3 → 4) 가 이 의존 graph 와
정합.

## 4. PR-N scope 매핑 (R3.9)

| PR | scope | sub-PR 형태 |
|---|---|---|
| **PR-2** (amdv proof body land) | Cluster 1 (amdv lower) + Cluster 2 (amdv upper) | PR-2a = cluster 1, PR-2b = cluster 2 |
| **PR-5d** (power proof body land) | Cluster 3 (power upper) + Cluster 4 (power lower) | PR-5d.1 = cluster 3, PR-5d.2 = cluster 4 |
| **PR-Verus-Backend** (Verus 본체 patch, R3.11) | z3 + OxiZ + adsmt 3-way trait 통일 + verdict mapping + `--report-abductive-on-unknown` reporter | 별도 sub-PR, P.3 cluster 작성과 분리.  upstream contribute-back path |

## 5. Per-cluster sub-PR 내부 작업 항목 (R3.7 정합)

각 cluster sub-PR 안에서:

1. **Verus proof body 작성** (`proofs/verus/src/<domain>/<upper|lower>/<file>.rs`)
2. **Rocq theory 통합** (P-redesign.4 R4.2=b — `~/y4-verus2rocq/` sibling
   도구가 adsmt-emit-rocq wrapper 로 `.v` emission; nested directory
   naming `theories/<Domain>/<Module>.v`, R4.6; Ltac2-only enforcement,
   R4.5; cluster 별 rolling, R4.7 — `docs/verus_to_rocq.md` 참조)
3. **Isabelle theory 통합** (P-redesign.5 R5.2 — `~/y4-verus2isabelle/`
   sibling 도구가 adsmt-emit-isabelle wrapper 로 `.thy` emission; flat
   underscore naming `Y4_<Domain>_<Module>.thy`, R5.4; Lean4 backend 제외
   R5.3 — `docs/verus_to_isabelle.md` §3.2 참조)
4. **cross-validation row** (cluster 완료 시 smt-cross-validation-tracker §2 에 row 추가, R3.6 / R3.12)
5. **본 tracker §6 의 진행 상태 row 갱신**

## 6. Per-cluster 진행 상태

| Cluster | LoC estimate | LoC actual | sub-PR | 시작 | 완료 | cross-val row |
|---|---|---|---|---|---|---|
| Cluster 1 (amdv lower) | ~2400 | — | PR-2a | — | — | — |
| Cluster 2 (amdv upper) | ~2700 | — | PR-2b | — | — | — |
| Cluster 3 (power upper) | ~3300 | — | PR-5d.1 | — | — | — |
| Cluster 4 (power lower) | ~1000 | — | PR-5d.2 | — | — | — |

## 7. R3.12 — Verus fork 측 adsmt third backend 의 활성 invariant 후보

abductive verdict 이 가치 있는 (z3/OxiZ 가 `unknown` 으로 timeout 가능성
↑ + higher-order quantifier 자연) invariant:

| AV | cluster | 사유 |
|---|---|---|
| AV5 `parent_thread_group_pinned` | amdv upper | S6.5 의 (cspace ∧ vspace) 일치 + ChangeParent atomicity 의 higher-order quantifier |
| AV12 `audit_per_cpu_order` | amdv lower | ∀ entry e ∃ read view r 의 alternating quantifier (∀∃ 형태) |
| AV15 `orphan_frame_absent` | amdv upper | ∀ frame f ∃ cap c 의 alternating quantifier + revoke chain |
| AV23 `sub_mode_transition_atomicity` | power lower | AEAD integrity check + 모든 lease suspended 의 conjunction quantifier |
| AV24 `mode_invariant_holds` | power upper | named sub-mode definition 의 polymorphic quantifier (defined → invariant 적용) |
| AV30 `smt_pair_power_state_sync` | power lower | SMT pair primary/sibling 의 pairwise quantifier |

위 6 invariant 가 default + adsmt opt-in 측정 (`--backend=adsmt` 명시).
나머지 31 invariant 는 z3 + OxiZ 만으로 충분 (first-order).

## 8. Lean4 backend retrofit watch (R3.10)

adsmt v1.1.x 도달 시점 (현재 leo4 v1.0.0-rc.4 + adsmt-lean-binding tag
정합 + L4 mslean4 path 의 `feat/mslean4-lecq-lecr-ipcs` branch 완료
대기):

1. P-redesign.4 의 adsmt-emit-rocq 와 짝지어 **adsmt-emit-lean** wrapper 도 정의
2. `verus_to_isabelle.md` §3.2 의 file path 표에 `<file>.lean.rs` 매핑 추가
3. 각 cluster 의 sub-PR retrofit 시 `.lean` emission 도 land
4. `cpu_virt_compat.md` §8 (4) Lean4 watch row 의 trigger 조건 충족 — 본
   tracker 의 §1 R3.10 결정 활성

watch 위치: `adsmt-integration-tracker.md` §10.5 (Lean4 / OxiLean 별도
watch) + `cpu_virt_compat.md` §8 (4).

## 9. cross-validation 측정 (R3.6 → smt-cross-validation-tracker §2)

cluster 별 batch — 각 cluster 의 모든 AV body 완료 후 1 회:

1. Verus 본체 patch (R3.11) land 확인
2. `just verus-cross-validate --cluster=<cluster-name>` 실행 (R6.6 의 3-tier env × R3.12 의 3 backend)
3. smt-cross-validation-tracker §2 baseline 표에 row 1 회 추가
4. 본 tracker §6 의 cross-val row 갱신

## 10. 갱신 path

- §6 per-cluster 진행 상태: cluster sub-PR 시작/완료 시
- §7 R3.12 invariant 후보: adsmt 의 abductive verdict 활용 결과 (z3/OxiZ
  unknown → adsmt 의 ranked hypothesis) 가 useful 한 경우 add/remove
- §8 Lean4 retrofit: adsmt v1.1.x 도달 시 1 회 sweep
- §3 의존 graph: 작성 중 발견되는 cross-cluster 의존 발견 시 row 추가
- Reserved AV (AV36~AV40): `power_safety.md` §1.4 위협 ledger 발견 시
  채움
