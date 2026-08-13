<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Attestation / measured boot — tenant 가 자기 데이터가 도는 TCB 를 암호학적으로 검증 (★ verified base 에 bind 하는 attestation)
created: 2026-08-13T13:25:25+09:00   # KST (UTC+9)
status: brainstorming (결정: §1 secure+measured 합성 / §2 DRTM 선호+vendor-neutral RoT / §3 verified base bind / §5 device attestation 별도 / §6 주 stance=Y4 trusted / ★§8.5 하드웨어 메커니즘 realization 은 spec 안 함 원칙 → docs/hw_mechanism_abstraction.md 로 승격).  side-channel §10 에서 예고
scope: multi-tenant accelerator hosting 의 remote attestation.
       secure vs measured boot / root of trust(SRTM·DRTM, AMD·Intel) / 측정 체인 /
       remote attestation 프로토콜 / device(accelerator) attestation / confidential VM 관계 / 검증
refs:
  - boot/{limine.conf,sel4.rules,README.md} (Limine→seL4→Y4 부팅 체인)
  - docs/cpu_virt_compat.md (AMD-V↔VT-x vendor-neutrality — RoT abstraction 토대)
  - docs/amdv_safety.md (AMD-V 안전장치 — SVM/SKINIT 맥락)
  - MEMORY/wavetensor_terms.md (TRNG / XChaCha20 256-bit key — nonce·session key)
  - .brainstormings/20260813-124610-side-channel-isolation.md §2/§10 (partition_id master key + attestation 예고)
  - .brainstormings/20260812-174402-capsule-fault-isolation-restart.md §2.1/§4.2 (attested surface = capsule 정책·partition)
  - .brainstormings/20260618-231936-cross-platform-strategy.md (multi-ISA — confidential-VM 벤더/ISA 중립 추상화 맥락, RISC-V CoVE)
  - CLAUDE.md §6 원칙 1(TCB 최소화)·5(verified base)·6(formal-first) / §8 D2(x86_64 first, arch rollout)
---

# Attestation / measured boot — verified 최소 TCB 를 tenant 에게 증명

## §0 프레임

**최종 구현 목표 기준.**  server-farm form factor 에서 tenant 는 자기 데이터가
**어떤 TCB 위에서 도는지**를 신뢰해야 한다.  원칙 1(TCB 최소화)이 "TCB 가
작다"는 *주장*이라면, **attestation 은 그 주장을 tenant 에게 암호학적으로
증명**하는 수단.  side-channel §10 에서 예고: "tenant 가 side-channel 방어
(+TCB)가 실제 활성임을 검증".

baseline: 부팅 체인(Limine→seL4→Y4), WaveTensor TRNG/XChaCha20, **형식 증명된
seL4**, AMD-V(cpu_virt_compat).

## §0.5 원칙 정합

- **atomic-free**(IPC §0.5): measurement 는 boot/lease-switch 의 sequential
  trusted path, log 는 append-only **single writer** — cross-CPU atomic 없음.
- **TCB 최소화**(원칙 1): ★ attestation 이 원칙 1 을 tenant 에게 *증명*하는
  수단이자, **attested surface 자체를 최소화**(측정 대상이 작을수록 강한 주장).
- **formal-first**(원칙 6): measurement/quote 로직을 Verus 로, 그리고 ★
  **attestation 을 verified base(seL4)에 bind**(§3).
- **capability**: attestation 을 lease/session key 에 bind(§4/§7).

## §1 (결정) secure boot vs measured boot — Y4 는 둘 다

- **secure boot**: 각 stage 가 **다음 stage 서명 검증**, 불량 시 halt.
  boot-time 로컬 **fail-closed**.
- **measured boot**: 각 stage 가 다음 stage 를 **해시 측정 → tamper-evident
  log(TPM PCR) extend → 실행**.  정책 판정은 이후 **remote attestation**(원격
  verifier = tenant)이 수행.
