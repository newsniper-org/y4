<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# 하드웨어/플랫폼/디바이스 메커니즘 추상화 정책

## 0. 지위 / 출처

Y4 의 **cross-cutting 설계 정책**.  `.brainstormings/20260813-132525-
attestation-measured-boot.md` §8.5 (opinion 1·2·3 통합)에서 도출되어 본
도메인 문서로 승격(2026-08-13).  attestation 맥락에서 처음 명료화됐으나 적용
범위는 Y4 전반 — CPU 가상화(`cpu_virt_compat.md`) · cross-platform · Root of
Trust · confidential computing · device attestation 등.

engineering 원칙(`CLAUDE.md` §6)의 **원칙 5(verified base + specialization-
only)** · **원칙 3(Rust-first, C 는 capsule ABI 뒤로)** · **원칙 6(formal-
first)** 을 "하드웨어 메커니즘" 축으로 구체화한 것.

## 1. 원칙 (normative)

> **Y4 spec 은 하드웨어·플랫폼·디바이스가 제공하는 벤더·ISA·디바이스별
> 메커니즘의 realization 을 고정하지 않는다.**  Y4 는 **추상 계약(abstract
> contract)만 고정**하고, 구체 backend·범위·구현시점은 **deployment ·
> form-factor(§D2 arch rollout) · cross-repo ABI** 가 결정한다.
>
> 메커니즘을 사용하는 코드는 **어떤 realization 이 쓰였는지 faithfully 보고**
> 할 수 있어야 하며, **수용 여부의 판단은 그 정보를 받는 측(relying party /
> remote verifier)의 정책**이다.

세 개의 하위 규칙:

1. **계약만 고정 (contract-only spec).**  Y4 는 각 메커니즘 class 에 대해
   trait/ABI 형태의 추상 계약을 정의한다.  계약은 backend-중립적이어야 하며
   특정 벤더/ISA/디바이스 용어를 계약 표면에 노출하지 않는다.
2. **realization 은 위임 (deferred realization).**  어떤 concrete backend 를,
   언제, 어느 arch/form-factor 에서 구현할지는 spec 이 아니라 form-factor
   rollout(§D2: x86_64 first, 타 arch 는 그 form-factor 작업 개시 때)과
   cross-repo ABI(예: WaveTensor RTL, `hiu_abi.md`)가 결정한다.
3. **provenance 보고 + relying-party 정책 (faithful provenance).**  보안
   관련 메커니즘은 사용된 realization 을 **discoverable / attestable** 하게
   노출해야 한다.  수용/거부는 Y4 가 spec 으로 못박지 않고 relying party
   (attestation 의 tenant 등)의 정책에 맡긴다.

## 2. 계약 vs realization — 무엇을 정하고 무엇을 안 정하나

| 메커니즘 class | Y4 가 고정하는 추상 계약 | Y4 가 정하지 않는 realization |
|---|---|---|
| CPU 가상화 | vendor-neutral VMM 인터페이스 (`cpu_virt_compat.md`) | AMD-V(SVM) / Intel VT-x / RISC-V H-ext |
| Root of Trust | TPM 2.0 conformance (PCR extend / quote / EK-AK 체인) | discrete dTPM / firmware fTPM / 기타 |
| measured-boot 개시 | RoT trait 이 measurement provenance 보고 | SKINIT / SENTER / RISC-V + DRTM·SRTM 택 |
| measurement-log 접근 | `MeasurementLogSource`: TCG2 event log + PCR/MR 획득(공통 파서·PCR replay) | UEFI config table / ACPI `TPM2` / coreboot cbmem / device-tree SML |
| confidential VM | vendor/ISA-neutral confidential-VM capability | AMD SEV-SNP / Intel TDX / RISC-V CoVE |
| device attestation | "device 가 identity+config 를 attest" | SPDM 채택 범위 (⏳ `hiu_abi.md` / RTL) |
| (일반) arch-특화 코드 | trait 뒤의 arch-neutral 로직 | per-arch backend (`sel4_backend` 형태) |

이 표는 **living registry** — 새 하드웨어 메커니즘이 등장하면 (추상 계약,
realization) 쌍으로 항목을 추가한다.

## 3. 경계 — 이 정책이 적용되지 않는 것 (Y4 가 직접 소유·설계)

이 정책은 **외부 하드웨어/플랫폼/디바이스 메커니즘**에만 적용된다.  **Y4 자신이
저작·산출하는 것**은 추상화-후-위임 대상이 아니라 Y4 가 설계·고정·검증할
deliverable 이다:

- **Y4 자신의 build·release 산출물** — RIM(Reference Integrity Manifest)
  공개, reproducible build 파이프라인, 서명 정책.
- **Y4-authored capability 모델 / spec / Verus·Rocq·Isabelle proof.**
- **Y4 의 IPC · allocator · capsule 격리 semantics.**

판별 기준: **"외부(벤더/ISA/디바이스)가 제공하는 메커니즘인가?"** → 예면 §1
정책(추상화, spec 안 함); **"Y4 가 만들어내는 산출물인가?"** → 예면 Y4 소유
(설계·고정).

## 4. 형식 검증과의 관계 (원칙 6)

- **추상 계약 = 검증 표면.**  Verus 로 증명되는 property 는 **추상 계약**에
  대해 진술한다("이 계약을 honoring 하는 어떤 backend 위에서도 성립").
- **backend = trusted boundary.**  concrete backend(`sel4_backend` 류)는
  계약을 honor 해야 하는 trusted 코드 — 계약 준수는 per-backend 확인(일부는
  trusted, 일부는 검증).  이는 Y4 가 이미 쓰는 패턴(`sel4_backend`,
  `ConfigSpace` mock↔real)의 일반화.
- 결과: backend 가 늘어도(새 벤더/ISA) **상위 property 재증명 불요** — 계약
  표면이 불변이므로.  cross-platform 확장 비용을 낮춘다.

## 5. governance — 새 메커니즘의 진입

1. 새 하드웨어 메커니즘 식별 → §2 registry 에 (추상 계약, realization) 쌍 추가.
2. 추상 계약을 trait/ABI 로 정의(backend-중립 표면).
3. 첫 backend 는 현재 form-factor(§D2: x86_64)에서 구현; 타 arch backend 는
   그 form-factor 작업 개시 때.
4. 보안 관련이면 §1 규칙 3(provenance 보고) 준수 — realization 을 attestable
   하게 노출.

## 6. cross-reference

- `docs/cpu_virt_compat.md` — 본 정책의 첫 구체 사례(AMD-V↔VT-x 벤더 중립성)
- `.brainstormings/20260618-231936-cross-platform-strategy.md` — multi-ISA
  (본 정책의 arch 축)
- `.brainstormings/20260813-132525-attestation-measured-boot.md` §8.5 — 본
  정책의 도출 기록(RoT / confidential-VM / device attestation)
- `docs/hiu_abi.md` + WaveTensor RTL — device attestation realization 의
  cross-repo ABI 출처
- `CLAUDE.md` §6 원칙 3·5·6 / §8 D2(arch rollout)
