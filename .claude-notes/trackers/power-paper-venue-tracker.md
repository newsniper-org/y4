<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Power management paper venue tracker

> **갱신 정책:** Phase C 종반 paper draft 시점에 본격 갱신 시작.
> 현재는 baseline placeholder (`power_arch.md` §6.4 의 venue fit 분석
> import).  paper submission 후 venue 별 review 결과 row 추가.

> **Cross-ref:** `power_arch.md` §6.4 (venue fit 분석) + §6.5 (paper
> artifact 형식) + §6.6 (재현성 패키지 위치 `/home/ybi/y4-paper-artifact/`)
> + §8.7 (본 tracker 의 갱신 path).

## 1. Baseline venue fit (`power_arch.md` §6.4 import)

| Venue | fit 평가 | power 측 evidence | 우선순위 |
|---|---|---|:---:|
| **SOSP workshop (PLOS)** | ◎ 가장 자연 fit | "verified hypervisor on a verified microkernel + power-mgr" 가 PLOS workshop 트랙 매칭 | **1 순위** |
| **IEEE S&P (Oakland)** | ◎ security framing 강함 | Hertzbleed / PLATYPUS / Plundervolt mitigation 의 verified hypervisor 측 진입 첫 사례 framing | **2 순위** |
| **SOSP main track** | ◎ 강한 fit (2027) | Atmosphere (SOSP '25) 의 후속 — verified kernel + power 도입 첫 사례.  Phase C 종반 결과 강도 + 차별점 evidence 충분 시 시도 | 3 순위 |
| **OSDI main track** | ○ 강한 fit (2027) | systems-evaluation 비중 ↑, microbench 산출물 충분 시 시도 | 3 순위 |
| **USENIX Security** | ○ side-channel 트랙 강 fit | Hertzbleed / PLATYPUS / Plundervolt 의 mitigation 사례 → S&P 와 양립 후보 | 4 순위 |
| **EuroSys** | ○ systems track | systems-heavy 가 더 적합한 경우 후보 | 4 순위 |
| **ASPLOS** | △ HW 측면 약함 | WaveTensor 통합 paper 별도 시 후보 | 5 순위 |
| **HOTOS** | △ workshop short paper | 초기 idea paper 형식 적합, full paper 부적합 | 5 순위 |

## 2. Submission deadline 추적

각 venue 의 typical deadline cycle (실제 deadline 은 매년 갱신 — Phase
C 종반 시점에 정확 추적):

| Venue | Cycle | Typical deadline | 다음 deadline 추적 |
|---|---|---|---|
| SOSP workshop (PLOS) | 매년 SOSP 와 동반 | SOSP submission 의 ~6 개월 전 | TBD (Phase C 종반 갱신) |
| IEEE S&P (Oakland) | 매년 5월 | 6 cycles per year (Apr / Jun / Aug / Oct / Dec / Feb) | TBD |
| SOSP main | 매년 10월 | submission ~5월 | TBD (SOSP '27 cycle) |
| OSDI main | 매년 7월 | submission ~12월 | TBD (OSDI '27 cycle) |
| USENIX Security | 매년 8월 | 3 cycles per year | TBD |
| EuroSys | 매년 4월 | submission ~10월 | TBD |
| ASPLOS | 매년 3월 | 4 cycles per year | TBD |
| HOTOS | 매년 5월 | submission ~1월 | TBD |

`power_arch.md` v1.0 frozen (2026-05-07) → Phase C 진입 → 본격 코드
작업 (PR-1~5) → microbench measurement (Phase C 종반) → paper draft
시작 → 본 tracker 의 deadline 적극 갱신.

## 3. paper artifact 산출물 준비 상태 (`power_arch.md` §6.5 cross-ref)

7 산출물 묶음 (USENIX / ACM / IEEE artifact badge Available + Functional
+ Reproducible 충족 목표):

| # | 산출물 | 상태 |
|---|---|---|
| (i) | Y4 GitHub repo 의 v1.0 frozen tag | (대기) Phase 4-power 마킹 후 git tag 작업 |
| (ii) | Verus 증명 산출물 (AV1~AV40) | (진행) statement-only frozen 완료, proof body 채움은 PR-3 + PR-5 본격 작업 |
| (iii) | qemu reproducibility script | (대기) Phase C 진입 후 작성 |
| (iv) | Isabelle skeleton | (대기) `y4-verus2isabelle` 도구 v1.0 + power fixture round-trip |
| (v) | power microbench 산출물 | (대기) Phase C 종반 측정 |
| (vi) | TPM-based reproducibility | (대기) qemu + swtpm 환경 구축 또는 hardware loaner |
| (vii) | logicutils-driven artifact verification | (대기) `freshcheck` + `stamp` + `lu-par --transaction` + `.lu-store/` 동봉 |

## 4. Submission 결과 row (Phase C 종반 paper draft 후)

| 일자 | Venue | Track | 결과 | 후속 작업 |
|---|---|---|---|---|
| (현재 비어 있음) | — | — | — | — |

## 5. 갱신 path (`power_arch.md` §8.7 정합)

1. Phase C 진입 직후: §3 의 산출물 준비 상태 갱신
2. Phase C 종반: §2 의 deadline 정확 추적 시작 + §3 모든 산출물 (i)~(vii)
   완료 검증
3. Paper draft 후 첫 submission: §4 에 row 추가
4. Review 결과 row 추가 (accept / reject / revise)
5. Re-submission 시 row 추가
6. 게시 후 venue + 인용 추적
