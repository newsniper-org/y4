<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Power management threat ledger

> **갱신 정책:** 새 CVE / 학술 논문 발견 시 row 추가 → `power_safety.md`
> §1.2 의 12 항목 catalog 의 어느 카테고리 (A side-channel / B direct
> attack / C DoS) 인지 분류 → mitigation 가능성 분석 → 기존 S15~S23
> 의 sub-decision 갱신 또는 신규 S24+ 추가.

> **Cross-ref:** `power_safety.md` §1.4 (v1.x patch 후속 위협 ledger)
> + §1.2 (12 항목 catalog) + §3 (S15~S23 안전장치) + §6.4 (v1.x patch /
> v2 정의).

## 1. Baseline 12 항목 threat catalog (`power_safety.md` §1.2 import)

`power_safety.md` v1.0 frozen (2026-05-07) 시점의 12 위협 + mitigation:

### A. Side-channel (cross-tenant 정보 추출)

| # | 위협 | Layer | Mitigation |
|---|---|:---:|---|
| A.1 | Hertzbleed | Lower | S15.5 dwell + S15.6 constant-freq + S15.7 SMT pair 동기 |
| A.2 | DVFS / P-state side-channel | Lower | S15.1~S15.3 권한 + MSR + CPUID 마스킹 |
| A.3 | C-state residency side-channel | Lower | S16.4 residency MSR + S16.5 L1D flush + S16.6 deterministic |
| A.4 | RAPL / energy counter (PLATYPUS) | Upper | S17.1 MSR 차단 + S17.2 virtio-rapl + S17.4 noise |
| A.5 | Thermal observable fingerprinting | Lower | S21.2 thermal MSR 마스킹 + S21.3 virtio-thermal |
| A.6 | Wake event fingerprinting | Upper | S22.3 lease binding + S22.4 spurious detection |
| A.7 | SMT cross-thread side-channel | Upper | S19.2 strict 동기 + S19.3 audit + S19.7 constant-freq + S19.8 lease assignment |

### B. Direct attack (host integrity / hardware 변조)

| # | 위협 | Layer | Mitigation |
|---|---|:---:|---|
| B.1 | Plundervolt voltage attack | Upper | S23.4 voltage range ±50 mV + S23.9 MSR `0x150` 차단 + S14 pending queue |
| B.2 | PSP / PCH power mailbox abuse | Upper | S23.2 S14 pending + S23.5 SMN/MMIO 격리 + S23.10 force-toggle |
| B.3 | ACPI _PSx / _CST / _PSV 우회 | Upper | S18.1 mediation + S18.2 화이트리스트 + S18.5 host operator only |
| B.4 | Wake-from-suspend replay | Upper | S20.4.2 epoch counter + S20.4.1 AEAD AD binding |
| B.5 | Wake source spoofing (Magic Packet) | Upper | S22.6 Y4-defined cryptographic signed packet + nonce replay |

### C. DoS / Resource exhaustion

| # | 위협 | Layer | Mitigation |
|---|---|:---:|---|
| C.1 | Thermal throttle / hardware damage | Upper | S21.1 thermal MSR 차단 + S21.4 _TCC host-only + S21.8 hardlimit emergency |
| C.2 | Lease suspend race | Lower | S20.3 atomicity (S13.2) + S20.4 integrity check |
| C.3 | Battery drain (handheld / laptop) | DoS | S22.4 spurious detection + 즉시 deep idle 재진입 + S22.10 InhibitAll |
| C.4 | Energy budget DoS (server-farm multi-tenant) | Upper | S17.8 per-VM energy budget + EnergyBudgetExceeded vmrun 거부 |

## 2. 새 위협 발견 row (Phase C 진입 후)

발견 시점 / CVE / 학술 논문 / 카테고리 / mitigation 가능성:

| 일자 | 출처 | 카테고리 | 위협 명칭 | mitigation 분석 | 적용 결정 |
|---|---|---|---|---|---|
| (현재 비어 있음) | — | — | — | — | — |

## 3. CVE 추적 (Phase C 진입 후 active)

power-related CVE 식별자 + 영향 분석:

| CVE | 일자 | 영향 | Y4 mitigation | row 매핑 |
|---|---|---|---|---|
| (현재 비어 있음) | — | — | — | — |

## 4. 학술 논문 추적

power-related side-channel / direct attack / DoS 학술 논문:

| Venue | 일자 | 논문 | 영향 | row 매핑 |
|---|---|---|---|---|
| (baseline 14 row 는 power-prior-art-ledger.md §1 — 본 tracker 는 신규 발견만) | — | — | — | — |

## 5. 갱신 path (`power_safety.md` §1.4 정합)

새 위협 발견 시 5-step path:

1. **위협 식별** — CVE 또는 paper publication
2. **카테고리 분류** — §1.2 의 A (side-channel) / B (direct attack) /
   C (DoS) 중 어디
3. **Mitigation 가능성 분석**:
   - 기존 S15~S23 의 sub-decision 갱신으로 차단 가능 → v1.x patch
     (sub-decision 추가 또는 const 조정)
   - 기존 메커니즘으로 부족 → **신규 S24+** 추가 (v1.x patch — `power_
     safety.md` §6.4 의 v1.x 분류)
4. **op_tag 추가** — 필요 시 S12.2 schema 의 v1.x patch 로 op_tag enum
   확장
5. **Verus invariant 추가** — `power_safety.md` §4 의 AV21~AV40 catalog
   의 reserved (AV36~AV40) 사용 또는 본 catalog 확장

## 6. 위협 carouseling 정책 — 카테고리 간 이동

위협이 시간에 따라 카테고리가 바뀔 수 있음 (e.g., side-channel 이 weaponize
되어 direct attack 으로 진화):

| 일자 | 위협 # | 이전 카테고리 | 새 카테고리 | 이유 |
|---|---|---|---|---|
| (현재 비어 있음) | — | — | — | — |
