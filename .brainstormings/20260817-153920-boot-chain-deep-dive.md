<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Boot chain deep-dive — Limine→seL4→Y4 실제 chain + measured/secure boot + DRTM (★ DRTM 이 다중 bootloader 다양성과 attestation 단일성을 양립시킴)
created: 2026-08-17T15:39:20+09:00   # KST (UTC+9)
status: brainstorming (attestation §10 / key-management §11 후보에서 진입).  결정 다수 + 미결 일부.  rev: §3.1 measured 소비 = 부트로더·펌웨어 무관 MeasurementLogSource(a') 결정 + §8 log-parser Verus 대상(verus-fork 안정화 이후)
scope: Y4 부팅 chain 최종 목표.  현 baseline(Limine v12.1.0→seL4 15.0.0→y4-roottask, qemu-smoke PASS) 위
       measured/secure boot / DRTM vs SRTM / 다중 bootloader tier / per-ISA boot RoT /
       transactional update+rollback / verified-base bind
refs:
  - boot/README.md + boot/limine.conf (실제 chain — Limine multiboot1 → seL4 → roottask module)
  - CLAUDE.md §4 reuse manifest(bootloader 우선순위 Limine/GRUB2-BLS/U-Boot/coreboot) / §8
  - docs/architecture.md §Bootloader(우선순위 표 + 근거)
  - .brainstormings/20260813-132525-attestation-measured-boot.md §1(secure+measured)·§2(DRTM 선호)·§3(측정 체인)
  - docs/hw_mechanism_abstraction.md (per-ISA boot RoT realization 추상화 — §5 적용)
  - .brainstormings/20260813-205326-key-management.md §1(boot-verify pubkey / KEK sealing)
  - .brainstormings/20260618-231936-cross-platform-strategy.md (U-Boot=ARM, per-ISA)
---

# Boot chain deep-dive — DRTM 이 bootloader 다양성과 attestation 단일성을 양립시킨다

## §0 프레임

**최종 구현 목표 기준.**  boot chain 은 attestation(§2~3 측정 체인)·key-
management(§1 boot-verify pubkey / KEK sealing)가 위임한 지점.  현재는 실제로
**Limine v12.1.0 → seL4 15.0.0 → y4-roottask → "Hello, Y4"**(qemu-smoke PASS)
까지 구현돼 있으나, **secure boot 키 enrollment 는 Phase E 인증 트랙**(미구현),
measured boot 도 미구현.  본 발제가 그 최종 형태를 탐색.

## §0.5 원칙 정합

- **atomic-free**(§0.5): boot 은 sequential(단일 CPU bring-up → SMP), hot path
  아님; measurement extend 는 append-only single writer — trivially 정합.
- **TCB 최소화**(원칙 1): ★ **측정 TCB 최소화** — DRTM 으로 firmware/bootloader
  를 측정 대상에서 제외(§3).
- **capability**: root task = capability 분배 시작점(seL4 boot 이 넘겨줌).
- **formal-first**(원칙 6): seL4 **verified boot** 상속; boot chain 대부분은
  measured/secure(증명 아님, §8 경계).

## §1 실제 chain (baseline, 구현됨)

```
firmware(UEFI/BIOS)
  → Limine v12.1.0            (1st bootloader, BSD-2, chain-loaded never linked)
      protocol = multiboot1   (seL4 x86_64 = Multiboot1 image, header 0x1BADB002)
  → seL4 15.0.0               (kernel.elf)
      module = y4-roottask.elf (seL4 가 first multiboot module 로 pickup)
  → y4-roottask               (entry 0x401000 → "Hello, Y4" 시리얼 출력)
```
- Limine 은 **native Limine protocol 아닌 multiboot1** 사용(seL4 가 MB1 image).
- root task ELF 는 **multiboot module**(`module_path` in limine.conf) → seL4
  `boot_module` 로 인식.
- cmake 호출 = logicutils-only(`sel4.rules`/`limine.rules`, lu-rule/lu-par).
- 현 milestone: qemu-smoke PASS(Phase B step 5).

## §2 다중 bootloader tier — form-factor 별 (CLAUDE §4/§8)

| tier | bootloader | license | 적용 form-factor |
|---|---|---|---|
| 1st | **Limine** | BSD-2 | x86 server/laptop(현 baseline) |
| 2nd | GRUB2-BLS | GPLv3 | 광범위 호환성 fallback |
| 3rd | U-Boot | GPLv2+ | ARM/RISC-V SoC·handheld(§5) |
| 4th | coreboot | GPLv2 | open-firmware 플랫폼 |

- **chain-loaded only, never linked** — GPL bootloader 도 링크 X(license 격리,
  licensing 정합).
- **배제**: systemd-boot(systemd-tied) / rEFInd(transactional-update hook 부재,
  §6).

## §3 (결정) measured boot: SRTM vs DRTM — ★ 다중 bootloader 와의 시너지

- **SRTM**: firmware→Limine→seL4→Y4→capsules 각 stage measure(PCR extend).
  → **각 bootloader tier 가 측정 지원해야** → 4-tier 가 측정 파편화(GRUB2/U-Boot/
  coreboot 측정 지원 상이).
- **DRTM**(attestation §2 선호): SKINIT/SENTER late-launch → 측정 root reset →
  **seL4+Y4 만 측정**(firmware/Limine 제외).

★ **헤드라인 — DRTM 이 bootloader 선택과 attestation 을 decouple**:
> DRTM 은 attested root 를 **어느 bootloader 로 왔든 동일한 최소 launch
> (seL4+Y4)** 로 만든다.  ⟹ **§2 의 다중 bootloader 다양성(form-factor 별)을
> 유지하면서도 attestation 을 파편화하지 않는다.**  bootloader 는 "DRTM launch
> 지점까지 데려다주는" 역할로 격하 — 신뢰·측정 부담이 bootloader 에서 사라짐.

즉 **다양성(§2)과 단일성(attestation)이 양립** — Y4 의 form-factor 별 bootloader
정책이 attestation 스토리를 조각내지 않는다.  SRTM 은 DRTM 미가용 플랫폼의
fallback(attestation §2, 무엇이 쓰였는지는 attestation 이 보고).

## §3.1 (결정) measured 소비 — 부트로더·펌웨어 무관 `MeasurementLogSource` (a')

**측정은 이미 표준이 한다.**  Limine v12.1.0 은 표준 준수 SRTM measured boot
를 내장 — `measured_boot`(Secure Boot 시 강제 on), **PCR 8**(cmdline/kernel
경로/module 경로) + **PCR 9**(limine.conf/kernel 이미지/module 이미지),
`EV_IPL` 이벤트, `EFI_TCG2_PROTOCOL`(+ TDX/SEV-SNP 는 `EFI_CC_MEASUREMENT_
PROTOCOL`, PCR→MR 자동 매핑), **UAPI Linux TPM PCR Registry(GRUB 관례) 준수**,
그리고 **digest 재현 절차 문서화**(→ RIM 직결, reproducible-build 발제).
⟹ **Y4 shim 불필요, Limine fork 불필요**(원칙 5 reuse + submodule-tracks-
upstream D3).

**Y4 는 그 결과를 표준으로 소비 (a')**: "TCG2 event log + PCR/MR 획득"을
**계약**으로 고정하고 로그 **소스는 추상화**(`MeasurementLogSource`):

| backend | 대상 |
|---|---|
| UEFI config table(`LINUX_EFI_TPM_EVENT_LOG` + `EFI_TPM2_FINAL_EVENTS_TABLE`) | UEFI x86 |
| ACPI `TPM2`(LAML/LASA log area) | UEFI/legacy-BIOS x86 (post-boot 지속) |
| coreboot cbmem TCPA log | coreboot 플랫폼 |
| **device-tree SML**(`linux,sml-base`/`linux,sml-size`) | **ARM/RISC-V/embedded (비-UEFI·비-ACPI)** — §5 per-ISA RoT 와 pairing |

- **왜 (a')**: bootloader-무관(Limine/GRUB2/U-Boot/coreboot 모두 같은 PCR
  extend + 같은 표준 log append, UAPI/TCG 규약 공통) **+ firmware-무관**(UEFI/
  BIOS/coreboot/DT).  Limine 외 부트로더를 쓰는 커스텀 배포판(§2) × 다중 ISA 를
  **동일 소비 코드**로 커버.  (b) Limine-특화 핸드오프는 부트로더마다 파편화
  → 기각.
- **hw_mechanism_abstraction 인스턴스**: 계약(TCG log+PCR/MR)만 고정, discovery
  realization 은 per-platform + 로그가 provenance 보고.  `docs/hw_mechanism_
  abstraction.md` registry 에 등재.
- **비용 구조**: TCG log 파싱 + PCR replay 는 **공통 1벌**(포맷 동일), 소스별
  차이는 얇은 discovery 백엔드뿐 — "파서 1 + discovery N".
- **MB1 nuance 해소**: PCR extend 는 boot protocol 무관(로드 artifact 측정);
  Y4 는 protocol-특화 핸드오프가 아니라 **표준 소스**에서 log 를 read →
  현 multiboot1 경로도 문제 없음.

## §4 (결정) secure boot — 서명 체인 (fail-closed)

- UEFI Secure Boot → **Limine(signed)** → Limine이 seL4 검증 → seL4/Y4 가
  capsule 검증.  **boot-verify pubkey**(key-mgmt §1) 사용.
- 불량 서명 = **halt**(로컬 fail-closed, attestation §1).
- DRTM(§3)과 상보: secure boot = **즉시 차단**, measured = **원격 증명**.  둘 다.
- 현 상태 Phase E 트랙(boot/README non-goal) — 최종 목표로 설계만.

## §5 (결정) per-ISA boot RoT — hw_mechanism_abstraction 적용 ★

- DRTM(SKINIT/SENTER)은 **x86 특화**.  ARM = **TF-A(Trusted Firmware-A) + ARM
  DRTM(DEN 0113) / TrustZone**; RISC-V = 자체 measured-boot.
- ⟹ **measured-boot 개시 메커니즘은 per-ISA realization** → **`docs/hw_
  mechanism_abstraction.md` 정책 적용**(attestation §8.5 registry 에 이미 등재).
  §2 의 **U-Boot(3rd, ARM)는 TF-A/ARM-DRTM 과 pairing**.
- Y4 계약: "boot RoT 가 seL4+Y4 launch 를 측정·보고"; realization(x86 SKINIT /
  ARM DEN0113 / RISC-V)은 per-ISA + attestation 이 provenance 보고.

## §6 (결정) transactional update + rollback

- rEFInd 배제 이유 = **transactional-update hook 부재**(CLAUDE §4).  Y4 는 **A/B
  slot / transactional update + 실패 시 rollback** 지향.
- **measured boot 상호작용**: update 후 측정값 변화 → attestation 이 새 버전
  반영(RIM 갱신, key-mgmt §11 / reproducible build 발제) → tenant 가 새 TCB
  재검증.  boot 실패 시 **이전 slot 으로 rollback**.
- systemd-boot 배제(systemd-tied) → **Limine + Y4 자체 transactional 스킴**.

## §7 verified-base bind — attestation §3 의 실제 지점

- seL4 **verified property = kernel post-boot**; boot code 는 seL4 일부(verified
  boot).  측정이 **seL4 + root task** 를 포착.
- **verified-base-bind**(attestation §3): 측정 seL4 해시 == tenant 가 신뢰하는
  proof 의 seL4 버전(15.0.0 핀).  ⟹ boot chain 이 "형식 증명된 microkernel 이
  부팅됨"을 증명하는 **물리적 지점** — measured boot × formal verification 이
  실제로 만나는 곳.

## §8 verification (경계)

- boot chain 대부분 = **trusted/measured**, Verus-proven 아님(firmware/bootloader
  C 코드; seL4 **verified boot 은 inherited**).
- Y4 기여 여지: **root task 초기 setup**(capability 분배 이전)의 일부 invariant
  는 Verus 가능(예: 초기 capability 분배가 C1/C2/C3 well-formed 상태로 시작).
- ★ **log-parser · PCR-replay = Verus 대상**(§3.1): `MeasurementLogSource` 의
  공통 파서(TCG `EV_IPL` 파싱)와 PCR replay(측정 chain 재계산 → 기대값/RIM
  대조)는 deterministic → **공통 계약에 대해 1벌** Verus(hw_mechanism_abstraction
  §4 정합; discovery 백엔드는 trusted boundary).  boot 에서 새 Verus proof 가
  생기는 **유일 지점**.  **단 verus-fork 가 최근 Verus 업스트림 breaking change
  대응 중이라 실제 증명 착수는 그 안정화 이후** — root-task invariant(§9)와는
  별개 항목이나 동일 verus-fork 타이밍에 걸린다.
- 검증 스토리 = **measured + secure boot + verified seL4(inherited)** — 새 Verus
  proof 는 위 §3.1 log-parser 한 곳으로 **최소**.  boot 은 증명보다 **측정·
  서명**으로 신뢰를 세운다(경계 명시).

## §9 결정 / 미결 요약

**결정 방향(강)**:
- baseline = 실제 Limine multiboot1 → seL4 → roottask(§1, 구현됨)
- **DRTM 선호 + ★ 다중 bootloader 와 시너지**(attested root 를 bootloader 선택과
  decouple → 다양성·단일성 양립, §3)
- secure boot 서명 체인 fail-closed + measured 와 상보(§4)
- **per-ISA boot RoT = hw_mechanism_abstraction 적용**(x86 SKINIT / ARM DEN0113 /
  RISC-V; U-Boot↔TF-A pairing, §5)
- **transactional A/B + rollback**(rEFInd 배제 이유의 실현, measured 와 연동, §6)
- boot chain = verified-base-bind 의 물리적 지점(§7)
- **measured 소비 = 부트로더·펌웨어 무관 `MeasurementLogSource`(a', §3.1)** —
  Limine 내장 표준 측정 사용(shim·fork X), Y4 는 표준 TCG log 를 UEFI/ACPI/
  coreboot/DT 백엔드로 소비; hw_mechanism_abstraction registry 등재

**미결(설계 필요)**:
- transactional update 스킴 구체(A/B slot 배치 / 측정값 관리 / rollback trigger)
- U-Boot + TF-A/ARM-DRTM pairing 구체(⏳ ARM form-factor)
- root task 초기 setup 의 Verus invariant 범위(§8) — **verus-fork 측 작업
  지연으로 보류(미결 유지, 별도 진행)**

**⏳ 선행 의존**:
- attestation 구현 — TPM 연동·sealing(§3~4, Phase E 인증 트랙)
- ARM/RISC-V form-factor(§5, D2 상 x86 이후)
- RIM / reproducible build(§6 update 측정값 — 다음 발제 후보)

## §10 다음 발제 후보

- **reproducible build / supply-chain** — RIM 공개 전제(attestation §9 유일 잔여
  미결) + §6 transactional update 의 known-good 측정값 출처 + KEK sealing 대조값.
  (지금까지 여러 발제가 수렴하는 마지막 축.)
