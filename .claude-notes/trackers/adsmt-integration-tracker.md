<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# adsmt integration tracker

> **갱신 정책:** adsmt 의 P5 도달 / v1.0 release / OxiZ 측 milestone /
> Y4 측 spec 갱신 결정 시점에 row 추가.  v1.0 통합 후 Y4 측 학술
> 차별점 (power_arch §6.1.8) + 산업 차별점 (§6.2) + prior art ledger
> (§6.7) 의 갱신 path 추적.

> **Source repo:** adsmt = `~/AD1/` + https://github.com/Honey-Be/adsmt
> (윤병익 author, BSD-2 OR Apache-2.0 OR LGPL-2.1+).

## 1. v1.0 unified vision (확인 2026-05-29)

```
adsmt v1.0 = adsmt-core + logicutils + OxiZ (integrated form)
```

원래 2-way (adsmt + logicutils) 였으나 2026-05-13 에 3-way 로 revised
— OxiZ (pure-Rust Z3 reimplementation) 가 third leg.

| Component | 역할 | License | 분량 / 신뢰도 |
|---|---|---|---|
| **lu-kb** (logicutils v0.x-smt) | user-facing KB surface — ACLP (abductive-deductive + constraint + type relations) | BSD-2-Clause | logicutils v0.x → v1.0 으로 bump (offset "+2" 제거) |
| **OxiZ** | SAT + classical SMT theories (LIA/LRA/BV/Arrays/Datatypes/FP/Strings/NIA) + math (Simplex/polynomial/CAD) + proof (DRAT/Alethe/LFSC) | Apache-2.0 | ~408k LoC, 6415 tests, **100% Z3 parity across 8 logics** |
| **adsmt** | abductive engine (SLD + minimize + rank + workflow) + HOL+HKT kernel + type-class layer + Lean4/Rocq first-class + lu-kb integration | BSD-2 OR Apache-2 OR LGPL-2.1+ | v0.17.0 (P4 단계, 2026-05-29 시점) |

**P5 결정 (2026-06-01 fetch 확인, option 5 채택)**:
- **option 5 — bidirectional embed**: adsmt + adsmt-contrib 가 두 독립
  repo 로 독자 release cadence 유지 + adsmt 안에 logicutils **absorbed**
  (`state/logicutils-frozen/docs/man/` 보관, adsmt 측 PKGBUILD 의 split
  package 로 export) + OxiZ 는 adsmt 측 fork (Honey-Be/oxiz) 의 transitive
  dep
- 이전 옵션 (a) merge + OxiZ dep / (b) fold into OxiZ extensions 둘 다
  closed (historical record)
- 근거: adsmt-contrib commit `a838525` 의 "bidirectional embed decision
  (P5 option 5)" 본문 + adsmt main commit `a937058` 의 9-PKGBUILD 측
  `state/logicutils-frozen/` 보존

## 2. Phased integration plan (P1-P5)

| Phase | Cycle | Status | 주요 사항 |
|---|---|---|---|
| P1 Bridge | v0.11 | ✅ | `oxiz_backend` feature in `adsmt-engine`, sits alongside `cadical_backend` |
| P2 Math | v0.13 | ✅ | `oxiz-math` Simplex import; v0.9 hand-rolled LIA Fourier-Motzkin retire |
| P3 Proof bridge | v0.15 | ✅ (landed 2026-05-14 commit `8bbf97e`) | `oxiz-proof` (DRAT/Alethe/LFSC) + `enable_writer` PR + 254 tests passing |
| P4 Coordination | v0.17 | ✅ | OxiZ 측 issues/PRs — ITP binding (Lean4 + Rocq equal priority), abduction trait; option C of v0.15 oxiz_drat_bridge (cert ⇄ oxiz-proof bidirectional 확장) |
| P5 v1.0 decision | v0.19 | ✅ (통과 추정) | testing 브랜치가 v1.0.0-rc.2 단계, P5 결정 완료된 듯 (구체 선택은 release notes 확인 시점) |
| **v1.0.0-rc.2 audit cycle** | **testing 브랜치** | **현재 (2026-06-01 시점)** | RC2.1~RC2.7 모두 resolved (warning sweep / cargo doc / bench / adsmt-lsp / contributions audit 13 findings).  HEAD = `450b986` "preparations for stable v1.0.0 release" (2026-05-31) — **stable release 임박** |

