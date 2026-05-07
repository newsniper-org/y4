<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Power Management 안전장치 사양

> **상태:** **v1.0 frozen** (2026-05-07, Phase 4-power 일괄 마킹).
> ARCH-II' v1.0 frozen (2026-05-05) 후 첫 신규 spec.  9 안전장치
> (S15~S23) + sub-decision + AV21~AV40 invariant catalog + form-factor
> + sub-mode + universal customizability + PR-5 분리 모두 sign-off
> (§6.3 sub-decision ledger).  짝 doc = `docs/power_arch.md` v1.0
> frozen.  ARCH-II' 측 4 doc (vmm_arch / amdv_safety / sel4_fork_policy
> / verus_to_isabelle) 와는 별도 v1.0 cycle (단 cross-cluster API 의존
> §6.2).  cpu_virt_compat 측 vendor-neutrality 정책과 정합.
>
> 이전 record: v0 spec 진입 (2026-05-05) — amdv_safety.md S1~S14 와
> 같은 catalog 형식으로 power domain 의 보안 측면 (Hertzbleed / DVFS
> side-channel / energy counter leak / firmware mailbox attack 등) 을
> 안전장치 S15+ 로 박는 spec.

본 doc 은 Y4 의 5 form factor (server-farm / 랩톱 / rack-mount /
핸드헬드+독 / 임베디드 SoC) 호스트 OS 측면에서 power management 의
**보안 안전장치 catalog**.  형상별 power profile 차이 + 사용자
override 정책은 §2 에서 다룬다.

---

## 1. 위협 모델

### 1.1 Threat actor 분류 (E)

| Tier | Actor | 능력 |
|---|---|---|
| 1 | **Malicious guest** | 별도 lease 보유, own VM 안 임의 코드 실행 권한.  ACPI eval / MSR access / VMMCALL hypercall 시도 가능, 단 host hardware 직접 access X |
| 2 | **Malicious lease holder** | host-wide 영향 시도, lease 권한 abuse — lease 내부에서 power policy 변경 시도, cross-tenant 데이터 추출 시도 |
| 3 | **Compromised capsule** | cluster 안 capsule 의 trust 침해 — capsule binary 변조 또는 supply chain attack 으로 cluster-scope-trusted 가정 깨짐.  Verus invariant 가 inductive 차단 (vmm_arch.md §2.3) |
| 4 | **External network attacker** | NIC magic packet / wake spoofing / Wake-on-LAN abuse, lease 미보유 외부.  DoS + battery drain |

### 1.2 위협 catalog (12 항목, 3 카테고리)

#### A. Side-channel (cross-tenant 정보 추출)

| 위협 | 메커니즘 | 영향 | Layer | Mitigation |
|---|---|---|:---:|---|
| **Hertzbleed** | DVFS frequency 의존성으로 secret-dependent timing 노출 | cross-tenant key extraction (실제 SIKE 공격 reproducible) | Lower | S15.5 dwell time + S15.6 constant-freq + S15.7 SMT pair 동기 |
| **DVFS / P-state side-channel** | Frequency change 가 wall-clock observable → secret-dependent computation 의 frequency 측정 | crypto key / secret data leak | Lower | S15.1 권한 + S15.2 MSR observable 차단 + S15.3 CPUID frequency 마스킹 |
| **C-state residency side-channel** | Deep C-state 진입 timing 이 cache state 노출 | cache footprint inference | Lower | S16.4 residency MSR 차단 + S16.5 L1D flush + S16.6 deterministic timing |
| **RAPL / energy counter leak (PLATYPUS)** | `MSR 0x611 / 0x619 / 0x639` energy counter read 가 cross-tenant power profile 노출 | PowerHammer / PLATYPUS 공격 | Upper | S17.1 RAPL MSR 차단 + S17.2 virtio-rapl + S17.4 noise injection |
| **Thermal observable fingerprinting** | thermal sensor read 가 cross-tenant workload fingerprinting | workload 식별 / VM placement inference | Lower | S21.2 thermal MSR 마스킹 + S21.3 virtio-thermal |
| **Wake event fingerprinting** | wake source 패턴이 다른 lease 의 사용 패턴 노출 | guest behavior fingerprinting | Upper | S22.3 lease binding + S22.4 spurious detection |
| **SMT cross-thread side-channel** | S5 SMT-aware grouping 무력화 — SMT pair 의 한쪽만 deep idle, 다른 쪽 host | side-channel 차단 깨짐 | Upper | S19.2 strict 동기 + S19.3 allow-mixed audit + S19.7 constant-freq + S19.8 lease assignment |

#### B. Direct attack (host integrity / hardware 변조)

| 위협 | 메커니즘 | 영향 | Layer | Mitigation |
|---|---|---|:---:|---|
| **Plundervolt voltage attack** | mailbox / VR MSR (`0x150`) 로 voltage under/over 변조 → SGX/TZ enclave fault injection | cryptographic key extraction, hardware 손상 | Upper | S23.4 voltage range ±50 mV + S23.9 MSR `0x150` 차단 + S14 pending queue |
| **PSP / PCH power mailbox abuse** | guest 가 firmware mailbox 로 power patch 적용 | host CPU clock skew, microcode unauthorized update | Upper | S23.2 S14 pending queue + S23.5 SMN/MMIO 격리 + S23.10 force-toggle |
| **ACPI _PSx / _CST / _PSV 우회** | guest 의 ACPI eval 이 host 의 _PSV / _PSx 의도와 다른 power state 강제 | host thermal throttle 우회 | Upper | S18.1 mediation + S18.2 화이트리스트 + S18.5 host operator only thermal threshold |
| **Wake-from-suspend replay attack** | suspend 시점 sealed blob capture → 후속 wake 시점 stale blob substitute → lease state rollback | cross-tenant secret rollback, integrity violation | Upper | S20.4.2 epoch counter + S20.4.1 AEAD AD binding |
| **Wake source spoofing (Magic Packet)** | 표준 Magic Packet capture + replay 또는 spoof | unauthorized wake, DoS, lease wake 강제 | Upper | S22.6 Y4-defined cryptographic signed packet + nonce replay 차단 |

#### C. DoS / Resource exhaustion

| 위협 | 메커니즘 | 영향 | Layer | Mitigation |
|---|---|---|:---:|---|
| **Thermal throttle / hardware damage** | guest 가 _TCC / _TJMAX MSR write 로 host thermal 한도 변경 | hardware damage / DoS | Upper | S21.1 thermal MSR 차단 + S21.4 _TCC host-only + S21.8 hardlimit emergency |
| **Lease suspend race / atomicity violation** | deep idle / hibernate 진입 시 lease state 의 atomicity 부재 → 재개 시 stale state 노출 | cross-tenant data leak | Lower | S20.3 atomicity (S13.2 패턴) + S20.4 integrity check |
| **Battery drain (handheld / laptop)** | spurious wake spam, deep idle 진입 차단 | battery 빠른 소진 | DoS | S22.4 spurious detection + 즉시 deep idle 재진입 + S22.10 InhibitAll force |
| **Energy budget DoS (server-farm multi-tenant)** | 악의 / 손상 guest 의 power-hammer (지속 high-energy workload) | host energy budget 고갈, multi-tenant 다른 lease 영향 | Upper | S17.8 per-VM energy budget cap + EnergyBudgetExceeded vmrun 거부 |

### 1.3 Form-factor 별 위협 가중치 (F)

| Form factor | 우선 위협 카테고리 | 근거 |
|---|---|---|
| **server-farm** | A (side-channel) > C (DoS) > B (direct) | multi-tenant 격리 우선, latency tail 영향 |
| **rack-mount** | A > C > B | 동일, 단 thermal envelope 우선 |
| **laptop** | C (battery drain) > A (side-channel) > B | 단일 사용자 + battery 우선 |
| **handheld + 독** | C (battery + cold-boot) > B (mode switch attack) > A | physical access risk + battery |
| **임베디드 SoC** | B (supply chain + direct hardware attack) > C (always-on resource) > A | physical access + long-life device |
| **certified profile** | B (direct + integrity) > A (compliance) > C | 의료/항공/금융 의무 mitigation |

가중치 → form-factor profile 의 default 정책 결정 (S15-S23 의 form-
factor 별 build-time const 가 본 가중치를 reflect).

### 1.4 v1.x patch 후속 위협 ledger

새 CVE / 학술 논문 발견 시 처리 path:

1. 위협 식별 → §1.2 catalog 의 어느 카테고리 (A/B/C) 인지 분류
2. mitigation 가능성 분석:
   - 기존 S15-S23 의 sub-decision 갱신으로 차단 가능 → v1.x patch
     (sub-decision 추가 또는 const 조정)
   - 기존 메커니즘으로 부족 → **신규 S24+** 추가 (v1.x patch — 본 doc
     §7.4 의 v1.x 분류 정합)
3. audit op_tag 추가 시 S12.2 schema 의 v1.x patch 로 추가 (op_tag enum
   확장)
4. Verus invariant 추가 시 power_safety.md §4 의 AV21+ catalog 에 추가
5. 모든 v1.x 갱신은 본 doc §6 동결 정책 (또는 짝 doc 의 v1.0 frozen
   유지) 정합

기록 위치: `.claude-notes/trackers/power-threat-ledger.md` (Phase C 진입 후
신설 — 발견 시점, CVE / paper reference, mitigation 적용 결정 기록).

---

## 2. 형상별 default power profile + 사용자 override + logicutils rule

결정 4 = (D): 기본 형상별 default profile + 사용자 override + logicutils
`tools/power.rules` 통합.

### 2.1 4 default form-factor + mobile sub-modes + certified overlay (M1~M13)

**Universal customizability 원칙 (M3, M4):** Y4 가 ship 하는 form-factor /
sub-mode 도 "**built-in**" 이 아닌 "**default definitions**" — user 가
override / removal 가능, default 와 user-defined 의 mechanism 동일.

#### 2.1.1 4 default form-factor (Y4 ship)

| Form factor | 우선순위 | 특성 |
|---|---|---|
| **server-farm host** | latency consistency | P-state 변동 최소, deep C-state 사용 0 (latency tail), RAPL audit 활성 |
| **rack-mount node** | thermal envelope | _PSV/_TCC 강제 적용, P-state 활용 적극, C-state OK |
| **mobile** (M1: laptop + handheld 통합) | battery + responsiveness + dual/tri-mode | aggressive C-state, P-state on demand, runtime PM ON.  3 default sub-mode (M2 — `dock` / `portable` / `transportation`) 자동 전환 |
| **임베디드 SoC** | always-on 또는 duty-cycle | minimal power state, deep idle 적극, wake source 제한 |

#### 2.1.2 Mobile 의 3 default sub-modes (M2)

| Sub-mode | 의미 | 특성 |
|---|---|---|
| **dock** | 독 연결 모드 (dock detect signal 활성) | throughput 우선, SMT on, deep C-state 완화, suspend OFF |
| **portable** (default) | battery 모드 (독 미연결, vehicle 외) | battery 우선, SMT off, aggressive deep C-state, suspend ON |
| **transportation** (M6) | 자동차 / 철도 / 항공 등 교통수단 | sudden power loss 대비 (suspend ON 적극, atomicity 강화), 캐빈 thermal 보수적 (~80°C 대비), wake source 적극 (vehicle bus / GPS / cellular), TPM fTPM 우선 (M9 — vibration 으로 dTPM 신뢰성 ↓), multi-tenant 중요도 ↓ |

mode signal (S19.6.3 dock-detect, S21.7 thermal mode signal, S22 wake
source signal, S22 vehicle bus / GPS detection 신규) 발화 시 자동 전환
— S19.6.3 의 L2 signal hub 가 sub-block 사이 토글.

#### 2.1.3 Default definitions — `tools/power.rules.d/` overlay merge (M3, M4)

logicutils `lu-rule` 의 numbered file overlay merge 패턴 (`boot/x86_64-
debug.rules` 정합):

```
tools/power.rules.d/
├── 00-default-server-farm.rules           # Y4 ship
├── 00-default-rack-mount.rules
├── 00-default-mobile.rules
├── 00-default-soc.rules
├── 00-default-mobile-dock.rules           # mobile 의 sub-mode default
├── 00-default-mobile-portable.rules
├── 00-default-mobile-transportation.rules
├── 00-default-certified.rules             # certified overlay
├── 00-default-aliases.rules               # M8 deprecated alias (laptop/handheld → mobile)
├── 50-user-edge-gateway.rules             # 사용자 추가 form-factor (예시)
├── 50-user-mobile-bicycle.rules           # 사용자 추가 mobile sub-mode (예시)
├── 90-myorg-server-farm-override.rules    # default 재정의 (NN > 00 우선)
└── 99-myorg-mobile-portable-override.rules
```

**Override semantics:** 같은 form-factor / sub-mode 이름이 여러 file 에
정의되면 NN 큰 쪽이 우선 (replace).  `removed = true` flag 로 default
removal 가능 (M13 — 단 detection rule + Verus AV24 의 references 깨짐
build-time fail).

#### 2.1.4 Custom form-factor / sub-mode 정의 (M3, M4 통합)

Form-factor 와 sub-mode 모두 동일 mechanism:

- **Build-time:** `tools/power.rules.d/<NN>-<name>.rules` 의 새 block
  추가
- **Runtime:** 새 정의 추가 X (boot 시점 fix — capsule Verus invariant
  변동 차단), runtime 에 등록된 정의 set 안에서만 `mode-set` 가능
- **Naming convention (M12):** form-factor name + sub-mode name 모두
  ASCII alphanumeric + hyphen, ≤ 32 char.  sub-mode block 이름 =
  `[<form-factor>-<sub-mode>]`

#### 2.1.5 Default definition removal (M13)

```
# tools/power.rules.d/95-no-soc.rules
[soc]
removed = true
```

위 file 추가 시 `soc` form-factor 가 effectively 사라짐.

**Build-time check (boot 시점 lu-rule build):**
- detection rule (§2.5) 가 references 하는 form-factor 가 모두 정의되어
  있어야 함 (alias 통한 redirection 도 OK)
- Verus AV24 의 references 하는 *named* sub-mode (`transportation` 등)
  가 정의되어 있어야 invariant 적용 — removed 시 invariant 자동 비활성

removal 시 사용자가 detection / Verus references 도 갱신 책임 (`tools/
power.rules.d/<NN>-detection-update.rules` overlay).

#### 2.1.6 certified overlay flag — form-factor 와 분리 (D=b, G=i')

`certified` 는 form-factor 와 mutually-exclusive 가 아닌 **overlay
flag** — 어느 form-factor 위에든 활성 가능:

- `server-farm + certified` (의료 데이터 호스팅 클러스터)
- `rack-mount + certified` (금융 콜로케이션)
- `mobile + certified + portable` (의료 진단 노트북, transportation
  variant 도 가능)
- `SoC + certified` (의료 임베디드 디바이스)

**Overlay 의미:** baseline form-factor 의 default 위에 `certified` rule
이 mergeover — 항상 *conservative direction* 으로 강화 (보안 / integrity
우선).  특정 const 는 `certified` 활성 시 cmdline 변경 자체 차단.

설정 채널:
- cmdline `y4.pm.certified=on|off` (default `off`) — boot 시점 1 회 결정,
  runtime 변경 X
- `tools/power.rules.d/00-default-certified.rules` 의 `[certified]`
  overlay block 이 모든 form-factor 위에 mergeover (default + user 합쳐)

`certified` 활성 시 영향:
- S10.2 MSR profile = `certified` 강제
- S15.6 `Y4_PM_CONSTANT_FREQ = ON`
- S20.2.5 `Y4_PM_TPM_REQUIRED = on` (TPM 부재 시 boot fail)
- S21.10 thermal = `conservative` 강제 + `force_default = conservative`
  + cmdline 변경 X
- S23.4 voltage offset ±0 mV
- S23.7 mailbox `strict`
- S22.10 wake force `policy` (InhibitAll 우회 차단)

certified 자체도 default definition — user override 가능
(`90-myorg-certified.rules` 의 `[certified]` block 으로 재정의).

#### certified flag — form-factor 와 **분리된 overlay** (D=b, G=i')

`certified` 는 baseline form-factor 와 **mutually exclusive 가 아닌
overlay flag** — 어느 form-factor 위에든 활성 가능:

- `server-farm + certified` (의료 데이터 호스팅 클러스터)
- `rack-mount + certified` (금융 콜로케이션)
- `laptop + certified` (의료 진단 노트북)
- `handheld + certified` (항공 의료 휴대 단말)
- `SoC + certified` (의료 임베디드 디바이스)

**Overlay 의미:** baseline form-factor 의 default 위에 `certified` rule
이 mergeover — 항상 *conservative direction* 으로 강화 (보안 / integrity
우선).  특정 const 는 `certified` 활성 시 cmdline 변경 자체 차단 (예:
`Y4_PM_VOLTAGE_*_OFFSET_MV = 0` 강제, S23.4 정합).

설정 채널:
- cmdline `y4.pm.certified=on|off` (default `off`) — boot 시점 1 회 결정,
  runtime 변경 X
- `tools/power.rules` 의 `[certified]` overlay block 이 모든 baseline
  block 위에 mergeover

`certified` 활성 시 영향:
- S10.2 MSR profile = `certified` 강제
- S15.6 `Y4_PM_CONSTANT_FREQ = ON`
- S20.2.5 `Y4_PM_TPM_REQUIRED = on` (TPM 부재 시 boot fail)
- S21.10 thermal = `conservative` 강제 + `force_default = conservative`
  + cmdline 변경 X
- S23.4 voltage offset ±0 mV
- S23.7 mailbox `strict`
- S22.10 wake force `policy` (InhibitAll 우회 차단)

### 2.2 Cmdline key 통합 표 (A=a, S15-S23 cross-ref)

S4 의 3-계층 ceiling 패턴 정합 (build-time const → cmdline runtime
override → per-VM cap).  cmdline key 의 namespace = `y4.pm.*`:

| Cmdline key | Default | 의미 | Cross-ref |
|---|---|---|---|
| `y4.pm.profile` | (auto-detect) | form-factor 선택 (default 또는 user-defined name) | §2.5 |
| `y4.pm.sub_mode` | (sub-mode auto-detect) | mobile 등의 sub-mode 명시 (`dock` / `portable` / `transportation` / user-defined) | §2.4 |
| `y4.pm.certified` | `off` | certified overlay flag | §2.1.6 |
| `y4.pm.cpufreq_governor` | (form-factor) | P-state governor | S15.4 |
| `y4.pm.min_pstate_dwell_ns` | 10_000_000 | DVFS dwell time | S15.5 |
| `y4.pm.constant_freq` | (form-factor) | constant-frequency mode | S15.6 |
| `y4.pm.smt_force` | `policy` | SMT force-toggle (S19.6 L0) | S19.6.1 |
| `y4.pm.cstate_max` | (form-factor) | deep C-state max | S16.2 |
| `y4.pm.cstate_deterministic` | (form-factor) | C-state deterministic timing | S16.6 |
| `y4.pm.rapl_audit_enable` | `on` | RAPL audit ON/OFF | S17.7 |
| `y4.pm.rapl_default_budget_j` | u64::MAX | per-VM energy budget default | S17.8 |
| `y4.pm.rapl_noise_bits` | 4 | RAPL LSB noise bits | S17.4 |
| `y4.pm.acpi_eval_timeout_ns` | 100_000_000 | ACPI eval timeout | S18.7 |
| `y4.pm.acpi_osi_extra` | (empty) | _OSI 추가 화이트리스트 | S18.4 |
| `y4.pm.thermal_psv_c` | (form-factor) | passive trip point | S18.5 / S21.5 |
| `y4.pm.thermal_tcc_c` | (form-factor) | thermal control circuit | S21.4 |
| `y4.pm.thermal_hardlimit_c` | (form-factor) | software hardlimit | S21.8 |
| `y4.pm.thermal_hysteresis_c` | 5 | thermal state hysteresis | S21.6 |
| `y4.pm.thermal_noise_bits` | 3 | thermal sensor noise bits | S21.2 |
| `y4.pm.thermal_force` | `policy` | thermal force-toggle | S21.9 |
| `y4.pm.deep_idle_lease_suspend` | (form-factor) | S20 suspend ON/OFF | S20.7 |
| `y4.pm.suspend_force` | `policy` | suspend force-toggle | S20.9 |
| `y4.pm.suspend_latency_ns` | 10_000_000 | suspend latency budget | S20.5 |
| `y4.pm.wake_latency_ns` | 5_000_000 | wake latency budget | S20.5 |
| `y4.pm.tpm_required` | `auto` | dTPM 의무 / 권장 / off | S20.2.5 |
| `y4.pm.tpm_pcr_policy` | (default 0+1+2+3+7) | TPM PCR binding | S20.2.4 |
| `y4.pm.wake_force` | `policy` | wake force-toggle | S22.10 |
| `y4.pm.voltage_min_offset_mv` | -50 | Plundervolt min | S23.4 |
| `y4.pm.voltage_max_offset_mv` | +50 | Plundervolt max | S23.4 |
| `y4.pm.mailbox_force` | `policy` | mailbox force-toggle | S23.10 |

총 30 cmdline key.  미지정 시 form-factor + certified overlay 의
default 적용.  `certified=on` 시 일부 key 의 cmdline 변경 자체 차단
(§2.1 표 참조).

### 2.3 Logicutils `tools/power.rules.d/` (B=a, M3, 정확 변수 set)

per-form-factor + per-sub-mode build flag 묶음 + capsule 측 행동 결정 —
`boot/x86_64-debug.rules` 패턴 정합 + numbered file overlay merge.  Y4
ship 의 default file 들의 본문 (실제 build-time const 30+ 항목):

