<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: WWAN 모뎀 직접 제어 + 상위 OS 서비스 분배 — Y4 switch-hub (★ 기지국이 복제/이상 단말로 인식 절대 금지 = 단일 네트워크-컨텍스트 불변식)
created: 2026-08-27T00:08:13+09:00   # KST (UTC+9)
status: brainstorming (discussion 수렴 → 구조화).  결정 다수 + 미결 일부
scope: Y4 가 WWAN 모뎀(들)을 소유하고 네트워크엔 단일 단말로 보이며 서비스를 상위 OS 에 분배.
       VoLTE/VoNR·SMS/MMS(consumer) + PS-LTE/LTE-R/FRMCS(MC via MCX) 동등, form-factor-agnostic.
       switch-hub M×N / 복제-방지 3층 불변식 / 데이터경로(internet=route, IMS=terminate) /
       3단 failover / 2계층 credential+UICC / baseband IOMMU / 검증
inviolable: 기지국(네트워크)이 이 단말을 절대로 복제/이상 단말로 인식해서는 안 된다.
refs:
  - .brainstormings/20260812-174402-capsule-fault-isolation-restart.md §2(tier)·§4(IOMMU)·§4.2(FA6 partition coherence)·§3/§5(fault/restart)
  - .brainstormings/20260813-205326-key-management.md §2(use-without-extraction)·§3(partition key namespace)
  - .brainstormings/20260813-124610-side-channel-isolation.md §2(partition master key)
  - .brainstormings/20260618-225136-soft-vs-hard-real-time.md (MCX hard latency) + 20260813-134809-scheduler-design.md (RT)
  - .brainstormings/20260813-132525-attestation-measured-boot.md §5(device attestation SPDM)
  - docs/hw_mechanism_abstraction.md (modem/RAT/IMS/UICC/IOMMU realization 추상화)
  - CLAUDE.md §1(form-factor: handheld+dock / embedded SoC) / §6 원칙 1·2·6
---

# WWAN 모뎀 switch-hub — 복제-단말 인식을 구조적으로 불가능하게

## §0 프레임

**최종 구현 목표 기준.**  Y4 가 WWAN 모뎀(1개 또는 다수)을 소유하고, **네트워크
에는 정확히 "단일 단말"로 보이면서**, 상위 서비스(음성/메시지/데이터/MCX)를
상위 OS(guest)들에 **capability 로 분배**한다.  대상은 **consumer**(VoLTE/VoNR/
SMS/MMS)와 **mission-critical**(PS-LTE/LTE-R/FRMCS via 3GPP MCX)을 **동등**하게,
**form-factor-agnostic**(handheld+dock / embedded SoC / laptop …)하게.

> ★ **불가침 제약**: 기지국(네트워크)이 이 단말을 **절대로 복제/이상 단말로
> 인식해서는 안 된다.**  MC 에서 오검출-블랙리스트는 **안전 재난**.

## §0.5 원칙 정합

- **capability**(원칙 2): 서비스(voice/SMS/bearer/MCX)·권한(USIM-AKA)·forwarding
  entry = 전부 capability.
- **TCB 최소화**(원칙 1): 모뎀-제어 상태기계 = 최소 trusted; **baseband = 미신뢰-
  confined**; Y4 데이터 router = 최소 Rust-safe(guest 가 full 스택).
- **partition**: MC↔consumer 격리 · guest 간 격리(partition/lease).
- **formal-first**(원칙 6): ★ **복제-방지 = verified 불변식**(§2/§9).
- **atomic-free**: 제어 경로는 sequential trusted path.
- **hw_mechanism_abstraction**: modem I/F·RAT·IMS 위치·UICC 형태·IOMMU = 전부
  realization 추상화 → form-factor-agnostic(§10).

## §1 (결정) 구도 — switch-hub (≠ bridge)

- **M×N**: **uplink 포트 = 가입자/모뎀**(각각 Y4 소유의 **단일 네트워크-컨텍스트**),
  **downlink 포트 = guest OS**(서비스 capability 수령).  정책 주도 **forwarding
  table**: `(guest, 서비스클래스) → (가입자, bearer/identity)`.
- ★ **switch(terminate+re-originate) vs bridge(passthrough)**: bridge 였다면 라디오/
  NAS 를 guest 로 투명 전달 → guest 가 네트워크 정체성 획득 → 독립 registration/
  AKA → **복제 위험**.  switch 는 **네트워크-대면 컨텍스트를 종단**하고 guest 엔
  **로컬 인터페이스만 재기원** → 복제-방지를 **구조적으로 강제**.  "switch 로
  한다"는 선택 자체가 안전 메커니즘.