## 3. adsmt ⇔ logicutils 의 v0.x cycle 규칙

(`~/AD1/.claude-memories/logicutils_version_rule.md`)

- **Version offset**: logicutils v0.x-smt minor = adsmt minor + 2
- **Immediate kb-syntax sync**: adsmt 측 lu-kb surface 변경 시 같은 cycle
  안에 logicutils 측 commit 도 동시
- v1.0 도달 시 offset 제거 (logicutils 1.0.0 = adsmt 1.0.0)

## 4. Fork strategy — OxiZ 측

(`~/AD1/.claude-memories/oxiz_relationship.md`)

- adsmt 측 fork: https://github.com/Honey-Be/oxiz, **strict superset**
  of upstream
- Submodule path: `~/AD1/external/oxiz/`
- Branch: `0.2.2` (upstream tag 매치) + `feat/enable-writer` (adsmt 측
  변경)
- `Cargo.toml [patch.crates-io]` 로 `oxiz-sat` / `oxiz-proof` /
  `oxiz-math` 를 fork submodule path 로 redirect

## 5. Y4 측 spec 갱신 ledger (v1.0 통합 후, v1.x patch 분류)

> **Hold pattern 완전 해제 (2026-06-01 사용자 결정)**: adsmt v1.0
> release 대기 X.  Y4 측 dependency 가 testing channel pin (rolling,
> §10.6) 이므로 stable release timing 과 spec 마킹 timing 의 정합
> 무의미.  본 §5 의 spec 갱신 = P-redesign.8 (즉시 진입 가능, §10.3)
> 의 산출물.  rolling 후속 갱신은 후속 v1.x patch 로.  관련 memory:
> `feedback_adsmt_v1_verification_redesign.md` (정책 변화 ledger) +
> `feedback_adsmt_testing_channel_pin.md` (channel pin 정책).

v1.0 도달 시 Y4 측 spec 의 학술/산업 차별점 강화 + cargo dep 정합:

| Doc / sub-section | 현 표현 | v1.0 후 갱신 |
|---|---|---|
| `power_arch.md` §6.1.8 학술적 차별점 | "ACLP-driven build orchestration (logicutils) + theory-aware SMT 통합" | **"unified verification toolkit (OxiZ + adsmt + lu-kb) 의 첫 verified hypervisor industrial adoption"** — pure-Rust SMT + abductive ITP frontend + ACLP build orchestration single coherent ecosystem |
| `power_arch.md` §6.7 prior art ledger | (현재 logicutils + ACLP 영역만) | + OxiZ (Apache-2, 100% Z3 parity) row + Verus + Z3 dependency cycle 의 supply chain 비교 |
| `power_arch.md` §6.2 산업 차별점 | (logicutils 기반 build infrastructure) | + unified toolkit 의 single Rust workspace + supply chain integrity ↑ |
| `power_arch.md` §5.2 workspace dependency 표 | `logicutils-core` (BSD-2) | unified `adsmt-logicutils-unified-toolkit` (또는 P5(b) 시 OxiZ extension crates) |
| `power_arch.md` §5.6 cargo dep 라이선스 정합 | (현재 tss-esapi BSD-2 / logicutils-core BSD-2) | + adsmt triple (BSD-2 OR Apache-2 OR LGPL-2.1+) + OxiZ Apache-2.0 |
| `vmm_arch.md` §1.1 참조 자료 통합 표 | (현재 7 row) | + adsmt + OxiZ row |
| `vmm_arch.md` §3.2 Verus version semantic caution | (Tracked/Ghost/PointsTo) | + "Verus 의 Z3 → OxiZ 교체 시 trust boundary 명시 (OxiZ 의 100% Z3 parity claim 신뢰성 검증)" |
| `verus_to_isabelle.md` §1.3 (T-iv) SMT-LIB hybrid | (현재 Verus Z3 query → Isabelle smt method replay) | + adsmt + OxiZ 통합 backend 명시 (Lean4/Rocq direct emit 가능) |
| `verus_to_isabelle.md` §3.3 cargo dep | `logicutils-core` (BSD-2) | unified toolkit (또는 OxiZ extension crates) |
| NOTICE | (현재 logicutils BSD-2 / tss-esapi BSD-2 / etc.) | + adsmt (triple license) + OxiZ (Apache-2.0) attribution |

