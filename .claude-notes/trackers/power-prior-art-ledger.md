<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Power management prior art ledger

> **갱신 정책:** 새 학술 논문 / CVE / 산업 도입 발견 시 row 추가 →
> `power_arch.md` §6.7 의 prior art 부재 주장 재평가 → 영향 시 §6.1 의
> 학술적 차별점 8 항목 갱신 (v1.x patch 형태로 power_arch.md 갱신).

> **Cross-ref:** `power_arch.md` §6.7 (initial 14 row, 본 파일의 baseline)
> + §8.6 (갱신 path).

## 1. Baseline ledger (2026-05-07 sign-off cycle 종료 시점)

`power_arch.md` §6.7 의 14 row 그대로 — 새 발견 시 본 파일에 row 추가.

| # | Prior art | Venue / 출처 | 본 doc 와 비교 | 결론 |
|---|---|---|---|---|
| 1 | Atmosphere | SOSP '25 (mars-research) | Verus + Rust full-featured microkernel, 7.5:1 proof-to-code ratio.  power management 측면 미진입 | power domain prior art 부재 |
| 2 | AWS Nitro Isolation Engine | re:Invent 2025 (AWS), Isabelle/HOL | Graviton5 EC2 verified isolation.  power management 미명시 | power domain prior art 부재 |
| 3 | Lightweight Hypervisor Verification | HOTOS '25 (EPFL) | top-down lock-step verification | capsule-level + power 통합 prior art 부재 |
| 4 | CoVE / ACE-RISCV | arXiv 2505.12995 (IBM, 2025) | RISC-V confidential computing + post-quantum crypto | ISA-agnostic 4-tier + lease integration prior art 부재 |
| 5 | Hertzbleed update | IEEE S&P 2025 | 5x faster Hertzbleed, mitigation = disable boost only | verified hypervisor capsule-level mitigation prior art 부재 |
| 6 | PLATYPUS | USENIX Sec '21 + 후속 | RAPL driver 권한 제약 + Intel microcode | verified RAPL isolation prior art 부재 |
| 7 | Plundervolt | USENIX Sec '20 + Intel firmware patch | hypervisor MSR write 차단 standard | verified voltage range bound prior art 부재 |
| 8 | automotive hypervisor 시장 | VOSySmcs (ISO-26262) / Synopsys VDK CES 2026 / 시장 $3.2B by 2030 | functional safety 인증 수준 | verified transportation power management prior art 부재 |
| 9 | Windows Server 2025 HVCI / TPM 2.0 | Microsoft 2025 | hypervisor + TPM single-platform integration | ISA-agnostic 4-tier + universal customizability prior art 부재 |
| 10 | BioMake | github.com/evoldoers/biomake (BSD-3) | bioinformatics pipeline + Prolog logic programming | bioinformatics 영역만, generic verified build 진입 0 |
| 11 | Modus | Datalog dialect for container images | deductive datalog 만 (abductive/constraint X), kernel verification 측 X | logic-enhanced + ACLP + verified hypervisor prior art 부재 |
| 12 | Reproducible Builds (Vienna 2025 summit) | reproducible-builds.org | mtime + best-effort determinism | content-based hash-driven + transactional + ACLP + verified hypervisor 통합 부재 |
| 13 | OSS Rebuild | Google 2025 | open source package ecosystem reproducibility | verified hypervisor 의 form-factor customizability 통합 부재 |
| 14 | Verifying Datalog Reasoning with Lean | ITP 2025 (dagstuhl) | datalog reasoning formal verification | build orchestration 통합 X.  logicutils 와 보완 가능 |
| 15 | ACLP (Abductive Constraint Logic Programming) | Kakas/Michael/Mourlas 1999, ScienceDirect / arXiv cs/0003020 | application = diagnosis / planning / formal verification / multi-agent / ontology reasoning | **build orchestration 측 application prior art 부재** |
| 16 | ALP (Abductive Logic Programming) | Kakas/Kowalski/Toni 1992 | hypothetical reasoning, abducible predicates | software build dependency analysis 측 prior art 부재 |
| 17 | SCIFF framework | IFF abductive framework derived | verifiable agent interaction in ALP | agent verification 영역, hypervisor build orchestration 미진입 |

## 2. Search ledger (어떤 query 로 어디까지 confirm 됐는지)

| 검색 시점 | Engine | Query | 결과 요약 |
|---|---|---|---|
| 2026-05-07 | WebSearch (Anthropic) | "verified hypervisor power management capsule formal verification 2025 2026" | Atmosphere / AWS Nitro / Lightweight Hypervisor Verification 발견, power 측 진입 X |
| 2026-05-07 | WebSearch | "Hertzbleed mitigation hypervisor formal verification 2025 2026" | Hertzbleed IEEE S&P 2025 update — disable boost only, verified hypervisor 측 mitigation 부재 |
| 2026-05-07 | WebSearch | "PLATYPUS RAPL isolation virtualization formal proof 2024 2025 2026" | PLATYPUS USENIX '21 + Intel microcode patch.  verified hypervisor RAPL 격리 prior art 부재 |
| 2026-05-07 | WebSearch | "Plundervolt mitigation verified hypervisor capsule 2024 2025 2026" | Intel firmware patch + hypervisor MSR write 차단 standard.  formal verification 측 prior art 부재 |
| 2026-05-07 | WebSearch | "RISC-V CoVE confidential VM formal verification 2025 2026" | ACE-RISCV (arXiv 2505.12995, IBM 2025).  power management 측면 미명시 |
| 2026-05-07 | WebSearch | "automotive transportation hypervisor formal verification power management 2025 2026" | VOSySmcs / Synopsys VDK / ISO-26262 functional safety.  formal verification + transportation lease integration 부재 |
| 2026-05-07 | WebSearch | "BioMake datalog build system formal verification kernel 2025 2026" | BioMake 의 bioinformatics + Prolog logic programming, verified system 영역 진입 0 |
| 2026-05-07 | WebSearch | "Abductive Constraint Logic Programming build orchestration verified system" | ACLP application 영역 = diagnosis/planning/formal verification/multi-agent/ontology, build orchestration 측 prior art 부재 confirm |

## 3. 갱신 path (`power_arch.md` §8.6 정합)

새 prior art 발견 시:
1. 본 ledger §1 에 row 추가 (다음 number = 18 부터)
2. §2 search ledger 에 검색 query / 결과 row 추가
3. `power_arch.md` §6.7 의 ledger row 도 같은 항목 추가 (v1.x patch — power_arch §7.4 분류)
4. §6.1 의 학술적 차별점 8 항목 의 prior art 부재 주장 재평가:
   - 새 prior art 가 Y4 의 차별점 영역과 정확 매치 → 차별점 약화 / 재정의 필요
   - 부분 overlap 만 → boundary 명시화로 차별점 보존
   - 무관 → ledger 만 갱신, 차별점 변경 0

## 4. 새 항목 추가 (Phase C 진입 후)

(현재 비어 있음 — Phase C 진입 후 새 발견 시 채움.)