```
# tools/power.rules (logicutils lu-rule 형식)

[server-farm]
Y4_PM_CPUFREQ_GOVERNOR              = "performance"
Y4_PM_CONSTANT_FREQ                 = ON       # S15.6 — Hertzbleed 차단
Y4_PM_DEEP_C_STATE_MAX              = "C1"     # S16.2 — latency tail
Y4_PM_CSTATE_ENTRY_DETERMINISTIC    = OFF      # S16.6
Y4_PM_RAPL_AUDIT                    = ON       # S17.7
Y4_PM_THERMAL_TCC_C                 = 90
Y4_PM_THERMAL_PSV_C                 = 80
Y4_PM_THERMAL_HARDLIMIT_C           = 95       # S21.10
Y4_PM_THERMAL_NOISE_BITS            = 4        # S21.2 — 보수적
Y4_PM_THERMAL_FORCE_DEFAULT         = "conservative"
Y4_PM_LEASE_SUSPEND                 = OFF      # S20.7
Y4_PM_TPM_REQUIRED                  = "on"     # S20.2.5
Y4_PM_SMT_INITIAL_STATE             = "on"     # S19.6.2
Y4_PM_WAKE_SOURCES                  = "nic-magic-packet, ipmi-bmc, rtc-alarm"
Y4_PM_MAILBOX_POLICY                = "strict" # S23.7

[rack-mount]
# server-farm 과 동일 패턴, thermal 만 완화
Y4_PM_CPUFREQ_GOVERNOR              = "ondemand"
Y4_PM_CONSTANT_FREQ                 = OFF
Y4_PM_DEEP_C_STATE_MAX              = "C2"
Y4_PM_THERMAL_TCC_C                 = 95
Y4_PM_THERMAL_PSV_C                 = 85
Y4_PM_THERMAL_HARDLIMIT_C           = 100
Y4_PM_THERMAL_NOISE_BITS            = 3
Y4_PM_THERMAL_FORCE_DEFAULT         = "policy"
Y4_PM_LEASE_SUSPEND                 = OFF
Y4_PM_TPM_REQUIRED                  = "on"
Y4_PM_SMT_INITIAL_STATE             = "on"
Y4_PM_WAKE_SOURCES                  = "nic-magic-packet, ipmi-bmc, rtc-alarm"
Y4_PM_MAILBOX_POLICY                = "strict"
Y4_PM_RAPL_AUDIT                    = ON

[mobile]
# baseline mobile (sub-mode 미지정 시 portable 적용)
Y4_PM_DEFAULT_SUB_MODE              = "portable"
Y4_PM_RAPL_AUDIT                    = ON

[mobile-portable]    # 00-default-mobile-portable.rules
Y4_PM_CPUFREQ_GOVERNOR              = "powersave"
Y4_PM_CONSTANT_FREQ                 = OFF
Y4_PM_DEEP_C_STATE_MAX              = "C6"
Y4_PM_CSTATE_ENTRY_DETERMINISTIC    = ON       # side-channel 우선, battery 영향 미미
Y4_PM_THERMAL_TCC_C                 = 88       # laptop 90 + handheld 85 의 중간
Y4_PM_THERMAL_PSV_C                 = 73
Y4_PM_THERMAL_HARDLIMIT_C           = 93
Y4_PM_THERMAL_NOISE_BITS            = 3
Y4_PM_THERMAL_FORCE_DEFAULT         = "policy"
Y4_PM_LEASE_SUSPEND                 = ON
Y4_PM_TPM_REQUIRED                  = "auto"   # fTPM 또는 dTPM 대부분 보유
Y4_PM_SMT_INITIAL_STATE             = "off"    # battery 우선
Y4_PM_WAKE_SOURCES                  = "lid-open, power-button, volume-button,
                                        mode-switch, nic-magic-packet, usb-device,
                                        rtc-alarm, battery-threshold"
Y4_PM_MAILBOX_POLICY                = "mediated"

[mobile-dock]    # 00-default-mobile-dock.rules
Y4_PM_CPUFREQ_GOVERNOR              = "ondemand"
Y4_PM_CONSTANT_FREQ                 = OFF
Y4_PM_DEEP_C_STATE_MAX              = "C2"
Y4_PM_CSTATE_ENTRY_DETERMINISTIC    = OFF
Y4_PM_THERMAL_TCC_C                 = 95
Y4_PM_THERMAL_PSV_C                 = 85
Y4_PM_THERMAL_HARDLIMIT_C           = 100
Y4_PM_THERMAL_FORCE_DEFAULT         = "policy"
Y4_PM_LEASE_SUSPEND                 = OFF      # docked, throughput 우선
Y4_PM_SMT_INITIAL_STATE             = "on"     # docked, throughput 우선
Y4_PM_WAKE_SOURCES                  = "lid-open, power-button, dock-detect,
                                        nic-magic-packet, usb-device, rtc-alarm"
# (그 외 항목은 [mobile] 와 동일, mergeover)

[mobile-transportation]    # 00-default-mobile-transportation.rules (M6, M9)
Y4_PM_CPUFREQ_GOVERNOR              = "ondemand"
Y4_PM_CONSTANT_FREQ                 = OFF
Y4_PM_DEEP_C_STATE_MAX              = "C3"     # 운전자 attention 가속 우선, deep idle 보수적
Y4_PM_CSTATE_ENTRY_DETERMINISTIC    = ON
Y4_PM_THERMAL_TCC_C                 = 95       # 캐빈 80°C 대비 + 추가 마진
Y4_PM_THERMAL_PSV_C                 = 80
Y4_PM_THERMAL_HARDLIMIT_C           = 100
Y4_PM_THERMAL_NOISE_BITS            = 3
Y4_PM_THERMAL_FORCE_DEFAULT         = "conservative"
Y4_PM_LEASE_SUSPEND                 = ON       # sudden power loss 대비, 적극 suspend
Y4_PM_SUSPEND_LATENCY_NS            = 5_000_000   # default 10ms → 5ms (ignition off race)
Y4_PM_TPM_REQUIRED                  = "auto"
Y4_PM_TPM_PREFER_FTPM               = ON       # M9 — vibration 으로 dTPM 신뢰성 ↓
Y4_PM_SMT_INITIAL_STATE             = "off"
Y4_PM_WAKE_SOURCES                  = "vehicle-bus, gps-movement, cellular,
                                        power-button, dock-detect, rtc-alarm,
                                        battery-threshold, mode-switch"
Y4_PM_MAILBOX_POLICY                = "mediated"
Y4_PM_MULTI_TENANT_PRIORITY         = LOW      # 단일 사용자 (driver / 승객) 가정

[soc]
Y4_PM_CPUFREQ_GOVERNOR              = "platform-driver"
Y4_PM_DEEP_C_STATE_MAX              = "C6"
Y4_PM_THERMAL_TCC_C                 = "platform-dependent"
Y4_PM_THERMAL_PSV_C                 = "platform-dependent"
Y4_PM_THERMAL_HARDLIMIT_C           = "platform-dependent"
Y4_PM_THERMAL_NOISE_BITS            = 3
Y4_PM_RAPL_AUDIT                    = OFF       # RAPL hardware 부재 다수
Y4_PM_LEASE_SUSPEND                 = "platform-dependent"
Y4_PM_TPM_REQUIRED                  = "auto"
Y4_PM_SMT_INITIAL_STATE             = "platform-dependent"
Y4_PM_WAKE_SOURCES                  = "platform-dependent"
Y4_PM_MAILBOX_POLICY                = "platform-driver-only"
```

#### Certified overlay (D=b)

`certified=on` 시 baseline 위에 mergeover.  baseline 의 어느 form-factor
든 본 overlay 적용:

```
[certified]   # overlay — baseline 의 위에 mergeover
Y4_PM_MSR_PROFILE                   = "certified"      # S10.2
Y4_PM_CONSTANT_FREQ                 = ON               # S15.6 강제
Y4_PM_TPM_REQUIRED                  = "on"             # S20.2.5
Y4_PM_THERMAL_FORCE_DEFAULT         = "conservative"   # S21.9
Y4_PM_THERMAL_NOISE_BITS            = 4
Y4_PM_VOLTAGE_MIN_OFFSET_MV         = 0                # S23.4 ±0 mV
Y4_PM_VOLTAGE_MAX_OFFSET_MV         = 0
Y4_PM_MAILBOX_POLICY                = "strict"         # S23.7
Y4_PM_WAKE_FORCE_DEFAULT            = "policy"         # S22.10 InhibitAll 차단
# 위 const 의 cmdline 변경 차단 (build-time fix only)
```

build orchestration 측이 form-factor + certified 선택 시 자동 적용 +
mergeover.

### 2.4 Mode signal definition + sources (F=a, M5, M7, M10, 통합 정의)

S19.6.3 / S21.7 / S22 의 mode signal 통합 — 발화 source 와 영향 capsule
의 single source of truth.  **String-keyed namespace (M5)** — universal
customizability 정합, default 와 user-defined sub-mode 동일 type:

```rust
enum ModeSignal {
    /// sub-mode 전환 — name 은 default (`dock` / `portable` / `transportation`)
    /// 또는 user-defined name. boot 시점 등록된 set 안에서 lookup
    SubModeChange { name: String },

    DockDetect { docked: bool },              // S22 — ACS event 또는 host operator manual
    BatteryThreshold { below_percent: u8 },   // S22.3 BatteryThreshold wake source trigger
    ThermalMode { state: ThermalState },      // S21.7 — Throttling/HardThrottling 진입/이탈
    VehicleBusDetect { active: bool },        // M7 — CAN / OBD-II / vehicle Ethernet AVB signal
    GpsMovementDetect { sustained_kmh: u32 }, // M7 — ≥ 5 km/h sustained 30 sec
    UserExplicit { sub_mode: String },        // `y4-hypercall power sub-mode <name>`
    SmtForceChange { from: SmtForce, to: SmtForce }, // S19.6.1 force-toggle
}

enum ThermalState { Cool, Throttling, HardThrottling, Emergency }
```

발화 hub: power-orchestrator.  signal → form-factor sub-block 전환 (예:
`[mobile-portable]` ↔ `[mobile-dock]` ↔ `[mobile-transportation]`) +
영향 capsule (cpufreq / lifecycle / acpi-pm / wakeup) 자동 통보.

#### Sub-mode 전환 atomicity (M10)

sub-mode 전환 = lease suspend → const 변경 적용 → wake.  S20 lease
suspend 패턴 정합:

```
1. power-orchestrator 가 mode signal 수신 → 새 sub-mode 결정
2. 모든 active lease 의 lease-pm capsule 에 suspend 요청 (S20.3 패턴)
3. 모든 vCPU vmexit 후 — sub-mode const 변경 적용 (cpufreq / thermal /
   wake / TPM 등 capsule 의 상태 갱신)
4. lease 들 wake (S20.4 — XChaCha20-Poly1305 AEAD integrity check + replay
   protection)
5. audit `SubModeTransition` (severity Info, severity Warning if
   integrity fail)
```

force-toggle (S19.6.1 / S20.9 / S21.9 / S22.10 / S23.10) 활성 시 mode
signal mask — 사용자 의지 우선 (단 thermal hardlimit emergency 는 모든
force 우회).

### 2.5 Form-factor + sub-mode detection (G=i', M7, M8)

부팅 시점 결정 흐름 — 2-step (form-factor 1 차, sub-mode 2 차):

#### 2.5.1 Form-factor detection

```
1. cmdline `y4.pm.profile=<name>` 명시 → 해당 form-factor 적용 (1 순위).
   <name> 은 default 또는 user-defined 등록된 form-factor name 안에서
2. M8 deprecated alias — `y4.pm.profile=laptop|handheld` 가 `mobile`
   로 자동 매핑 + audit `DeprecatedFormFactorAlias` (Info, boot 1 회).
   v2 까지 유지, 그 후 reject
3. cmdline 미지정 → DMI / device tree + ACPI table 자동 감지:
    - DMI System Type = "Notebook" / "Laptop" → mobile
    - DMI Chassis Type = "Tablet" / "Hand Held" → mobile
    - DMI Chassis Type = "Portable" / "Convertible" → mobile
    - DMI Chassis Type = "Rack Mount Chassis" → rack-mount
    - SMBIOS BIOS Characteristics Server Available → server-farm
    - device tree / FDT 의 compatible = "*-soc" → soc
    - ARM device tree 의 chassis-type → 매핑
   detection rule 자체도 default — `tools/power.rules.d/00-default-
   detection.rules` 의 한 항목, user override 가능
4. detection 실패 또는 모호 시 conservative default = `rack-mount`
```

#### 2.5.2 Sub-mode detection (mobile 전용 + user-defined form-factor)

mobile (또는 sub-mode 보유한 user-defined form-factor) 의 boot 시점
sub-mode 결정:

```
1. cmdline `y4.pm.sub_mode=<name>` 명시 → 해당 sub-mode 적용 (1 순위).
   <name> 은 default (`dock` / `portable` / `transportation`) 또는
   user-defined sub-mode name
2. cmdline 미지정 → 자동 감지:
    a. transportation detection (M7):
       - vehicle bus signal active (CAN / OBD-II / vehicle Ethernet AVB)
       - GPS movement ≥ 5 km/h sustained 30 sec
       → 둘 중 하나라도 만족 시 transportation
    b. dock detection: dock detect signal (ACS event) active → dock
    c. 위 둘 모두 부재 → portable (mobile 의 default sub-mode)
3. 부팅 후 mode signal (§2.4) 발화 시 자동 전환 (S20 atomicity 패턴, M10)
```

#### 2.5.3 certified flag — detection 과 분리 (G=i')

`y4.pm.certified=on|off` 는 **별도 cmdline key** — form-factor /
sub-mode detection 결과와 무관하게 boot 시점에 1 회 결정:

```
1. cmdline `y4.pm.certified=on` → certified overlay 활성 (form-factor
   + sub-mode 위에 mergeover)
2. cmdline 미지정 또는 `off` → certified overlay 비활성
3. runtime 변경 X — boot 시점 fix
```

근거: certified 는 *compliance posture* — hardware / firmware / software
의 모든 layer 가 boot 시점에 일관 강화.  runtime 토글 시 prior-boot 의
non-certified state 가 audit chain 에 포함되어 compliance 깨짐.

### 2.6 새 form-factor / sub-mode 추가 path (H=a, M3, M4, M12, M13, v1.x patch 분류)

Form-factor 와 sub-mode 모두 동일 mechanism (M3 / M4 symmetric).

#### 2.6.1 새 form-factor 추가

```
예: 50-user-edge-gateway.rules

[edge-gateway]   # naming convention (M12): ASCII alphanumeric + hyphen, ≤ 32 char
Y4_PM_CPUFREQ_GOVERNOR              = ...
Y4_PM_DEEP_C_STATE_MAX              = ...
... (모든 30+ 변수 정의)
```

| Step | 작업 |
|---|---|
| 1 | `tools/power.rules.d/<NN>-<name>.rules` 의 새 `[<name>]` block (NN > 00 if override default) |
| 2 | §1.3 form-factor 별 위협 가중치 표 갱신 (새 row 추가, default rules 자체는 자동 mergeover) |
| 3 | detection rule 갱신 (`tools/power.rules.d/<NN>-detection.rules` overlay) |
| 4 | S15-S23 의 form-factor 별 표 (S16.2 / S17.7 / S18.5 / S20.7 / S21.10 / S22.2 / S23.7) 에 row 추가 |
| 5 | sub-decision invariant 변경 X → v1.x patch 분류 (§7 동결 정책 정합) |
| 6 | 기존 `[certified]` overlay 가 새 form-factor 에도 mergeover 가능한지 검증 (compliance posture 의 universal 보장) |

#### 2.6.2 새 sub-mode 추가 (form-factor 의 sub-mode 일 때)

```
예: 50-user-mobile-bicycle.rules

[mobile-bicycle]   # form-factor `mobile` 의 새 sub-mode
Y4_PM_DEEP_C_STATE_MAX              = "C3"
Y4_PM_THERMAL_TCC_C                 = 90
Y4_PM_LEASE_SUSPEND                 = ON
Y4_PM_SMT_INITIAL_STATE             = "off"
Y4_PM_WAKE_SOURCES                  = "gps-movement, power-button, rtc-alarm,
                                        battery-threshold"
Y4_PM_DETECTION                     = "ble-gps-bicycle-sensor"   # 사용자 정의 detection signal
... (모든 30+ 변수 정의)
```

| Step | 작업 |
|---|---|
| 1 | `[<form-factor>-<sub-mode>]` block 추가 (M12 naming) |
| 2 | (선택) detection rule 추가 — sub-mode 자동 감지 source 정의 |
| 3 | mode signal source 추가 — `ModeSignal::SubModeChange { name }` namespace 등록 |
| 4 | sub-decision invariant 변경 X (default Verus AV24 가 *named* sub-mode 만 strict, 그 외 generic) |

#### 2.6.3 Default definition removal (M13)

```
# 95-no-soc.rules
[soc]
removed = true

# 또는 일부 sub-mode 만 removal
[mobile-transportation]
removed = true
```

**Build-time check (lu-rule build 시점):**
- detection rule 의 references 모두 정의되어 있어야 함 (alias 통한 redirection 도 OK)
- Verus AV24 의 references 하는 *named* sub-mode 가 정의되어 있어야 invariant 적용
- removed 시 invariant 자동 비활성, references 깨짐 시 build fail

removal 시 사용자가 detection / Verus references 도 갱신 책임
(`tools/power.rules.d/<NN>-detection-update.rules` overlay).

#### 2.6.4 Default override (M3, M13)

같은 이름의 form-factor / sub-mode 가 여러 file 에 정의되면 **NN 큰 쪽이
우선 (replace semantics)**:

```
# 90-myorg-server-farm-override.rules — Y4 의 default 보다 보수적
[server-farm]
Y4_PM_DEEP_C_STATE_MAX              = "C0"      # default C1 → C0 강화
Y4_PM_RAPL_NOISE_BITS               = 6         # default 4 → 6 강화
... (override 하고 싶은 변수만, 나머지는 default mergeover)
```

### 2.7 3-layer 우선순위 (E=a, S4 패턴 정합)

S4 의 3-계층 ceiling 패턴을 power 측에도 적용:

| Layer | 결정 시점 | 결정 주체 | 의미 |
|---|---|---|---|
| **L1 — build-time const** | build | 형상 빌더 (`tools/power.rules.d/` overlay merge) | baseline default + user override mergeover 결과 (default + user file 합산, NN 큰 쪽 우선) + form-factor + sub-mode + certified overlay 적용 |
| **L2 — cmdline override** | boot | 시스템 운영자 | runtime override, ≤ L1 의 보수적 방향 (또는 baseline 무관 cmdline-only key).  Limine multiboot1 cmdline 으로 전달 |
| **L3 — runtime CLI / force-toggle / sub-mode change** | runtime | host operator | runtime 변경 (S19/S20/S21/S22/S23 force-toggle 5 개 + sub-mode change `y4-hypercall power sub-mode <name>`), audit Warning |

#### 변경 가능 layer 별 const 분류

| const | L1 | L2 | L3 |
|---|:---:|:---:|:---:|
| `Y4_PM_CONSTANT_FREQ` | ◎ | ◎ | ✗ (build-time fix) |
| `Y4_PM_DEEP_C_STATE_MAX` | ◎ | ◎ | ✗ |
| `Y4_PM_VOLTAGE_*_OFFSET_MV` | ◎ | ◎ (단 certified=on 시 차단) | ✗ |
| `Y4_PM_RAPL_DEFAULT_BUDGET_J` | ◎ | ◎ | ◎ (per-VM `set-cap` CLI) |
| `Y4_PM_THERMAL_TCC_C` | ◎ | ◎ | ◎ (`thermal-set --tcc` CLI) |
| `Y4_PM_*_FORCE_DEFAULT` | ◎ | ◎ | ◎ (force-toggle CLI) |
| `Y4_PM_TPM_REQUIRED` | ◎ | ◎ | ✗ (boot 시점 fix) |
| `Y4_PM_SMT_INITIAL_STATE` | ◎ | (cmdline `y4.amdv.smt_grouping` 으로 격리 모드만) | ◎ (SMT force-toggle, S19) |

certified=on 시 일부 const 의 L2 / L3 변경 차단 (§2.1 의 certified
overlay 영향 표 정합).

---

## 3. 안전장치 catalog (S15~S20+, 본격 spec sign-off cycle 대기)

본 §3 은 sign-off cycle (sub-decision iteration) 으로 진입.  S15 부터
순차 진행.

### S15 — cpufreq governor 격리 (P-state)

**ARCH-II' 매핑:**
- `cpufreq` capsule — 본체.  P-state 결정 + governor 적용 + DVFS dwell
  time 강제.
- `msr-bitmap` capsule — P-state MSR (IA32_PERF_CTL/STATUS, AMD
  P-state MSR) trap forward.
- `cpuid-emul` capsule — frequency leaf (CPUID 0x15 / 0x16) 응답 마스킹.
- `audit` capsule — `PStateChange` op_tag entry 기록.
- power-orchestrator — P-state 변경 결정 dispatch + form-factor
  profile 적용.

guest 가 host CPU frequency 변경 또는 frequency observable 측정으로
cross-tenant secret extraction 시도하는 path 차단 (Hertzbleed family).

#### S15.1 — P-state 변경 권한 (A=a-i)

| 주체 | 권한 |
|---|---|
| **host operator** (root task) | P-state 변경 ◎ (governor 선택 + cmdline override) |
| **lease holder** | P-state 변경 X — own VM 안에서도 변경 0 |
| guest | 0 (S15.2 의 trap path) |

근거: lease holder 도 P-state 변경 권한 부여 시 host-wide impact 발생
(다른 tenant 의 frequency observable 변동) → Hertzbleed cross-tenant
attack 의 trigger 경로.  host operator 에 권한 집중.

#### S15.2 — P-state observable 차단 (B=b-i)

`msr-bitmap` capsule 의 S10.1 mandatory entry 에 다음 추가 (v1.x patch
형태로 amdv_safety.md S10.1 갱신):

| MSR | 권한 | 비고 |
|---|---|---|
| `IA32_PERF_STATUS (0x198)` | guest read 차단 | 현재 P-state 노출 |
| `IA32_PERF_CTL (0x199)` | guest read+write 차단 | P-state 강제 변경 |
| `MSR_PSTATE_CURRENT_LIMIT (0xC0010061)` | guest read 차단 | AMD 의 P-state limit |
| `MSR_PSTATE_CTL (0xC0010062)` | guest read+write 차단 | AMD P-state control |
| `MSR_PSTATE_STATUS (0xC0010063)` | guest read 차단 | AMD P-state 현재 상태 |
| `MSR_PSTATE_DEF_BASE (0xC0010064-0xC001006B)` | guest read 차단 | AMD P-state definition (8 개) |

write 시도 → S10.4 default-deny audit (op_tag = `MsrAccessDenied`) +
guest 에 #GP(0).

#### S15.3 — CPUID frequency leaf 마스킹 (C=c-i)

`cpuid-emul` capsule 의 CPUID emulation table 갱신 (S2 의 CPUID 정책
sub-section 정합):

| Leaf | 응답 정책 |
|---|---|
| **CPUID 0x15** (TSC frequency) | base/max frequency 비트는 cpuid-emul capsule 의 **fixed value** 로 마스킹 (form-factor profile 의 nominal frequency).  실제 host frequency 노출 0 |
| **CPUID 0x16** (Processor Frequency Information) | base / max / bus frequency 모두 fixed value 마스킹.  v1.0 default = base = max = nominal (Hertzbleed reproducibility 차단) |

fingerprinting / Spectre-variant detection 차단.

#### S15.4 — Governor 정책의 capsule scope (D=d-i)

per-form-factor build-time 결정 (logicutils `tools/power.rules` 의
`Y4_PM_CPUFREQ_GOVERNOR` 변수) + cmdline override
(`y4.pm.cpufreq_governor=performance|ondemand|powersave|userspace`).
v1.0 미지원 governor: `userspace` 는 host operator only 의 manual P-state
조작용, default OFF.

per-VM cap 으로 governor 변경 X — host-wide impact 차단.

#### S15.5 — DVFS frequency change rate 제한 (E=e-i)

```
Y4_PM_MIN_PSTATE_DWELL_NS    build-time const, default 10_000_000 (10 ms)
                              cmdline `y4.pm.min_pstate_dwell_ns` override
```

power-orchestrator 가 P-state 변경 사이 최소 dwell time 강제:

```
fn change_pstate(new_state: PState) -> Result<(), Y4Error> {
    let now = read_tsc();
    if now - last_pstate_change_tsc < Y4_PM_MIN_PSTATE_DWELL_NS_AS_TSC {
        return Err(Y4Error::PStateRateLimited);
    }
    cpufreq_capsule.apply(new_state)?;
    last_pstate_change_tsc = now;
    Ok(())
}
```

근거: secret-dependent rapid frequency oscillation 의 reproducibility
임계 차단 (Hertzbleed 의 frequency change rate 측정 분해능 < 10 ms
일반).  10 ms 는 secret extraction 의 SNR 을 1/N 로 줄임.

#### S15.6 — Constant-frequency mode (F=f-i)

```
Y4_PM_CONSTANT_FREQ   build-time const, default OFF
                      cmdline `y4.pm.constant_freq=on|off`
```

`Y4_PM_CONSTANT_FREQ=ON` 시:
- power-orchestrator 가 boot 시 nominal P-state 고정
- 모든 P-state 변경 syscall reject (host operator 도 변경 0 — change
  하려면 reboot)
- DVFS / Turbo 비활성

자동 활성 form-factor:
- **server-farm profile** — latency tail 차단 + Hertzbleed 차단
- **certified profile** (S10.2 의 certified MSR profile 과 짝) — 의료/
  항공/금융 인증 트랙

#### S15.7 — SMT pair frequency 동기 (G=g-i)

S5 의 SMT-aware grouping (`Y4_AMDV_SMT_GROUPING = isolate-pairs`) 정합:

`isolate-pairs` 모드일 때 SMT pair 양쪽이 **같은 P-state 강제** —
cpufreq capsule 의 P-state apply 가 logical APIC ID 의 SMT sibling
을 자동 같은 상태로 set.

근거: SMT pair 의 한쪽만 다른 P-state → cross-thread frequency
observable (FastSpec / SMT-Hertzbleed variant).  S5 의 SMT 격리와 짝.

`allow-mixed` 모드는 SMT 격리 자체 약화 — 본 강제 적용 X.

#### S15.8 — Audit (S12 정합) (H=h-i)

모든 P-state 변경이 `audit` capsule 의 ring buffer 에 기록.  S12.2
schema 에 v1.x patch 로 op_tag 추가:

```rust
enum OpTag {
    // 기존 ...
    PStateChange,        // S15
    PStateRateLimited,   // S15.5 dwell time 위반
    // ...
}

enum AuditPayload {
    // ...
    PStateChange {
        cpu_id:    LogicalApicId,
        from:      PState,
        to:        PState,
        requester: Subject,    // host_operator | power_orchestrator_internal
    },
}
```

anomaly detector rule (S12.6 정합) 추가:
- `PStateRateLimited` ≥ 10 / 분 → warning (host operator 의 잘못된 cmd
  또는 internal bug)

### S16 — C-state side-channel 차단

**ARCH-II' 매핑:**
- `cpufreq` capsule — C-state 진입 결정 + 깊이 강제 + cache flush 정책.
- `lease-pm` capsule — deep C-state ≥ C3 시 lease state suspend trigger
  (S20 본체 짝).
- `msr-bitmap` capsule — C-state residency MSR trap.
- `audit` capsule — `CStateTransition` op_tag entry.
- power-orchestrator — HLT vmexit dispatch + form-factor profile 적용.

#### S16.1 — C-state 진입 권한 (A=a-i)

