<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Boot chain deep-dive — Limine→seL4→Y4 실제 chain + measured/secure boot + DRTM (★ DRTM 이 다중 bootloader 다양성과 attestation 단일성을 양립시킴)
created: 2026-08-17T15:39:20+09:00   # KST (UTC+9)
status: brainstorming (attestation §10 / key-management §11 후보에서 진입).  결정 다수 + 미결 일부.  rev: §3.1 measured 소비 = 부트로더·펌웨어 무관 MeasurementLogSource(a') 결정 + §8 log-parser Verus 대상(verus-fork 안정화 이후) + §6 transactional=per-chunk A/B(chunk-K seL4+roottask / chunk-P capsule+config)+Y4 mini-selector 결정
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

## §6 (결정) transactional update + rollback — per-chunk A/B + Y4 mini-selector

rEFInd 배제 이유 = transactional-update hook 부재(CLAUDE §4); systemd-boot 배제
= systemd-tied.  Y4 는 **A/B dual-slot + 실패 시 rollback** 을 자체 스킴으로 —
이는 외부 하드웨어 메커니즘이 아니라 **Y4-owned 산출물**(RIM/reproducible-build
과 같은 부류; hw_mechanism_abstraction §3 경계의 Y4 소유 쪽 → **registry 대상
아님**).

### §6.1 boot-state = GPT 파티션 속성 + Y4 mini-selector (bootloader-무관)
- **boot-state** = GPT 파티션 속성(`priority`/`tries`/`successful`, ChromeOS
  cgpt 방식) — on-disk, **bootloader·firmware 무관**(a' 와 동일 이식성 논리).
- **selector = 단일 Y4 mini-selector** — 어느 부트로더(Limine/GRUB2/U-Boot/
  coreboot)든 이 mini-selector 하나를 payload 로 로드; 이후 GPT 속성 read →
  slot 선택 → chain.  ⟹ **per-bootloader adapter 0**(부트로더는 "mini-selector
  까지 데려다주는" loader 로 격하, §3 테마).
- mini-selector 무결성 = **secure boot 서명(§4) + anti-rollback floor(§6.4)** 로
  담보(DRTM 시 measured TCB 밖 pre-launch 이나 이 둘로 커버; SRTM 시 Limine 이
  측정).

### §6.2 ★ per-chunk A/B — 변경빈도 층화
Y4 이미지를 단일 slot 이 아니라 **덩어리(chunk)별 독립 A/B**:
- **chunk-K = seL4 kernel + y4-roottask** (verified core, 드물게 변경) — {1A,1B}
- **chunk-P = capsule set + config** (driver/policy 층, 자주 변경) — {2A,2B}
- mini-selector 가 **chunk 별로** slot 선택 → boot set = 선택된 chunk-K ⊕ 선택된
  chunk-P.
- **rationale**: Y4 의 신뢰·변경빈도 층화와 정합 — verified seL4 core 는 드물게
  변경(변경 시 verified-base-bind §7 재트리거), capsule/policy 는 자주.  각 층이
  자기 변경빈도에 맞는 update train 을 가짐(작은 update 단위, 독립 rollback).
- ★ **cross-chunk 호환 계약**: chunk-P 가 요구하는 **kernel-ABI 범위**를 선언;
  mini-selector 는 **호환되는 조합만 선택**.  rollback 도 조합 호환을 보존 —
  각 chunk 의 "good(successful)" slot 들은 **항상 호환 baseline** 을 이룬다.

### §6.3 update flow (power-fail-safe, per-chunk)
1. update client 가 대상 chunk 의 **inactive slot** 에 신규 이미지 write(active
   무손상 → 중단돼도 inactive 만 오염)
2. 검증(그 slot 의 RIM = 서명/hash)
3. **원자적 commit-to-try**: 해당 chunk inactive 의 priority↑ + tries=N (GPT 단일
   write)
4. 재부팅 → mini-selector 가 그 chunk 의 신규 slot 선택(**호환 확인**) + tries 감소
5. 정상 도달 시 **mark-good(successful=1)** → 영구 commit; 실패 시 tries 소진 →
   **그 chunk 만 auto-rollback**(타 chunk 무영향; 이전 slot 손대지 않아 항상 good)

### §6.4 rollback trigger + ★ anti-rollback
- **트리거**: (a) commit 전 tries 소진 — panic/hang 은 **watchdog** 재부팅으로
  tries 감소 → 반복 실패 시 rollback; (b) **health-check 실패**(capsule up /
  accelerator 도달 / attestation self-check) → mark-good 생략 + 재부팅.
- ★ **anti-rollback(security)**: safety-rollback(이전 good 복귀)은 허용하되
  악의적 downgrade(구 취약 버전 강제)는 차단 — **chunk 별 TPM NV monotonic
  counter 의 min-version floor**; floor 미만은 unseal/attestation 실패.

### §6.5 measured-boot / attestation 루프 (§3.1 연동)
- **chunk 별 RIM**; mini-selector 가 **선택한 chunk slot-id 들을 측정**(PCR
  extend) → attested TCB = **실제 부팅된 조합(chunk-K ⊕ chunk-P)** 과 일치,
  trial/committed 상태도 attestable.
- update 후 측정값 변화 → tenant 가 신규 조합 RIM 으로 재검증; rollback 시 측정값
  이전 조합으로 → tenant 가 rollback 인지(key-mgmt §11 / reproducible-build 발제).

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
- **transactional update = per-chunk A/B + Y4 mini-selector**(§6): boot-state=GPT
  속성(bootloader/firmware 무관), chunk-K(seL4+roottask)/chunk-P(capsule+config)
  독립 A/B(변경빈도 층화)+cross-chunk 호환 계약; power-fail-safe flow; rollback=
  tries 소진/health-fail; anti-rollback=chunk 별 TPM NV min-version floor;
  Y4-owned(registry 대상 아님)

**미결(설계 필요)**:
- **data/영속 파티션** 처리(별도 non-A/B 파티션 + 스키마 변경 시 migration) — §6 열린 항목
- cross-chunk 호환 descriptor 포맷 + mini-selector 호환 선택 알고리즘 세부(§6.2)
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
