<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Verus ↔ seL4 Isabelle/HOL 통합 tracker (R7, 2026-06-03)

> **목적**: Y4 의 Verus invariant (AV1~AV40, 4 cluster) 의 verify 결과를
> seL4 의 기존 Isabelle/HOL 코드 (L4.verified, https://github.com/seL4/l4v,
> Y4 외) 와 같은 verification logic 으로 통합.
>
> **상태**: R7 sign-off 2026-06-03.  Y4 측 spec/tracker 갱신 + sibling repo
> 폐기 + consumer pattern adapt 완료.  PR-Verus-Backend (별 세션) land
> 후 첫 emit milestone 활성 가능.
>
> **Plan ledger**: `~/.claude/plans/jazzy-gliding-puppy.md`

## 1. R7.1~R7.12 결정 ledger

| Item | 결정 | 상태 |
|---|---|---|
| **R7.1a** docs/verus_to_rocq.md 삭제 (P-redesign.4 산출물 revert) | git rm | ✅ |
| **R7.1b** docs/verus_to_isabelle.md §3 supersede note (§3.6 만 유효) | §3 introduction + §3.2 header 의 supersede note | ✅ |
| **R7.1c** proofs/coq/README.md sibling reference revert (R4.1 manual 3 theory 유지) | adsmt-emit-rocq 통합 section rewrite + 도구 row supersede | ✅ |
| **R7.1d** adsmt-integration-tracker §10.3 P-redesign.4/.5 superseded mark | ⚠️ "superseded by R7 sign-off (2026-06-03)" | ✅ |
| **R7.2** proofs/verus/justfile consumer pattern adapt | 8 recipes: verify / verify-adsmt / verify-cvc5 / verify-oxiz / verify-adsmt-fast / aot-bake / cross-check / emit-isabelle / emit-rocq | ✅ |
| **R7.3** PR-Verus-Backend scope 확장 (emit + AOT + JIT) | pr-verus-backend-tracker §1.5 + §1.6 + §4 P-vb.10/11/12 + 분량 ~500 → ~850 LoC | ✅ |
| **R7.4** AOT prelude bank cache directory | default `<verus-fork>/target-verus/aot-cache/`, env override `VERUS_ADSMT_AOT_CACHE_DIR` | ✅ |
| **R7.5** JIT trace load | `-V jit-trace-load` + `--jit-trace-load=<path>` path flag (default off, optional manual) | ✅ |
| **R7.6** branch pin 정책 + soundness milestone | testing branch (rolling) — adsmt = `03f33a9` (rc.29), adsmt-contrib = `33349dc` (rc.28).  rc.28+ sound + rc.29+ complete 모두 ✅.  stable v1.0.0 release 시 stable-v1 channel 로 전환 예정 | ✅ |
| **R7.7** .thy 산출물 위치 | `<Y4>/proofs/isabelle/` 신설 — 2-layer imports chain (Layer 1 cluster sub-grouping `Y4_AmdvSafety.thy` + `Y4_PowerSafety.thy` + Layer 2 per-AV flat list + top-level `Y4.thy` 가 양 layer 모두 imports) | ✅ |
| **R7.8** .v (Rocq) 산출물 위치 | `<Y4>/proofs/coq/theories/Generated/.gitkeep` 신설 + `_CoqProject` glob 추가 + R4.1 manual 3 theory (theories/{Sel4,IPC,Lease}/*.v) 별도 위치 | ✅ |
| **R7.9** Cross-check CI 통합 | proofs/verus/justfile 의 cross-check recipe + av-proof-body-tracker §5 cluster 별 batch (R3.6 정합) + smt-cross-validation-tracker §2 row 추가 | ✅ (recipe) |
| **R7.10** L4.verified inbound contract trust marker | verus_to_isabelle.md §1.7 갱신 — PR-Verus-Backend land + adsmt rc.28+/rc.29+ 도달 후 trust 가능 명시 | ✅ |
| **R7.11** 첫 emit milestone | Cluster 1 (amdv lower) PR-2a + AV1 `intercept_floor_holds` + `Y4_AmdvSafety_Lower_InterceptFloor.thy` | ⚠️ **blocked by adsmt declare-datatypes (2026-06-04)** — AV1 proof body 작성 + Z3 verify (54 verified, 0 errors) ✅, 단 `just verify-adsmt` 가 lu-smt 의 parameterized constructor 미지원으로 fail.  Request 신설: `.local-requests-to/adsmt/2026-06-04-declare-datatypes-parameterized.md` |
| **R7.12** Verification end-to-end | vargo build → just verify-adsmt → just emit-isabelle → just coq → just cross-check → (manual) l4v import | ⚠️ R7.11 block 으로 일부 step 보류 (`just verify` Z3 default ✅) |

## 2. Per-cluster emission 진행 record

| Cluster | sub-PR | Verus proof body | `.thy` emit | `.v` emit | cross-check |
|---|---|---|---|---|---|
| Cluster 1 (amdv lower) | PR-2a | AV1 ✅ (intercept_floor.rs, Z3 54 verified) | ⚠️ block (adsmt declare-datatypes) | ⚠️ block | ⚠️ block |
| Cluster 2 (amdv upper) | PR-2b | (대기) | (대기) | (대기) | (대기) |
| Cluster 3 (power upper) | PR-5d.1 | (대기) | (대기) | (대기) | (대기) |
| Cluster 4 (power lower) | PR-5d.2 | (대기) | (대기) | (대기) | (대기) |

각 cluster sub-PR 의 단일 row 갱신 = av-proof-body-tracker §5 의 6 step
정합 (Verus → verify-adsmt → emit-isabelle → emit-rocq → cross-check
→ 본 tracker §2 + §6 갱신).

## 3. L4.verified inbound milestone (사용자 manual)

R7.10 정합 — l4v repo 의 Y4 측 통합 X.  seL4 팀이 자체 환경에서 import.

| Milestone | 시점 | 작업 |
|---|---|---|
| 1. PR-Verus-Backend land | ✅ **2026-06-03** | verus-fork backend-pluggable HEAD 가 R3.11+R3.12+R7.3 의 모든 patch + AOT + JIT + emit recipe 모두 ship.  pr-verus-backend-tracker §4 P-vb.1~P-vb.12 모두 ✅ |
| 2. adsmt rc.28+ + rc.29+ 도달 | ✅ 2026-06-03 | rc.29 = `03f33a9` |
| 3. 첫 emit milestone (R7.11) | (진행 중, 2026-06-03) | AV1 `.thy` 생성 + l4v import 사용자 manual 확인 |
| 4. Cluster 1 (amdv lower) full emit | (대기, 3 후) | PR-2a land + 12 AV 의 `.thy` |
| 5. Cluster 2 (amdv upper) full emit | (대기, 4 후) | PR-2b land + 9 AV 의 `.thy` |
| 6. Cluster 3 (power upper) full emit | (대기, Phase C) | PR-5d.1 land + 11 AV 의 `.thy` |
| 7. Cluster 4 (power lower) full emit | (대기, Phase C) | PR-5d.2 land + 5 AV 의 `.thy` |
| 8. Paper artifact 제출 (Phase C 종반) | (대기) | proofs/isabelle/ snapshot + unified-toolkit-pin.lock + verus-fork submodule pin 의 hash chain |

## 4. Architecture (R7 sign-off, single source of truth = Verus fork)

```
Y4 측 Verus invariant
(proofs/verus/src/{amdv,power}/{upper,lower}/*.rs)
    │
    │ just verify-adsmt   (proofs/verus/justfile)
    ▼
verus-fork/source/target-verus/release/verus -V adsmt
(PR-Verus-Backend, R3.11+R3.12+R7.3 scope)
    │
    │ (Verus fork 내부 adsmt 연동 + cert JSON 자동 생성)
    ▼
adsmt cert JSON (target/adsmt-cert/<crate>.cert.json)
    │
    │ just emit-isabelle  →  adsmt-emit-isabelle <cert>.json > <Y4>/proofs/isabelle/Y4_*.thy
    │ just emit-rocq      →  adsmt-emit-rocq <cert>.json > <Y4>/proofs/coq/theories/Generated/<file>.v
    ▼
.thy (proofs/isabelle/Y4_*.thy)    .v (proofs/coq/theories/Generated/*.v)
    │
    │ (paper artifact 측 첨부 + l4v 팀이 자체 import — R7.10 사용자 manual)
    ▼
seL4 L4.verified (https://github.com/seL4/l4v) — Y4 외, 사용자 manual
```

## 5. Cross-ref

- `~/.claude/plans/jazzy-gliding-puppy.md` (본 plan 전체 결정 ledger)
- `<Y4>/docs/verus_to_isabelle.md` §1.7 (inbound contract trust marker
  R7.10) + §3 (R7.1b supersede note) + §2.6 (theory file naming) + §3.6
  (unified-toolkit-pin)
- `<Y4>/proofs/verus/justfile` (R7.2 consumer pattern)
- `<Y4>/proofs/isabelle/README.md` (R7.7 layout + l4v inbound)
- `<Y4>/proofs/isabelle/Y4.thy` + `Y4_AmdvSafety.thy` + `Y4_PowerSafety.thy`
  (R7.7 Layer 1 + top-level)
- `<Y4>/proofs/coq/_CoqProject` (R7.8 Generated/ glob)
- `<Y4>/proofs/coq/theories/Generated/` (R7.8 emit destination)
- `<Y4>/unified-toolkit-pin.lock` (R7.6 baseline rc.29)
- `<Y4>/.claude-notes/trackers/pr-verus-backend-tracker.md` §1.5 / §1.6
  / §4 P-vb.10/11/12 (R7.3 scope 확장)
- `<Y4>/.claude-notes/trackers/av-proof-body-tracker.md` §5 (R7 per-
  cluster 6 step 갱신)
- `<Y4>/.claude-notes/trackers/smt-cross-validation-tracker.md` §6
  (R7 measurement command 갱신)
- `<Y4>/.claude-notes/trackers/adsmt-integration-tracker.md` §7 (R7
  watch row) + §10.3 (P-redesign.4/.5 superseded mark)
- `<Y4>/CLAUDE.md` §5 (proofs/isabelle/ row + verus-fork R7.3 scope 명시)

## 6. 갱신 path

- §2 cluster row: cluster sub-PR (R3.7 정합) 의 6 step 완료 시
- §3 milestone: 사용자 명시 시점에 추가 (paper artifact 제출, l4v import 사용자 확인 등)
- §1 R7.X 결정의 미해결 (R7.11/R7.12) 의 ✅ 마킹: 첫 emit milestone 완료 시 (PR-Verus-Backend land 는 이미 ✅ 2026-06-03)
- R7.6 baseline commit hash: rolling (사용자 명시 시점에 `.lock` 갱신)

## 7. 미해결

1. **adsmt declare-datatypes parameterized constructors (active, 2026-06-04)**
   — Y4 R7.11 milestone block.  lu-smt v1.0.0-rc.29 의 SMT-LIB parser
   가 nullary constructor 만 지원.  Verus vstd 가 발화하는 parameterized
   `(declare-datatypes ...)` parse fail.  Request 신설: `.local-requests-
   to/adsmt/2026-06-04-declare-datatypes-parameterized.md` (mirror:
   `~/AD1/.local-requests-from/Y4/2026-06-04-declare-datatypes-
   parameterized.md`).  reply 도달 + lu-smt patch land 후 `.lock` baseline
   갱신 + R7.11 milestone 재시도
2. **paper artifact 첨부 메커니즘** — `proofs/isabelle/*.thy` snapshot +
   verus-fork submodule pin + adsmt commit pin 의 hash chain (Phase C
   종반 sub-cycle)
2. **L4.verified 측 spec 과의 explicit connection** — Y4 의 AV invariant
   의 assumes/shows 가 l4v 의 어떤 spec lemma (KernelSchedule_A, KernelMmu_A
   등) 와 짝지을지 (Phase C 이후 의제로 deferred)
3. **`proofs/coq/theories/Generated/` 와 manual `theories/<Domain>/
   <Module>.v`** 의 import 관계 (R4.1 의 3 theory 가 generated 측 import
   하는지)