**host operator + power-orchestrator only — guest 직접 진입 0.**

guest 의 HLT / MWAIT 발화는 S2 의 `INTERCEPT_HLT` mandatory mask + (S16.3
의) MWAIT intercept 로 vmexit → power-orchestrator 가 form-factor profile
+ host operator policy 에 따라 결정 → cpufreq capsule 호출.

| 주체 | 권한 |
|---|---|
| **host operator** (root task) | C-state 진입/이탈 결정 ◎ (cmdline `y4.pm.cstate_max` 또는 runtime CLI) |
| **power-orchestrator** (internal) | HLT/MWAIT vmexit 응답으로 자동 진입 결정 (B 의 max 안에서) |
| **lease holder** | 0 — own vCPU 의 idle 신호만 발화, 실제 C-state 결정 X |
| guest | 0 |

#### S16.2 — C-state 깊이 상한 (B=b-i)

```
Y4_PM_DEEP_C_STATE_MAX    build-time const, per-form-factor
                           cmdline `y4.pm.cstate_max=C0|C1|C2|C3|C6` override
```

per-form-factor default (`tools/power.rules` §2.3 정합):

| Form factor | default max |
|---|---|
| server-farm | **C1** (latency tail 차단 + Hertzbleed 차단) |
| rack-mount | C2 |
| laptop | **C6** (battery 우선) |
| handheld + 독 | battery 모드 = C6, 독 모드 = C2 (mode switch 시 lease suspend 짝) |
| SoC | C6 또는 platform 별 deepest |

cmdline 미지정 시 build-time default.  cmdline override 는 build-time
이상 X (max 만 줄일 수 있음).

#### S16.3 — MWAIT hint 마스킹 (C=c-i)

S2 의 mandatory mask 에 MWAIT 추가 (현재 INVLPG/PAUSE/MWAIT/MONITOR 는
intercept 미포함, v1.0 power 측의 별도 추가 — amdv_safety.md S2 의 v1.x
patch 형태):

```
INTERCEPT_MWAIT      추가 mandatory (S16 본체)
INTERCEPT_MONITOR    짝
```

cpufreq capsule 의 MWAIT handler:

```rust
fn handle_mwait(eax: u32, ecx: u32) -> Result<CState, Y4Error> {
    let target = parse_mwait_eax(eax)?;          // EAX = target C-state hint
    let break_enable = (ecx & 1) != 0;           // ECX bit 0 = interrupt-as-break

    let max = pm_rules::deep_c_state_max();      // S16.2 의 form-factor max
    let actual = if target > max {
        log::warn!("guest mwait C{} > max C{}, substituting", target, max);
        max                                        // silent substitution
    } else {
        target
    };

    cpufreq_capsule.enter_cstate(actual, break_enable)
}
```

silent substitution → guest workload 호환 (Linux idle path) + 깊이 통제.

#### S16.4 — C-state residency observable 차단 (D=d-i)

`msr-bitmap` capsule 의 S10.1 mandatory entry 갱신 (v1.x patch):

| MSR | 권한 | 비고 |
|---|---|---|
| `MSR_PKG_C2_RESIDENCY (0x60D)` | guest read 차단 | Intel package C2 residency |
| `MSR_PKG_C3_RESIDENCY (0x3F8)` | guest read 차단 | Intel package C3 |
| `MSR_PKG_C6_RESIDENCY (0x3F9)` | guest read 차단 | Intel package C6 |
| `MSR_PKG_C7_RESIDENCY (0x3FA)` | guest read 차단 | Intel package C7 |
| `MSR_CORE_C3_RESIDENCY (0x3FC)` | guest read 차단 | core C3 |
| `MSR_CORE_C6_RESIDENCY (0x3FD)` | guest read 차단 | core C6 |
| `MSR_CORE_C7_RESIDENCY (0x3FE)` | guest read 차단 | core C7 |
| `MSR 0xC0010073` (AMD C-state base) | guest read+write 차단 | AMD C-state config |

read 시도 → S10.4 default-deny audit + #GP(0).

#### S16.5 — C-state 진입 시 cache flush 정책 (E=e-i)

**C-state ≥ C3 진입 직전 partial TLB + L1D flush.**

| C-state | flush 정책 |
|---|---|
| C0 / C1 | flush 0 (성능 우선, cache 보존이 wake latency ↓) |
| C2 | partial TLB flush (SVM ASID 단위) |
| **C3 이상** | partial TLB + **L1D flush** (Foreshadow / L1TF mitigation 정합) |
| C6 / C7 | C3 동일 + LLC flush 는 hardware 자동 (deep idle 시 cache 자동 유실) |

cpufreq capsule 의 enter_cstate:

```rust
fn enter_cstate(target: CState, ...) -> Result<(), Y4Error> {
    if target >= CState::C3 {
        flush_partial_tlb(current_asid());
        flush_l1d();           // Intel: L1D_FLUSH_CMD MSR write
                               // AMD: equivalent op
    }
    hardware_enter_cstate(target)
}
```

L1TF / Foreshadow 같은 deep-idle wake 시 cache state leak 차단.

#### S16.6 — C-state 진입/이탈 timing deterministic (F=f-ii)

```
Y4_PM_CSTATE_ENTRY_DETERMINISTIC    per-form-factor build-time
                                     cmdline `y4.pm.cstate_deterministic=on|off`
```

per-form-factor default:

| Form factor | deterministic |
|---|---|
| **server-farm** | **OFF** — latency 우선, deterministic delay 가 tail latency 영향 |
| rack-mount | OFF |
| **laptop** | **ON** — side-channel 우선, battery 영향 미미 |
| handheld + 독 | battery 모드 = ON, 독 모드 = OFF |
| SoC | platform 별 |

ON 시:
- C-state 진입 시 fixed entry delay 추가 (build-time `Y4_PM_CSTATE_ENTRY_FIXED_NS`,
  default 50 µs) — cache state observable 차단
- 이탈 시 fixed exit delay (default 100 µs) — wake timing 측정 불가

OFF 시:
- hardware 의 native C-state latency 그대로 — 성능 우선

#### S16.7 — SMT pair C-state 동기 (G=g-i)

S5 / S15.7 의 `isolate-pairs` 모드 정합 — SMT pair 양쪽이 **같은
C-state 강제**:

cpufreq capsule 의 enter_cstate 가 logical APIC ID 의 SMT sibling
도 함께 진입 (둘 다 ready 일 때만, 한쪽만 ready 면 양쪽 C0 유지).

근거: SMT pair 한쪽만 deep idle → 다른 쪽 host workload 의 cache /
power state observable (cross-thread side-channel).  S5 의 SMT 격리와
짝.

`allow-mixed` 모드 (SMT 격리 자체 약화) → 본 강제 적용 X.

#### S16.8 — Lease state suspend 진입 임계 (H=h-i)

`lease-pm` capsule 의 suspend trigger 임계는 **C-state ≥ C3** —
S20 본체 (별도 spec) 의 진입 조건.  본 §S16 은 trigger 임계 명시만:

```
if cstate >= CState::C3 {
    lease_pm_capsule.suspend(current_lease)?;
}
```

Suspend 본체 (XChaCha20 key SRAM 보호 / HMAC integrity check / atomicity
보장 / wake 시 restore) 는 **S20** 에서 정의.

#### S16.9 — Audit (S12 정합) (I=i-i)

S12.2 schema 에 v1.x patch 로 op_tag 추가:

```rust
enum OpTag {
    // 기존 ...
    CStateTransition,    // S16.9 — C0..C7 진입/이탈
    // ...
}

enum AuditPayload {
    // ...
    CStateTransition {
        cpu_id:   LogicalApicId,
        from:     CState,
        to:       CState,
        trigger:  enum { GuestHlt, GuestMwait, OperatorOverride, Internal },
        sibling_smt: Option<LogicalApicId>,    // SMT pair 동기 시
    },
}
```

severity 정책:
- C0 ↔ C1 / C0 ↔ C2 transition → severity = **Trace** (volume ↑↑, S12.3
  trace tier circular)
- C0 ↔ C3 이상 transition → severity = **Info** (priority tier)
- `OperatorOverride` trigger → severity = **Info** (priority)
- 모든 deep C-state ≥ C6 진입 → severity = **Info**

anomaly detector rule (S12.6 정합) 추가:
- per-CPU C-state ≥ C6 진입 빈도 ≥ 1000 / 분 → warning (jitter / power
  bug 가능성)

### S17 — RAPL / energy counter 격리

**ARCH-II' 매핑:**
- `rapl` capsule — 본체.  RAPL MSR access mediation + virtio-rapl
  paravirt interface server-side + per-VM energy budget tracking +
  noise injection.
- `msr-bitmap` capsule — RAPL MSR trap forward.
- `cpuid-emul` capsule — RAPL feature bit 마스킹.
- `audit` capsule — `RaplRead` / `EnergyBudgetSet` op_tag entry.
- power-orchestrator — RAPL-based DVFS / thermal cap 의 internal channel
  hub.

