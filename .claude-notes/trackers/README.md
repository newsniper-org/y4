<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# `.claude-notes/trackers/` — 진행 중 추적 파일

본 디렉터리는 Y4 의 spec 진행 중에 **지속 갱신** 되는 ledger / tracker
파일 묶음.  새 정보 (CVE / 학술 논문 / venue deadline / 위협 발견 등)
가 도착할 때마다 row 추가 / cell 갱신.

`.claude-notes/` 의 sibling 파일 (예: `amd-v-verified-survey.md`) 와의
구분:

| 종류 | 위치 | 성격 |
|---|---|---|
| **Tracker / ledger** (지속 갱신) | `.claude-notes/trackers/` (본 디렉터리) | 새 정보 도착 시 row 추가, status 갱신, cell 수정 |
| **Design memo / decision archive** (갱신 종료) | `.claude-notes/` | 결정 시점의 record, 채택 후 갱신 종료, historical reference |
| **Completed work archive** | `.claude-notes/_completed/` | 종료된 work item 보관 |

## 현재 / 예정 tracker 파일

(Phase C 진입 후 실제 작성 — 본 README 는 placeholder + 정책 ledger.)

| 파일 | 역할 | 신설 시점 | Cross-ref |
|---|---|---|---|
| `power-prior-art-ledger.md` | `power_arch.md` §6.7 의 prior art ledger 의 갱신 — 새 학술 논문 / CVE / 산업 도입 발견 시 row 추가 + §6.1 의 8 학술적 차별점 의 prior art 부재 주장 재평가 | Phase C 진입 후 | `power_arch.md` §6.7 / §8.6 |
| `power-paper-venue-tracker.md` | `power_arch.md` §6.4 의 venue 후보 (SOSP workshop / PLOS / IEEE S&P / SOSP main / OSDI / USENIX Security / EuroSys / ASPLOS / HOTOS) deadline 추적 + paper draft → submission timeline 정합 | Phase C 종반 paper draft 시점 | `power_arch.md` §6.4 / §8.7 |
| `power-threat-ledger.md` | `power_safety.md` §1.4 의 v1.x 위협 ledger — 새 CVE / 학술 논문 발견 시 row 추가 + §1.2 의 12 항목 catalog 갱신 + §3 의 안전장치 (S15~S23) 의 mitigation 영향 재평가 | Phase C 진입 후 | `power_safety.md` §1.4 / §7.3 |
| `adsmt-integration-tracker.md` | adsmt (`~/AD1/`, newsniper-org/adsmt) 의 v1.0 unified vision (P5 option 5 bidirectional embed — adsmt + adsmt-contrib 독자 release cadence + logicutils absorbed) 통합 watch + Y4 측 spec 갱신 ledger.  P-redesign.1~8 sub-cycle 의 진행 상태 + testing channel pin 정책 + Lean4 / OxiLean watch | **2026-05-29 (active)** | adsmt 측 `~/AD1/.claude-memories/{logicutils_version_rule,oxiz_relationship}.md` + Y4 측 `power_arch.md` §6.1.8 + §6.7 + §5.2 / `vmm_arch.md` §1.1 + §3.2 / `verus_to_isabelle.md` §1.3 + §3.6 / `cpu_virt_compat.md` §8 / `unified-toolkit-pin.toml` / NOTICE |
| `smt-cross-validation-tracker.md` | Verus 의 z3 + OxiZ + adsmt (R3.12 opt-in third) backend cross-validation 실험 plan + 측정 결과.  paper artifact 의 Reproducible 자격 evidence + 학술적 차별점 §6.1.8 의 quantitative support | **2026-06-01 (active)** | `adsmt-integration-tracker.md` §8 + `verus_to_isabelle.md` §3.6 + `power_arch.md` §6.5 (vii) + `av-proof-body-tracker.md` §7 / §9 |
| `av-proof-body-tracker.md` | AV1~AV40 (amdv 21 + power 16 active) proof body sub-cluster 별 작성 plan + 진행 상태.  4 cluster = amdv lower (12) + amdv upper (9) + power upper (11) + power lower (5).  R3.1~R3.12 sign-off (12 결정), PR-N scope 매핑, R3.12 abductive verdict 활용 invariant 후보 6, Lean4 retrofit watch | **2026-06-01 (active)** | `adsmt-integration-tracker.md` §10.3 + `amdv_safety.md` §5 / §6 + `power_safety.md` §4 + `verus_to_isabelle.md` §3.2 / §3.6 + `smt-cross-validation-tracker.md` §2 + `cpu_virt_compat.md` §8 (4) |
| `pr-verus-backend-tracker.md` | Verus 본체 patch (R3.11 + R3.12 + R7.3 의 산출물) — `-V <key>` extended-multi flag (oxiz / adsmt / report-abductive-on-unknown / emit-isabelle / emit-rocq / aot-prelude / jit-trace-load).  본 작업은 **별도 Claude Code 세션** 에서 진행 (`<Y4>/verus-fork/` submodule, branch `backend-pluggable`).  12 phase (P-vb.1~P-vb.12), 분량 ~850 LoC | **2026-06-01 (active, preparation)** | `av-proof-body-tracker.md` §1 R3.11 + R3.12 + §7 + `smt-cross-validation-tracker.md` §9 + `verus_to_isabelle.md` §3.6 + `unified-toolkit-pin.toml` `[verus]` + `y4-sel4-integration-tracker.md` §1 R7.3 |
| `y4-sel4-integration-tracker.md` | R7.1~R7.12 sign-off (2026-06-03) — Y4 Verus ↔ seL4 Isabelle/HOL 통합 plan.  sibling repo 폐기 + Verus fork single source of truth + consumer pattern + `proofs/isabelle/` 신설 + L4.verified inbound 사용자 manual.  per-cluster emission 진행 record + l4v 측 inbound milestone watch | **2026-06-03 (active)** | `~/.claude/plans/jazzy-gliding-puppy.md` + `docs/verus_to_isabelle.md` §1.7 / §3 + `proofs/isabelle/README.md` + `proofs/verus/justfile` + `unified-toolkit-pin.lock` + `pr-verus-backend-tracker.md` §1.5 / §1.6 / §4 P-vb.10~12 + `av-proof-body-tracker.md` §5 + `adsmt-integration-tracker.md` §7 / §10.3 + `CLAUDE.md` §5 |
| (추가 tracker 들 — Phase C 진입 후 발견 시 추가) | — | — | — |

## 정책

### git tracking
**Git-tracked** — `vmm_arch.md` §5.4 의 `.claude-notes/` 정책 정합.
design 흔적 보존이 contribute-back paper / 코드 리뷰 / 산업 도입 시
audit reference.

### 갱신 권한
- Y4 contributor (host operator / lease holder 의 OS 측 사용자 권한과
  무관) — git push 가능한 모든 사용자
- Claude Code 가 sign-off cycle 또는 microbench 측정 시 자동 갱신 가능

### Tracker → Archive 전이
tracker 가 갱신 종료될 시점 (예: paper 게시 후 venue tracker 갱신 종료):
1. 본 디렉터리에서 `.claude-notes/_completed/` 로 이동
2. cross-ref doc 갱신 (file path 갱신)
3. 본 README 의 표 갱신

### 파일 명칭 convention
- `<domain>-<kind>.md` 형식: `<domain>` = power / amdv / vmm / wavetensor /
  etc.  `<kind>` = ledger / tracker / survey / etc.
- 예: `power-prior-art-ledger.md` (domain=power, kind=prior-art-ledger)