## 6. Y4 측 활용 path (v1.0 통합 후 simplified)

| Path | v1.0 통합 후 의미 | Y4 측 진입 시점 |
|---|---|---|
| (A) Verus 의 Z3 backend 대체 | OxiZ 채택 (100% Z3 parity, pure-Rust) — Verus 측 호환성 검증 자연 | PR-3 (Verus 명세 + paper artifact) 시점 |
| (D) logicutils ACLP solver 강화 | adsmt 의 abductive engine (이미 unified toolkit 일부) | PR-5a (orchestrator + audit hook) 시점 |
| (B/C) verus_to_isabelle (T-iv) backend | unified toolkit 의 단일 backend | PR-4 시점 |
| (F) paper artifact cross-validation | OxiZ ↔ Z3 자연 정합 | Phase C 종반 microbench 와 함께 |
| (G) pure-Rust verification stack | 자동 만족 (toolkit 자체) | v1.0 통합 시 자동 |
| (N) Lean4 + Rocq first-class | adsmt prover_emit/lean 의 reference path | PR-4 시점 (verus_to_isabelle 도구 확장) |

## 7. Watch milestones

| 일자 | Milestone | 영향 | row 매핑 |
|---|---|---|---|
| 2026-05-13 | adsmt v0.x 의 3-way unified vision revision (originSession `32a1dc0d`) | Y4 측 학술 차별점 강화 path 확정 | §1 |
| 2026-05-14 | adsmt P3 landed (commit `8bbf97e`) | OxiZ proof bridge stable | §2 |
| 2026-05-29 | Y4 측 adsmt integration tracker 신설 (본 file) | v1.0 도달 시 spec 갱신 ledger 활성 | §1, §5 |
| (대기) | adsmt P5 (v0.19) v1.0 decision | (a) merge + OxiZ dep / (b) fold into OxiZ | §2 |
| 2026-05-29 | Y4 측 hold pattern 결정 — adsmt v1.0 release 전까지 verification workflow 본격 변경 X | 사용자 명시 정책, `feedback_adsmt_v1_verification_redesign.md` 신설 | §5 (Hold pattern) |
| 2026-05-31 | adsmt testing 브랜치 v1.0.0-rc.2 진입 (윤병익, RC2.1~RC2.7 audit cycle) | P5 (v0.19) 통과 신호 | §2 |
| 2026-05-31 | adsmt-contrib Debian channel model lockstep (main/testing/stable) | adsmt main 과 lockstep, 사용자 instruction | §2 |
| 2026-05-31 | adsmt testing HEAD = "preparations for stable v1.0.0 release" + RC2.7 13 audit findings 모두 resolved | **v1.0.0 stable release 임박 신호** | §2 |
| 2026-06-01 (오전) | Y4 측 verification workflow 재설계 *논의* 시작 | hold pattern 부분 해제 | §5 (Hold pattern banner 갱신) |
| 2026-06-01 (P-redesign.1) | 11 결정 ledger sign-off (R4=b'/R6=b' 즉시 trigger) + testing channel pin 정책 (§10.6) | hold pattern 사실상 해제 | §10.2, §10.6 |
| **2026-06-01 (P-redesign.8 trigger)** | **Hold pattern 완전 해제 — P-redesign.8 도 stable release 기다리지 X** (사용자 결정) | testing channel pin (rolling) 정합, 모든 P-redesign sub-cycle 즉시 진입 가능 | §10.3, §10.4, §10.6 |
| (대기) | adsmt v1.0.0 stable release | rolling 따라가는 testing channel 기준 — Y4 측 spec 영향 minimal (rolling 후속 갱신은 v1.x patch) | §10.6 |

## 8. Cross-validation experiment plan (P-redesign.6 ✅ 2026-06-01)

> **본격 plan + measurement 정책:** `.claude-notes/trackers/smt-cross-
> validation-tracker.md` (R6.1~R6.10 sign-off).  본 §8 은 핵심 영역
> summary 만.