- **결정 — 합성**: secure boot(로컬 fail-closed) + measured boot(tenant
  검증용 attestable chain).  둘은 상보 — 전자는 즉시 차단, 후자는 원격 증명.

## §2 (결정) Root of Trust — DRTM 선호 + vendor-neutral

- HW RoT = **TPM 2.0-conformant RoT** + CPU 측정 명령.  **구체 realization
  (discrete dTPM / firmware fTPM / 기타)은 Y4 가 세세히 정하지 않는다** —
  abstract RoT trait 뒤의 **deployment·form-factor 선택**.  Y4 의 계약은 **TPM
  2.0 conformance**(PCR extend / quote / EK-AK 체인)뿐.  (threat-model 상
  dTPM 이 측정 TCB 가 작아 DRTM 정신과 부합하나 **mandate 아님**.)
- ★ **vendor-neutral**(cpu_virt_compat 확장): **AMD SKINIT + AMD-SP(PSP)**
  ↔ **Intel TXT/SENTER + SINIT ACM**.  abstract RoT trait + arch backend
  (AMD-V↔VT-x 벤더 중립성의 attestation 축).
- **SRTM vs DRTM**:
  - SRTM = power-on 부터 firmware→bootloader→… 전부 측정 → 측정 TCB 큼(firmware
    포함).
  - DRTM = late-launch 명령(SKINIT/SENTER)이 측정 환경을 **중간에 reset** →
    firmware/Limine 제외한 **작은 측정 TCB**.
- **결정 — DRTM 선호**: 측정 TCB 를 최소화(원칙 1 정합, 지저분한 firmware
  제외).  SRTM 은 fallback(DRTM 미가용 플랫폼).  단 구체 late-launch 명령
  (SKINIT/SENTER/RISC-V 등)과 DRTM/SRTM 중 무엇이 쓰였는지는 **attestation 이
  faithfully 보고**하고 수용은 **remote verifier(tenant) 정책** — Y4 는
  fallback 경계를 spec 으로 못박지 않는다(§8.5).

## §3 (결정) 측정 체인 — attested surface + verified base bind ★

DRTM 선호 시: **SKINIT/SENTER → 측정된 launch env = seL4 + Y4**(firmware/
Limine 제외).  측정 항목:

- **seL4 binary** — ★ **== 형식 증명된 seL4 버전**
- Y4 root task + **capsule 정책**(어느 driver, 격리 config; capsule §2.1)
- **partition config**(partition_id 배정; side-channel §2 master key)
- **side-channel 방어 활성**(time protection / SMT gang / cache coloring;
  side-channel §10)
- lease terms(본 tenant 세션에 bind; §7)

★ **verified base 에 bind (Y4 고유 차별점)**: 측정된 seL4 해시 == tenant 가
신뢰하는 *proof 의* seL4 버전 해시.  ⟹ attestation 이 "these bytes booted"
가 아니라 **"형식 증명된 microkernel 이 부팅됐다"** 를 증명.  **measured boot
× formal verification 결합** — 기존 attestation(측정만) 대비 Y4 만의 주장
(paper §).

## §4 (결정) remote attestation 프로토콜

```
tenant(verifier) --nonce--> Y4/TPM
                            quote = Sign_AK(측정 log ‖ nonce ‖ session-binding)
tenant <--quote-- Y4
tenant: (1) 서명 체인 검증 AK → EK/VCEK → vendor root (genuine HW)
        (2) 측정값 대조 reference/RIM (known-good Y4 release)
        (3) nonce freshness (replay 방지)
        (4) session/lease binding 확인
```
- freshness **nonce** = tenant 제공; platform 측 randomness = **WaveTensor
  TRNG**.
