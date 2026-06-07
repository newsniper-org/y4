<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Isabelle/HOL theory 산출물

Y4 가 Verus invariant (AV1~AV40 + alloc/ipc/capsules 51 verified) 를
adsmt cert 측으로 verify 한 후 `adsmt-emit-isabelle` CLI 가 cert →
`.thy` 변환한 결과.

L4.verified (`https://github.com/seL4/l4v`, Y4 외) 측 통합의 entry —
seL4 팀이 자체 Isabelle 환경에서 본 디렉터리의 `Y4.thy` 또는 cluster
sub-grouping (`Y4_AmdvSafety.thy` / `Y4_PowerSafety.thy`) 또는 individual
AV theory 를 cherry-pick import 가능.

## 생성 흐름 (R7 sign-off 2026-06-03)

```
proofs/verus/src/{amdv,power}/{upper,lower}/*.rs   (Verus invariant)
    │
    │ cd proofs/verus && just verify-adsmt
    │   → <Y4>/verus-fork/source/target-verus/release/verus -V adsmt
    │      (PR-Verus-Backend land 후, R3.11+R3.12 / R7.3 scope)
    │   → target/adsmt-cert/<crate>.cert.json (cert JSON 자동 생성)
    │
    │ cd proofs/verus && just emit-isabelle
    │   → adsmt-emit-isabelle <cert>.json > proofs/isabelle/Y4_*.thy
    │      (adsmt-contrib testing branch pin, R7.6)
    ▼
proofs/isabelle/Y4_*.thy   (본 디렉터리)
```

## Theory file layout (R7.7 — 2-layer imports chain)

**Layer 1 — cluster sub-grouping** (이 layer 가 cluster 단위 import):
- `Y4_AmdvSafety.thy` — amdv cluster (21 AV) 의 모든 sub theory imports
- `Y4_PowerSafety.thy` — power cluster (16 active AV) 의 모든 sub theory imports

**Layer 2 — per-Verus-module 1 .thy** (verus_to_isabelle.md §2.6 정합,
flat underscore naming):
- amdv lower (12): `Y4_AmdvSafety_Lower_<Module>.thy`
- amdv upper (9):  `Y4_AmdvSafety_Upper_<Module>.thy`
- power upper (11): `Y4_PowerSafety_Upper_<Module>.thy`
- power lower (5):  `Y4_PowerSafety_Lower_<Module>.thy`

예: `Y4_AmdvSafety_Lower_InterceptFloor.thy` (AV1), `Y4_AmdvSafety_Upper_Npt.thy`
(AV2+AV2-D), `Y4_AmdvSafety_Lower_Audit.thy` (AV11+AV12+AV13, shared).

**Top-level — `Y4.thy`**:
```isabelle
theory Y4
  imports
    Main
    (* Layer 1 — cluster sub-grouping *)
    Y4_AmdvSafety
    Y4_PowerSafety
    (* Layer 2 — flat list (cherry-pick 옵션) *)
    Y4_AmdvSafety_Lower_InterceptFloor
    Y4_AmdvSafety_Upper_Npt
    (* ... 모든 per-AV theory *)
begin
end
```

**양 layer 공존 의도**: seL4 팀이 cluster 단위 (Layer 1) 로 일괄 import
하거나 individual AV (Layer 2) 로 cherry-pick 모두 가능.  top-level
`Y4.thy` 가 양 layer 모두 imports 하므로 어느 쪽이든 single entry.

## Dependency

본 디렉터리의 모든 `.thy` 의 단일 dependency:
```isabelle
imports Main
```

Isabelle `Main` 측 classical machinery (HOL.Classical, HOL.Eq 등) 만
사용 — 외부 sessions / AFP entry / l4v 측 theory 모두 X.  seL4 팀이
자체 환경에서 `imports` chain 의 추가 없이 본 디렉터리만 ROOTS 경로에
추가하면 build.

`adsmt-emit-isabelle` 의 classical axiom 처리 (`~/adsmt-contrib/adsmt-
emit-isabelle/src/lib.rs:39~73`) 가 보장 — Isabelle `Main` 이 이미
import 하므로 추가 imports line 0.

## l4v 측 inbound contract

`verus_to_isabelle.md` §1.7 의 6 step 정합:

1. Y4 가 PR-4 (paper artifact) 의 `.thy` 산출물 + `unified-toolkit-pin.lock`
   + `verus-fork/` submodule commit pin (gitlink) 모두 첨부
2. seL4 팀이 본 `proofs/isabelle/` 를 자체 l4v 환경의 ROOTS 에 추가
3. (T-i) sorry 채우기 / (T-ii) axiom 수용 / (T-iv) SMT replay 자동 채움
   결정 — verus_to_isabelle.md §1.4 mode flag
4. (T-iv) SMT replay fail 시 manual Isar 또는 도구 재실행
5. import path 변경 = v2 (incompatible) — 기존 use 무영향 추가는 v1.x patch
6. trust ledger: (T-i) seL4 책임 / (T-ii) Y4 책임 / (T-iv) Isabelle stdlib 책임

**Trust 가능 조건** (R7.10):
- PR-Verus-Backend land 후 (Verus fork 의 `-V adsmt` flag 정합 확인)
- adsmt rc.28+ + rc.29+ 도달 후 (sound + complete 확보, R7.6 정합)
- 본 `unified-toolkit-pin.lock` baseline 의 commit hash 가 위 두 조건 만족

## Cross-validation

`smt-cross-validation-tracker.md` §2 의 cluster 별 row 가 본 디렉터리의
.thy 산출물의 z3 vs adsmt cross-check 결과 record.  본 .thy 가 사용자
의도 한 invariant 를 정확히 표현하는 evidence.

## 현 상태 (2026-06-03)

- 본 디렉터리 신설 ✅ (R7.7)
- 첫 emit milestone = AV1 `intercept_floor_holds` (Cluster 1 amdv lower
  PR-2a, R7.11) — `Y4_AmdvSafety_Lower_InterceptFloor.thy`
- PR-Verus-Backend (R3.11+R3.12+R7.3) land 대기 — Verus fork 의 `-V adsmt`
  + `-V emit-isabelle` flag 작동 시점부터 본 디렉터리 활성

## Cross-ref

- `<Y4>/docs/verus_to_isabelle.md` §1.7 (inbound contract) + §2.6 (theory
  file naming) + §3.6 (unified-toolkit-pin)
- `<Y4>/.claude-notes/trackers/y4-sel4-integration-tracker.md` (R7.1~R7.12
  ledger + 진행 record)
- `<Y4>/.claude-notes/trackers/av-proof-body-tracker.md` §2 (per-cluster
  file layout) + §5 (per-cluster sub-PR 작업 항목)
- `<Y4>/.claude-notes/trackers/pr-verus-backend-tracker.md` §1 + §4
  (PR-Verus-Backend scope + phase)
- `<Y4>/unified-toolkit-pin.toml` + `.lock` (adsmt + adsmt-contrib + oxiz
  pin)
- `~/.claude/plans/jazzy-gliding-puppy.md` (본 plan 의 전체 결정 ledger)