## §2 (결정) ★ 복제-방지 = 단일 네트워크-컨텍스트 불변식 (3층)

**근본 원인**: 하나의 IMSI 에 네트워크-대면 상태가 **복수** 존재 → 복제 판정.
핵심 기전 = **AKA + SQN 역동기화**(두 주체가 같은 USIM 자격으로 병렬 AKA →
USIM SQN 이 네트워크 기대와 어긋나 Sync failure) + 중복 registration/동시 위치갱신.

**3층 방어** (하나라도 뚫리면 복제 신호):

| 층 | 불변식 | 강제 수단 |
|---|---|---|
| **session** | 네트워크-대면 상태기계 종단 | switch(§1) |
| **registration** | 가입자당 단일 NAS + 단일 IMS registration; guest 자가등록 불가 | terminate-then-re-originate(§3) |
| **credential** | guest 는 USIM-AKA 구동 불가; vSIM ⟂ 실 K/AKA | 2계층 credential + UICC 유일중재(§5) |

★ **Verus 불변식**: `∀ 가입자 s: 활성 네트워크 컨텍스트 ≤ 1` ∧ `∀ guest g:
g 는 USIM-AKA(s) capability 없음 ∧ s 에 자가 registration 불가` ∧ `forwarding
table 이 한 가입자에 두 컨텍스트를 매핑하지 않음`.  MC 에서 "복제로 안 보임을
증명"이 killer feature.

## §3 (결정) 서비스 다중화 & 데이터 경로 — internet=route/NAT, IMS=terminate-then-re-originate ★

**서비스 클래스별로 guest-facing 가상 디바이스를 달리한다**:

| 서비스 | guest 가 보는 것 | Y4 처리 |
|---|---|---|
| **internet** | **가상 NIC (virtio-net)** | **route/NAT** (opaque IP) |
| voice (VoLTE/VoNR) | 로컬 IMS/SIP endpoint | **terminate-then-re-originate** (B2BUA) |
| SMS/MMS | SMS 인터페이스 | terminate → 라우팅 |
| MCX (MC) | MCX API/broker | terminate → broker |

- **근거**: **internet = 불투명 IP**(PDU session 은 이미 Y4 단일-컨텍스트의 bearer
  → guest IP 실어보내도 정체성 무관, 복제 위험 0) → **route/NAT** 가능.  **IMS
  서비스 = 정체성·registration 을 실음** → raw 를 guest 로 흘리면 독립 등록 가능
  → **반드시 종단**.  internet 이 route-가능한 **유일** 클래스.
- **Y4 역할**: 데이터 = **셀룰러 라우터/NAT 게이트웨이**(guest 가상 LAN(LAN) →
  셀룰러 PDU session/APN(WAN); DHCP·NAT·firewall·per-guest QoS·격리); 서비스 =
  **IMS B2BUA/broker**.
- **Y4-mediated demux 무조건**: baseband → **Y4-owned ring 에만 DMA** → Y4 가
  demux·검증 후 guest 로(per-guest 직접 DMA 경로 **없음**).  demux 키 = PDU
  session / NAT 매핑.  이 경계가 **미신뢰 baseband 출력의 파싱 방화선**(defense-
  in-depth).  Y4 router = 최소 Rust-safe L2/L3(guest 가 full TCP/IP) → TCB 최소.
- **NAT 차폐** + inter-guest 격리(별 가상 LAN) = 보너스 보안.

## §4 (결정) 3단 standard-first failover ladder

1. **모뎀-네이티브**(단일 컨텍스트 *내부* RAT/NSA/EN-DC/EPS-fallback/SRVCC/CA/
   inter-RAT 재선택) — **Y4 무개입**.  이는 3GPP 표준·인증 거동이고 기지국이
   기대하는 바 → Y4 개입은 이중-failover thrashing → **이상 거동 = 오검출 위험**.
2. **표준 서비스-계층 연속성**(IMS 연속성 · **MCX service continuity/resilience**)
   — Y4 재발명 X.
3. **Y4 hypervisor failover** — 위 둘이 못 넘는 경계에만: **cross-subscription /
   cross-modem**(FRMCS↔PS-LTE, modem-A↔modem-B), cross-operator.