**위협:** PLATYPUS (USENIX Security '21) / PowerHammer / RAPL-based
side-channel.  guest 가 RAPL energy counter read 로 cross-tenant
secret-dependent power profile 측정 → AES / RSA secret 추출.

#### S17.1 — RAPL MSR access 차단 (A=a-i)

`msr-bitmap` capsule 의 S10.1 mandatory entry 갱신 (v1.x patch).
**모든 guest read+write 차단:**

| MSR | Vendor | 권한 |
|---|---|---|
| `MSR_RAPL_POWER_UNIT (0x606)` | Intel | guest read+write 차단 |
| `MSR_PKG_RAPL_POWER_LIMIT (0x610)` | Intel | guest read+write 차단 |
| `MSR_PKG_ENERGY_STATUS (0x611)` | Intel | guest read+write 차단 |
| `MSR_PKG_PERF_STATUS (0x613)` | Intel | guest read+write 차단 |
| `MSR_PKG_POWER_INFO (0x614)` | Intel | guest read+write 차단 |
| `MSR_DRAM_RAPL_POWER_LIMIT (0x618)` | Intel | guest read+write 차단 |
| `MSR_DRAM_ENERGY_STATUS (0x619)` | Intel | guest read 차단 |
| `MSR_DRAM_PERF_STATUS (0x61B)` | Intel | guest read 차단 |
| `MSR_DRAM_POWER_INFO (0x61C)` | Intel | guest read 차단 |
| `MSR_PP0_POWER_LIMIT (0x638)` | Intel | guest read+write 차단 |
| `MSR_PP0_ENERGY_STATUS (0x639)` | Intel | guest read 차단 |
| `MSR_PP0_POLICY (0x63A)` | Intel | guest read+write 차단 |
| `MSR_PP1_POWER_LIMIT (0x640)` | Intel | guest read+write 차단 |
| `MSR_PP1_ENERGY_STATUS (0x641)` | Intel | guest read 차단 |
| `MSR_PP1_POLICY (0x642)` | Intel | guest read+write 차단 |
| `MSR_CORE_ENERGY_STATUS (0xC001029A)` | AMD | guest read 차단 |
| `MSR_PKG_ENERGY_STATUS (0xC001029B)` | AMD | guest read 차단 |
| `MSR_RAPL_PWR_UNIT (0xC0010299)` | AMD | guest read+write 차단 |

read 시도 → S10.4 default-deny audit (op_tag = `MsrAccessDenied`) +
guest 에 #GP(0).

#### S17.2 — virtio-rapl paravirt interface (B=b-i)

Linux guest 호환을 위해 virtio paravirt RAPL interface — guest 의
`/sys/class/powercap/` 가 host 직접 노출 X, **per-VM 격리된 energy view**
만 받음:

```rust
// rapl capsule 의 server-side API
fn virtio_rapl_get_energy(lease: LeaseCap, domain: RaplDomain) -> u64 {
    // host 의 실제 energy 값 X — per-VM tracking 의 결과
    let raw = lease.energy_tracker.get(domain);
    let masked = raw & !((1 << Y4_PM_RAPL_NOISE_BITS) - 1);  // S17.4 noise
    masked
}
```

guest 의 `intel-rapl` / `amd_energy` driver 가 본 paravirt interface
와 통신.  host 의 실제 RAPL 값은 **read 0** — guest 가 cross-tenant
power profile 추정 가능성 차단.

per-VM tracking 메커니즘:
- vmrun 시작 / 종료 시점에 host RAPL counter snapshot (host operator
  scope, capsule internal)
- delta 를 per-lease energy tracker 에 누적
- guest 의 virtio query 시 누적 값 반환 (S17.4 noise 적용)

#### S17.3 — host operator RAPL access (C=c-i)

`rapl` capsule 의 server-side API:

```
host_operator
    → rapl_capsule.read_energy(domain)
        → AuditEntry { op_tag: RaplRead, ... }  (S17.5 audit)
        → returns: (raw_value: u64)
```

CLI (sibling repo `/home/ybi/y4-hypercall/`):

```
y4-hypercall energy show                    # 모든 RAPL domain 의 host 측 값
y4-hypercall energy show --vm <id>          # per-VM budget 사용량
y4-hypercall energy set-cap --vm <id> --budget-j 1000
                                            # H 의 per-VM budget 설정
```

host operator 가 RAPL MSR 직접 read X — 항상 capsule mediation.

#### S17.4 — cross-tenant noise injection (D=d-i)

```
Y4_PM_RAPL_NOISE_BITS    build-time const, default 4
                          cmdline `y4.pm.rapl_noise_bits=N` (0 ≤ N ≤ 8)
```

`rapl` capsule 의 모든 read 응답 (virtio + host operator + per-VM budget
tracker 읽기) 에 **LSB N bit AND-mask** 강제:

```rust
fn read_energy_masked(domain: RaplDomain) -> u64 {
    let raw = read_rapl_msr(domain);
    raw & !((1u64 << Y4_PM_RAPL_NOISE_BITS) - 1)
    // default 4 bit → 16-unit quantization
}
```

근거: PLATYPUS 의 attack SNR 이 LSB resolution 의존.  4-bit 마스킹 →
16-unit quantization → secret-dependent signal 의 SNR 1/16 로 ↓.
8-bit 마스킹 (extreme) 시 256-unit, secret extraction 사실상 차단.
v1.0 default 4 bit = 사용성 (energy budget tracking 정확도) + 보안
balance.

#### S17.5 — Audit (S12 정합) (E=e-i)

S12.2 schema 에 v1.x patch 로 op_tag 추가:

```rust
enum OpTag {
    // 기존 ...
    RaplRead,            // S17.5 — host operator 의 RAPL read
    EnergyBudgetSet,     // S17.5 — per-VM energy budget 설정/변경
    EnergyBudgetExceeded,// S17.8 — per-VM budget 초과 시 vmrun reject
    // ...
}
```

severity 정책:
- 모든 entry = severity **Info** (priority tier, S12.3 priority block-
  on-full).  read 빈도 낮음 (host operator manual) → audit cost 작음.
- `EnergyBudgetExceeded` = severity **Warning** — anomaly detector trigger

#### S17.6 — RAPL-based thermal cap 호환 (F=f-i)

host kernel 의 RAPL ↔ cpufreq / DVFS integration 패턴 보존:

```
power-orchestrator (internal channel)
    ├── rapl capsule (energy 측정 + budget tracking)
    └── cpufreq capsule (P-state 제한 적용)
```

RAPL-based DVFS / thermal cap 결정은 **power-orchestrator 안에서**
일어남 — capsule 외부 노출 0.  Linux 의 RAPL → CPUFreq governor signal
패턴과 동일 의미, 단 격리 보존.

#### S17.7 — Form-factor 별 RAPL audit (G=g-i)

per-form-factor build-time `Y4_PM_RAPL_AUDIT` (`tools/power.rules`):

| Form factor | default |
|---|---|
| server-farm | **ON** (compliance + multi-tenant tracking) |
| rack-mount | ON |
| laptop | ON (default) |
| handheld + 독 | ON |
| **SoC** | **OFF** (RAPL hardware 부재 다수, 측정 불가) |

cmdline `y4.pm.rapl_audit_enable=on|off` override.  OFF 시 S17.5 audit
entry generation 0 — 단 S17.1 의 MSR 차단은 그대로 강제.

#### S17.8 — RAPL energy budget per-VM cap (H=h-i)

```
Y4_PM_RAPL_DEFAULT_BUDGET_J    build-time const, default = u64::MAX (unlimited)
                                cmdline `y4.pm.rapl_default_budget_j` override
```

lease 발급 시 per-VM budget 설정 (host operator API):

```rust
struct LeaseCap {
    // ...
    energy_budget_j:  u64,    // 누적 허용 energy (Joule)
    energy_used_j:    u64,    // 현재 누적 사용량
}

fn vmrun(vcpu: VcpuId, ...) -> Result<(), Y4Error> {
    let lease = vcpu_to_lease(vcpu);
    if lease.energy_used_j >= lease.energy_budget_j {
        audit.append(EnergyBudgetExceeded { lease_id: lease.id, ... });
        return Err(Y4Error::EnergyBudgetExceeded);
    }
    // 정상 vmrun ...
}
```

근거:
- server-farm 의 multi-tenant 환경에서 power budget 분배
- 악의적 / 손상된 guest 의 power-DoS 차단 (Spectre-style power-hammer
  mitigation)
- v1.0 default = unlimited (호환성 우선) — operator 가 cap 설정 시
  만 강제

CLI: `y4-hypercall energy set-cap --vm <id> --budget-j 1000` (S17.3
정합).

#### S17.9 — CPUID RAPL feature bit 마스킹 (I=i-i)

`cpuid-emul` capsule 의 CPUID emulation table 갱신:

| Leaf | 마스킹 비트 |
|---|---|
| `CPUID 0x6.EAX[14]` (RAPL Power Limit Notification) | 0 강제 |
| `CPUID 0x6.EAX[15]` (HWP / Hardware P-states) | 0 강제 (S15 의 host operator only governor 와 짝) |
| `CPUID 0x80000007.EDX[16]` (AMD Core Performance Boost) | 0 강제 (Turbo 마스킹, S15.6 constant-freq mode 정합) |
| `CPUID 0x80000008.EBX[12]` (AMD RAPL2) | 0 강제 |
| `CPUID 0x14` (Intel Processor Trace) sub-leaves | 0 강제 (PT-based side-channel 차단, v1.x 의 별도 검토 후보) |

guest 가 RAPL / Turbo / HWP feature 미보유로 인식 → driver 자동 fallback.

### S18 — ACPI _PSx / _CST / _PSV 검증

**ARCH-II' 매핑:**
- `acpi-pm` capsule — 본체.  per-VM virtual DSDT 생성 + AML interpreter
  server-side + mutating method mediation + thermal threshold 적용 +
  ACPI table integrity check.
- `cpufreq` capsule — _PSx eval 결과를 P-state 적용으로 forward (S15
  mediation 와 짝).
- `audit` capsule — `AcpiMethodEval` op_tag entry.
- `firmware-approval` capsule — ACPI table hash mismatch 시 lease revoke
  trigger (S14 정합).
- power-orchestrator — _SHUTDOWN / _S5 / 형상별 thermal cap 의 dispatch.

#### S18.1 — Guest ACPI eval mediation 모델 (A=a-i)

**모든 guest 의 ACPI method eval 이 acpi-pm capsule 의 server-side
interpreter 경유.**  host AML interpreter 직접 노출 X — guest 는 own
copy 의 virtual DSDT view 만 받고, host DSDT 의 mutating method 는
mediation:

```
guest OS (Linux ACPICA)
    → AML eval (own virtual DSDT 안)
        ├── read-only method (S18.2 화이트리스트) → 직접 응답
        └── mutating method (S18.2 화이트리스트) → acpi-pm capsule mediation
                → host 정책 적용
                → 가짜 결과 (per-VM 격리된 view) 반환
                → audit (S18.9)
```

#### S18.2 — DSDT method 화이트리스트 (B=b-i)

| 분류 | Methods | mediation |
|---|---|---|
| **read-only (informational)** | `_OSI` (D 의 화이트리스트), `_OS`, `_PIC`, `_HID`, `_UID`, `_CID`, `_STR`, `_BBN`, `_ADR` | 직접 응답 (per-VM view) |
| **mutating — power state** | `_PSV` (passive thermal), `_PSx` (P-state list), `_CST` (C-state list), `_TCC` (thermal control), `_TZD` (thermal zone devices), `_PCT` (P-state control), `_PPC` (perf present capabilities) | acpi-pm capsule mediation, host 정책 적용 후 응답 |
| **mutating — battery** | `_PSL` (power source list), `_PSR` (power source), `_PIF` (power info), `_BIF` (battery info), `_BST` (battery status) write | mediation, 가짜 값 반환 (handheld/laptop 형상의 form-factor profile 또는 host 의 실제 값 마스킹) |
| **mutating — Sx** | `_S0`~`_S5`, `_PTS` (prepare to sleep), `_WAK` (wake) | F=f-i 의 power-orchestrator forward |
| **금지** | `_PRW` (wake setting write), `_DIS` (disable device), `_SRS` (set resource settings), `_SRT` (set thermal trip), 임의 SSDT load | acpi-pm capsule reject + audit Warning |

#### S18.3 — Per-VM virtual DSDT (C=c-i)

acpi-pm capsule 이 lease 발급 시 자동 생성:

```
virtual DSDT 의 base 구성:
- 1 (또는 N) vCPU CPU object — guest 의 multi-vCPU 와 정합
- minimal device tree:
    * virtio-rapl (S17.2 의 paravirt RAPL)
    * virtio-thermal (S21 의 paravirt thermal)
    * virtio-battery (laptop / handheld profile 만)
    * virtio-pm (deep idle / wake source — S20 / S22 짝)
- guest 의 _OSI 응답 화이트리스트 (D)
- _S0 / _S5 만 정의, _S3 / _S4 는 form-factor 의존
- _CST 응답 = power_safety.md S16.2 의 form-factor max 와 정합
```

guest 가 보는 DSDT ≠ host DSDT — host 의 PCI topology / device 정보
노출 0.  host PCIe device passthrough 는 Phase D 의 별도 표면
(IOMMU programming capsule).

#### S18.4 — _OSI string handling (D=d-i)

acpi-pm capsule 의 _OSI 응답 화이트리스트 — Linux kernel 의 `acpi_osi=`
패턴 정합:

| _OSI string | 응답 |
|---|---|
| `"Linux"` | TRUE |
| `"Windows 2024"`, `"Windows 2023"`, ..., `"Windows 2009"` | TRUE (호환성) |
| `"Y4"` | TRUE (Y4-specific path 식별) |
| `"Module Device"`, `"Processor Device"`, `"3.0 _SCP Extensions"`, `"Extended Address Space Descriptor"` | TRUE |
| 그 외 | FALSE |

`Y4` string 은 Y4-aware DSDT method 가 host policy 인지 확인용 (예:
"Y4 environment 면 alternate path 사용").

cmdline `y4.pm.acpi_osi_extra=...` 로 추가 string 화이트리스트 가능
(host operator only).

#### S18.5 — _PSV / _TCC thermal threshold (E=e-i)

host operator 만 thermal threshold 설정:

```
Y4_PM_THERMAL_PSV_C    build-time const, default 85 (Celsius)
                        per-form-factor 차등 (logicutils tools/power.rules)
                        cmdline `y4.pm.thermal_psv_c` override

Y4_PM_THERMAL_TCC_C    build-time const, default 95
                        cmdline `y4.pm.thermal_tcc_c` override
```

per-form-factor default:

| Form factor | _PSV (passive) | _TCC (critical) |
|---|---|---|
| server-farm | 80°C | 90°C |
| rack-mount | 85°C (default) | 95°C (default) |
| laptop | 75°C | 90°C |
| handheld + 독 | 70°C / 85°C (mode 별) | 85°C / 95°C |
| SoC | platform 별 |  |

guest 의 `_PSV` write attempt → acpi-pm capsule mediation 가 **reject
+ audit Warning** (`AcpiMethodEval` with op = `_PSV` write attempt,
severity Warning).

#### S18.6 — ACPI Sx state 진입 권한 (F=f-i)

host operator only.  guest 의 `_S5` (shutdown) / `_S3` (sleep) 발화 시:

```
guest OS calls _S5 ()
    → acpi-pm capsule mediation
        → power-orchestrator 에 forward
            → form-factor 별 정책 적용:
                * server-farm: own VM 만 destroy (host shutdown X)
                * laptop: same — own VM 만 destroy
                * handheld: own VM 만, host 는 영향 0
            → lifecycle capsule (cross-cluster) 의 lease revoke trigger (S13)
        → guest 에 success 응답 (own VM 종료 진행)
    → audit (S18.9 의 `AcpiSxStateEnter` op_tag)
```

guest 가 host 전체 shutdown 또는 sleep 강제 0.

#### S18.7 — ACPI eval timeout (G=g-i)

```
Y4_PM_ACPI_EVAL_TIMEOUT_NS    build-time const, default 100_000_000 (100 ms)
                                cmdline `y4.pm.acpi_eval_timeout_ns` override
```

acpi-pm capsule 의 AML interpreter 가 method eval 시작 시점부터 100 ms
초과 시:
1. interpreter 강제 abort
2. guest 에 `Y4Error::Timeout` 반환
3. audit (op_tag = `AcpiMethodTimeout`, severity Warning)

근거: DSDT 안의 악성 / 손상된 method (무한 loop, 깊은 recursion) 가
host 의 acpi-pm capsule 을 wedge 하는 DoS 차단.  S4 deadline 패턴 정합.

100 ms 는 정상 ACPI method 의 최악 case (수백 IO port poll 포함) 를
모두 흡수하면서 DoS 임계 안전.

#### S18.8 — ACPI table integrity (H=h-i)

acpi-pm capsule 이 부팅 시점에 host ACPI table 의 SHA-256 hash 기록:

| Table | hash 기록 |
|---|---|
| RSDP (Root System Description Pointer) | ◎ |
| RSDT / XSDT | ◎ |
| FADT (Fixed ACPI Description Table) | ◎ |
| DSDT (Differentiated System Description Table) | ◎ |
| SSDT (Secondary SSDT) — 부팅 시 load 된 모든 SSDT | ◎ |
| MADT (Multiple APIC Description Table) | ◎ |
| MCFG, HPET, SLIT, SRAT, BGRT, ... | ◎ (모든 ACPI table) |

매 mutating method eval 시 (또는 주기적으로 — 1 분마다) hash 재검증:

```rust
fn before_acpi_method_eval(method: &Method) -> Result<(), Y4Error> {
    if !verify_acpi_table_hashes() {
        audit.append(AcpiTableTampered { ... });
        firmware_approval.queue_revoke_all_leases();   // S14 정합
        return Err(Y4Error::SecurityViolation);
    }
    Ok(())
}
```

hash 변경 = firmware-mutating attack 가능성 → audit Critical + 모든
lease revoke trigger (S14 firmware-approval capsule 와 짝).

#### S18.9 — Audit (S12 정합) (I=i-i)

S12.2 schema 에 v1.x patch 로 op_tag 추가:

```rust
enum OpTag {
    // 기존 ...
    AcpiMethodEval,         // S18.9 — mutating method eval (severity Info)
    AcpiMethodEvalReadOnly, // S18.9 — read-only method (severity Trace)
    AcpiMethodTimeout,      // S18.7 — timeout (severity Warning)
    AcpiMethodRejected,     // S18.5 / S18.2 금지 method (severity Warning)
    AcpiSxStateEnter,       // S18.6 — Sx 진입 (severity Info)
    AcpiTableTampered,      // S18.8 — hash mismatch (severity Critical)
    // ...
}

enum AuditPayload {
    // ...
    AcpiMethodEval {
        method_name:  String,        // 예: "_PSV"
        args:         Vec<u64>,
        result:       Result<u64, Y4Error>,
        duration_ns:  u64,
    },
    AcpiTableTampered {
        table:        String,        // "DSDT" 등
        expected_sha: [u8; 32],
        actual_sha:   [u8; 32],
    },
}
```

severity 정책:
- read-only method = **Trace** (volume ↑, S12.3 trace tier circular)
- mutating method = **Info** (priority tier)
- timeout / rejected = **Warning** (priority + anomaly detector trigger)
- table tampered = **Critical** (priority + 즉시 lease revoke)

anomaly detector rule (S12.6 정합) 추가:
- `AcpiMethodTimeout` ≥ 5 / 분 → warning, 잠재 DoS 시도
- `AcpiMethodRejected` ≥ 10 / 분 → warning
- `AcpiTableTampered` 1 회 → **즉시 모든 lease revoke** (S14 정합)

### S19 — SMT power gating ↔ S5 정합

**ARCH-II' 매핑:**
- `cpufreq` capsule — P-state SMT 동기 (S15.7), C-state SMT 동기 (S16.7),
  hardware SMT enable/disable.
- `lifecycle` capsule (cross-cluster) — S5 SMT-aware grouping +
  S5.3 CPU offline 자동 re-pin + SMT toggle 시 vCPU migrate.
- `audit` capsule — SMT 관련 5 op_tag entry.
- power-orchestrator — SMT pair power state 결정 hub + 3-tier force-
  toggle dispatch (F=f-iv').

S5 의 `Y4_AMDV_SMT_GROUPING` (boot-time fix) 가 host 측 SMT pair 의
cross-thread side-channel **격리 모드**.  S19 는 power 측 본체 — SMT
pair 의 power state (P-state / C-state / power gate) 동기 보장 + 한쪽
만 power off 되어 다른 쪽 측정 가능 path 차단 + convertible 디바이스
대응.

#### S19.1 — SMT pair power gating 권한 (A=a-i)

**host operator + power-orchestrator only.**  guest 의 SMT thread
개별 power gate 0 — guest 는 own vCPU 단위 idle 만 신호, 실제 SMT
sibling power gate 결정은 power-orchestrator.

| 주체 | 권한 |
|---|---|
| host operator | force-toggle (F1, S19.6) + runtime CLI `y4-hypercall power smt-*` |
| power-orchestrator | follow-policy 모드의 자동 결정 (S19.6 의 L1/L2) |
| lease holder | 0 — own vCPU 의 idle 신호만 |
| guest | 0 |

#### S19.2 — Strict 동기 (isolate-pairs 모드, B=b-i)

`Y4_AMDV_SMT_GROUPING = isolate-pairs` (S5 boot-time) 일 때:

cpufreq capsule + lifecycle capsule 의 모든 power 변경 operation 이
**SMT pair 양쪽 동시 적용**:

| 변경 항목 | 동기 강제 |
|---|---|
| P-state | pair 양쪽 같은 P-state (S15.7) |
| C-state | pair 양쪽 같은 C-state (S16.7) |
| Power gate state | pair 양쪽 같은 gate state (S19) |
| Hardware online/offline | pair 양쪽 동시 (S19.4) |

한쪽만 변경 시도 (예: bug 또는 손상된 capsule 의 partial apply) →
**reject + audit** (`SmtPairDesyncAttempt`, severity Warning).

#### S19.3 — Allow-mixed 모드 정책 (C=c-i)

`Y4_AMDV_SMT_GROUPING = allow-mixed` 일 때:
- S19 의 동기 강제 적용 X (격리 모드 자체가 약화됨)
- boot 시점 audit Warning 1 회: `SmtAllowMixedActive` (operator 가
  보안 약화 명시 인지)
- F=force-on + allow-mixed 시 **추가 audit Critical** (F3=a, S19.6.3
  본문)

#### S19.4 — SMT thread offline (D=d-i)

host operator only.  D=d-i 의 권한이 F (S19.6) 의 mode signal 의 한
경로로 흡수 — 같은 mechanism:

```
y4-hypercall power smt-offline <apic-id>
    → power-orchestrator
        → isolate-pairs 모드 시: pair 양쪽 함께 offline 강제
            (한쪽만 → reject, audit SmtPairDesyncAttempt)
        → allow-mixed 모드 시: 단일 thread offline 가능
        → lifecycle capsule: 해당 thread 에 묶인 vCPU 자동 migrate (S5.3)
        → cpufreq capsule: hardware offline (Hot-unplug equivalent)
        → audit SmtThreadOffline (severity Info)
```

#### S19.5 — SMT pair lockstep entry/exit atomicity (E=e-i)

cpufreq capsule 의 `enter_pstate` / `enter_cstate` / `power_gate` 가
SMT pair 양쪽에 atomic 적용:

```rust
fn enter_pstate_smt_synced(pair: SmtPair, target: PState) -> Result<(), Y4Error> {
    let lock = pair.acquire_lock();
    let primary_done = apply_pstate(pair.primary_apic, target);
    let sibling_done = apply_pstate(pair.sibling_apic, target);
    match (primary_done, sibling_done) {
        (Ok(_), Ok(_)) => { lock.release(); Ok(()) }
        _ => {
            // rollback both
            apply_pstate(pair.primary_apic, lock.previous_pstate);
            apply_pstate(pair.sibling_apic, lock.previous_pstate);
            lock.release();
            audit.append(SmtPairDesyncAttempt { ... });
            Err(Y4Error::SmtSyncFailed)
        }
    }
}
```

IPI broadcast (양쪽 thread 동시 깨우기) + per-pair spinlock + rollback
on partial failure.  중간 race window 0.

#### S19.6 — 3-tier force-toggle (F=f-iv')

사용자 최우선 override (KDE Plasma "screen lock / sleep inhibit" 패턴
정합):

```
[L0 user force-toggle]   force-on | follow-policy | force-off    ← 최우선
       ↓ (follow-policy 시에만 L1 → L2 가 작동)
[L1 logicutils rule]     form-factor profile 의 default
       ↓
[L2 mode signal event]   dock-detect / battery-low / 사용자 명시 mode
```

##### S19.6.1 — L0 user force-toggle

```rust
enum SmtForce {
    ForceOn,        // 모든 mode signal 무시, SMT 항상 enable
    FollowPolicy,   // L1 + L2 자동 적용 (default)
    ForceOff,       // 모든 mode signal 무시, SMT 항상 disable
}

// state 보관: power-orchestrator 의 internal cell
const Y4_PM_SMT_FORCE_DEFAULT: SmtForce = SmtForce::FollowPolicy;
```

설정 채널:
- cmdline `y4.pm.smt_force=on|off|policy` (boot 시점, F2=ii)
- runtime CLI:
  ```
  y4-hypercall power smt-force on
  y4-hypercall power smt-force off
  y4-hypercall power smt-force policy
  y4-hypercall power smt-status      # 현 force 상태 + 실 SMT enable 상태 + 마지막 mode signal
  ```

권한 (F1=i): **host operator only** — multi-tenant 일관성, lease
holder 의 host-wide SMT 변경 권한 부여 X (single-user laptop / handheld
형상의 사용자 = host operator 와 동일 person, KDE 사용자 경험 그대로).

persistence (F2=ii): cmdline default + runtime override.  reboot 시
cmdline 으로 fallback (config 파일 없는 Y4 minimalism — root task
init system 미보유).

##### S19.6.2 — L1 logicutils rule

`tools/power.rules` 의 form-factor 정책:

```
[server-farm]
Y4_PM_SMT_INITIAL_STATE = on    # throughput 우선

[rack-mount]
Y4_PM_SMT_INITIAL_STATE = on

[laptop]
Y4_PM_SMT_INITIAL_STATE = on    # default on, mode signal 따라 toggle

[handheld]
Y4_PM_SMT_INITIAL_STATE = off   # battery 우선 default

[handheld-dock-mode]
Y4_PM_SMT_INITIAL_STATE = on    # docked 시 throughput

[soc]
Y4_PM_SMT_INITIAL_STATE = platform-dependent
```

L0 == FollowPolicy 시 L1 의 default 가 boot 시점 적용.

##### S19.6.3 — L2 mode signal event (F4=a)

L0 == FollowPolicy + L1 default 위에서 평가:

| Signal source | Trigger | SMT 동작 |
|---|---|---|
| **dock detect** (ACS event 또는 host operator manual) | handheld → docked | L1 의 `[handheld-dock-mode]` rule 적용 → SMT on |
| **dock detect** | docked → handheld | L1 의 `[handheld]` rule 적용 → SMT off |
| **battery threshold** | < 20% (handheld / laptop) | SMT off (battery 절약) |
| **사용자 명시 mode** | `y4-hypercall power mode-set <profile>` | profile 의 SMT 설정 적용 |

thermal mode signal 은 S21 (별도 안전장치) 에서 다룸 — 본 §S19.6 범위
외.

##### S19.6.4 — 상호작용 명세

| L0 → L0' transition | 동작 |
|---|---|
| FollowPolicy → ForceOn | 즉시 SMT on (L1/L2 와 무관), audit `SmtForceToggle` (Warning) |
| FollowPolicy → ForceOff | 즉시 SMT off, audit `SmtForceToggle` (Warning) |
| ForceOn → FollowPolicy | L1 + L2 즉시 재평가, 필요 시 SMT toggle, audit `SmtForceToggle` (Warning) + 후속 `SmtRuntimeToggle` (Info) |
| ForceOff → FollowPolicy | L1 + L2 즉시 재평가, 필요 시 SMT toggle, audit `SmtForceToggle` + 후속 `SmtRuntimeToggle` |
| ForceOn → ForceOff (또는 역) | 즉시 SMT 상태 변경, audit `SmtForceToggle` |
| Any L0 + L2 mode signal | force-on / force-off 시 mode signal **무시 + audit 0** (force 가 mask).  follow-policy 시 자동 적용 + audit `SmtRuntimeToggle` |

##### S19.6.5 — F3=a force-on + allow-mixed 시 의무 audit

`Y4_AMDV_SMT_GROUPING = allow-mixed` (boot-time) + L0 = ForceOn 동시
활성 시:

```
audit.append(SmtForceToggle {
    new_state: ForceOn,
    grouping_mode: AllowMixed,
    severity: Critical,    // 명시적 보안 약화
    note: "ForceOn + allow-mixed = SMT cross-thread side-channel 차단 0.
           operator 가 명시 인지하고 활성.  Hertzbleed / SMT-MDS / L1TF
           cross-thread variant 노출 가능."
});
```

**Critical severity** — priority tier block-on-full + anomaly detector
즉시 host operator 통지.  보안 정책 변경의 가장 중요한 record.

#### S19.7 — SMT-Hertzbleed mitigation (G=g-i)

`Y4_PM_CONSTANT_FREQ` (S15.6) ON 시 SMT pair 양쪽 freq 변경 0 강제 —
cpufreq capsule 의 P-state apply 가 reject (양쪽 모두).  Hertzbleed
의 SMT variant 차단.

S15.6 와 정합:
- server-farm + certified profile 의 자동 활성과 동일 path
- F=ForceOn + constant_freq=ON = SMT enable 유지 + freq 고정 (most
  conservative configuration)

#### S19.8 — SMT pair 의 lease assignment (H=h-i)

`isolate-pairs` 모드 + SMT enable 상태:

| pair 상태 | 허용 |
|---|---|
| pair 양쪽이 같은 lease 의 vCPU | ◎ (cross-tenant 격리 자연 만족) |
| pair 한쪽 = lease A, 다른 쪽 = host idle | ◎ (cross-tenant 격리 만족, sibling 이 idle 이라 measurement noise 0) |
| pair 양쪽이 host idle | ◎ (lease 없음) |
| pair 한쪽 = lease A, 다른 쪽 = lease B | **✗ reject + audit** (HyperThreading-based side-channel — L1TF / TAA / SMT-MDS) |
| pair 한쪽 = lease A, 다른 쪽 = host workload (root task) | **✗ reject + audit** |

SMT off 상태 (F = ForceOff 또는 dock-mode 의 SMT off):
- pair sibling thread = hardware-offline → APIC ID 표면에서 사라짐
- lease 의 cpu_id 후보 = primary thread 의 APIC ID 만
- pair 자체 사라지므로 위 표 자동 만족 (vacuously true)

S5 의 cpu_id 검증 (lifecycle capsule) 이 SMT off APIC ID 거부 — 이미
boot-time 또는 runtime SMT toggle 시점에 lifecycle 의 valid APIC set
갱신 (S5.3 의 offline detection 메커니즘 재사용).

#### S19.9 — Audit (S12 정합) (I=i-i)

S12.2 schema 에 v1.x patch 로 op_tag 5 개 추가:

```rust
enum OpTag {
    // 기존 ...
    SmtPairSync,           // S19.2 — pair 동기 적용 (severity Trace, volume ↑↑)
    SmtPairDesyncAttempt,  // S19.2 / S19.5 — 한쪽만 변경 시도 reject (severity Warning)
    SmtThreadOffline,      // S19.4 — thread offline (severity Info)
    SmtRuntimeToggle,      // S19.6.3 — L1/L2 mode signal 의 toggle (severity Info)
    SmtForceToggle,        // S19.6.1 / S19.6.4 — L0 force-toggle 변경 (severity Warning, F3=a 시 Critical)
    SmtAllowMixedActive,   // S19.3 — boot 시점 1 회 (severity Warning)
    // ...
}

enum AuditPayload {
    // ...
    SmtForceToggle {
        from:       SmtForce,
        to:         SmtForce,
        grouping_mode: SmtGrouping,
        requester:  Subject,        // host_operator | cmdline_boot
        note:       Option<String>, // F3=a Critical 시 명시 메시지
    },
    SmtRuntimeToggle {
        from:       bool,           // SMT enabled before
        to:         bool,           // SMT enabled after
        signal:     ModeSignal,     // dock-detect | battery-low | manual-mode
    },
}
```

anomaly detector rule (S12.6 정합) 추가:
- `SmtPairDesyncAttempt` ≥ 1 / 분 → warning (capsule bug 또는 손상)
- `SmtForceToggle` Critical (F3=a) → 즉시 host operator 통지 (notification
  channel S12.7 / G=g-i 정합)

### S20 — Deep idle 시 lease suspend semantics

**ARCH-II' 매핑:**
- `lease-pm` capsule — 본체.  S16.8 의 trigger (C-state ≥ C3) 발화 시
  lease 의 secret state (XChaCha20 key + HKDF segment key) 를 ISA-별
  best-available secure storage 로 sealing + wake 시 atomic restore +
  integrity check.
- `cpufreq` capsule — C-state 진입 직전 lease-pm 호출 (S16.5 cache flush
  와 짝).
- `wakeup` capsule (S22 본체) — wake source signal 의 spoofing 차단 +
  lease-pm 의 wake hook 발화.
- `audit` capsule — `LeaseSuspend*` op_tag entry.
- power-orchestrator — suspend / wake decision dispatch + force-toggle.

위협: Plundervolt 류의 wake-from-deep-idle 공격, cold-boot variant,
spurious wake 통한 stale state 노출.

#### S20.1 — Suspend trigger 임계 (A=a-i)

**C-state ≥ C3** (S16.8 정합).  C0 / C1 / C2 시 suspend X (얕은 idle
은 hardware power state cutoff 부재 → lease state 보존 자체 안전).

```rust
fn on_cstate_enter(target: CState, lease: LeaseCap) -> Result<(), Y4Error> {
    if target >= CState::C3 && form_factor_allows_suspend() {
        lease_pm_capsule.suspend(lease)?;     // S20.3 atomicity
    }
    Ok(())
}
```

#### S20.2 — 보호된 storage 위치 — ISA-agnostic 4-tier (B=b-i', TQ.1~TQ.9 통합)

```
Tier 1 — CPU hardware-attested secure storage (ISA-별 best-available)
    x86-64 AMD:   PSP-protected SRAM (default),
                   SEV-SNP secure memory (SEV-SNP 호스트),
                   PSP-fTPM (firmware TPM via PSP — Tier 1 sub-case, TQ.6)
    x86-64 Intel: TXT-protected memory (default),
                   TDX secure memory (TDX 호스트, Sapphire Rapids+),
                   ME-fTPM 또는 PTT (Platform Trust Technology) — Tier 1 sub-case
    AArch64:      TrustZone Secure World memory (default, ARMv8-A+),
                   CCA Realm-protected (ARMv9 호스트),
                   TZ-fTPM (firmware TPM via TZ — Tier 1 sub-case)
    POWER:        PEF Secure Memory (Ultravisor-managed, POWER9+)
    IBM Z:        Secure Execution facility (z15+)
    RISC-V:       PMP-protected region + Keystone-style (default, 모든 RV64),
                   CoVE secure memory (CoVE-capable 호스트)

Tier 1.5 — 외장 dTPM 2.0 + XChaCha20-Poly1305 master key seal (TQ.1 신설)
    weak-TEE platform 의 dTPM (discrete TPM 2.0) 보유 케이스:
    - SPARC TPM (일부 Sun/Oracle 서버),
    - 일부 MIPS64 board,
    - older ARM TZ-less SoC + dTPM,
    - dTPM 보유 RISC-V (TPM-over-SPI),
    - x86-64 의 dTPM 보유 server (fTPM 미보유 또는 보안 강화 목적)

    메커니즘 (TQ.1):
    - lease-별 fresh XChaCha20-Poly1305 master key 생성
    - master key 를 TPM 2.0 의 SRK 로 PCR-bound seal (TQ.2)
    - plaintext 는 master key 로 AEAD encrypt + AD = epoch (S20.4.2)
    - TPM session encryption (HMAC + AES-CFB) 의무 활성 (TQ.3) —
      LPC/SPI bus snooping (TPM Genie / logic analyzer) 차단
    - wake 시 TPM unseal → master key 복원 → AEAD decrypt
    - PCR mismatch (measured boot fail) → unseal X → lease revoke

Tier 2 — Encrypted in-memory (universal fallback)
    XChaCha20-Poly1305 AEAD sealed DRAM,
    key = HKDF-Expand(boot-time secret, per-lease nonce).
    모든 ISA 에서 동작.
    Tier 1 + Tier 1.5 부재 또는 detection 실패 시 자동 fallback.

Tier 3 — Suspend 거부 (Tier 1 / 1.5 / 2 모두 부재 시)
    suspend 의 wake-time integrity 보장 0 → S20.1 trigger 발화 자체 X.
    form-factor profile 에서 "no-suspend" 강제.
```

##### S20.2.1 — Capsule abstraction

```rust
trait SecureStorage {
    fn seal(&self, lease_id: LeaseId, plaintext: &[u8])
        -> Result<SealedHandle, Y4Error>;
    fn unseal(&self, handle: SealedHandle)
        -> Result<Vec<u8>, Y4Error>;
    fn integrity_check(&self, handle: SealedHandle)
        -> Result<(), Y4Error>;
    fn tier(&self) -> SecureStorageTier;
}

enum SecureStorageTier {
    Tier1,    // CPU hardware (PSP / TXT / SEV-SNP / TDX / TZ / CCA / PEF / SE / PMP+Keystone / CoVE / fTPM)
    Tier1_5,  // 외장 dTPM 2.0 + AEAD master key seal (TQ.1)
    Tier2,    // pure XChaCha20-Poly1305 sealed DRAM
    Tier3,    // 거부
}
```

부팅 시점에 platform 감지 → 가장 높은 tier 선택.  platform-specific
backend 는 별도 sub-crate:

```
Y4/capsules/pm-lease-pm/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── storage_psp.rs            (x86-64 AMD, Tier 1 + PSP-fTPM sub-case)
│   ├── storage_txt.rs            (x86-64 Intel, Tier 1 + PTT sub-case)
│   ├── storage_sev_snp.rs        (Tier 1)
│   ├── storage_tdx.rs            (Tier 1)
│   ├── storage_tz.rs             (AArch64 TrustZone, Tier 1 + TZ-fTPM sub-case)
│   ├── storage_cca.rs            (AArch64 ARMv9 CCA, Tier 1)
│   ├── storage_pef.rs            (POWER, Tier 1)
│   ├── storage_se.rs             (IBM Z Secure Execution, Tier 1)
│   ├── storage_pmp.rs            (RISC-V PMP+Keystone, Tier 1)
│   ├── storage_cove.rs           (RISC-V CoVE, Tier 1)
│   ├── storage_tpm_aead.rs       (외장 dTPM 2.0 + AEAD, Tier 1.5)  ← 신설
│   └── storage_xchacha.rs        (Tier 2 universal fallback)
```

##### S20.2.2 — Detection 로직

```rust
fn detect_secure_storage_backend() -> Box<dyn SecureStorage> {
    // Priority 1: CPU hardware Tier 1 (fTPM 포함)
    if let Some(tier1) = detect_tier1_hardware() {
        return tier1;
    }

    // Priority 2: 외장 dTPM 2.0 detection
    // ACPI TPM2 table (TCG ACPI Specification) 또는 device tree 의
    // /tpm@... node 또는 TPM CRB / FIFO interface 의 vendor ID
    if let Some(tpm) = detect_dtpm2() {
        let session = setup_encrypted_session(&tpm);  // TQ.3 의무
        let policy = default_pcr_policy();             // TQ.2: PCR 0+1+2+3+7
        return Box::new(TpmAeadStorage::new(tpm, policy, session));
    }

    // Priority 3: pure software Tier 2
    if can_use_dram_xchacha() {
        return Box::new(XchachaDramStorage::new());
    }

    // Tier 3 — suspend 자체 X
    Box::new(NoSuspendStorage)
}
```

##### S20.2.3 — Tier 1.5 의 TPM 2.0 backend (TQ.1, TQ.3, TQ.9)

`tss-esapi` Rust crate (TCG TSS 2.0 ESAPI binding, BSD-2 호환) 를
cargo dependency 로 link.  Y4 의 single-license Apache-2.0 + BSD-2
단방향 호환 (P3.6 §3.3 logicutils 패턴 정합).

```rust
// storage_tpm_aead.rs
use tss_esapi::*;

struct TpmAeadStorage {
    tpm:        EsapiContext,
    pcr_policy: PcrSelectionList,    // TQ.2: PCR 0+1+2+3+7
    session:    AuthSession,         // TQ.3: HMAC + AES-CFB encrypted session
}

impl SecureStorage for TpmAeadStorage {
    fn tier(&self) -> SecureStorageTier { SecureStorageTier::Tier1_5 }

    fn seal(&self, lease_id: LeaseId, plaintext: &[u8]) -> Result<SealedHandle, _> {
        let master_key: [u8; 32] = generate_xchacha20_key();   // 256-bit
        let nonce_aead: [u8; 24] = fresh_nonce();              // 192-bit XChaCha20

        // 1. TPM 에 master key seal (PCR-bound + encrypted session)
        let tpm_sealed_key = self.tpm.create_sealed_object(
            &self.session,
            &master_key,
            &self.pcr_policy,
        )?;

        // 2. plaintext 를 master key 로 XChaCha20-Poly1305 AEAD
        let aead_blob = aead_encrypt(plaintext, &master_key, &nonce_aead,
                                     additional_data: &epoch.to_le_bytes());

        // 3. master key 를 메모리에서 secure zeroize
        secure_zero(&mut master_key);

        Ok(SealedHandle {
            lease_id,
            tier: SecureStorageTier::Tier1_5,
            tpm_sealed_key,
            aead_blob,
            nonce: nonce_aead,
            epoch,
        })
    }

    fn unseal(&self, handle: SealedHandle) -> Result<Vec<u8>, _> {
        // 1. TPM 에서 master key unseal — 현 PCR 가 seal 시점 PCR 와 매치 시에만
        let master_key = self.tpm.unseal(&handle.tpm_sealed_key, &self.session)
            .map_err(|_| Y4Error::SecurityViolation)?;   // PCR mismatch → fail
        // 2. AEAD decrypt + Poly1305 tag 검증 + AD epoch 검증 (S20.4)
        aead_decrypt(handle.aead_blob, &master_key, ...)
    }
}
```

##### S20.2.4 — PCR binding 정책 (TQ.2)

default PCR set: **PCR 0 + 1 + 2 + 3 + 7** — measured boot 의 핵심 PCR.

| PCR | 측정 대상 |
|---|---|
| 0 | BIOS / UEFI firmware code |
| 1 | BIOS / UEFI configuration + platform 설정 |
| 2 | Option ROM code (PCIe option ROM 등) |
| 3 | Option ROM configuration |
| 7 | Secure Boot policy (PK / KEK / db / dbx) |

근거: PCR 0/1/2/3/7 는 platform integrity 의 핵심.  4/5/6/8+ 는 OS-level
변동 (kernel update / module load 등) 으로 자주 변경되어 PCR mismatch
유발 → unseal 깨짐 → 정상 OS update 도 lease revoke 발화.

cmdline override: `y4.pm.tpm_pcr_policy=<list>` (host operator only,
expert mode).

##### S20.2.5 — TPM 의무 / 권장 / off (TQ.4)

```
y4.pm.tpm_required=auto|on|off    cmdline, default = auto

auto  — TPM detection 시 Tier 1.5 활용, 부재 시 Tier 2 fallback
on    — TPM 부재 시 boot fail (certified profile, host operator 강제)
off   — TPM detection 무시, Tier 2 fallback (testing / bypass)
```

per-form-factor `tools/power.rules` (TQ.5):

| Form factor | `Y4_PM_TPM_REQUIRED` |
|---|---|
| **server-farm** | **on** (compliance, multi-tenant 신뢰 root 의무) |
| **rack-mount** | **on** |
| laptop | auto (대부분 fTPM 또는 dTPM 보유 — Tier 1 또는 Tier 1.5 자동) |
| handheld + 독 | auto (Tier 1 의 TZ/SEP 우선, TPM 부재 OK) |
| SoC | auto (platform 의존) |
| **certified profile** (S10.2 와 짝) | **on** (의료 / 항공 / 금융 트랙) |

##### S20.2.6 — fTPM vs dTPM 분리 (TQ.6)

| 종류 | Tier | 근거 |
|---|---|---|
| **fTPM** (PSP-fTPM, ME-fTPM/PTT, TZ-fTPM) | **Tier 1 sub-case** | CPU 자체 hardware root + secure-world 안 |
| **dTPM** (discrete TPM 2.0 chip on motherboard) | **Tier 1.5** | 외부 chip + bus 의존 (LPC/SPI), bus snooping 가능성 (단 TQ.3 의 session encryption 으로 차단) |

detection 우선순위: Tier 1 (fTPM 포함) → Tier 1.5 (dTPM) → Tier 2 →
Tier 3.

##### S20.2.7 — TPM detection / mismatch audit (TQ.7)

S12.2 schema 에 v1.x patch 로 op_tag 3 개 추가:

```rust
enum OpTag {
    // 기존 ...
    TpmDetected,        // 부팅 시 1 회 (severity Info)
    TpmAbsent,          // 부팅 시 1 회, fallback Tier 명시 (severity Info)
    TpmPcrMismatch,     // measured boot fail — wake unseal X (severity Critical)
    // ...
}
```

`TpmPcrMismatch` 1 회 발생 → **즉시 모든 lease revoke** (S14 firmware-
approval revoke chain 정합 — measured boot 변조 신호, host integrity 심대).

##### S20.2.8 — Verus invariant `tpm_pcr_consistency` (TQ.8, AV21 후보)

power_safety.md §4 의 Verus invariant catalog (AV21~AV30+) 에 추가:

```
AV21 (Lower) tpm_pcr_consistency(handle):
    forall handle: SealedHandle,
        tier(handle) == Tier1_5 ==>
            (unseal_succeeds(handle) ==>
                pcr_at_unseal() == pcr_bound_at_seal(handle))
```

statement: Tier 1.5 backend 사용 시 unseal 시점의 PCR 가 seal 시점 PCR
와 동일 (PCR mismatch 면 unseal 실패).  power_arch.md §6 의 AV21 entry,
proof file = `proofs/verus/src/power/lower/tpm_consistency.rs`.

#### S20.2.9 — Endian 정합

POWER / SPARC / AArch64 의 endian 무관 지원 — Verus invariant statement
가 byte-order 의존 X (모든 spec 이 abstract integer 사용, P3.5 §2.1
width-agnostic 패턴).  실제 backend 구현 시 endian 각자 처리.

TPM 2.0 wire protocol 은 big-endian 고정 (TCG specification) — `tss-esapi`
crate 가 endian 변환 책임.  Y4 측 spec 영향 0.

#### S20.3 — Suspend atomicity (C=c-i, S13.2 패턴)

S13.2 mid-vmrun race resolve 패턴 재사용:

```
1. lease-pm capsule 에 suspend 요청 도착
2. 모든 vCPU 의 vmrun 상태 확인
   - vmrun 중 vCPU 존재 시: suspend 보류, retry timer (1 ms) 등록
   - 모두 vmexit 상태 시: 진행
3. cpufreq capsule 의 cache flush (S16.5 의 partial TLB + L1D)
4. lease 의 모든 segment key + audit key 를 SecureStorage::seal
5. 메모리상의 plaintext key 를 secure zeroize
6. C-state 하드웨어 진입
```

retry 임계 = `Y4_PM_SUSPEND_RETRY_MAX` (default 10 회 = 10 ms 한도).
초과 시 suspend abort + audit `LeaseSuspendDeferred` (severity Info).
DoS 차단 — 무한 retry 0.

#### S20.4 — Wake 시 integrity check (D=d-i, 정정 2026-05-05)

**Primitive: XChaCha20-Poly1305 AEAD + replay protection.**

원안 HMAC-SHA256 → XChaCha20-Poly1305 AEAD 로 정정.  근거:
1. **Y4 정합** — S12.5 audit ring + S13.9 sibling segment + WaveTensor
   HIU (192-bit nonce / 256-bit key) 와 단일 primitive (XChaCha20 family)
   통일.  cryptographic 표면 ↑ 0
2. **Single AEAD** — encrypt + MAC 이 하나의 key + 하나의 nonce + 하나의
   tag (RFC 7539 / IETF draft XChaCha20-Poly1305)
3. **ISA-agnostic uniform** — Bernstein ARX design 이 SIMD software 에서
   일관 성능 + cache-timing pitfall 0 (모든 ISA 에서 constant-time)
4. **Tier 분리 책임** — Tier 1 의 hardware-attested storage 는 자체
   integrity 제공 (SEV-SNP RMP / TDX MKTME-i / SGX MAC tree / TZ secure
   memory / CCA Realm tracking / PEF Ultravisor-managed pages / IBM Z
   Secure Execution / TPM PCR-seal) → 추가 software MAC redundant.
   Tier 2 만 XChaCha20-Poly1305 의 Poly1305 MAC 사용

##### S20.4.1 — Tier-별 integrity 책임

```rust
fn wake_lease(handle: SealedHandle) -> Result<(), Y4Error> {
    let storage = lease_pm_capsule.storage_backend();
    let secrets = match storage.tier() {
        // Tier 1: hardware-attested integrity 사용
        // (PSP / TXT / SEV-SNP / TDX / TZ / CCA / PEF / SE / TPM-seal /
        //  PMP+Keystone / CoVE) — 추가 software MAC X
        SecureStorageTier::Tier1 => storage.unseal_with_hardware_integrity(handle),

        // Tier 2: XChaCha20-Poly1305 AEAD — Poly1305 tag 검증
        // (universal fallback, 모든 ISA)
        SecureStorageTier::Tier2 => storage.unseal_with_aead(handle),

        // Tier 3: suspend 자체 X — wake 호출 0
        SecureStorageTier::Tier3 => unreachable!(),
    }
    .map_err(|_| {
        audit.append(LeaseWakeIntegrityFail {
            lease_id: handle.lease_id,
            tier:     storage.tier(),
            reason:   IntegrityFailReason::TagMismatch,
        });
        lifecycle.revoke_lease(handle.lease_id);    // S13 lease revoke
        Y4Error::SecurityViolation
    })?;

    // S20.4.2: replay protection check
    if !replay_check(&secrets, handle)? {
        audit.append(LeaseWakeIntegrityFail {
            lease_id: handle.lease_id,
            tier:     storage.tier(),
            reason:   IntegrityFailReason::EpochReplay,
        });
        lifecycle.revoke_lease(handle.lease_id);
        return Err(Y4Error::SecurityViolation);
    }

    restore_lease_secrets(secrets);
    Ok(())
}
```

##### S20.4.2 — Replay protection (별도 추가)

위협: attacker 가 suspend 시점의 sealed blob capture → 후속 wake 시점에
stale blob substitute → lease state rollback (이전 secret state 복원).

차단: sealed blob 안에 **monotonic epoch counter** (lease-별, suspend
횟수) 포함 + wake 시점에 expected epoch 비교.

```rust
struct SealedHandle {
    lease_id:      LeaseId,
    epoch:         u64,           // monotonic counter, lease 별 (suspend 횟수)
    ciphertext:    Vec<u8>,       // XChaCha20-Poly1305 AEAD ciphertext + tag
    nonce:         [u8; 24],      // 192-bit XChaCha20 nonce (Y4 표준)
}

// lease-pm capsule 의 internal state (per-lease)
struct LeasePmState {
    expected_wake_epoch: u64,     // 다음 wake 의 기대 epoch
    // ... 기존 필드
}

fn suspend_lease(lease: LeaseCap) -> Result<SealedHandle, Y4Error> {
    let state = lease_pm_state(lease.id);
    state.expected_wake_epoch += 1;
    let handle = SealedHandle {
        lease_id:   lease.id,
        epoch:      state.expected_wake_epoch,
        nonce:      generate_nonce(),
        ciphertext: storage.seal_with_aead(
            lease_secrets(lease),
            additional_data: &state.expected_wake_epoch.to_le_bytes(),  // AEAD AD
        )?,
    };
    Ok(handle)
}

fn replay_check(secrets: &[u8], handle: SealedHandle) -> Result<bool, Y4Error> {
    let state = lease_pm_state(handle.lease_id);
    if handle.epoch != state.expected_wake_epoch {
        return Ok(false);   // replay 또는 stale handle
    }
    Ok(true)
}
```

핵심:
- **AEAD additional data** 에 epoch 포함 → ciphertext tampering + AD
  tampering 모두 Poly1305 tag 검증으로 차단
- **expected_wake_epoch 는 lease-pm capsule 의 internal monotonic
  counter** — sealed blob 외부에 보관, attacker 가 capture 0
- suspend 마다 epoch 증가 (overflow 시 lease revoke — u64 overflow 는
  사실상 불가능 + 안전 fallback)
- wake 시 `handle.epoch == expected_wake_epoch` 검증 — 1 회 wake 만
  허용, replay 0

##### S20.4.3 — Failure 처리

integrity 또는 replay check fail = lease 의 secret state 변조 / 재생
가능성 (Plundervolt / cold-boot variant / replay attack) → **즉시
lease revoke + audit Critical**.

```rust
enum IntegrityFailReason {
    TagMismatch,    // AEAD Poly1305 tag 불일치
    EpochReplay,    // S20.4.2 replay 검출
    HardwareDeny,   // Tier 1 hardware-attested integrity fail
}
```

audit op_tag `LeaseWakeIntegrityFail` 의 payload 에 `reason` 추가 —
forensic 가치 ↑.

#### S20.5 — Latency budget (E=e-i)

```
Y4_PM_SUSPEND_LATENCY_NS    build-time const, default 10_000_000 (10 ms)
Y4_PM_WAKE_LATENCY_NS       build-time const, default  5_000_000  (5 ms)
                             cmdline `y4.pm.suspend_latency_ns` /
                                      `y4.pm.wake_latency_ns` override
```

power-orchestrator 가 budget 초과 감지 시:

| operation | 초과 동작 |
|---|---|
| **suspend** | abort + lease revoke (S13) + audit `LeaseSuspendTimeout` (Critical) — DoS 차단 |
| **wake** | integrity check 강제 진행 (실패 시 S20.4 처리), 그러나 audit `LeaseWakeTimeout` (Warning) |

#### S20.6 — Cross-VM suspend 동시성 (F=f-i)

**Lock-free 동시 multiple lease suspend OK.**  근거:
- 각 lease 의 segment key 독립 (S13.9 HKDF-Expand 의 lease-별 master
  key)
- audit ring buffer / lifecycle capsule 의 sink-only 그래프 (vmm_arch
  §2.4) 가 cross-cluster race 자연 차단
- Tier 1 hardware storage 의 atomicity 는 hardware 가 보장 (PSP/TXT/
  TZ/PEF/SE 등 모두 multi-context 안전)
- Tier 2 XChaCha20 도 nonce 독립 → race 0

global lock 없이도 lease 사이 격리 유지.

#### S20.7 — Form-factor 별 suspend 정책 (G=g-i)

`tools/power.rules`:

| Form factor | suspend 기본 | 근거 |
|---|---|---|
| server-farm | **OFF** (강제) | latency tail 차단 |
| rack-mount | OFF | 동일 |
| **laptop** | **ON** (default) | battery 절약 |
| **handheld + 독** | battery 모드 ON / 독 모드 OFF | dual-mode (power_arch.md §3.2) |
| SoC | duty-cycle 의 idle 구간만 ON | platform 별 |

cmdline `y4.pm.deep_idle_lease_suspend=on|off` override (host operator).

#### S20.8 — Wake source 정합 (H=h-i)

S22 (wakeup capsule) 의 wake source 도착 → spoofing 차단 → power-
orchestrator → lease-pm wake hook:

```
wake_signal (interrupt / GPE / device wake)
    → wakeup capsule (S22): spoofing 검증 + 정당성 검사
    → power-orchestrator: form-factor profile 에 따른 wake 결정
    → lease-pm capsule: wake_lease(sealed_handle) — S20.4 integrity check
        ├── pass: secrets restore + vCPU resume
        └── fail: lease revoke (S13)
    → audit LeaseWakeStart / LeaseWakeIntegrityFail
```

spoofing 차단 본체는 S22, lease-pm 은 wake hook 만.

#### S20.9 — 사용자 force toggle (I=i-i, S19 패턴 정합)

KDE Plasma 의 "screen lock / sleep inhibit" 패턴 재사용:

```rust
enum SuspendForce {
    ForceOn,        // 무조건 suspend (battery 절약 강제, laptop close-lid)
    FollowPolicy,   // form-factor profile + cmdline + L2 mode signal (default)
    ForceOff,       // 무조건 suspend X (게임 / 발표 inhibit, always-on)
}

const Y4_PM_SUSPEND_FORCE_DEFAULT: SuspendForce = SuspendForce::FollowPolicy;
```

설정 채널:
- cmdline `y4.pm.suspend_force=on|off|policy`
- runtime CLI:
  ```
  y4-hypercall power suspend-force on
  y4-hypercall power suspend-force off
  y4-hypercall power suspend-force policy
  y4-hypercall power suspend-status   # 현 force 상태 + 실 suspend 활성 + 마지막 wake 시각
  ```

권한: **host operator only** (S19.6.1 F1=i 패턴).
persistence: cmdline default + runtime override (S19.6.1 F2=ii 패턴).
audit: `LeaseSuspendForceToggle` (severity Warning).

#### S20.10 — Audit (S12 정합) (J=j-i)

S12.2 schema 에 v1.x patch 로 op_tag 추가:

```rust
enum OpTag {
    // 기존 ...
    LeaseSuspendStart,         // S20.3 — suspend 시작 (severity Info)
    LeaseSuspendComplete,      // S20.3 — suspend 완료 (severity Info)
    LeaseSuspendDeferred,      // S20.3 retry 임계 도달 (severity Info)
    LeaseSuspendTimeout,       // S20.5 — suspend latency 초과 (severity Critical)
    LeaseWakeStart,            // S20.4 — wake 시작 (severity Info)
    LeaseWakeIntegrityFail,    // S20.4 — HMAC fail (severity Critical)
    LeaseWakeTimeout,          // S20.5 — wake latency 초과 (severity Warning)
    LeaseSuspendForceToggle,   // S20.9 — force-toggle 변경 (severity Warning)
    // ...
}
```

anomaly detector rule (S12.6 정합):
- `LeaseSuspendTimeout` 1 회 → host operator 즉시 통지
- `LeaseWakeIntegrityFail` 1 회 → 즉시 모든 lease revoke + Critical
  (S14 firmware-approval revoke chain 정합 — 시스템 변조 신호)
- `LeaseSuspendDeferred` ≥ 100 / 분 → warning (deep idle 진입 자체가
  반복 fail 하는 환경 — vmrun 무한 발화 등)

### S21 — Thermal throttle 정책 (_TCC / _TJMAX)

**ARCH-II' 매핑:**
- `acpi-pm` capsule — _TCC / _PSV mediation (S18.5 정합) + virtio-thermal
  paravirt server-side + per-VM virtual thermal zone.
- `cpufreq` capsule — thermal threshold 도달 시 P-state cap 적용 + thermal
  hysteresis tracking.
- `msr-bitmap` capsule — thermal MSR access 차단 + read 마스킹.
- `cpuid-emul` capsule — thermal/Turbo/HDC feature bit 마스킹.
- `audit` capsule — `Thermal*` op_tag entry.
- `lifecycle` capsule (cross-cluster) — hardlimit 도달 시 vCPU pause +
  lease throttle.
- power-orchestrator — thermal mode signal hub (S19.6 L2 와 합류).

위협:
- guest 가 thermal MSR write 로 host thermal 한도 변경 → hardware damage
  / DoS / Plundervolt-style under-volt 우회
- guest 가 _PSV 무력화로 thermal throttle 회피 → host CPU 과열
- thermal sensor read 가 cross-tenant workload fingerprinting

#### S21.1 — Thermal MSR write 차단 (A=a-i)

`msr-bitmap` capsule 의 S10.1 mandatory entry 갱신 (v1.x patch):

| MSR | Vendor | 권한 |
|---|---|---|
| `IA32_THERM_CONTROL (0x19A)` | Intel | guest read+write 차단 |
| `IA32_THERM_INTERRUPT (0x19B)` | Intel | guest read+write 차단 |
| `IA32_PACKAGE_THERM_INTERRUPT (0x1B2)` | Intel | guest read+write 차단 |
| `MSR_TEMPERATURE_TARGET (0x1A2)` | Intel | guest read+write 차단 (TJMAX 변경) |
| `MSR_THERMAL_DIODE_OFFSET (0xC0010055)` | AMD | guest read+write 차단 |
| `MSR_HWCR (0xC0010015)` | AMD | guest write 차단 (thermal-related bits 포함) |
| `MSR_RAPL_POWER_LIMIT (0x610)` | Intel | (S17.1 의 RAPL 차단과 중복, thermal interaction) |

write 시도 → S10.4 default-deny audit (op_tag = `MsrAccessDenied` —
S15 Pthermal MSR 의 super-set) + guest 에 #GP(0).

#### S21.2 — Thermal MSR read 마스킹 (B=b-i)

read 허용 + LSB 마스킹 — guest 가 host 온도 abstract 만 보고, fine-
grained workload fingerprinting 차단:

```
Y4_PM_THERMAL_NOISE_BITS    build-time const, default 3
                              cmdline `y4.pm.thermal_noise_bits=N` (0 ≤ N ≤ 6)
```

| MSR | read 처리 |
|---|---|
| `IA32_THERM_STATUS (0x19C)` | LSB 3 bit AND-mask (8°C quantization) |
| `IA32_PACKAGE_THERM_STATUS (0x1B1)` | LSB 3 bit AND-mask |
| `MSR_TEMPERATURE_TARGET (0x1A2)` read | TJMAX 자체는 host operator 정책 → fixed value 마스킹 (host 의 실제 TJMAX 노출 X) |

근거:
- 3 bit ≈ 8°C resolution → workload signature 차단 (대부분의 fingerprinting
  attack 은 sub-1°C 분해능 의존)
- 6 bit (max) ≈ 64°C resolution → fingerprinting 사실상 불가, 단 thermal
  throttle 정확도 ↓ (Linux thermal driver 호환성 risk)
- v1.0 default 3 bit = balance

#### S21.3 — virtio-thermal paravirt interface (C=c-i)

acpi-pm capsule 의 server-side virtio-thermal (S18.3 의 per-VM virtual
DSDT 정합):

```rust
// guest 가 보는 thermal zone:
struct VirtioThermalZone {
    name:           "Y4 Virtual Thermal Zone",
    current_temp:   u32,    // S21.2 마스킹된 host temp 또는 fixed (form-factor 별)
    psv_trip_c:     u32,    // host 의 _PSV (form-factor profile 격리 복사본, S21.5)
    tcc_trip_c:     u32,    // host 의 _TCC 격리 복사본, S21.4
    hot_trip_c:     u32,    // hardlimit (S21.8) 의 격리 복사본
    crit_trip_c:    u32,    // 동일
    cooling_devices: Vec<VirtioCoolingDevice>,    // virtio-cpufreq 의 cooling state
}
```

guest 의 Linux `thermal-zone` driver 가 본 paravirt interface 와 통신.
host 의 실제 thermal 직접 노출 X.

#### S21.4 — _TCC 한도 변경 권한 (D=d-i, S18.5 정합)

```
Y4_PM_THERMAL_TCC_C    build-time const, per-form-factor (S21.10)
                        cmdline `y4.pm.thermal_tcc_c=...` override
                        runtime CLI `y4-hypercall power thermal-set --tcc <C>`
```

권한: **host operator only.**  guest 의 _TCC write attempt → S18.5 의
acpi-pm mediation 가 reject + audit Warning (`AcpiMethodRejected`).

#### S21.5 — _PSV 한도 변경 권한 (E=e-i, S18.5 정합)

```
Y4_PM_THERMAL_PSV_C    build-time const, per-form-factor (S21.10)
                        cmdline `y4.pm.thermal_psv_c=...` override
                        runtime CLI `y4-hypercall power thermal-set --psv <C>`
```

권한: **host operator only.**  guest 의 _PSV write attempt → reject +
audit Warning.

S18.5 의 본문이 본 sub-decision 의 base — S18.5 의 form-factor 표가
S21.10 으로 일치되어 갱신.

#### S21.6 — Thermal hysteresis (F=f-i)

```
Y4_PM_THERMAL_HYSTERESIS_C    build-time const, default 5
                                cmdline `y4.pm.thermal_hysteresis_c=N`
```

cpufreq capsule 의 thermal state machine:

```
state Cool:
    if temp >= PSV: → Throttling
state Throttling:
    if temp <= (PSV - HYSTERESIS): → Cool
    if temp >= TCC: → HardThrottling
state HardThrottling:
    if temp <= (TCC - HYSTERESIS): → Throttling
    if temp >= HARDLIMIT: → Emergency  (S21.8)
state Emergency: ...  (S21.8 의 vCPU pause)
```

근거: hysteresis 0 시 thermal 임계 근처에서 throttle 진입/이탈 oscillation
→ DVFS storm + audit volume 폭증.  5°C hysteresis 가 표준 (Linux thermal
governor default 와 동일).

#### S21.7 — Thermal mode signal — S19.6 L2 합류 (G=g-i)

thermal threshold 도달 시 power-orchestrator 가 thermal mode signal 발화:

```
state Throttling 진입:
    → power-orchestrator.thermal_mode_signal(ThrottleActive)
        → S19.6 L2 mode signal 합류 (S19.6.3 표 와 짝):
            * SMT toggle (form-factor 의 follow-policy 시 off 적극)
            * cpufreq capsule: P-state cap 적용 (max P-state 강제)
            * cstate: deep idle 진입 적극 (max C-state 활용)
        → audit ThermalThresholdReached (Warning)

state Cool 복귀:
    → power-orchestrator.thermal_mode_signal(ThrottleClear)
        → 자동 해제 (이전 form-factor profile 복귀)
        → audit ThermalCoolingComplete (Info)
```

S19 force-toggle 의 ForceOn 시 SMT 그대로 — thermal mode signal 은
mask 됨 (사용자 의지 우선, 단 thermal hardlimit 도달 시에만 force 무시
S21.8).

#### S21.8 — Hardware damage 방어 hardlimit (H=h-i)

```
Y4_PM_THERMAL_HARDLIMIT_C    build-time const, per-form-factor
                              vendor-별 TJMAX 또는 TJMAX-5°C (default)
                              cmdline `y4.pm.thermal_hardlimit_c=...` override
                              runtime CLI `y4-hypercall power thermal-set --hardlimit <C>`
```

도달 시 power-orchestrator 의 emergency response:

```rust
fn on_thermal_hardlimit(temp: u32) -> ! {
    // S21.8 emergency
    audit.append(ThermalHardlimitReached { temp, ... });   // Critical

    // 1. 모든 vCPU 즉시 pause (lifecycle capsule cross-cluster)
    lifecycle.pause_all_vcpus();

    // 2. 모든 lease 의 vmrun 거부 (lease throttle)
    for lease in active_leases() {
        lease.vmrun_throttled = true;
    }

    // 3. host operator emergency notification (S12.7 push channel)
    notify_host_operator_emergency(EmergencyKind::ThermalHardlimit, temp);

    // 4. cpufreq capsule: 모든 CPU lowest P-state + 모든 가능한 C-state 진입
    cpufreq.emergency_lowest_pstate();

    // 5. 사용자 acknowledge 까지 대기 (CLI 의 `power thermal-resume` 또는 reboot)
    wait_for_thermal_clearance();
}
```

근거:
- hardware 자체 thermal trip 은 last resort (TCC throttle / TJMAX 자동
  pause / catastrophic shutdown)
- software-side hardlimit 은 그 보다 5°C 마진 — 정상 동작 안에서 detect
  + graceful pause + lease throttle
- S19 ForceOn / S20 ForceOff 모두 무시 (사용자 force 보다 hardware 보호
  우선).  단 thermal_force = aggressive (S21.9 J=j-i) 도 hardlimit 까지만

#### S21.9 — 사용자 force toggle (J=j-i, S19.6 / S20.9 패턴)

KDE Plasma 패턴 정합:

```rust
enum ThermalForce {
    Conservative,    // thermal threshold 강하게: PSV/TCC 의 -10°C 적용 (longer battery / safer)
    FollowPolicy,    // form-factor profile + cmdline + L2 mode signal (default)
    Aggressive,      // PSV/TCC 의 +10°C 완화 (gaming / 발표 모드).  단 hardlimit 까지만
}

const Y4_PM_THERMAL_FORCE_DEFAULT: ThermalForce = ThermalForce::FollowPolicy;
```

설정 채널:
- cmdline `y4.pm.thermal_force=conservative|policy|aggressive`
- runtime CLI:
  ```
  y4-hypercall power thermal-force conservative
  y4-hypercall power thermal-force policy
  y4-hypercall power thermal-force aggressive
  y4-hypercall power thermal-status   # 현 force + 실 PSV/TCC + 현재 temp
  ```

권한: **host operator only** (F1 패턴 정합).
persistence: cmdline default + runtime override (F2 패턴 정합).
audit: `ThermalForceToggle` (severity Warning).

aggressive 의 hardlimit 보호: aggressive 적용으로 PSV/TCC 가 +10°C 되어도
hardlimit (S21.8) 는 그대로 — hardware damage 는 절대 보호.

#### S21.10 — Form-factor 별 thermal profile (I=i-i, logicutils `tools/power.rules`)

S18.5 의 form-factor 표 갱신 — TCC + PSV + hardlimit + thermal_noise_bits
+ thermal_force_default 까지 통합:

| Form factor | TCC | PSV | hardlimit | noise_bits | force default |
|---|---|---|---|---|---|
| **server-farm** | 90°C | 80°C | 95°C | 4 (보수적) | conservative |
| **rack-mount** | 95°C | 85°C | 100°C | 3 (default) | policy |
| **laptop** | 90°C | 75°C | 95°C | 3 | policy |
| **handheld + 독 (battery 모드)** | 85°C | 70°C | 90°C | 3 | conservative |
| **handheld + 독 (독 모드)** | 95°C | 85°C | 100°C | 3 | policy |
| **SoC** | platform 별 (cmdline 의무) | platform 별 | platform 별 | platform 별 | platform 별 |
| **certified profile** | conservative 강제 | conservative 강제 | TJMAX-5°C | 4 | conservative (cmdline 변경 X) |

#### S21.11 — CPUID thermal feature 마스킹 (K=k-i)

`cpuid-emul` capsule 의 CPUID emulation table 갱신 (S15.6 / S17.9 와 짝):

| Leaf | 비트 | 마스킹 |
|---|---|---|
| `CPUID 0x6.EAX[0]` | Digital Thermal Sensor | 0 강제 (host thermal 노출 차단) |
| `CPUID 0x6.EAX[1]` | Turbo Boost | 0 (S15.6 constant-freq 짝) |
| `CPUID 0x6.EAX[5]` | PLN (Power Limit Notification) | 0 |
| `CPUID 0x6.EAX[6]` | ECMD (Extended Clock Modulation Duty) | 0 |
| `CPUID 0x6.EAX[13]` | HDC (Hardware Duty Cycle) | 0 |
| `CPUID 0x6.EBX[0:3]` | Number of Interrupt Thresholds in DTS | 0 |
| `CPUID 0x6.ECX[0]` | Hardware Coordination Feedback Capability | 0 |

guest 는 thermal sensor / Turbo / HDC / PLN feature 부재로 인식 →
Linux thermal driver 자동 fallback (또는 virtio-thermal 사용).

#### S21.12 — Audit (S12 정합) (L=l-i)

S12.2 schema 에 v1.x patch 로 op_tag 5 개 추가:

```rust
enum OpTag {
    // 기존 ...
    ThermalThresholdReached,    // S21.7 — _PSV / _TCC 도달 (severity Warning)
    ThermalHardlimitReached,    // S21.8 — hardlimit 도달, 즉시 vCPU pause (severity Critical)
    ThermalCoolingComplete,     // S21.7 — Cool 상태 복귀 (severity Info)
    ThermalConfigChange,        // host operator 의 _PSV/_TCC/hardlimit 변경 (severity Info)
    ThermalForceToggle,         // S21.9 — force-toggle 변경 (severity Warning)
    ThermalMsrAccessDenied,     // S21.1 — guest write 차단 (severity Trace, volume ↑)
    // ...
}
```

anomaly detector rule (S12.6 정합):
- `ThermalHardlimitReached` 1 회 → host operator 즉시 emergency 통지 +
  hardware-protection escalation
- `ThermalThresholdReached` ≥ 100 / 분 → warning (지속 thermal stress —
  cooling 시스템 점검 권고)
- `ThermalMsrAccessDenied` ≥ 1000 / 분 → warning (guest 의 반복 시도,
  potential exploit 시도)

### S22 — Wake source routing 격리

**ARCH-II' 매핑:**
- `wakeup` capsule — 본체.  wake source enable + 화이트리스트 + lease
  binding + spurious wake detection + priority arbitration + magic packet
  signature 검증.
- `lease-pm` capsule — S20.8 의 wake hook (wakeup capsule signal 받아
  integrity check + restore).
- `acpi-pm` capsule — _PRW (Power Resources for Wake) eval mediation
  (S18.2 정합) + virtio-pm 의 wake source 관리.
- `audit` capsule — `Wake*` op_tag entry.
- power-orchestrator — wake routing dispatch + form-factor profile 적용.

위협:
- guest / 외부 attacker 의 spurious wake interrupt → DoS, battery 형상
  critical (battery 빠른 소진)
- guest 의 wake source spoofing → cross-tenant wake 강제
- wake source priority / mask 변경 → host operator 의도 우회

#### S22.1 — Wake source enable 권한 (A=a-i)

**host operator + power-orchestrator only.**  guest 의 wake source
enable 0 — guest 의 ACPI `_PRW` (Power Resources for Wake) write
attempt 는 S18.2 의 mediation 가 reject + audit Warning
(`AcpiMethodRejected`).

| 주체 | 권한 |
|---|---|
| host operator | runtime CLI `y4-hypercall power wake-source <add|remove|list>` |
| power-orchestrator (internal) | follow-policy 모드의 자동 결정 (form-factor profile 적용) |
| lease holder | own VM scope 의 virtio-pm wake source (`virtio-rtc` alarm 등) 만 등록 가능, host hardware wake 0 |
| guest | 0 |

#### S22.2 — Form-factor 별 화이트리스트 (B=b-i, default-deny)

`tools/power.rules` 의 form-factor 정책:

```
[server-farm]
Y4_PM_WAKE_SOURCES = "nic-magic-packet, ipmi-bmc, rtc-alarm"

[rack-mount]
Y4_PM_WAKE_SOURCES = "nic-magic-packet, ipmi-bmc, rtc-alarm"

[laptop]
Y4_PM_WAKE_SOURCES = "lid-open, power-button, nic-magic-packet, usb-device,
                      rtc-alarm, battery-threshold"

[handheld]
Y4_PM_WAKE_SOURCES = "dock-detect, power-button, volume-button, mode-switch,
                      rtc-alarm, battery-threshold, network"

[handheld-dock-mode]
Y4_PM_WAKE_SOURCES = "lid-open, power-button, dock-detect, nic-magic-packet,
                      usb-device, rtc-alarm"

[soc]
Y4_PM_WAKE_SOURCES = "platform-dependent"     # GPIO / RTC 가 일반
```

화이트리스트 외 wake event 는 spurious wake 로 분류 (S22.4).

#### S22.3 — Wake source 의 lease binding (C=c-i)

`LeaseCap` struct 갱신:

```rust
struct LeaseCap {
    // 기존 ...
    wake_sources: Vec<WakeSource>,    // 본 lease 가 받을 수 있는 wake source 목록
}

enum WakeSource {
    NicMagicPacket {
        mac:        MacAddr,           // own VM 의 virtual MAC
        vlan_tag:   Option<u16>,
        signing_key: Aes256Key,        // F=f-i.b 의 cryptographic key
    },
    UsbDevice {
        vid_pid:    Option<(u16, u16)>,    // None = any USB
        device_class: Option<u8>,
    },
    RtcAlarm {
        absolute_ts: u64,              // virtio-rtc 의 alarm 시각
    },
    LidOpen,
    PowerButton,
    DockDetect,
    BatteryThreshold {
        below_percent: u8,
    },
    // ...
}
```

multi-tenant 환경:
- NIC magic packet 도착 → wakeup capsule 의 packet matcher 가 destination
  MAC + VLAN 조합으로 lease 식별 → 해당 lease 만 wake
- power button → 모든 lease wake X (host operator 의 routing 정책 적용,
  S22.8)
- RTC alarm → alarm 등록한 lease 만 wake

#### S22.4 — Spurious wake detection (D=d-i)

wake event 도착 시 wakeup capsule 의 wake-cause register 검증:

```rust
fn on_wake_event(event: WakeEvent) -> Result<(), Y4Error> {
    // 1. 화이트리스트 검증 (S22.2)
    if !form_factor_whitelist().contains(&event.source) {
        audit.append(SpuriousWakeDetected {
            event,
            reason: "not in whitelist",
        });
        cpufreq.reenter_deepest_cstate();    // 즉시 deep idle 재진입
        return Err(Y4Error::SpuriousWake);
    }

    // 2. lease binding 검증 (S22.3)
    let target_lease = match_event_to_lease(&event)?;
    if target_lease.is_none() {
        audit.append(SpuriousWakeDetected {
            event,
            reason: "no matching lease binding",
        });
        cpufreq.reenter_deepest_cstate();
        return Err(Y4Error::SpuriousWake);
    }

    // 3. magic packet 의 cryptographic signature 검증 (S22.6)
    if let WakeSource::NicMagicPacket { signing_key, .. } = &event.source {
        verify_magic_packet_signature(&event, signing_key)?;
    }

    // 4. 정상 wake — power-orchestrator 에 forward
    power_orchestrator.dispatch_wake(event, target_lease);
    audit.append(WakeRouting { lease: target_lease.id, event });
    Ok(())
}
```

spurious wake 의 영향:
- 즉시 deep idle 재진입 → battery 손실 최소화 (battery 형상 critical)
- audit Warning → anomaly detector 가 빈도 추적

#### S22.5 — Wake source priority (E=e-i)

build-time priority list (logicutils 통합):

| Priority | 종류 |
|---|---|
| 0 (highest) | **Emergency** — thermal hardlimit (S21.8) |
| 1 | host operator command (CLI 의 `power wake-now`) |
| 2 | **Critical wake** — power button, lid open, dock detect, mode switch |
| 3 | **Network wake** — NIC magic packet (verified, S22.6) |
| 4 | **Device wake** — GPE, USB device wake |
| 5 (lowest) | **Scheduled** — RTC alarm, battery threshold |

동일 priority 내에서는 FIFO.  power-orchestrator 의 wake queue 가
priority-aware:

```rust
struct WakeQueue {
    by_priority: [VecDeque<WakeEvent>; 6],
}

fn dequeue_next(&mut self) -> Option<WakeEvent> {
    for prio in 0..6 {
        if let Some(ev) = self.by_priority[prio].pop_front() {
            return Some(ev);
        }
    }
    None
}
```

#### S22.6 — Network wake authentication (F=f-i.b)

**Y4-defined cryptographic signed wake packet** — 표준 Magic Packet 의
spoofing 위험을 회피.

packet format:

```
+--------+--------+----------+------------+----------+
| 6 byte | 16x    | 8 byte   | 16 byte    | 16 byte  |
| 0xFF   | dest   | nonce    | lease_id   | tag      |
+--------+--------+----------+------------+----------+
                  ↑ Y4 extension (표준 magic packet body 의 trailer)
                  ← XChaCha20-Poly1305 AEAD payload →
```

검증:
- standard Magic Packet body (6 byte 0xFF + 16x dest MAC) 가 prefix
- Y4 extension 8-byte nonce + 16-byte lease_id + 16-byte Poly1305 tag
- key = `lease.wake_sources[NicMagicPacket].signing_key` (lease 발급 시
  host operator 가 설정, 또는 자동 생성)
- AEAD verify (key, nonce, additional_data = standard Magic Packet body,
  ciphertext = lease_id, tag = 16-byte) → success 시 lease_id 추출 →
  lease wake 발화
- nonce replay 차단: lease-pm capsule 의 internal monotonic counter
  (S20.4.2 패턴 정합) — 같은 nonce 재사용 시 audit `MagicPacketReplay`

표준 Magic Packet (SecureOn password 옵션) 미사용:
- SecureOn 의 password 가 plain text on wire — sniffing 으로 노출
- spoofing 가능 (capture + replay)
- Y4-defined 가 cryptographic signature + replay protection 제공

#### S22.7 — USB wake / device wake (G=g-i)

USB device wake 의 화이트리스트 — `LeaseCap.wake_sources` 의
`UsbDevice { vid_pid, device_class }` 항목:

| 정책 | 의미 |
|---|---|
| `vid_pid = Some((vid, pid))` | 해당 specific USB device 만 wake 가능 |
| `vid_pid = None, device_class = Some(c)` | 해당 device class (HID / Mass Storage 등) 만 |
| `vid_pid = None, device_class = None` | 모든 USB device wake (laptop default) |

**host 의 USB device 직접 wake X** — guest 가 USB attach detect 의
wake 를 own VM scope 만 받음.  passthrough 는 Phase D 의 별도 표면
(IOMMU programming capsule + per-device BAR cap, vmm_arch.md §8.8) 까지
차단.

#### S22.8 — Lid / button wake — form-factor wake routing (H=h-i)

physical button (power / lid / volume / convertible mode switch) wake
는 항상 **host operator level**:

```
physical button wake event
    → wakeup capsule (S22.4 spurious 검증)
    → power-orchestrator
        → form-factor profile 의 wake routing 결정:
            * laptop (lid open) → default lease 또는 last-active lease wake
            * handheld (mode switch) → S19.6.3 의 dock-detect / handheld-mode
              transition trigger + 활성 lease wake
            * server-farm (이론적 power button) → host operator notification
              (immediate wake X — operator 결정 대기)
        → lease 결정 후 wake forward
    → audit WakeRouting
```

button wake 의 routing 결정은 host operator 정책 — guest 가 직접 받음
0.

#### S22.9 — RTC alarm wake — virtio-rtc per-lease (I=i-i)

lease 가 자체 alarm 등록:

```rust
// lease 측 (guest 측 paravirt API)
virtio_rtc.set_alarm(absolute_ts: u64);

// wakeup capsule 측
fn on_virtio_rtc_alarm(lease_id: LeaseId, ts: u64) {
    // host RTC chip 의 alarm 으로 직접 등록 X
    // wakeup capsule 의 internal alarm scheduler 가 지속 주시
    // 가장 빠른 alarm 의 시각으로 host RTC alarm 1 개만 set
    self.alarm_scheduler.add(lease_id, ts);
    self.update_host_rtc_alarm();
}
```

host RTC chip 직접 access 0 — 모든 alarm 은 wakeup capsule 의 internal
scheduler 가 관리.  host RTC 의 single alarm slot 은 가장 빠른 alarm
의 시각으로만 set, 그 외 alarm 은 wake 후 capsule 이 재발화.

#### S22.10 — 사용자 force toggle (J=j-i, S19.6 / S20.9 / S21.9 패턴)

```rust
enum WakeForce {
    InhibitAll,      // 모든 wake 차단 (긴 deep idle, do-not-disturb)
    FollowPolicy,    // form-factor profile + cmdline + lease binding (default)
    AllowAll,        // 모든 화이트리스트 wake 적극 (always-available, server-farm 의
                     //   compliance 또는 operator 의무)
}

const Y4_PM_WAKE_FORCE_DEFAULT: WakeForce = WakeForce::FollowPolicy;
```

설정 채널:
- cmdline `y4.pm.wake_force=inhibit-all|policy|allow-all`
- runtime CLI:
  ```
  y4-hypercall power wake-force inhibit-all
  y4-hypercall power wake-force policy
  y4-hypercall power wake-force allow-all
  y4-hypercall power wake-status   # 현 force + 화이트리스트 + 직전 wake event
  ```

권한: **host operator only** (F1 패턴).
persistence: cmdline default + runtime override (F2 패턴).
audit: `WakeForceToggle` (severity Warning).

InhibitAll 의 emergency 예외:
- thermal hardlimit (S21.8 의 priority 0) 는 InhibitAll 무시 — hardware
  보호 우선, S20.9 ForceOff + S21.9 hardlimit 와 동일 패턴

#### S22.11 — Audit (S12 정합) (K=k-i)

S12.2 schema 에 v1.x patch 로 op_tag 6 개 추가:

```rust
enum OpTag {
    // 기존 ...
    WakeEventReceived,         // S22.4 — 모든 wake event (severity Trace, volume ↑)
    SpuriousWakeDetected,      // S22.4 — 화이트리스트 / lease binding 불일치 (severity Warning)
    WakeSourceConfigChange,    // S22.1 — host operator 의 wake source 추가/제거 (severity Info)
    MagicPacketRejected,       // S22.6 — F=f-i.b authentication / replay fail (severity Warning)
    WakeRouting,               // S22.4 — 정상 wake 의 lease routing 결정 (severity Info)
    WakeForceToggle,           // S22.10 — force-toggle 변경 (severity Warning)
    // ...
}

enum AuditPayload {
    // ...
    SpuriousWakeDetected {
        event:  WakeEvent,
        reason: String,    // "not in whitelist" / "no matching lease binding" / "magic packet sig fail"
    },
    MagicPacketRejected {
        source_mac:   MacAddr,
        dest_mac:     MacAddr,
        reason:       enum { SignatureMismatch, NonceReplay, UnknownLease },
    },
}
```

anomaly detector rule (S12.6 정합):
- `SpuriousWakeDetected` ≥ 5 / 분 → warning (DoS 시도 가능성, battery
  형상에서 power 손실)
- `MagicPacketRejected` (NonceReplay) 1 회 → warning + lease 의 wake key
  rotation 권고 host operator 통지
- `MagicPacketRejected` (SignatureMismatch) ≥ 10 / 분 → warning, brute-
  force 시도 가능성

### S23 — PSP / PCH power mailbox approval (S14 정합)

**ARCH-II' 매핑:**
- `psp-pm` capsule — 본체.  vendor 별 mailbox backend (SMU / PCH PMC /
  PMIC / OCC / SE / SBI) + access mediation + voltage range 검증 + dry-
  run simulation.
- `firmware-approval` capsule (cross-cluster, vmm_arch.md 의 기존
  capsule) — pending queue 에 mailbox operation entry forward (S14 정합).
- `msr-bitmap` capsule — voltage MSR (Intel `IA32_VR_MISC_CTL 0x150`,
  AMD SMU MSR) 차단 + read 마스킹.
- `cpuid-emul` capsule — power feature bit 마스킹 (S15.6 / S17.9 /
  S21.11 super-set).
- `npt` capsule — SMN BAR / mailbox MMIO 의 NPT 매핑 차단.
- `io-bitmap` capsule — mailbox port (PCH PMC port 등) 의 IO 차단.
- `audit` capsule — `Mailbox*` op_tag entry (S14 op_tag set 확장).
- power-orchestrator — mailbox dispatch hub + S14 forward.

위협:
- **Plundervolt** (USENIX Security '20) 류 — voltage 변조로 SGX/TZ
  enclave fault injection 유발 → cryptographic key extraction
- under-volt: voltage 낮춰 cryptographic operation fault
- over-volt: voltage 높여 hardware 손상
- microcode patch via mailbox (S14 의 PATCH_LOADER MSR 외 alternative
  channel)
- fuse blow / calibration override (irreversible)

#### S23.1 — Vendor 별 power mailbox 식별 (A=a-i)

psp-pm capsule 의 vendor backend 매트릭스:

| Vendor / Platform | Mailbox / Channel | Backend module |
|---|---|---|
| **AMD PSP** | SMU mailbox at `MSR 0xC0010140-0xC0010143` 또는 SMN (System Management Network) BAR | `mailbox_amd_smu.rs` |
| AMD SVI2 | Serial Voltage Interface command | `mailbox_amd_svi2.rs` |
| **Intel PCH** | PMC (Power Management Controller) mailbox + PMSU | `mailbox_intel_pmc.rs` |
| Intel VR | `MSR 0x150 IA32_VR_MISC_CTL` 또는 SVID interface | `mailbox_intel_vr.rs` |
| **ARM SoC PMIC** | vendor-specific (Qualcomm RPMh, NXP SCU, Samsung S2MPS, MTK PMIC 등) | `mailbox_arm_pmic_*.rs` |
| **POWER OCC** | On-Chip Controller mailbox (PIB / TOD / OCC) | `mailbox_power_occ.rs` |
| **IBM Z** | SE (Support Element) mailbox + HMC (Hardware Management Console) | `mailbox_z_se.rs` |
| **RISC-V** | SBI 의 SUSP / HSM extension + vendor-specific (SiFive, Andes, T-Head) | `mailbox_riscv_sbi.rs` |
| **SPARC / MIPS64** | platform-별 (Oracle ILOM 등) — 대부분 host operator 만 | `mailbox_legacy.rs` (limited support) |

`Y4/capsules/pm-psp-pm/`:
```
src/
├── lib.rs
├── mailbox_amd_smu.rs
├── mailbox_amd_svi2.rs
├── mailbox_intel_pmc.rs
├── mailbox_intel_vr.rs
├── mailbox_arm_pmic_qcom.rs
├── mailbox_arm_pmic_nxp.rs
├── mailbox_power_occ.rs
├── mailbox_z_se.rs
├── mailbox_riscv_sbi.rs
└── mailbox_legacy.rs
```

부팅 시점 platform 감지 → 적절 backend dispatch.  미지원 platform 은
mailbox feature 0 (S22 의 wakeup 와 별개로, mailbox 전체가 disable —
Tier 3-equivalent fallback).

#### S23.2 — Power mailbox access 권한 (B=b-i, S14 통합)

**모든 guest access 차단 + S14 firmware-approval capsule 의 pending
queue 로 trap forward.**

```rust
fn on_mailbox_access(vcpu: VcpuId, op: MailboxOp) -> Result<(), Y4Error> {
    // 1. Voltage range 검증 (S23.4)
    if !voltage_range_valid(&op) {
        audit.append(MailboxAccessRejected {
            op,
            reason: "voltage range violation (D=d-i)",
            severity: Critical,
        });
        return Err(Y4Error::SecurityViolation);
    }

    // 2. S14 firmware-approval pending queue 에 entry 추가
    let entry = firmware_approval.queue(FirmwareOp::MailboxOperation {
        vendor: detect_vendor(),
        target: op.target,
        opcode: op.opcode,
        payload: op.payload,
    }, Scope::HostWide /* 또는 vm-local — S23.3 등급별 */)?;

    audit.append(MailboxAccessQueued { entry_id: entry.id, op });

    // 3. guest 에 'pending' 응답 (S14 patterns 정합 — token-based wait
    //    또는 timeout 시 reject 결정 후 응답)
    Ok(())
}
```

S14 의 host-wide warning + token confirm (S14 D=d-ii) + dry-run (S14
E=e-i) + per-VM whitelist (S14 F=f-ii) 그대로 적용.

#### S23.3 — Voltage operation 의 3-등급 분류 (C=c-i)

```rust
enum MailboxOpClass {
    /// (1) read-only — current voltage / temperature / frequency query
    /// psp-pm mediation 으로 마스킹된 값 반환 (S21.2 thermal noise 패턴)
    ReadOnly,

    /// (2) reversible mutation — voltage step 변경, V/F curve 조정
    /// pending queue + audit Info
    ReversibleMutation,

    /// (3) irreversible / destructive — microcode patch via mailbox,
    /// fuse blow, calibration override
    /// pending queue + S14 dry-run 의무 + token confirm + audit Critical
    Destructive,
}
```

분류 별 처리:

| 등급 | scope | dry-run 의무 | token confirm | audit severity |
|---|---|---|---|---|
| ReadOnly | vm-local | X | X | Trace (`MailboxReadMasked`, volume ↑) |
| ReversibleMutation | host-wide | optional | X (D=d-i 범위 안) | Info |
| Destructive | host-wide | **의무** (S14 E=e-i `dry-run` CLI 적용) | **의무** (S14 D=d-ii token) | **Critical** |

mailbox opcode 별 등급 매핑은 vendor backend 가 결정 (mailbox spec
참조).  분류 미명 opcode 는 default Destructive (보수적).

#### S23.4 — Voltage range 한도 (D=d-i, Plundervolt 차단)

```
Y4_PM_VOLTAGE_MIN_OFFSET_MV    build-time const, default -50 (mV)
                                cmdline `y4.pm.voltage_min_offset_mv` override
Y4_PM_VOLTAGE_MAX_OFFSET_MV    build-time const, default +50 (mV)
                                cmdline `y4.pm.voltage_max_offset_mv` override
```

근거:
- Plundervolt 공격은 -100 mV 이상 under-volt 필요 (typical)
- ±50 mV = 정상 V/F curve 조정 범위 (CPU vendor 의 published nominal
  range 안)
- 초과 시 pending queue 도 거치지 X — S23.2 의 즉시 reject + Critical
  audit

```rust
fn voltage_range_valid(op: &MailboxOp) -> bool {
    match op.semantic_class() {
        VoltageDelta(mv) => {
            (Y4_PM_VOLTAGE_MIN_OFFSET_MV..=Y4_PM_VOLTAGE_MAX_OFFSET_MV)
                .contains(&mv)
        }
        _ => true   // non-voltage op 은 별도 검증
    }
}
```

certified profile (의료/항공/금융 트랙) 의 옵션:
- `y4.pm.voltage_min_offset_mv=0` + `y4.pm.voltage_max_offset_mv=0` →
  voltage 변경 자체 0 (Plundervolt 사실상 불가능)

#### S23.5 — Mailbox SMN / BAR / MMIO 격리 (E=e-i, S3 + S11 정합)

**npt capsule + io-bitmap capsule 동시 차단:**

```rust
// npt capsule 측 (S3 정합)
// 부팅 시점 SMN / mailbox MMIO range 식별 → guest NPT 매핑 거부
fn on_npt_map(host_frame: HostFrameCap, guest_pa: GuestPaddr) -> Result<(), Y4Error> {
    let host_pa = host_frame.physical_address();
    if is_in_mailbox_mmio_range(host_pa) {
        audit.append(NptMappingRejected { host_pa, reason: "mailbox MMIO" });
        return Err(Y4Error::SecurityViolation);
    }
    // ... 정상 매핑 진행
}
```

io-bitmap capsule (S11 정합):
- form-factor 별 mailbox port (PCH PMC 의 IO port range, vendor-specific)
  추가 default-block
- S11.2 default-block 표 갱신 (S23 의 mailbox port 추가)

이중 차단 — guest 가 SMN BAR direct mapping 또는 IO port 어느 경로로도
mailbox 도달 0.

#### S23.6 — CPUID power feature 마스킹 (F=f-i)

`cpuid-emul` capsule 의 추가 마스킹 (S15.6 / S17.9 / S21.11 super-set):

| Leaf | 비트 | 마스킹 |
|---|---|---|
| `CPUID 0x80000007.EDX[7]` | HwPstate | 0 강제 (AMD HW P-state) |
| `CPUID 0x80000007.EDX[9]` | CPB (Core Performance Boost) | 0 (S15.6 짝) |
| `CPUID 0x80000008.EBX` | AMD power feature bits 일괄 | 0 (RAPL2, FastShortRepStosb 등) |
| `CPUID 0x6.EAX[1]` | Turbo Boost (Intel) | 0 (S15.6 / S21.11 짝) |
| `CPUID 0x80000022` | AMD AmdFeature_Pmu_V2 일부 | guest mask 검토 |

guest 가 power mailbox feature / Turbo / HW P-state 미보유로 인식 →
software fallback path 사용.

#### S23.7 — Form-factor 별 mailbox 정책 (G=g-i)

`tools/power.rules`:

```
[server-farm]
Y4_PM_MAILBOX_POLICY = strict       # 모든 mutation 의무 차단, read-only audit only

[rack-mount]
Y4_PM_MAILBOX_POLICY = strict

[laptop]
Y4_PM_MAILBOX_POLICY = mediated     # read-only mediation 허용 + mutation pending

[handheld-battery-mode]
Y4_PM_MAILBOX_POLICY = mediated

[handheld-dock-mode]
Y4_PM_MAILBOX_POLICY = mediated

[soc]
Y4_PM_MAILBOX_POLICY = platform-driver-only   # PMIC 의 platform driver 만 host operator 권한

[certified]
Y4_PM_MAILBOX_POLICY = strict       # ±0 mV voltage + 의무 차단
```

`strict` = read-only mediation 도 audit 진행, mutation 은 항상 reject.
`mediated` = read-only OK + mutation pending.  `platform-driver-only`
= host operator 의 platform driver 만 mailbox access (guest path 0).

#### S23.8 — S14 firmware-approval 와의 통합 (H=h-i, single source of truth)

S14 의 firmware-approval pending queue 에 **신규 entry type 추가**
(별도 queue 신설 X):

```rust
enum FirmwareOp {
    // 기존 (S14 본체) ...
    MicrocodeUpdate { payload: Vec<u8>, payload_hash: Sha256 },
    SmiInvocation { port: u16, value: u8 },
    UefiCapsule { capsule: Vec<u8> },
    AcpiMethodMutating { method: String, args: Vec<u64> },

    // S23 신규 entry type
    MailboxOperation {
        vendor:  MailboxVendor,
        target:  MailboxTarget,    // SMU / PMC / PMIC / OCC / SE / SBI
        opcode:  u32,
        payload: Vec<u8>,
        class:   MailboxOpClass,    // S23.3 의 3-등급
    },
}
```

S14 의 4 op_tag (`FirmwareApprovalQueued`, `Decided`, `WhitelistChanged`)
는 mailbox entry 도 동일 path — single source of truth 보존.

S23 의 별도 op_tag (S23.10 의 `Mailbox*`) 는 **psp-pm capsule 측 audit
sub-detail** — S14 의 entry record 의 부속.

#### S23.9 — MSR `0x150` (IA32_VR_MISC_CTL) 직접 차단 (I=i-i)

`msr-bitmap` capsule 의 S10.1 mandatory entry 갱신 (v1.x patch, S15 /
S17 / S21 패턴 정합):

| MSR | 권한 | 비고 |
|---|---|---|
| `IA32_VR_MISC_CTL (0x150)` | guest write 차단, read 마스킹 | Intel undervolting interface — Plundervolt 의 standard channel |
| `IA32_OVERCLOCKING_STATUS (0x195)` | guest read+write 차단 | Intel overclocking |
| `MSR_PKG_CST_CONFIG_CONTROL (0xE2)` | guest write 차단 | Intel C-state 동작 변경 |
| AMD SMU MSR `0xC0010140-0xC0010143` | guest read+write 차단 | SMU mailbox MSR (vendor-specific, S23.1 의 backend 와 짝) |

write 시도 → S10.4 default-deny audit (`MsrAccessDenied`) + #GP(0).
read (cap 마스킹) 은 fixed value 반환 (host 의 실제 voltage offset 노출
0).

#### S23.10 — 사용자 force toggle (J=j-i, S19/S20/S21/S22 패턴)

```rust
enum MailboxForce {
    Strict,         // 모든 mailbox mutation 차단 (S14 pending 도 자동 reject)
                    //   — 의료/항공/금융 인증 트랙 default
    FollowPolicy,   // form-factor profile (S23.7) + cmdline (default)
    Permissive,     // read-only mediation 만, mutation 도 pending 거치되 Critical 만 token confirm
}

const Y4_PM_MAILBOX_FORCE_DEFAULT: MailboxForce = MailboxForce::FollowPolicy;
```

설정 채널:
- cmdline `y4.pm.mailbox_force=strict|policy|permissive`
- runtime CLI:
  ```
  y4-hypercall power mailbox-force strict
  y4-hypercall power mailbox-force policy
  y4-hypercall power mailbox-force permissive
  y4-hypercall power mailbox-status   # 현 force + 정책 + 직전 mailbox event
  ```

권한: **host operator only** (F1 패턴).
persistence: cmdline default + runtime override (F2 패턴).
audit: `MailboxForceToggle` (severity Warning).

Strict 의 emergency 예외:
- thermal hardlimit (S21.8 priority 0) 도 mailbox 사용 시 — power-
  orchestrator 의 emergency response 가 strict 우회 가능 (host 보호 우선,
  audit Critical 동반).  단 voltage range 한도 (S23.4) 는 Strict 에서도
  강제.

#### S23.11 — Audit (S12 정합) (K=k-i)

S12.2 schema 에 v1.x patch 로 op_tag 5 개 추가 (S14 op_tag set 의 sub-
detail):

```rust
enum OpTag {
    // 기존 ...
    MailboxAccessQueued,    // S23.2 — pending queue 적재 (severity Info; S14 FirmwareApprovalQueued 의 sub-detail)
    MailboxAccessApproved,  // S23.2 — host operator 승인 후 적용 (severity Info)
    MailboxAccessRejected,  // S23.2 / S23.4 — voltage range 위반 시 Critical, 그 외 Warning
    MailboxReadMasked,      // S23.3 ReadOnly — psp-pm mediation 의 read 마스킹 (severity Trace, volume ↑)
    MailboxForceToggle,     // S23.10 — force-toggle 변경 (severity Warning)
    // ...
}

enum AuditPayload {
    // ...
    MailboxAccessRejected {
        op:       MailboxOp,
        reason:   enum {
            VoltageRangeViolation { requested_mv: i32, allowed_min: i32, allowed_max: i32 },
            StrictPolicyActive,
            UnknownOpcode,
            ScopeMismatch,
        },
    },
}
```

anomaly detector rule (S12.6 정합):
- `MailboxAccessRejected` (VoltageRangeViolation) 1 회 → host operator
  즉시 통지 + 잠재 Plundervolt 시도 (Critical)
- `MailboxAccessQueued` ≥ 10 / 분 → warning (mailbox spam, DoS 시도)
- `MailboxReadMasked` ≥ 10000 / 분 → warning (guest 의 fingerprinting
  시도)

---

## 4. Verus invariant catalog (AV21~AV40)

### 4.1 카탈로그 구조

amdv_safety.md §5 의 AV1~AV20 후 자연 연속 (AV21~AV40, 예비 확장 포함).
**statement only v1.0 frozen** (proof body 는 PR-5 진입 시 채움 —
formal-first 의 statement-first sign-off, amdv_safety.md §5.1 패턴
정합).

power_arch.md §5.x 의 2-축 layout 정합 — `proofs/verus/src/power/{upper,
lower}/` + 각 layer 안에 per-capsule + per-안전장치 파일.

### 4.2 Invariant 표 (15 항목 + Phase D 1 + reserved 5)

| AV | Layer | 책임 capsule | Invariant | Safety / 결정 | Proof file | Status |
|---|:---:|---|---|---|---|---|
| **AV21** | Lower | lease-pm + audit | `tpm_pcr_consistency(handle)`: tier(handle) == Tier1_5 ⟹ (unseal_succeeds(handle) ⟹ pcr_at_unseal == pcr_bound_at_seal(handle)) | S20.2.8 | `lower/tpm_consistency.rs` | v1.0 |
| **AV22** | Upper | power-orchestrator | `sub_mode_consistency`: `forall fact: FormFactor, m: SubMode, m ∈ allowed_sub_modes(fact) ⟹ rule_defined(fact, m)` (default + user-defined 합쳐 등록된 set 안에서만 mode-set) | M11 / §2.1 | `upper/sub_mode.rs` | v1.0 |
| **AV23** | Lower | lease-pm + power-orchestrator | `sub_mode_transition_atomicity`: transition 시점에 모든 lease 가 suspended 상태에서만 const 변경 적용 + AEAD integrity check 후 wake (S20 패턴 정합) | M11 / §2.4 / S20.3 | `lower/sub_mode_transition.rs` | v1.0 |
| **AV24** | Upper | power-orchestrator | `mode_invariant_holds`: named sub-mode 가 정의되어 있을 때만 specific invariant 적용.  예: `defined("transportation") ⟹ transportation_sudden_power_loss_safe`. removed (M13) 시 invariant 자동 비활성 | M11 / M13 / §2.6 | `upper/mode_invariants.rs` | v1.0 |
| **AV25** | Upper | psp-pm + msr-bitmap | `voltage_range_bound`: `forall op: MailboxOp, voltage_delta(op) ∈ [Y4_PM_VOLTAGE_MIN_OFFSET_MV, Y4_PM_VOLTAGE_MAX_OFFSET_MV]` | S23.4 | `upper/voltage_range.rs` | v1.0 |
| **AV26** | Upper | wakeup | `magic_packet_replay_protection`: `forall pkt: MagicPacket, accept(pkt) ⟹ pkt.nonce ∉ seen_nonces(pkt.lease_id)` + nonce 갱신 | S22.6 | `upper/magic_packet.rs` | v1.0 |
| **AV27** | Upper | acpi-pm + cpufreq + lifecycle | `thermal_hardlimit_emergency`: `forall t: Temperature, t ≥ HARDLIMIT_C ⟹ all_vcpus_paused() ∧ lease_throttled()` | S21.8 | `upper/thermal_hardlimit.rs` | v1.0 |
| **AV28** | Upper | wakeup | `wake_source_whitelist`: `forall ev: WakeEvent, accept(ev) ⟹ ev.source ∈ form_factor_whitelist() ∧ matches_lease_binding(ev)` | S22.2 / S22.4 | `upper/wake_whitelist.rs` | v1.0 |
| **AV28-D** | Upper | wakeup + Phase D IOMMU | `wake_source_iommu_consistent`: forward-compat hook (Phase D 의 IOMMU programming capsule 도입 후 본문) | S22.4 + Phase D | `upper/wake_iommu.rs` | **Phase D** |
| **AV29** | Upper | rapl + audit | `rapl_energy_budget_enforced`: `forall lease: LeaseCap, vmrun_allowed(lease) ⟹ lease.energy_used_j < lease.energy_budget_j` | S17.8 | `upper/rapl_budget.rs` | v1.0 |
| **AV30** | Lower | cpufreq + lifecycle | `smt_pair_power_state_sync`: `isolate_pairs() ⟹ forall pair: SmtPair, pstate(pair.primary) == pstate(pair.sibling) ∧ cstate(pair.primary) == cstate(pair.sibling)` | S19.2 / S19.7 | `lower/smt_sync.rs` | v1.0 |
| **AV31** | Lower | cpufreq | `dvfs_dwell_time`: `forall t1, t2: pstate_change_times, t2 > t1 ⟹ (t2 - t1) ≥ Y4_PM_MIN_PSTATE_DWELL_NS` | S15.5 | `lower/dvfs_dwell.rs` | v1.0 |
| **AV32** | Upper | acpi-pm + firmware-approval | `acpi_table_integrity`: `forall t: AcpiTable, alive(t) ⟹ sha256(t) == boot_recorded_sha256(t)` (mismatch 시 lease revoke chain, S14 정합) | S18.8 | `upper/acpi_integrity.rs` | v1.0 |
| **AV33** | Upper | lease-pm | `wake_epoch_monotonic`: `forall handle: SealedHandle, unseal_succeeds(handle) ⟹ handle.epoch == expected_wake_epoch(handle.lease_id)` + `expected_wake_epoch` monotonic increment per suspend | S20.4.2 | `upper/wake_epoch.rs` | v1.0 |
| **AV34** | Lower | power-orchestrator | `force_toggle_masks_mode_signal`: `forall force: ForceState, force ≠ FollowPolicy ∧ mode_signal_received() ⟹ effective_state == force_state` (S19/S20/S21/S22/S23 force 5 개 모두 적용) | M5 / S19.6.4 + 4 force 정합 | `lower/force_mask.rs` | v1.0 |
| **AV35** | Upper | power-orchestrator | `boot_fix_form_factor`: form-factor + certified flag 는 boot 시점 결정 후 runtime 변경 X.  sub-mode 만 mode-set 으로 변경 가능 (AV23 atomicity 정합) | §2.5 / M10 | `upper/boot_fix.rs` | v1.0 |
| **AV36** | — | — | reserved (v1.x patch — §1.4 위협 ledger 발견 시 채움) | reserved | — | — |
| **AV37** | — | — | reserved | reserved | — | — |
| **AV38** | — | — | reserved | reserved | — | — |
| **AV39** | — | — | reserved | reserved | — | — |
| **AV40** | — | — | reserved | reserved | — | — |

### 4.3 Layer 분포 통계

- **Upper (cross-tenant / cross-CPU / cross-VM)**: AV22 / AV24 / AV25 /
  AV26 / AV27 / AV28 / AV28-D / AV29 / AV32 / AV33 / AV35 = **11
  invariant** (Phase D forward-compat 1 포함)
- **Lower (within-cluster / capsule cooperation)**: AV21 / AV23 / AV30 /
  AV31 / AV34 = **5 invariant**

power 측의 Upper:Lower 비율 (11:5) 이 amdv 측 (Upper 9:Lower 12) 과
다른 이유: power 의 위협 모델 (§1.2) 이 cross-tenant attack (Plundervolt
/ PLATYPUS / Hertzbleed cross-tenant) 비중 ↑.

### 4.4 v1.x patch 추가 invariant slot (S=a, AV36~AV40)

§1.4 의 v1.x 위협 ledger 발견 시 새 invariant 추가:
- 새 CVE / 학술 논문 → §1.2 catalog 에 row 추가 → mitigation 으로 새
  AV# 추가 (AV36 부터 채움)
- AV40 도달 시 v2 (incompatible) 단계로 catalog 확장 또는 §5.1 의 amdv
  catalog 와 마찬가지로 reserved 확장

### 4.5 Phase D forward-compat (R=a)

AV28-D `wake_source_iommu_consistent` — Phase D 진입 시 IOMMU programming
capsule + per-device BAR cap 도입 후 statement body 채움.  본 v1.0
spec 에서는 forward-compat hook 만 (proof file path 예약):

```
proof file: proofs/verus/src/power/upper/wake_iommu.rs

// v1.0
proof fn wake_source_iommu_consistent_stub() {
    // Phase D 본문 채움
}
```

amdv_safety.md AV2-D 패턴 정합 (`npt_iommu_consistent`).

---

## 5. PR-5 분리 계획 (Phase C 8 번째 단계)

`phase_plan.md` §C 의 8 번째 단계 = PR-5 power-mgr.  vmm_arch.md §6.3
의 4-PR 패턴 정합 + amdv_safety.md §6.2 의 sub-PR 분리 옵션 정합.

### 5.1 PR-5 sub-PR 분리 (A=a, DAG sink-first)

PR-5 는 7 신규 workspace member (power-orchestrator + 6 capsule) 동시
도입.  단일 PR 보다 **DAG sink-first 4 sub-PR 분할**:

| Sub-PR | 내용 | 의존 (선결) | timeline |
|---|---|---|---|
| **PR-5a** | `Y4/power-orchestrator/` 신설 + `audit` capsule 의 power op_tag 확장 (S12.2 v1.x patch — `Wake*` / `CState*` / `Thermal*` / `LeaseSuspend*` / `Sub-mode*` / 등 30+ 신규 op_tag).  orchestrator 가 6 capsule cap 분배의 single source (vmm_arch.md §2.4 DAG 정합) | PR-1 + PR-2 머지 또는 review 진입 | Phase C 중반 |
| **PR-5b** | `Y4/capsules/pm-lease-pm/` (S20 본체).  XChaCha20-Poly1305 AEAD + replay protection + 4-tier secure storage (Tier 1 / Tier 1.5 dTPM / Tier 2 DRAM / Tier 3 거부) + ISA-별 backend 11 sub-module + tss-esapi cargo dep | PR-5a + lifecycle capsule (vmm) cross-cluster API 안정 | Phase C 중반 |
| **PR-5c** | `Y4/capsules/pm-cpufreq/` + `pm-msr-bitmap-extension` (S15 / S16 / S19 의 P-state / C-state / SMT 동기 + msr-bitmap mandatory entry 확장).  AV30 `smt_pair_power_state_sync` + AV31 `dvfs_dwell_time` proof body | PR-5b | Phase C 종반 |
| **PR-5d** | `Y4/capsules/pm-{acpi-pm,rapl,wakeup,psp-pm}/` (S17 / S18 / S21 / S22 / S23 의 ACPI mediation + RAPL paravirt + wake source whitelist + power mailbox approval) | PR-5c | Phase C 종반 |

각 sub-PR 의 review 부담 분산 + 의존성 cycle 짧음.

### 5.2 산출물 매트릭스

| 산출물 | 위치 | PR | 의존 |
|---|---|---|---|
| **seL4 측 D1a' 패치** (B=a) — power MSR / ACPI mediation / SMI handling | `third_party/sel4-patches/100-power-*.patch` (sel4_fork_policy.md §6.3 numbering, PR-1 의 후속 series — 단일 mainline submission) | **PR-1 의 후속 series, 동일 mainline submission timing** | sel4_fork_policy.md v1.0 frozen + 회귀 게이트 G1~G7 통과 |
| **`Y4/power-orchestrator/`** | 신규 workspace member, ~600 LoC budget (`Y4_PM_ORCHESTRATOR_LOC_BUDGET`) | PR-5a | vmm_arch.md §2.2.1 의 800 LoC budget 과 별도 |
| **`Y4/capsules/pm-{6}/`** | 신규 6 sub-crate (lease-pm / cpufreq / acpi-pm / rapl / wakeup / psp-pm) | PR-5b ~ PR-5d | 위 sub-PR 표 |
| **Verus 명세 (AV21~AV40)** | `proofs/verus/src/power/{upper,lower}/` | **amdv 측 PR-3 의 짝** (C=a — 단일 paper artifact 의 다른 module) | PR-5b ~ PR-5d 머지 + amdv PR-3 진행 |
| **Isabelle skeleton (AV21~AV40)** | `y4-verus2isabelle` 도구 산출물의 일부 | **amdv 측 PR-4 와 통합** (D=a — 같은 도구가 amdv + power 둘 다 처리, P1.4 §5.3 / P3.6 §3.2 single tool 정합) | `y4-verus2isabelle` v1.0 + power fixture round-trip 검증 |
| **`tools/power.rules.d/`** Y4 ship default | `00-default-*.rules` 8 파일 (server-farm / rack-mount / mobile / soc + mobile-{dock,portable,transportation} + certified) | PR-5a 와 함께 (boot 시점 사용 시작) | logicutils `lu-rule` |
| **`tools/sel4-fork-check.sh`** G7 확장 (F=a) | timing-equal trace check | PR-1 의 후속 (`100-power-*.patch` 와 짝) | sel4_fork_policy.md §3.6 G6 정합, v1.x patch 단계에서 강제 |

### 5.3 Workspace member 통합 갱신 (G=a)

vmm_arch.md §5.1 의 **16 → 23 workspace member**:

```
기존 (vmm_arch.md §5.1 후) — 16 멤버:
  y4-alloc, y4-capsules, y4-ipc, y4-roottask, y4-scudo-sys
  y4-vmrun-orchestrator
  y4-capsules-vmm-{vmcb,npt,msr-bitmap,io-bitmap,firmware-approval,
                    cpuid-emul,npf-handler,audit,nested-request,lifecycle}

추가 (PR-5) — 7 멤버:
  y4-power-orchestrator
  y4-capsules-pm-{cpufreq,acpi-pm,psp-pm,rapl,wakeup,lease-pm}

신규 합계: 23 workspace member
```

vmm_arch.md §5.1 의 표는 power_safety.md frozen 후 v1.x patch 로 갱신
(amdv 측 frozen 변경 0).

### 5.4 회귀 게이트 G7 (F=a, sel4_fork_policy.md §3.6 정합)

sel4_fork_policy.md §3.6 의 G6 (timing-equal optional, v1.0) 의 sub:

```
G7 — power-related syscall timing-equal
    KernelDebugBuild=ON 의 power-related syscall latency trace
    (HLT mediation / MWAIT mediation / SMI handling / ACPI eval mediation
     / wake event handling 등) 이 upstream seL4 와 ±5% 안.
    v1.0 optional, v1.x patch 에서 강제.
```

PR-1 의 후속 patch series 머지 시점에 측정 microbench 산출물도 함께
제출 (Phase C 종반).

### 5.5 Contribute-back 형태 (H=a, vmm_arch.md §6.3 + amdv_safety.md §6.2 정합)

PR-1 ~ PR-5 의 4-way contribute-back:

| 산출물 | 게시 plan |
|---|---|
| **C 패치 (D1a + D1a')** | seL4 mainline PR-1 (raw-SVM + power MSR/ACPI mediation 통합 단일 submission) |
| **Rust capsule 코드 (vmrun + power-orchestrator + 16 capsule)** | Y4 워크스페이스 (Y4 GitHub) — PR-2 + PR-5a~d |
| **Verus 명세 (AV1~AV40)** | Y4 frozen tag + paper artifact (PR-3 + PR-5 의 power 측 module) |
| **Isabelle skeleton** | `y4-verus2isabelle` 도구 산출물 (PR-4 + power fixture) |

paper venue: vmm_arch.md §6.4 의 1 순위 SOSP workshop / PLOS, 2 순위
SOSP / OSDI main track — power 측의 학술적 차별점 (power_arch.md §6 의
4 항목, 특히 "Hertzbleed / DVFS side-channel 의 capsule-level 격리 +
Verus invariant") 가 paper 의 추가 evidence.

---

## 6. 동결 정책 (frozen / sign-off)

본 doc 은 v0 spec.  `v1.0 frozen` 마킹 조건:

### 6.1 sign-off 조건

- **§1 위협 모델** (12 항목 catalog + 4 threat actor + form-factor 가중치
  + v1.x ledger path) 사용자 sign-off
- **§2 form-factor + sub-mode + universal customizability** — 4 default
  form-factor + mobile 의 3 default sub-mode + certified overlay + 30+
  cmdline key + `tools/power.rules.d/` overlay merge + ModeSignal
  String-keyed + detection 2-step + 새 form-factor/sub-mode 추가 path
  + 3-layer 우선순위 사용자 sign-off
- **§3 9 안전장치 catalog (S15~S23)** 모두 사용자 sign-off — sub-decision
  포함 (§6.3 ledger)
- **§4 Verus invariant catalog (AV21~AV40)** statement 사용자 sign-off
  (proof body 는 PR-5 진입 시 채움 — formal-first 의 statement-first
  sign-off, amdv_safety §5.1 패턴 정합)
- **§5 PR-5 분리 계획 (4 sub-PR + D1a' patch series + Verus + Isabelle
  통합)** 사용자 sign-off
- 짝 doc **`docs/power_arch.md` v1.0 frozen** 와 짝 (§6.2)

### 6.2 짝 doc 일괄 frozen 의존

본 doc 의 v1.0 frozen 은 **다음 짝 doc 의 v1.0 frozen 과 짝으로만 발화**
— power domain 의 별도 v1.0 cycle (vmm_arch.md / amdv_safety.md /
sel4_fork_policy.md / verus_to_isabelle.md 4 doc 의 v1.0 frozen 과
**별도 cycle**, 단 cross-cluster capsule API 의존):

| Doc | 짝 frozen 조건 |
|---|---|
| `docs/power_arch.md` | capsule 분해 (6 capsule + power-orchestrator) + lease integration + PR split |
| `docs/power_safety.md` (본 doc) | 14 안전장치 (S15~S23) + AV21~AV40 catalog |

cross-cluster API 의존 (vmm_arch.md 의 frozen capsule 재사용):
- `audit` capsule (S12 schema) — power op_tag 30+ 추가 (v1.x patch)
- `lifecycle` capsule — sub-mode transition + lease pause/throttle
- `firmware-approval` capsule (S14) — `MailboxOperation` entry type 추가

본 cross-cluster 의존 갱신은 amdv_safety.md / vmm_arch.md 의 v1.x patch
(§7.4 정합) — 4 doc frozen 변경 0.

### 6.3 안전장치 sub-decision sign-off ledger

S15~S23 + S20.2 Tier 1.5 (TQ.1~TQ.9) + Mobile merger (M1~M13) 의 각
sub-decision 채택 record (amdv_safety §7.3 패턴):

| Safety / Decision | sub-decision 묶음 | 채택 |
|---|---|---|
| S15 cpufreq governor 격리 | (a-i)~(h-i) host operator only + MSR observable 차단 + CPUID frequency 마스킹 + form-factor governor + 10 ms dwell + constant_freq + SMT pair 동기 + audit | 2026-05-05 |
| S16 C-state side-channel 차단 | (a-i)~(i-i) host operator only 진입 + per-form-factor C-state max + MWAIT silent C1 substitution + residency MSR 차단 + L1D flush + deterministic timing + SMT 동기 + lease suspend trigger + audit | 2026-05-05 |
| S17 RAPL 격리 | (a-i)~(i-i) MSR 18 항목 차단 + virtio-rapl + capsule mediation + 4-bit LSB noise + audit + RAPL↔cpufreq internal channel + form-factor audit + per-VM budget + CPUID 마스킹 | 2026-05-05 |
| S18 ACPI mediation | (a-i)~(i-i) AML interpreter mediation + DSDT 화이트리스트 + per-VM virtual DSDT + _OSI 화이트리스트 + thermal threshold host-only + Sx host-only + 100 ms timeout + ACPI hash integrity + audit | 2026-05-05 |
| S19 SMT power gating | M1=a~M13=a + (a-i)~(k-i) host operator only + isolate-pairs strict 동기 + allow-mixed Warning + thread offline + IPI atomic + **3-tier force-toggle (KDE 패턴) + force-on+allow-mixed Critical** + constant_freq SMT + lease assignment + 6 audit op_tag | 2026-05-05 |
| S20 deep idle lease suspend | (a-i)~(j-i) C3 trigger + **ISA-agnostic 4-tier secure storage (Tier 1 + Tier 1.5 dTPM + Tier 2 + Tier 3)** + S13 패턴 atomicity + **XChaCha20-Poly1305 AEAD + replay protection** + 10/5 ms latency + lock-free 동시성 + form-factor 정책 + S22 wakeup 정합 + KDE force-toggle + 8 audit op_tag | 2026-05-05 |
| S20.2 Tier 1.5 sub-decisions (TQ.1~TQ.9) | TQ.1 Tier 1.5 신설 + TQ.2 PCR 0+1+2+3+7 + TQ.3 session encryption 의무 + TQ.4 cmdline + TQ.5 form-factor + TQ.6 fTPM Tier1 / dTPM Tier1.5 + TQ.7 audit + TQ.8 AV21 + TQ.9 tss-esapi crate | 2026-05-05 |
| S21 thermal throttle | (a-i)~(l-i) thermal MSR 7 항목 차단 + 3-bit LSB noise + virtio-thermal + _TCC/_PSV host-only + 5°C hysteresis + S19.6 L2 mode signal + TJMAX hardlimit emergency + KDE force-toggle (conservative/policy/aggressive) + form-factor profile + CPUID 마스킹 + 6 audit op_tag | 2026-05-05 |
| S22 wake source routing | (a-i)~(k-i) host operator + power-orchestrator only + form-factor 화이트리스트 + lease binding + spurious detection + 6-tier priority + **Y4-defined cryptographic signed magic packet** + USB VID:PID + form-factor wake routing + virtio-rtc + KDE force-toggle (inhibit-all/policy/allow-all) + 6 audit op_tag | 2026-05-05 |
| S23 PSP/PCH mailbox | (a-i)~(k-i) 9-vendor mailbox 매트릭스 + S14 pending queue 통합 + 3-등급 분류 + ±50 mV voltage range + NPT+IO 격리 + CPUID 마스킹 + form-factor 정책 + S14 single source of truth + MSR 0x150 차단 + KDE force-toggle (strict/policy/permissive) + 5 audit op_tag | 2026-05-05 |
| Mobile merger | M1=a (laptop+handheld→mobile) + M2=a (3 default sub-mode dock/portable/transportation) + M3=a (build-time `tools/power.rules.d/` overlay) + M4=a (form-factor symmetric) + M5=a (String-keyed namespace) + M6=a (transportation 본격 spec) + M7=a (vehicle-bus+GPS detection) + M8=a (deprecated alias) + M9=a (transportation fTPM 우선) + M10=a (S20 atomicity) + M11=a (AV22~AV24 generic) + M12=a (naming) + M13=a (default removal) | 2026-05-05 |
| §1 threat model | A~G 12 항목 catalog + 4 threat actor + 3 카테고리 + Layer column + form-factor 가중치 + v1.x ledger | 2026-05-05 |
| §2 form-factor / customizability | M1~M13 + cmdline 30 + logicutils overlay + ModeSignal + 2-step detection + 3-layer 우선순위 + universal customizability | 2026-05-05 |

### 6.4 v1.x patch / v2 의 정의

amdv_safety §7.4 패턴 정합 + power-specific:

| 분류 | 정의 |
|---|---|
| **v1.x patch (backwards-compatible)** | (i) AV21~AV40 statement **약화 X** (강화는 OK).<br>(ii) form-factor / sub-mode 의 default definition 갱신 = patch (mechanism 자체 변경 X).<br>(iii) 새 form-factor / sub-mode / 안전장치 (S24+) 추가 = patch (기존 S15~S23 약화 X 한정).<br>(iv) audit op_tag enum 의 새 variant 추가 = patch.<br>(v) build-time const default 값 조정 = patch.<br>(vi) cross-cluster capsule API 갱신 (audit op_tag / lifecycle / firmware-approval) = patch (각 cluster 의 v1.x patch).<br>(vii) AV28-D Phase D body 채움 = patch (forward-compat hook 활성화). |
| **v2 (incompatible)** | AV statement 약화 또는 ModeSignal namespace (string-keyed) 변경 또는 `tools/power.rules.d/` overlay merge mechanism 변경 또는 4-tier secure storage 신뢰 model 변경. |

frozen 후 v1.x patch 는 PR review + paper artifact 업데이트, v2 는
별도 재검토 cycle (S15~S23 + Mobile merger + ARCH-II' 측 호환성 재검토
+ paper revision).

### 6.5 짝 doc — `power_arch.md` v1.0 frozen 조건 (mirror)

`docs/power_arch.md` 의 v1.0 frozen 조건 (본 doc §6 에 mirror — 양 doc
의 짝 frozen 강제):

- §1 핵심 결정 (8 axis) sign-off
- §2 capsule 분해 (6 capsule + power-orchestrator + DAG 의존 그래프 +
  trust model + lease integration) sign-off
- §3 lease integration (form-factor 별 suspend 정책 + sudden power loss
  대비 transportation 강화) sign-off
- §4 PR split 매트릭스 sign-off
- §5 repo 구조 (vmm_arch.md §5.1 의 16 → 23 workspace member, v1.x
  patch path) sign-off

### 6.6 frozen 후 진입 가능 작업

본 spec frozen → **PR-5 진입 차단 해제** (phase_plan.md §C 의 8 단계 중
8 번째):

1. ✅ §6.1 5 sign-off 조건 모두 만족
2. ✅ `power_arch.md` v1.0 frozen 짝 (§6.2)
3. (열림) **PR-5a** — `power-orchestrator` + `audit` capsule power op_tag
   확장 진입 (§5.1)
4. (열림) **PR-5b** — `lease-pm` capsule + 4-tier secure storage + 11
   ISA backend 진입
5. (열림) **PR-5c** — `cpufreq` + `msr-bitmap-extension` 진입
6. (열림) **PR-5d** — `acpi-pm` / `rapl` / `wakeup` / `psp-pm` 진입
7. (열림) `tools/power.rules.d/` 의 user override 가능 (build-time, §2.3)
8. (열림) Verus AV21~AV40 proof body 채움 (PR-3 짝, §5.5)
9. (열림) `y4-verus2isabelle` 의 power fixture round-trip 검증 (PR-4
   짝, §5.5)

§5.5 의 4-way contribute-back 진입.

---

## 7. 미해결 / 추가 결정 필요

amdv_safety §8 패턴 정합.

### 7.1 닫힘 ledger (sign-off 또는 sub-decision 으로 해결됨)

| # | 항목 | 닫힘 사유 |
|---|---|---|
| 1 | Mobile merger (laptop + handheld → mobile, dual-mode → tri-mode) | M1~M13 sign-off (2026-05-05) — `mobile` 단일 form-factor + 3 default sub-mode (dock/portable/transportation) + universal customizability |
| 2 | TPM 외장 dTPM 의 보안 등급 | TQ.1~TQ.9 sign-off — Tier 1.5 신설, fTPM=Tier 1 sub-case / dTPM=Tier 1.5, PCR 0+1+2+3+7 binding, session encryption 의무, tss-esapi crate |
| 3 | Wake integrity check primitive 선택 | HMAC-SHA256 → **XChaCha20-Poly1305 AEAD** 정정 (2026-05-05) — Y4 정합 + single AEAD + ISA-agnostic uniform + Tier 분리 책임 |
| 4 | Replay protection | S20.4.2 — `expected_wake_epoch` lease-pm internal monotonic counter + AEAD AD = epoch + 매 suspend 마다 increment |
| 5 | ISA-별 secure storage 4-tier | S20.2 — Tier 1 (CPU hardware: PSP/TXT/SEV-SNP/TDX/TZ/CCA/PEF/SE/PMP/CoVE) + Tier 1.5 (외장 dTPM + AEAD master key seal) + Tier 2 (XChaCha20-Poly1305 sealed DRAM universal) + Tier 3 (suspend 거부) |
| 6 | Universal customizability principle | M3~M4 sign-off + §2.1 — Y4 ship 의 form-factor / sub-mode 도 "default definitions" — built-in 아닌, override / removal 가능, default 와 user-defined 의 mechanism 동일 (`tools/power.rules.d/` overlay merge) |
| 7 | Mode signal namespace | M5 — String-keyed (default 와 user-defined sub-mode 동일 type) |

### 7.2 v1.x patch 미해결 ledger

(현 시점 비어 있음 — frozen 시 추가될 항목 모두 §6.4 의 v1.x patch 분류
로 편입.)

### 7.3 Phase C 진입 후 신규 unresolved

PR-5a~d 진입 직후 결정:

1. **Microbench measurement** — S15.5 dwell 10 ms / S20.5 latency budget
   (10 ms suspend / 5 ms wake) / S22 spurious threshold 의 실제 microbench
   측정.  Phase C 종반에 `qemu-smoke` + capsule cluster + KernelDebugBuild=ON
   환경에서 측정 → G7 timing-equal 의 input + paper artifact 의 evaluation
   data.
2. **`tools/power.rules.d/` syntax 표준** — overlay merge 의 정확한
   grammar 결정 (TOML / INI / 자체 lu-rule syntax 의 어느 것).  logicutils
   의 기존 `lu-rule` syntax (`boot/x86_64-debug.rules`) 와 정합 검토 후
   결정.  M3/M4 mechanism 명시 후 syntax 분리.
3. **Vehicle bus signal driver 위치** — S22.6 / M7 의 CAN / OBD-II /
   vehicle Ethernet AVB driver 가 `y4-drivers` repo 에 추가될지, `wakeup`
   capsule 안에 통합될지.  Phase C 진입 후 결정 — 주류 옵션은 y4-drivers
   sibling repo (transportation form-factor 의 기본 driver set) + wakeup
   capsule 의 abstract signal API 분리.
4. **Plundervolt 외 mitigation** — S23.4 voltage range 외 추가 mitigation
   검토:
   - SGX/TZ enclave 측의 fault detection (Y4 의 Phase D enclave 도입 시
     통합)
   - voltage glitch detection (hardware-side, vendor-specific)
   - 추가 audit/anomaly rule (기존 anomaly rule 보강)

### 7.4 Phase D 진입 시 검토 영역

`docs/vmm_arch.md` §8.8 + 본 doc 의 forward-compat hook 들 — Phase D
진입 시 spec patch 로 추가:

1. **AV28-D body** — IOMMU programming capsule + per-device BAR cap
   도입 후 `wake_source_iommu_consistent` invariant body 채움 (forward-
   compat hook 활성화, §6.4 v1.x patch).
2. **per-capsule restart policy** (vmm_arch §8.4 정합) — power capsule
   (cpufreq / acpi-pm / psp-pm / rapl / wakeup / lease-pm) 도 포함, fault
   recovery 의 cluster revoke vs partial restart 결정.  v1.0 default =
   cluster revoke (vmm_arch §2.5 정합).
3. **Disk-backed audit persistence** (S12.4 forward-compat hook) —
   power op_tag (30+ 추가, §6.2 cross-cluster) 도 disk dump 대상에 포함.
4. **R-α / R-γ 의 wake source 정합** — Phase D 의 KVM ioctl 프록시 /
   paravirt agent 가 nested guest 의 wake event 도 처리.  guest-안-VM
   의 wake source 가 host 의 wake-from-deep-idle 과 정합.
5. **PCIe device passthrough + power management** — Phase D 의 IOMMU +
   per-device BAR 가 own driver guest 의 wake source / power state 와
   정합 (S22.7 USB wake / device wake 의 Phase D 활성화 path).
6. **Hardware enclave (SGX / TZ Realm) 도입 시 Plundervolt 의 secondary
   mitigation** — enclave 측 fault detection + Y4 측 voltage range
   bound (S23.4) 의 보안 layer 통합.

### 7.5 v2 (incompatible) 후보

frozen 후 v2 (incompatible) 단계에서 검토할 변경:

1. **ModeSignal namespace 변경** — string-keyed → typed enum (universal
   customizability 약화 시 검토).  paper review 또는 산업 도입 피드백
   에서 string-keyed 의 type-safety 부족이 issue 될 시 재검토.
2. **`tools/power.rules.d/` overlay merge mechanism 변경** — numbered
   file overlay → JSON / TOML database 등의 structured config.  logicutils
   진화 또는 별도 config tool 도입 시.
3. **4-tier secure storage trust model 변경** — Tier 분리 책임 재구성
   (예: hardware 자체 integrity 만 신뢰, software MAC 폐지 / 또는 모든
   tier 가 software MAC 추가).  암호 primitive 진화 (post-quantum 등)
   시 재검토.

### 7.6 Cross-cluster capsule API 의존 ledger

본 doc 의 power capsule 들이 amdv 측 capsule (vmm_arch.md §2.1) 에
의존하는 API surface 명시 — 본 의존은 amdv / vmm_arch 의 v1.x patch
형태로 (4 doc frozen 변경 0):

| 의존 capsule (amdv 측) | 추가 API surface | 짝 sub-decision |
|---|---|---|
| `audit` (S12) | power op_tag 30+ enum 확장: `PStateChange` / `PStateRateLimited` (S15) / `CStateTransition` (S16) / `RaplRead` / `EnergyBudgetSet` / `EnergyBudgetExceeded` (S17) / `AcpiMethodEval` (read-only/timeout/rejected/SxStateEnter/TableTampered, 6) (S18) / `SmtPairSync` / `SmtPairDesyncAttempt` / `SmtThreadOffline` / `SmtRuntimeToggle` / `SmtForceToggle` / `SmtAllowMixedActive` (S19) / `LeaseSuspend*` 8 + `LeaseWakeIntegrityFail.reason` (S20) / `ThermalThresholdReached` / `HardlimitReached` / `CoolingComplete` / `ConfigChange` / `ForceToggle` / `MsrAccessDenied` (S21) / `WakeEventReceived` / `SpuriousWakeDetected` / `WakeSourceConfigChange` / `MagicPacketRejected` / `WakeRouting` / `WakeForceToggle` (S22) / `Mailbox*` 5 (S23) / `TpmDetected` / `TpmAbsent` / `TpmPcrMismatch` (S20.2.7) / `SubModeTransition` / `DeprecatedFormFactorAlias` (M8) | S15-S23 의 audit sub-decision + M8 |
| `lifecycle` | `pause_all_vcpus()` (S21.8 thermal hardlimit) + `revoke_lease(lease_id)` (S20.4 wake integrity fail / S18.8 ACPI table tampered / S22.6 magic packet replay) + sub-mode transition hook (S20 패턴 정합) | S20 / S18 / S21 / S22 |
| `firmware-approval` (S14) | `FirmwareOp::MailboxOperation { vendor, target, opcode, payload, class }` enum variant 추가 | S23.8 single source of truth |
| `npt` (S3) | mailbox MMIO range 의 NPT mapping 거부 (`is_in_mailbox_mmio_range(host_pa)` predicate) | S23.5 |
| `msr-bitmap` (S10.1) | mandatory entry 30+ 항목 추가 (P-state MSR 6 / C-state residency MSR 8 / RAPL MSR 18 / thermal MSR 7 / voltage MSR 4 / SMU vendor MSR) | S15.2 / S16.4 / S17.1 / S21.1 / S23.9 |
| `io-bitmap` (S11) | mailbox port (PCH PMC port range) default-block 추가 | S23.5 |
| `cpuid-emul` (S2 의 sub) | power feature bit 마스킹 추가 (CPUID 0x6 thermal 6 / 0x15 + 0x16 frequency / 0x80000007 power feature / 0x80000008 RAPL2 등) | S15.3 / S17.9 / S21.11 / S23.6 |

본 의존 갱신은 amdv_safety.md / vmm_arch.md 의 v1.x patch (§7.4 정합) —
4 doc frozen 변경 0.

### 7.7 frozen 후 즉시 작업 항목

본 doc + 짝 doc `power_arch.md` 양쪽 v1.0 frozen 후 즉시 시작:

1. `tools/power.rules.d/` directory 신설 + 8 default file 작성:
   - `00-default-server-farm.rules`
   - `00-default-rack-mount.rules`
   - `00-default-mobile.rules` + `00-default-mobile-{dock,portable,
     transportation}.rules` (4 파일)
   - `00-default-soc.rules`
   - `00-default-certified.rules`
   - `00-default-aliases.rules` (M8 deprecated alias)
   - `00-default-detection.rules` (§2.5 detection rule)
2. `Y4/power-orchestrator/` workspace member scaffold (Cargo.toml 만,
   body 는 PR-5a — `Y4_PM_ORCHESTRATOR_LOC_BUDGET = 600` 적용 + LoC 검사
   CI hook).
3. `Y4/capsules/pm-{cpufreq,acpi-pm,psp-pm,rapl,wakeup,lease-pm}/` 6
   sub-crate scaffold (각 capsule 의 lib.rs + Cargo.toml + 빈 module
   tree, body 는 PR-5b~d).