- ★ **session/lease binding**: quote 에 **session key + granted lease
  capability** 를 bind → relay/TOCTOU 방지("attested TCB == session key 를
  쥔 TCB").
- Y4 는 **release 별 expected 측정값(RIM)을 공개**(reproducible build 뒷받침)
  — verifier 가 대조할 known-good.

## §5 (결정) device attestation — accelerator 는 별도 ★ (놓치기 쉬움)

플랫폼 attestation(§3~4)은 CPU 메모리·SW 만 커버한다.  **WaveTensor
accelerator 내부 상태**(partition/lease/masking config, RTL·firmware
identity)는 **별도 device attestation** 이 필요.

- 표준: **SPDM**(DMTF DSP0274, PCIe/CXL device attestation) + PCIe IDE.
  WaveTensor 가 SPDM-유사 attestation(accelerator identity + partition
  config)을 노출.
- **완전한 tenant attestation = 플랫폼(§3~4) ⊕ device(§5)**.  둘 중 하나만으론
  불충분.
- ⏳ `hiu_abi.md` v1.0 frozen + WaveTensor RTL 측 SPDM 노출(cross-repo ABI)에
  의존.

## §6 (결정) confidential computing 관계 — Y4 trusted vs SEV-SNP/TDX ★ 긴장

- **주 stance (a) — Y4 = trusted 최소-TCB hypervisor**: tenant 는 measured
  boot 로 Y4 를 신뢰(§3~4).  Y4 는 *자기로부터 숨을 필요가 없다* — verified
  seL4 + 측정이 신뢰 근거.  원칙(최소 TCB + verified)과 정합 → **주 노선**.
- **부 stance (b, 선택적) — confidential VM guest**: tenant 가 Y4 조차
  불신하려면 hypervisor 를 **adversary 로 두고** HW 가 guest 를 암호화·attest.
  이 메커니즘은 **vendor/ISA-neutral 추상 capability** 로 모델링 — 구체 backend
  = **AMD SEV-SNP / Intel TDX / RISC-V CoVE**(Confidential VM Extension;
  smmtt + TSM).  이들 모두 **hypervisor 를 적으로 가정** → Y4 의 "trusted
  capability provider" 모델과 철학적으로 상충 → **defense-in-depth 옵션**으로만,
  주 노선은 (a).
- ★ **RISC-V CoVE 시사점**: RISC-V 가 이를 **ISA 표준 자체에 포함**시키는
  흐름은, confidential-VM 이 벤더 확장이 아니라 **portable ISA-level primitive**
  로 수렴함을 뜻함.  ⟹ Y4 는 "우선순위를 정하기"보다 **추상화 뒤에 두는 게
  옳다** — opinion-1 의 RoT 추상화 / cpu_virt_compat 벤더중립성 / cross-platform
  발제와 동일 패턴.  **backend 우선순위는 attestation spec 이 아니라
  form-factor arch rollout(D2: x86_64 first, 타 arch 는 form-factor 개시 때)이
  결정.**
- ★ 주의: SEV-SNP/TDX/CoVE 모두 **CPU 메모리만** 보호 — accelerator 내부는
  여전히 §5 device attestation 필요(confidential VM 이 accelerator 를 못 가림).

## §7 세션 binding + key 관리

- **nonce**(TRNG) freshness / **session key ↔ quote** bind / **lease
  capability ↔ quote** bind → 3중 결합으로 attested TCB 를 이 세션에 고정.
- **AK**(hardware-rooted, TPM EK/AMD VCEK 체인) / session = XChaCha20 256-bit
  key.  key lifecycle 은 **TCB 의 일부** → 검증 대상(§8).

## §8 verification (formal-first)

- measurement/quote 로직 **Verus**: "quote 가 측정된 컴포넌트를 **정확히
  반영**"(누락/위조 불가) / "nonce freshness → **replay 방지**" / "session·
  lease binding 정확".
- ★ **verified base bind**(§3): attestation 이 임의 바이트 아니라 **verified
  seL4** 를 증명 — measured boot × formal verification 의 결합을 명시적 주장.
- append-only measurement log = single trusted writer(§0.5).

## §8.5 (결정 원칙) 하드웨어/플랫폼/디바이스 메커니즘은 추상화 — realization 은 spec 하지 않는다

> **→ 본 원칙은 도메인 문서 `docs/hw_mechanism_abstraction.md` 로 승격됨
> (2026-08-13).  이하는 최초 도출 기록(historical reference) — canonical
> 정의는 그 문서.**

opinion 1(RoT)·2(confidential-VM)·3 이 하나의 원칙으로 수렴:

> Y4 attestation spec 은 하드웨어/플랫폼/디바이스가 제공하는 **벤더·ISA·
> 디바이스별 메커니즘의 realization 을 고정하지 않는다.**  **추상 계약만
> 고정**하고, 구체 backend·범위·구현시점은 **deployment · form-factor(D2) ·
> cross-repo ABI** 가 결정.  attestation 은 *어떤 realization 이 쓰였는지*
> faithfully 보고하고, **수용 여부는 remote verifier(tenant) 정책**.

이 원칙으로 **"spec 하지 않음"** 처리되는 것들:

| 항목 | 추상 계약 (Y4 가 고정) | realization (정하지 않음) |
|---|---|---|
| platform RoT | TPM 2.0 conformance | dTPM / fTPM / 기타 (opinion 1) |
| measured-boot 명령 | RoT trait 이 provenance 보고 | SKINIT / SENTER / RISC-V + DRTM·SRTM 택 |
| confidential VM | vendor/ISA-neutral capability | SEV-SNP / TDX / CoVE (opinion 2) |
| device attestation | "device 가 identity+config attest" | SPDM 채택 범위 (⏳ hiu_abi/RTL) |

**경계 — 이 원칙에 해당하지 않는 것 (Y4 가 직접 소유·설계)**: **RIM 공개 +
reproducible build 파이프라인**은 외부 하드웨어 메커니즘이 아니라 **Y4 자신의
build·release 산출물**.  추상화 대상이 아니라 Y4 가 설계·구현할 deliverable
— 별도 발제로 다룸(§10).

## §9 결정 / 미결 요약

**결정 방향(강)**:
- secure boot + measured boot 합성(§1)
- **DRTM 선호 + vendor-neutral RoT**(AMD SKINIT/PSP ↔ Intel TXT, §2)
- **RoT realization 은 추상화** — Y4 계약은 TPM 2.0 conformance 뿐, dTPM/
  fTPM/기타는 deployment·form-factor 선택(Y4 spec 아님, §2)
- ★ **verified base(seL4)에 bind** — measured boot × formal verification(§3)
- nonce(TRNG) + session key + lease capability **3중 binding**(§4/§7)
- **device attestation(SPDM) 별도**, 플랫폼 ⊕ device(§5)
- 주 stance = **Y4 trusted**; confidential-VM(SEV-SNP/TDX/**RISC-V CoVE**)은
  **vendor/ISA-neutral 추상 capability** 의 defense-in-depth 옵션 — 우선순위는
  form-factor arch rollout(D2)이 결정, attestation spec 아님(§6)
- ★ **§8.5 원칙**: 하드웨어/플랫폼/디바이스 메커니즘의 realization 은 spec
  하지 않고 추상 계약만 고정(RoT / measured-boot 명령 / confidential-VM /
  device attestation) — 무엇이 쓰였는지는 attestation 이 보고, 수용은 tenant
  정책

**미결(설계 필요) — §8.5 로 "spec 안 함" 처리 후, Y4 가 직접 소유하는 것만 남음**:
- **RIM 공개 + reproducible build 파이프라인** 구체 (외부 메커니즘 아닌 Y4
  build·release 산출물, §8.5 경계 — 별도 발제 §10)

**⏳ 선행 의존**:
- `hiu_abi.md` v1.0 frozen — device attestation config 출처(§5)
- WaveTensor RTL 측 SPDM 노출 — cross-repo ABI(§5)
- boot 실제 구현 — Limine measured-boot 지원 vs DRTM 경로(§2~3)

## §10 다음 발제 후보

- **scheduler design** — SMT gang / lease switch / RT budget / μarch flush
  경계 통합
- key management deep-dive — AK/session key lifecycle, TRNG/XChaCha20 결합
- reproducible build / supply-chain — RIM 공개의 전제(§4)
