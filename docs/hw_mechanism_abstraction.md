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
| HIU 장치 클록 하한 (2026-10-09) | 플랫폼이 장치 클록 하한 `f_dev_min` 을 알린다 — Y4 는 사상표 소거 대기 ≥ 2^LNID_W / f_dev_min 로 쓴다 (`hiu_abi.md` §2.5) | 보드 기술 · 펌웨어 표 |
| HIU 장치 reset 경로 (2026-10-09 · 2026-10-22 보강) | 플랫폼이 *증명한* reset 경로만 쓴다 — 없으면 장치 격리 뒤 결함 표시 (`hiu_abi.md` §2.6). 그리고 **플랫폼이 알리지 않는 장치 reset(링크 · 기능 수준 reset 포함)은 없다** — 장치 reset 은 Y4 에 통지되거나 플랫폼 reset 과 묶인다. 정본 4항의 결속 전제(「마지막 장치 reset 뒤에」 `e-defined.typ:208`)를 Y4 step 장부가 거짓으로 믿지 않게 하는 전제이고, 「장치 reset 을 플랫폼 reset 에 잇는 것은 플랫폼 결선의 몫이다」(`:216`). reset 세대 장치 워드는 v1.x 등재다(`b-gaps.typ:2126` ⑹ @ `395e5b1`; `hiu_abi.md` §2.5 5e) | FLR ↔ reset 핀 결선 · 보드 리셋 회로 · reset 통지 경로 |
| HIU 창 거절 비트 가시 대기 상한 (2026-10-16 · 2026-10-18 정정) | 플랫폼이 구성 *기입의 거절*이 낸 loud 비트(창 거절 집합 W = `[5]` · `[6]` · `[7]` · `[15]` 가운데 기입 시점 원천)가 `0xF80` 에 보이기까지의 상한 `T_FLT` 를 알린다 — Y4 는 `0xE14` A = 0 뒤 `[6]` 폴링, OWN_SET 되읽기가 거절을 보인 뒤 `[7]` 확인, FP 관문(적재 뒤 `[15]` 확인)과 창 · 회전 기준선 대기에 쓴다. 정본은 사이클 수를 걸지 않으므로 ABI 가 아니다. **실행 시점 사건의 상한이 아니다** — runaway(「`[15]` 의 원천」 (4))의 `[15]` 가 언제 보이는가도, 그것이 `0xF08` `[1]` 의 하강보다 늦게 보이지 않는다는 것도 정본은 규범으로 두지 않는다 (WaveTensor 정본 `b16fc07` `e-defined.typ:466`, `hiu_abi.md` §2.3 ⑷ · §2.4 3 · §2.7) | 버스 · 어댑터 지연 표 |
| HIU step 걷기 정지 대기 상한 (2026-10-18 · 2026-10-22 정정) | 플랫폼이 선택 열쇠의 결속을 거둔 `0xF04` 기입이 수락된 뒤 `0xF08` `[1]` = 0 이 관측되기까지 기다릴 상한 `T_WALK` 와 연속 소진 상한 `K_WALK`(≥ 1)를 알린다 — Y4 는 재적재 전 정지 확인(`fp_quiesce` — 정본 E.1.1 「호스트 해석 규범」 4항의 재적재 순서, `e-defined.typ:208`; 그 관측이 덮는 것은 FP_STATUS 행 「`[1]` 의 읽기 순서」 `:463`)의 폴링 상한으로 쓴다. 정본은 「반영 전이면 장치는 그 읽기의 수락을 반영 뒤로 미룰 수 있다」(`:463` — 유계 장치 의무)고 하므로 `T_WALK` 는 그 읽기 정지를 포함한 상한이다. **MMIO 읽기 완료 시한은 장치의 유계 읽기 정지(`0xF08` · `0xE00` COMMIT 배압)보다 길고, 읽기 완료 오류는 데이터(all-ones)로 Y4 에 오지 않는다** — 오류가 all-ones 로 읽혀 E.1.1 2항 마스크를 거치면 `[1]` = 1 로 보여 소진으로 조용히 흡수된다. `[1]` 은 장치 전역 순간값이고 「v1.0 에서 일의 원천은 외부 제시라 투입을 멈출 수단이 없다」(`e-defined.typ:332`) — 그래서 소진은 장치 결함이 아니라 능력 부재로 보고 그 창의 step 적재를 생략하며, `K_WALK` 창 연속으로 소진되면 그 장치의 step 기능만 끈다(`StepOff` — 장치 격리 아님, 2026-10-22 사용자 결정; Verus `fp_quiesce_action`, `hiu_abi.md` §2.7) | 버스 · 어댑터 지연 표 · 완료 시한 설정 · 투입 부하 특성 |
| HIU 구성 페이지 접근 속성 (2026-10-16 · 2026-10-18 보강) | 구성 페이지는 UC · 비 write-combining 이고, 경로가 32b 정렬 접근을 쪼개거나 합치지 않는다 — 정렬 주소의 부분 바이트 쓰기는 정본 경계 밖이다 (`hiu_abi.md` §1.2.1 (C)). 그리고 경로(가속기 쪽 엔드포인트 브리지 · 호스트 버스 어댑터 포함)는 한 구성 페이지에 대한 **쓰기 → 읽기 순서를 보존**한다 — 뒤에 낸 읽기가 앞의 쓰기보다 먼저 장치 포트에 닿지 않는다. 정본 E.7 7항(OWN_SET · OWN_CLR 뒤 OWN_STATUS 되읽기)과 FP_STATUS 행 「`[1]` 의 읽기 순서」(en = 0 기입 뒤의 `0xF08` 읽기)는 *장치 포트에서의* 순서만 약속하고 「끝-끝 순서는 플랫폼의 성질이다」(`e-defined.typ:1388` · `:463`) — 두 확인이 이 전제에 선다. 정본의 보장 조건은 「버스 수락된 *뒤에* 버스 수락된」 읽기이므로 이 전제는 같은 사이클 수락을 허용하지 않는 꼴로 읽는다(WaveTensor 쪽 정본 경계 밖 — 제품 · 플랫폼의 약속, C12) | MTRR/PAT · IOMMU · 브리지/어댑터 설정 |
| HIU LBID 슬롯 수 | 플랫폼(또는 v1.x 장치 워드)이 실재 물리 슬롯 수를 알린다 — Y4 는 LBID 대행 검증의 상한으로 쓴다 (`hiu_abi.md` §2.8) | 보드 기술 · 펌웨어 표 |
| HIU LNID 등록 용량 · 배달 지점 수 (2026-10-16) | 플랫폼(또는 v1.x 장치 워드)이 LNID 등록 수락 용량과 실재 배달 지점 수를 알린다 — Y4 는 `lnid_write_ok`(슬롯 < 등록 용량 · 지점 < 배달 지점 수)의 상한으로 써서 「용량 밖 슬롯」 · 「없는 배달 지점」 거절(`[6]`)을 일으키지 않고 피한다. 정본은 두 수를 `0xFF8` 로 알리지 않는다 (`hiu_abi.md` §2.8). 용량 밖 거절은 V=1 등록에만 걸리고 V=0 기입은 용량 밖이어도 효과도 fault 도 없다(정본 `b16fc07` `03-hiu.typ:1072` — D3; 「없는 배달 지점」도 V = 1 등록에만 — `:1075`, C7) — 그러니 V=0 에까지 거는 `idx < cap` 은 Y4 의 보수 정책이다 | 보드 기술 · 펌웨어 표 |
| HIU step 프로그램 깊이 P · stepMax (2026-10-16 · 2026-10-18 정정) | **장치가 `0xF0C` FP_SHAPE 로 알린다** — `[8:0]` $P$ · `[17:9]` stepMax · `[31:24]` 부호화 번호 (정본 `b16fc07` `e-defined.typ:463`). Y4 는 `0xFFC` major = 1 일 때만 해석하고 정의된 비트로 가린 값(`& 0xFF03_FFFF`)을 `fp_prog_ok`(슬롯 < P · 색인 < stepMax) · `fp_bind_ok` 의 상한으로 써서 「`[15]` 의 원천」 (1) · (3) 을 일으키지 않고 피한다. 플랫폼 레지스트리의 (P, stepMax)는 **교차 확인용**으로 내린다 — 판 0 · mock 동안(워드 전체 0)에만 유일 출처이고, 판 값 뒤에는 `0xF0C` 와 다르면 장치 모순(`DeviceFault`, 2026-10-18 사용자 결정 U2)이다. stepMax = 0 은 「step 기능 없음」(적재 생략)이다(참조 RTL 은 stepMax ≥ 2 — RTL 사실). RTL 파도 W6(`efafb14`)가 `0xF0C` 를 배선했고 판 0 동안 워드 전체 0 이다 (`e-defined.typ:463` FP_SHAPE 행; Verus `fp_shape_verdict`, `hiu_abi.md` §1.4) | 보드 기술 · 펌웨어 표 (교차 확인) |
| HIU 플랫폼 step 이미지 · 실행 의미 식별자 (2026-10-18) | 플랫폼이 이 구현에 맞춘 step 이미지 목록과, 각 이미지를 고르는 선택 열쇠(`[15:8]` 연산 종류 ‖ `[7:0]` 정밀도)와, 그 구현의 실행 의미 식별자를 알린다 — v1.0 에서 Y4 는 **플랫폼이 지정한 이미지만** 싣고 테넌트는 선택 열쇠로 고르기만 한다(2026-10-18 사용자 결정 U1). 정본은 「부호화 번호는 배치만 이름하고 실행 의미를 식별하지 않는다」고, 「v1.0 에는 실행 의미를 식별하는 장치 워드가 없다」고 적는다 (`03-hiu.typ:1692`) — 그래서 4항 「드라이버는 플랫폼(과 도구 사슬)이 그 구현에 맞춘 것으로 지정한 이미지만 결속한다」(`e-defined.typ:208`)를 Y4 가 지려면 맞춤의 근거가 플랫폼에서 와야 한다. 실행 의미가 어긋난 이미지는 장치 그물(`[15]`) 없이 조용히 틀린 결과를 낸다 — 「걷기 순서가 동결돼 있으므로 (4) 를 내지 않는다」(`:466`) — 그러니 이 목록의 맞춤이 유일한 방어다. 목록의 이미지는 한 걷기 안에서 접수 규약 래치(0 = a · 1 = b · 2 = c) 밖의 스크래치 래치를 먼저 쓰기 전에 읽지 않는다 — 걷기 사이 래치 잔류(RTL 사실 — 규범 아님, 정본의 지위는 C9)를 U1 이 닫는 조건이고 이미지 목록의 검증 항목이다 (`hiu_abi.md` §1.1 · §2.7 「step 적재 규칙」 1) | 플랫폼 · 도구 사슬 서명 목록 |
| HIU OWN_SET 재시도 상한 `K_OWN` · 소유권 표 형상 힌트 (2026-10-18) | 플랫폼이 OWN_SET 거절(정본 E.7 6항, `e-defined.typ:1387`) 뒤 다른 프레임으로 다시 이양할 상한 `K_OWN` 을 알린다 — Y4 는 되읽기로 거절을 확인할 때마다 다른 프레임으로 `K_OWN` 회까지 다시 이양하고, 그래도 못 채우면 테넌트에 거친 「풀 확장 실패」만 보고한다(프레임 · 시점 비공개 — 2026-10-18 사용자 결정 U4, `y4_abi.md` §3). 소유권 표 형상(용량 · 연관도 · 색인 함수)은 판의 일이 아니므로(`03-hiu.typ:1689`) 플랫폼이 비ABI 힌트로 줄 수는 있어도 Y4 할당기에 박지 않는다 — 정합성은 되읽기만 진다 (`hiu_abi.md` §2.7) | 보드 기술 · 펌웨어 표 |

이 표는 **living registry** — 새 하드웨어 메커니즘이 등장하면 (추상 계약,
realization) 쌍으로 항목을 추가한다.  HIU 행의 정본 인용(`e-defined.typ` · `03-hiu.typ`)은
WaveTensor 동결 sha `b16fc07`(정본 트리 `2dd78f80b9d44354416716d75bd235b088ac8a01`) 기준이다(동결 뒤 사실 커밋은 「@ `395e5b1`」).

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