> **즉시 진입 (R6=b')**: stable release 기다리지 X.  첫 baseline 측정
> 은 P-redesign.2 의 Verus `--backend=oxiz` flag 가 작동 가능해진 시점
> (cargo `[patch.crates-io]` git dep 적용 후) 즉시.

paper artifact 의 Reproducible 자격 + 학술 차별점 evidence:

1. **OxiZ vs Z3 비교** — Y4 의 Verus 측 51 verified invariant 의 SMT
   질의를 두 solver 로 발화, 결과 / 시간 / proof cert 비교
2. **adsmt 의 abductive minimal explanation** — Y4 의 `power_arch.md`
   §6.1.8.2 의 3 query (Which targets are stale / smallest set to
   regenerate / Why being rebuilt) 의 actual implementation
3. **trust boundary 다양화** — Verus → adsmt → Lean4 + Rocq + Isabelle
   의 multi-ITP cross-validation
4. **artifact reproducibility** — `freshcheck` + `stamp` + `lu-par
   --transaction` (logicutils 의 unified toolkit 일부) 로 hash-driven
   강제

## 9. Rust toolchain version pin (v1.0 통합 시 결정)

| Component | 현 version | Y4 측 통합 시 결정 |
|---|---|---|
| Rust edition | adsmt: 2024 / OxiZ: TBD / logicutils: 2024 / Y4: 2024 | 모두 edition 2024 정합 OK |
| Cargo resolver | adsmt: 3 / Y4: 3 | OK |
| Y4 측 verus-bin | rolling (AUR) | unified toolkit 의 P5 결정에 따라 verus-pin 추가 |
| Y4 측 isabelle-pin | Isabelle 2024+ | OK |
| Y4 측 adsmt-pin (신규) | (대기) | unified toolkit v1.0 release 후 pin file 신설 (`adsmt-pin.toml` 또는 `unified-toolkit-pin.toml`) |

## 10. Y4 verification workflow 재설계 ledger (P-redesign.1, 2026-06-01)

### 10.1 사용자 trigger (2026-06-01)

R1=(a') "Lean4 만 제외한 verifications 전체" + R4=(b') / R6=(b')
"지금 당장 / stable release 기다리지 말고 즉시" — adsmt testing
브랜치가 v1.0.0-rc.2 + "preparations for stable v1.0.0 release"
단계 도달 + 13 audit findings 모두 resolved 라 hold pattern 사실상
해제, **본격 재설계 진입**.

### 10.2 11 결정 ledger (P-redesign.1)

| Item | 결정 | 비고 |
|---|---|---|
| **R1** scope | (a') Verification 전체 — **Lean4 제외** | Lean4 / OxiLean 측이 adsmt stable v1.0.0 release blocker (사용자 명시, 2026-06-01).  adsmt 측 자체의 미해결 area 라 Y4 측에서도 deferred.  영향: R7 의 adsmt-emit-rocq + adsmt-emit-isabelle 만 활용 (adsmt-emit-lean 미활용), verus_to_isabelle §1.3 (T-iv) 의 backend 도 Lean4 direct 미선택, y4-verus2isabelle 의 multi-ITP support 도 Isabelle + Rocq 만 (Lean4 deferred) |
| **R2** Verus Z3 backend → OxiZ | (b) **dual backend** (z3 + OxiZ, cross-validation) | paper artifact §6.5 (vii) reproducibility 강화 + supply chain diversity |
| **R3** Verus + adsmt pin 형식 | (b) **unified-toolkit-pin.toml** single | adsmt v1.0 의 3-way unified vision 정합 |
| **R4** AV proof body 시점 | (b') **지금 당장 sub-cluster 별 점진적** (stable release 기다리지 X) | adsmt testing v1.0.0-rc.2 가 충분히 안정적 (RC2.7 13 findings 모두 resolved).  4 sub-cluster = amdv lower / amdv upper / power lower / power upper.  순서는 P-redesign.3 sign-off 에서 결정 |
| **R5** Rocq theory 진입 순서 | (d) **3 theory 동시** (Y4.Lease.Spec + Y4.IPC.Refinement + Y4.Sel4.Wrapper) | adsmt-emit-rocq 가 multi-theory 일괄 처리 가능 |
| **R6** SMT cross-validation 시점 | (b') **stable release 기다리지 말고 즉시** | adsmt testing v1.0.0-rc.2 사용으로 baseline 측정 즉시 진입.  P-redesign.6 본격 |
| **R7** y4-verus2isabelle 도구 형태 | (a) **adsmt-contrib 의 adsmt-emit-isabelle wrapper** | 분량 ~2250 → ~500 LoC wrapper.  Lean4 제외 (R1 정합) |
| **R8** unsafe + proof 짝 lint | (a) 재설계 cycle 의 일부 (별도 sub-PR, P-redesign.7) | adsmt 의 type-class layer 활용 가능성 |
| **R9** verus_to_isabelle §1.3 (T-iv) backend | (a) **adsmt-cli 의 lu-smt binary 통합** | logicutils CLI protocol 의 단일 entry |
| **R10** sub-cycle 명명 | (a) **P-redesign.1~N** (sign-off cycle 패턴 정합) | — |
| **R11** spec 의 v2 가능성 | (a) **모두 v1.x patch** (mechanism 변경 X) | adsmt 가 그대로 backend, semantic 동일 |

### 10.3 Sub-cycle 분할 (P-redesign.1~8)

| Cycle | 내용 | 의존 | 시점 |
|---|---|---|---|
| **P-redesign.1** | 재설계 scope + 11 결정 ledger (본 sub-section) | 0 | ✅ **2026-06-01 완료** |
| **P-redesign.2** | Verus dual backend (z3 + OxiZ) integration 측 spec — `verus_to_isabelle.md` §3.6 갱신 (`unified-toolkit-pin.toml`).  Lean4 제외 명시 | P-redesign.1 | ✅ **2026-06-01 완료** (sub-table 3 = adsmt + adsmt-contrib + oxiz, logicutils absorbed, P5 option 5 정합; Y4/unified-toolkit-pin.toml + .lock 신설; cargo `[patch.crates-io]` git dep mechanism; Verus `--backend=z3|oxiz|dual` flag; cpu_virt_compat §8 Lean4 watch row 추가) |
| **P-redesign.3** | AV1~AV40 proof body 의 sub-cluster 별 작성 plan (4 cluster 순서 결정 + 분량 추정) | P-redesign.2 ✅ | 즉시 진입 가능 (R4=b') |
| **P-redesign.4** | Rocq theory 3 (Y4.Lease.Spec + Y4.IPC.Refinement + Y4.Sel4.Wrapper) + adsmt-emit-rocq 통합 spec | P-redesign.2 ✅ | 즉시 진입 가능 |
| **P-redesign.5** | `y4-verus2isabelle` 의 adsmt-emit-isabelle wrapper 재정의 (P3.6 §3.2 갱신, Lean4 제외) | P-redesign.2 ✅ | 즉시 진입 가능 |
| **P-redesign.6** | paper artifact SMT cross-validation 실험 plan (OxiZ ↔ Z3) | microbench infra | ✅ **2026-06-01 완료** (`.claude-notes/trackers/smt-cross-validation-tracker.md` 신설; R6.1~R6.10 sign-off; baseline 측정은 첫 Verus dual backend 실행 후) |
| **P-redesign.7** | unsafe + proof 짝 lint 자동화 spec (adsmt type-class layer 활용) | P-redesign.3 | — |
| **P-redesign.8** | Y4 spec v1.x patch 일괄 마킹 (power_arch + vmm_arch + verus_to_isabelle + cpu_virt_compat + amdv_safety + NOTICE) | P-redesign.2~7 완료 | **즉시 진입 가능 (2026-06-01 사용자 결정 — release 기다리지 X)**.  testing channel pin (§10.6) 기준 마킹, rolling 후속 갱신은 v1.x patch 로 |

### 10.4 hold pattern 의 변화

| 시점 | 정책 |
|---|---|
| 2026-05-29 | adsmt v1.0 release 전까지 verification workflow 본격 변경 X (hold pattern) |
| 2026-06-01 (오전) | 재설계 *논의* 시작 (부분 해제) |
| 2026-06-01 (P-redesign.1 sign-off) | R4=(b') + R6=(b') 로 본격 작업도 진입 — 단 P-redesign.8 (spec 일괄 마킹) 만 stable release 후 deferred |
| **2026-06-01 (P-redesign.8 도 즉시 trigger)** | **Hold pattern 완전 해제** — P-redesign.8 도 stable release 기다리지 X (사용자 결정).  Y4 측이 testing channel pin (rolling, §10.6) 이라 stable release 와 spec 마킹 timing 의 정합 무의미 — testing channel 기준 일괄 마킹, rolling 갱신은 후속 v1.x patch 로 |

### 10.5 Lean4 / OxiLean 영역의 별도 watch

R1=(a') 의 Lean4 제외 사유 — adsmt 측 자체 blocker:
- Y4 측에서 본 항목 본격 진입 시점 = adsmt 측이 Lean4/OxiLean 측 blocker 해결 + adsmt stable v1.0.0 release 후
- 본격 Y4 활용 path = §6 의 (N) Lean4 + Rocq first-class 의 Lean4 측
- 본 영역의 watch = adsmt 측 issue tracker / OxiLean repository / adsmt-emit-lean crate 상태
- Phase D 또는 v2 단계로 재고려

### 10.6 Y4 측 dependency channel pin 정책 (2026-06-01 사용자 결정)

Y4 가 adsmt + adsmt-contrib 둘 다 **testing channel pin** — rolling
release 패턴:

| Repo | Channel | Pin 형식 |
|---|---|---|
| **adsmt** (`~/AD1/` / `newsniper-org/adsmt`) | **testing** | branch head 또는 commit hash + branch comment.  rolling 따라가는 패턴 |
| **adsmt-contrib** (`~/adsmt-contrib/` / `newsniper-org/adsmt-contrib`) | **testing** | 동일 |
| OxiZ | adsmt 의 `external/oxiz/` submodule pin 의 transitive 추종 | adsmt 측이 fork (Honey-Be/oxiz) `feat/enable-writer` branch 사용, Y4 측은 adsmt 의 선택 그대로 |
| logicutils | adsmt 의 `external/logicutils/` submodule pin 의 transitive 추종 (v1.0 통합 후 사실상 adsmt 안) | adsmt 측 통합 그대로 |

**근거**: adsmt 의 main + testing 둘 다 rolling release 패턴으로 운영
(사용자 명시 2026-06-01).  Y4 측이 testing channel 을 pin 으로 추종
하면 adsmt 측 latest stabilisation 작업의 즉시 흡수 가능 — Y4 측
verification workflow 의 evolve 와 정합.

**Pin 파일 형식 결정 (P-redesign.2 sign-off 시점)**:

```toml
# unified-toolkit-pin.toml (예시)
[adsmt]
repo    = "https://github.com/newsniper-org/adsmt"
channel = "testing"
# rolling — branch HEAD 추종, 단 build-time 시점에 commit hash 캡쳐
# (reproducible build 위해)
captured_at = "(build-time stamp)"

[adsmt-contrib]
repo    = "https://github.com/newsniper-org/adsmt-contrib"
channel = "testing"
captured_at = "(build-time stamp)"
```

**Stable release 와의 정합 (2026-06-01 결정 — P-redesign.8 도 release
기다리지 X)**: Y4 측 channel 은 **testing 그대로 유지 (rolling)** — adsmt
v1.0.0 stable release 도달 후에도 channel 전환 X (옵션 (i) 채택, 사용자
결정).  근거:
- Y4 spec v1.x patch 본격 진입 시점이 stable release 도달 무관 (즉시)
- testing channel 의 rolling pattern 이 Y4 측 work 의 evolve 와 정합
- paper artifact submission 시점에만 build-time commit hash capture
  (reproducible build 위해) — channel 자체는 testing 유지

이전 옵션 (ii) stable 전환 / (iii) hybrid 는 closed — 본 ledger 의
historical record 로만 보존.

**Cargo.toml 측 [patch.crates-io] 활용**: adsmt 측 fork (Honey-Be/oxiz)
처럼 Y4 측도 `[patch.crates-io]` 로 adsmt + adsmt-contrib 의 testing
branch 를 path 또는 git dep 으로 redirect.
