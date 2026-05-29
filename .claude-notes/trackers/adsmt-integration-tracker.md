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

**P5 결정 옵션** (v0.19 cycle):
- (a) adsmt + logicutils merge + OxiZ as pinned dependency
- (b) adsmt 가 OxiZ 안의 extension crates (`oxiz-lean` / `oxiz-rocq` /
  `oxiz-abduce`) 로 fold

## 2. Phased integration plan (P1-P5)

| Phase | Cycle | Status | 주요 사항 |
|---|---|---|---|
| P1 Bridge | v0.11 | ✅ | `oxiz_backend` feature in `adsmt-engine`, sits alongside `cadical_backend` |
| P2 Math | v0.13 | ✅ | `oxiz-math` Simplex import; v0.9 hand-rolled LIA Fourier-Motzkin retire |
| P3 Proof bridge | v0.15 | ✅ (landed 2026-05-14 commit `8bbf97e`) | `oxiz-proof` (DRAT/Alethe/LFSC) + `enable_writer` PR + 254 tests passing |
| **P4 Coordination** | **v0.17** | **현재 (2026-05-29 시점)** | OxiZ 측 issues/PRs — ITP binding (Lean4 + Rocq equal priority), abduction trait; option C of v0.15 oxiz_drat_bridge (cert ⇄ oxiz-proof bidirectional 확장) |
| P5 v1.0 decision | v0.19 | (대기) | (a) merge + OxiZ dep / (b) fold into OxiZ extensions |

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
| (대기) | adsmt v1.0 release | Y4 측 §5 ledger 의 v1.x patch 진입 | §5 |
| (대기) | Y4 측 spec v1.x patch 본격 갱신 | power_arch / vmm_arch / verus_to_isabelle / NOTICE 모두 | §5 |

## 8. Cross-validation experiment plan (Phase C 종반)

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