★ **clean 경계 (no overlap / no gap)**: Y4 가 **모뎀 capability 조회** → 네이티브
지원의 **여집합만** 담당.  switch/broker 는 모뎀-네이티브 전환에 resilient(자기
failover 덧걸지 않음).  원칙 5(표준 재사용) + 복제-안전(예상범위 유지) 동시 만족.

## §5 (결정) 2계층 credential + USIM/eUICC 접근 모델

USIM 이 crux — AKA(`AUTHENTICATE` APDU, MILENAGE/TUAK)는 **K 가 카드 밖 미유출**
채 카드 *내부* 계산.  이 함수 구동자 = 네트워크 정체성 구동자.

- **Y4 = 유일 UICC-접근 권한(capability-gated)**: 모든 UICC/eUICC APDU 가 Y4
  UICC-broker 통과.  guest = **raw APDU/AUTHENTICATE 접근 0**.  "UICC-AKA
  capability" = 단일-컨텍스트 소유자만, guest 미부여.
- ★ **2계층 credential** (credential 층 terminate-then-re-originate):
  - **Y4 ↔ network** = 실제 USIM AKA(K in tamper-resistant 카드, 단일 컨텍스트)
  - **guest ↔ Y4** = 로컬 vSIM(key-mgmt 발급), 실 K/AKA 와 **암호학적 분리**
  - guest telephony 는 실 IMS 아닌 **Y4 로컬 IMS proxy** 에 등록 → 실 AKA 불요.
  ⟹ 손상 guest 는 로컬 creds 뿐 → **실 네트워크 인증 구조적 불가 → 복제 불가**.
- **APDU firewall**: `AUTHENTICATE`·키 자료 guest 미전달; 정책상 비민감 조회만
  큐레이팅.
- **eUICC/iSIM + 다중 가입자**: Y4 가 **LPA** 소유(프로필 download/enable/switch;
  guest 전환 불가); **MEP**(Multiple Enabled Profiles)로 다중 가입자; remote
  provisioning(SM-DP+) via Y4.  UICC 형태(physical/eUICC/iSIM) = realization.
- 정합: USIM = key-mgmt §2(use-without-extraction)의 **문자적 사례**; vSIM =
  partition-scoped key(key-mgmt §3); UICC-AKA = capability §2 단일-보유.

## §6 (결정) baseband IOMMU confinement

- **위협**: 거대·폐쇄·**미신뢰** DMA-capable 펌웨어, rogue gNB/기형 시그널링으로
  원격 공격 가능.  ★ **"직접 제어" = 제어 인터페이스(QMI/MBIM over confined rings)
  유일 소유**이지 baseband 내부 신뢰 아님 → baseband = 미신뢰-confined 3rd-tier
  capsule(capsule §2).
- **자기 IOMMU domain**(capsule §4.2 partition_id-keyed): DMA 를 **granted ring
  창(MHI transfer/event ring / 집적은 shared-mem QMI)에만** 국한 → guest/Y4/
  accelerator/타 device/USIM 자료 **도달 불가**.  **IR** 로 IRQ confine.
- **fault/restart**(capsule §3/§5): 감지 → **fence**(IOMMU/IR declaw) → **modem
  reset**(PCIe FLR / 집적 SSR) → 재-attach(단일 컨텍스트 재수립).  reset 간극은
  **tier-3 cross-modem failover**(§4)로 MC 유지.
- **IOMMU realization**: PCIe(VT-d/AMD-Vi, per-BDF, MHI) / 집적 SoC(**ARM SMMU**,
  stream-ID) / USB(DMA master 는 **xHCI** → xHCI confine + USB 데이터 검증) —
  추상화(§10).
  - ※ **집적 SoC 모뎀은 ARM 에 한정하지 않는다**: **RISC-V(RISC-V IOMMU 표준
    스펙, device-ID)** 나 기타 ISA(POWER/ARCv3 등, cross-platform 발제)를 채택한
    SoC 모뎀도 염두 — SoC IOMMU realization 은 ISA 별로 다르되 "baseband 은
    granted ring 만 도달" 계약은 불변(§10 hw_mechanism_abstraction).
- 펌웨어 무결성 = modem secure boot + **device attestation(SPDM, attestation §5)**;
  IOMMU 는 서명 펌웨어에도 **defense-in-depth**.
- 불변식(Verus): `baseband IOMMU domain ⊆ {Y4 모뎀 ring}` ∧ fence-before-reclaim
  (capsule FA3/FA6 확장).

## §7 (결정) MC + consumer 동등 + partition 격리

- 둘 다 1급 서비스 클래스.  **MC = QoS/preemption 우선**(MCPTT floor/preemption,
  전용 QCI/5QI bearer) + **consumer 로부터 강한 격리**(partition/lease) — 손상된
  consumer guest 가 MCX 를 못 건드림.
- **form-factor-agnostic**: modem I/F·RAT·IMS 위치·UICC·IOMMU 전부 realization
  추상화(§10) → 특정 form-factor 에 안 묶임.

## §8 attestation & 규제/인증

- MC 네트워크/MCX 가 요구하는 **device integrity** → Y4 measured-boot(attestation)
  로 UICC-broker·switch 가 verified Y4 임을 증명; eUICC SGP.22 인증서/EID.
- **인증(GCF/PTCRB/operator IOT)**: Y4 가 제어 경로(IMS proxy/switch)에 있으므로
  인증 대상에 포함 — 고려 필요(§11 미결).
- ★ MC 에서 오검출-블랙리스트 = 안전 재난 → **"복제로 안 보임"의 형식 증명**(§2/§9)
  이 차별점.

## §9 verification

- 복제-방지 불변식 계층 = **Verus 대상**: §2 3층(session/registration/credential)
  + §5 2계층 credential + §6 baseband confinement.  capability + information-flow
  불변식.
- **경계**: crypto primitive 강도(AKA/MILENAGE/TUAK)는 **trusted assumption**
  (USIM 은 인증된 tamper-resistant 요소) — Verus 는 "정체성 컨텍스트가 단일함"을
  증명하지 "암호가 안전함"을 증명하지 않는다(key-mgmt §9 경계와 동형).
- ⏳ verus-fork 안정화 이후 착수(다른 Verus 대상과 동일 타이밍).

## §10 hw_mechanism_abstraction 정합

| realization (정하지 않음) | 계약 (Y4 고정) |
|---|---|
| modem 제어 I/F: QMI / MBIM / AT | switch-broker 인터페이스 |
| RAT: LTE / 5G NR (SA/NSA) | 서비스 연속성 계약(§4) |
| IMS 위치: on-modem / host 스택 | 단일 registration 종단(§3) |
| UICC 형태: physical / eUICC / iSIM | Y4 유일 중재 + AKA 미노출(§5) |
| IOMMU: VT-d / AMD-Vi / ARM SMMU / **RISC-V IOMMU** / 기타 ISA SoC IOMMU / xHCI | baseband 은 granted ring 만(§6) |

계약 = switch-hub + 단일-컨텍스트 + 복제-방지 불변식.  realization 다양성이
form-factor-agnostic 을 준다.

## §11 결정 / 미결 요약

**결정 방향(강)**:
- switch-hub(≠bridge) M×N + 정책 forwarding(§1)
- ★ 복제-방지 = **단일 네트워크-컨텍스트 3층 불변식**(session/registration/
  credential), Verus 증명(§2/§9)
- 데이터경로: **internet=route/NAT, IMS 서비스=terminate-then-re-originate**;
  Y4-mediated demux 무조건; 서비스클래스별 가상 디바이스(§3)
- **3단 standard-first failover ladder** + capability-조회 clean 경계(§4)
- **2계층 credential + UICC 유일중재**(vSIM ⟂ 실 AKA), eUICC MEP(§5)
- **baseband IOMMU confinement**(미신뢰 3rd-tier, granted ring 만) + fault→tier-3
  failover(§6)
- MC=consumer 동등 + partition 격리(§7) · hw_mechanism_abstraction 추상화(§10)

**미결(설계 필요)**:
- MCX 깊이(MCPTT floor control / MCData / MCVideo / MC KMS 정합) — ⏳ MCX 표준
  정합 별도
- IMS 위치 기본 권장(on-modem vs host) — realization 이나 기본값 가이드
- 다중화 시 operator-provisioned multi-identity(IMS multi-IMPU/multi-device)
  의존 범위 vs 단일-단말-거동 유지
- vSIM 로컬 인증 프로토콜 구체(guest↔Y4)
- 인증(GCF/PTCRB/operator IOT) 전략 — Y4 가 인증 경계에 포함되는 범위

**⏳ 선행 의존**:
- verus-fork 안정화 — 복제-방지 형식 증명(§9)
- 실제 modem HW/드라이버 + IOMMU 드라이버(capsule roadmap)
- MCX 표준 스택(3GPP TS 23.280/23.379 계열)

## §12 다음 발제 후보

- observability / telemetry in minimal TCB
- (사용자 지시 대기)
