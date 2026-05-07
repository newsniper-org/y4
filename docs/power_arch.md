<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Power Management 아키텍처 — capsule-decomposed

> **상태:** **v1.0 frozen** (2026-05-07, Phase 4-power 일괄 마킹).
> 12 axis 핵심 결정 + 6 capsule + 1 orchestrator 분해 + lease integration
> + PR split + 23 workspace member repo 구조 + 학술적/산업적 차별점
> (logicutils ALP+CLP+Type Relations 통합 포함) 모두 sign-off
> (§7.3 P-arch sub-decision ledger).  짝 doc = `docs/power_safety.md`
> v1.0 frozen.  cpu_virt_compat 측 vendor-neutrality 정책과 정합.
>
> 이전 record: v0 design draft 진입 (2026-05-05) — ARCH-II' (vmm_arch.md)
> 의 capsule 분해 패턴을 power domain 에 적용.  6 capsule + 1
> orchestrator.

본 문서는 Y4 의 power management capsule cluster 디자인.  `power_safety.md`
의 S15+ 안전장치들이 *어디 (어느 capsule) 에* 구현되며 *어떻게* 격리되는지
정의.

---

## 1. 핵심 결정

| 축 | 결정 |
|---|---|
| Base | seL4 + ARCH-II' (vmm_arch.md v1.0 frozen) — 변경 없음 |
| **Power-mgr 위치** | **Y4 의 capsules 패턴 안 — 6 capsule + 1 orchestrator** (cpufreq / acpi-pm / psp-pm / rapl / wakeup / lease-pm — `power_safety.md` §5 cross-ref) |
| Threat model | power-mgr core (orchestrator) trusted, intercept handler 들 capsule 격리.  vmm_arch.md §2.3 의 trust model 패턴 재사용.  **Power-specific threat catalog**: `power_safety.md` §1.1 의 4-tier threat actor + §1.2 의 12 항목 catalog (Side-channel 7 / Direct attack 5 / DoS 4) cross-ref |
| 검증 도구 | Verus (**AV21~AV40**: 활성 15 + Phase D forward-compat 1 + reserved 5) — vmm_arch.md 와 같은 chain (`proofs/verus/src/power/{upper,lower}/`) |
| 검증 기법 | VeriSMo 2-layer concurrency 패턴 (vmm_arch.md §3.1 정합) |
| 형상별 정책 | logicutils `tools/power.rules.d/` overlay merge (`power_safety.md` §2.3) |
| **Customizability** | Y4 가 ship 하는 form-factor / sub-mode 도 **default definitions** (built-in 아님), user override / removal 가능 (`tools/power.rules.d/` numbered file overlay merge — NN > 00 우선).  default 와 user-defined 의 mechanism 동일.  `power_safety.md` §2.1.3 cross-ref |
| **Secure storage** | **ISA-agnostic 4-tier** — Tier 1 (CPU hardware: PSP/TXT/SEV-SNP/TDX/TZ/CCA/PEF/SE/PMP+Keystone/CoVE + fTPM Tier 1 sub-case) / Tier 1.5 (외장 dTPM 2.0 + XChaCha20-Poly1305 master key seal + PCR 0+1+2+3+7 binding + session encryption) / Tier 2 (XChaCha20-Poly1305 sealed DRAM universal fallback) / Tier 3 (suspend 거부).  `power_safety.md` §S20.2 cross-ref |
| **Cross-cluster capsule reuse** | **amdv 측 7 capsule 재사용** (audit / lifecycle / firmware-approval / npt / msr-bitmap / io-bitmap / cpuid-emul) — power 측 추가 capsule 신설 0, surface 확장은 amdv 측 v1.x patch 형태.  `power_safety.md` §7.6 cross-ref |
| seL4 인터페이스 | **D1a' patch series** (`third_party/sel4-patches/100-power-*.patch`) — PR-1 의 후속 patch series, 동일 mainline submission timing.  power MSR mediation + ACPI eval mediation + SMI handling.  `sel4_fork_policy.md` §3 패치 형식 + §6.3 numbering + Strictly Additive Fork 정합 |
| 라이선스 | Apache-2.0 (Y4 single-license).  참조 자료는 §1.1 표 |
| 학술적 차별점 | §6 cross-ref (4 항목, sign-off 후 갱신 시 본 row 도 따라감) |

### 1.1 참조 자료 통합 표 (E=a, vmm_arch §3.3 패턴 정합)

power 측의 모든 외부 참조 자료 ledger.  코드 import 0 또는 algorithmic
port + attribution 보존:

| 자료 | 출처 / venue | 라이선스 | 차용 형태 | 위치 |
|---|---|---|---|---|
| **Linux cpufreq / cpuidle / thermal / RAPL driver** | `linux/drivers/cpufreq/`, `cpuidle/`, `thermal/`, `powercap/` | GPL-2 | **알고리즘 reference only, code import 0** (Apache-2 X GPL 호환 X) | NOTICE Acknowledgements (Methodology Inspiration), `~/y4-upstream-refs/linux/` (read-only) |
| **FreeBSD cpufreq / acpi** | `freebsd/sys/dev/cpufreq/`, `acpica/` | BSD-2 | algorithmic port + attribution 보존 | NOTICE reuse manifest, `~/y4-upstream-refs/freebsd/` |
| **DragonFly powerd** | DragonFlyBSD `usr.sbin/powerd/` | BSD-3 | algorithmic port (DragonFly LWKT 와 짝, vmm_arch §3.3 정합) | NOTICE reuse manifest, `~/y4-upstream-refs/dragonfly/` |
| **ARM TrustZone TF-A (Trusted Firmware-A)** | `trusted-firmware-a` | BSD-3 | **PSCI (Power State Coordination Interface) reference** + TBBR (Trusted Board Boot Requirements) + TZ-fTPM reference (S20.2 Tier 1 의 fTPM sub-case) | NOTICE reuse manifest |
| **tss-esapi crate** | TCG TSS 2.0 ESAPI binding (Rust) | BSD-2 | cargo dep (Tier 1.5 dTPM backend, P3.6 §3.3 정합) | NOTICE reuse manifest, `Cargo.toml` |
| **Plundervolt paper** (USENIX Security '20) | Murdock et al. | (paper) | mitigation reference (S23.4 voltage range bound) | NOTICE Acknowledgements (Methodology Inspiration) |
| **PLATYPUS paper** (USENIX Security '21) | Lipp et al. | (paper) | mitigation reference (S17 RAPL 격리) | NOTICE Acknowledgements |
| **Hertzbleed paper** (USENIX Security '22) | Wang et al. | (paper) | mitigation reference (S15 cpufreq 격리) | NOTICE Acknowledgements |

---

## 2. Capsule 분해 — 6 개 capsule + 1 orchestrator

### 2.1 Capsule 목록 (vmm_arch §2.1 패턴 정합 6-column)

| # | Capsule | 책임 (1-line summary) | 1차 안전장치 | 협력 amdv 측 capsule (cross-cluster) | 관련 AV | Proof file |
|---|---------|----|---|---|---|---|
| 1 | **cpufreq** | P-state governor + DVFS dwell + C-state state machine + SMT pair power state 동기 + thermal cap 적용 + hardware SMT enable/disable | S15, S16, S19, S21 (P-state cap) | msr-bitmap (P-state/C-state/thermal MSR 차단) + cpuid-emul (frequency feature mask) + audit + lifecycle (S5 SMT grouping) | AV30 / AV31 | `lower/dvfs_dwell.rs` + `lower/smt_sync.rs` |
| 2 | **acpi-pm** | per-VM virtual DSDT + AML interpreter mediation + _TCC / _PSV thermal threshold + Sx state forward + ACPI table SHA-256 integrity check | S18, S21 (thermal threshold) | firmware-approval (ACPI table tamper revoke) + audit + cpuid-emul | AV27 / AV32 | `upper/thermal_hardlimit.rs` + `upper/acpi_integrity.rs` |
| 3 | **psp-pm** | vendor 별 9-platform mailbox backend + voltage range 검증 + Plundervolt mitigation + 3-등급 분류 dispatch | S23 | firmware-approval (`MailboxOperation` entry) + msr-bitmap (voltage MSR 차단) + cpuid-emul (power feature mask) + npt (mailbox MMIO 거부) + io-bitmap (mailbox port 차단) + audit | AV25 | `upper/voltage_range.rs` |
| 4 | **rapl** | RAPL MSR mediation + virtio-rapl paravirt + 4-bit LSB noise + per-VM energy budget + RAPL ↔ cpufreq internal channel | S17 | msr-bitmap (RAPL MSR 차단) + audit (`RaplRead` / `EnergyBudget*`) + cpuid-emul (RAPL feature mask) + lifecycle (vmrun gate) | AV29 | `upper/rapl_budget.rs` |
| 5 | **wakeup** | wake source whitelist + lease binding + spurious detection + 6-tier priority + Y4-defined cryptographic signed magic packet + virtio-rtc + sub-mode signal hub | S22, M7 (vehicle bus / GPS detection) | lease-pm (S20.8 wake hook) + acpi-pm (_PRW mediation) + audit | AV26 / AV28 / AV28-D | `upper/wake_whitelist.rs` + `upper/magic_packet.rs` |
| 6 | **lease-pm** | C3 trigger + ISA-agnostic 4-tier secure storage + XChaCha20-Poly1305 AEAD + replay protection epoch + atomicity (S13.2 패턴) + form-factor 별 suspend 정책 | S20, S20.2 (Tier 1.5 dTPM) | wakeup (S20.8 wake hook source) + lifecycle (lease revoke chain) + audit (`LeaseSuspend*` 8 op_tag) + firmware-approval (TPM PCR mismatch revoke) | AV21 / AV33 | `lower/tpm_consistency.rs` + `upper/wake_epoch.rs` |

### 2.2 Power Orchestrator (trusted core)

`y4-power-orchestrator` — workspace member, 가장 작은 trusted 구성 요소.

책임:
- C-state 진입/이탈 결정 (어느 CPU 가 어떤 C-state)
- form-factor profile 적용 (logicutils `tools/power.rules.d/` overlay
  merge 결과의 `Y4_PM_*` const)
- 6 capsule cluster 의 dispatch hub
- vmrun-orchestrator (ARCH-II') 와의 협력 — vCPU 의 deep idle 진입은
  power-orchestrator 에 위임
- Sub-mode transition hub (§2.4 ModeSignal — String-keyed namespace 의
  lookup + atomicity)
- 5 force-toggle 의 master state (SMT / suspend / thermal / wake /
  mailbox — `power_safety.md` S19.6 / S20.9 / S21.9 / S22.10 / S23.10)

#### 2.2.1 LoC budget (B=a)

`Y4_PM_ORCHESTRATOR_LOC_BUDGET` build-time const, **default 600 LoC**.
CI 자동 검사 — line count (주석 / 빈줄 / test 제외) 가 budget 초과 시
build fail.  vmm_arch §2.2.1 패턴 정합 (`Y4_AMDV_ORCHESTRATOR_LOC_BUDGET
= 800`) 와 **별도** budget:

| Trusted core | LoC budget |
|---|---|
| `vmrun-orchestrator` (ARCH-II') | 800 |
| `power-orchestrator` (본 doc) | 600 |
| **합계 trusted core** | **1400 LoC** (검증 표면 정량화) |

#### 2.2.2 IPC 인터페이스 추상 형태 (C=a, vmm_arch §2.2.2 패턴)

orchestrator ↔ 6 capsule 의 모든 통신은 message-typed IPC.  vmm_arch
§8.1 의 `CapsuleMsg` enum 과 **별도 namespace** (`PmCapsuleMsg`):

```rust
enum PmCapsuleMsg {
    // cpufreq capsule (5 verb)
    CpufreqSetPstate(LogicalApicId, PState),
    CpufreqEnterCstate(LogicalApicId, CState, BreakEnable),
    CpufreqQueryDwell(LogicalApicId),
    CpufreqSmtPairSync(SmtPair, PState, CState),
    CpufreqSetGovernor(GovernorKind),

    // acpi-pm capsule (6 verb)
    AcpiPmEvalMethod(VmId, MethodName, Args),
    AcpiPmSetThermalThreshold(ThresholdKind, Celsius),
    AcpiPmSxEnter(VmId, SxState),
    AcpiPmTableHashCheck(TableName),
    AcpiPmVirtioDsdtBuild(VmId),
    AcpiPmRoutePrwEvent(VmId, PrwEvent),

    // psp-pm capsule (4 verb)
    PspPmDispatchMailbox(MailboxOp),
    PspPmValidateVoltageRange(VoltageDelta),
    PspPmDryRun(MailboxOp),
    PspPmReadMasked(MailboxTarget),

    // rapl capsule (5 verb)
    RaplReadEnergy(RaplDomain),
    RaplVirtioGetEnergy(VmId, RaplDomain),
    RaplSetBudget(VmId, EnergyBudget),
    RaplCheckBudget(VmId),
    RaplInternalThermalChannel(ThermalCapHint),

    // wakeup capsule (8 verb)
    WakeupRegisterSource(VmId, WakeSource),
    WakeupRemoveSource(VmId, WakeSourceId),
    WakeupOnEvent(WakeEvent),
    WakeupVerifyMagicPacket(MagicPacket),
    WakeupSetForceToggle(WakeForce),
    WakeupModeSignalSubMode(SubModeName),
    WakeupModeSignalDockDetect(DockState),
    WakeupModeSignalVehicleBus(VehicleBusState),

    // lease-pm capsule (6 verb)
    LeasePmSuspend(VmId, CState),
    LeasePmWake(VmId, SealedHandle),
    LeasePmSetForceToggle(SuspendForce),
    LeasePmTpmDetect(),
    LeasePmEpochQuery(VmId),
    LeasePmRevokeAll(),
}
```

총 **34 verb** (5+6+4+5+8+6).  concrete msgport msg type / scheme verb
encoding 은 §8 unresolved 의 추후 결정 (vmm_arch §8.1 결정 패턴 정합 —
msgport enum primary + scheme verb fallback debug-only).

### 2.3 vmrun-orchestrator 와의 협력

- vCPU 가 `HLT` 발화 (S2 의 INTERCEPT_HLT 가 mandatory) → vmexit →
  vmrun-orchestrator 가 power-orchestrator 에 IPC ("vCPU N idle, suggest
  deep idle")
- power-orchestrator 가 form-factor profile 에 따라 결정 — mobile-portable
  형상은 deep C-state 진입 권고, server-farm 은 C1 max
- deep idle 진입 시 lease-pm capsule 이 lease state suspend (S20)
- `pause_all_vcpus` (S21.8 thermal hardlimit) / `revoke_lease` (S20.4
  wake integrity fail) 콜백은 lifecycle capsule (cross-cluster) 사용 —
  vmrun-orchestrator 영역과 정합

### 2.4 Capsule trust model — power-specific cross-cluster 의존 (D=a)

vmm_arch §2.3 정합 + power-specific 차이:

| 주체 | trust 등급 | 격리 보장 |
|---|---|---|
| seL4 microkernel + D1a + D1a' | **fully trusted** | TCB 본체 |
| `power-orchestrator` | **fully trusted** | TCB 보조, 600 LoC budget |
| `vmrun-orchestrator` (vmm_arch) | **fully trusted** | TCB 보조, 800 LoC budget |
| 6 power capsule | **cluster-scope-trusted** | 자기 cluster 안에서 신뢰, 다른 cluster / tenant 에 대해서는 격리 invariant 강제 |
| 7 amdv 측 capsule (cross-cluster reuse) | **cluster-scope-trusted** | amdv 측 v1.0 frozen 의 trust scope 의존 |
| guest | **untrusted** | ACPI eval / MSR access 모두 capsule 측 input validation |

#### Cross-cluster trust 의존

power capsule 들은 amdv 측 cluster 의 trust scope 안에서만 작동.  cross-
cluster API 호출 시 amdv 측 cluster-scope-trusted 가정 의존.  boundary
check Verus invariant:

```
forall pm_capsule, amdv_capsule: cross_cluster_call(pm_capsule, amdv_capsule)
    ⟹ amdv_cluster_trusted(amdv_capsule) ∧
       caps_compatible(pm_capsule.lease, amdv_capsule.lease)
```

amdv 측 cluster 가 lease revoke 시 power 측도 자동 revoke (lifecycle
capsule chain) — cross-cluster lifetime 정합 (vmm_arch §2.5 + S13 정합).

### 2.5 Capsule 의존 그래프 — DAG 보장 (E=a, cross-cluster sink 갱신)

```
                    power-orchestrator
                          │  (single source — 6 power capsule cap 분배)
       ┌────────┬─────────┼──────────┬─────────┐
       ▼        ▼         ▼          ▼         ▼          ▼
    cpufreq  acpi-pm   psp-pm     rapl      wakeup    lease-pm
       │        │         │          │         │          │
       │        └─────────┼──────────┘         │          │
       │       horizontal │ 0 (DAG 보장)       │          │
       │                  │                    │          │
       │  uses amdv API   │  uses amdv API     │          │
       │  (read-only      │  (cross-cluster    │          │
       │   surface)       │   call)            │          │
       ▼                  ▼                    ▼          ▼
  ┌────────────────────────────────────────────────────────────────┐
  │ amdv 측 cross-cluster sink (vmm_arch §2.4, 본 doc cross-cluster) │
  │                                                                 │
  │  audit (S12)         — power op_tag 30+ 추가 (v1.x patch)       │
  │  lifecycle           — pause_all_vcpus / revoke_lease /         │
  │                        sub-mode transition hook                 │
  │  firmware-approval   — MailboxOperation entry variant 추가      │
  │                                                                 │
  │  npt                 — mailbox MMIO 거부 (S23.5 — psp-pm 측)    │
  │  msr-bitmap          — power MSR 30+ entry (S15/S16/S17/S21/   │
  │                        S23 — cpufreq/rapl/psp-pm 측)            │
  │  io-bitmap           — mailbox port 차단 (S23.5 — psp-pm 측)    │
  │  cpuid-emul          — power feature mask (S15.3/S17.9/S21.11/  │
  │                        S23.6 — 모든 power capsule 측)            │
  └────────────────────────────────────────────────────────────────┘
```

**의존 방향 명시:**
- power-orchestrator → 6 power capsule (single direction)
- 6 power capsule 의 horizontal 의존 0 (DAG 보장)
- 6 power capsule → amdv 측 7 capsule (cross-cluster API call) — power
  → amdv 단방향, amdv 측은 power 의존 0
- amdv 측 sink-only (audit / lifecycle / firmware-approval) 은 sink, 다른
  capsule 의존 X
- amdv 측 surface providers (npt / msr-bitmap / io-bitmap / cpuid-emul)
  은 power capsule 의 boundary check 책임 (entry 추가는 v1.x patch)

Verus invariant `power_dag_acyclic` (P-arch.7 sign-off 시 §4 의 PR split
표 정합) — 본 그래프의 acyclicity 가 power-orchestrator 의 cap 분배 +
cross-cluster 의 단방향성으로 inductive 닫힘.

### 2.6 Capsule fault 시 거동 (F=a)

**v1.0 default: capsule fault → cluster 전체 lease revoke** (vmm_arch
§2.5 정합, lifecycle capsule chain).

**Phase D forward-compat — per-capsule fallback 정책 후보:**

Phase D 의 per-capsule restart policy (vmm_arch §8.4) 도입 시 power
capsule 별 fallback 검토:

| Capsule | v1.0 default | Phase D 후보 fallback (recoverable case) |
|---|---|---|
| `cpufreq` | cluster revoke | conservative P-state (lowest non-idle) + audit, fault 원인 분석 후 restart |
| `acpi-pm` | cluster revoke | safe-method-only mode (read-only method 만 허용) + virtio-DSDT 재구축 |
| `psp-pm` | cluster revoke | mailbox dispatch disable + mailbox MSR 차단 강화 (모든 op reject) |
| `rapl` | cluster revoke | audit only (RAPL read 차단, 단 budget enforcement 는 software cache 로 유지) |
| `wakeup` | cluster revoke | inhibit-all wake (S22.10 force-toggle InhibitAll 강제) |
| `lease-pm` | cluster revoke | **non-recoverable** (secret state 손상 가능 → 항상 cluster revoke) |

`lease-pm` 만 항상 cluster revoke (Phase D 도) — secret state contamination
가능성으로 fail-safe 강제.  나머지 5 capsule 은 Phase D 에서 partial
restart 가능성 검토.

### 2.7 Capsule cluster scope (G=a, host-global vs per-VM)

vmm_arch §2.6 패턴 + power-specific (hardware sharing 본성):

| Capsule | Instance scope | 근거 |
|---|---|---|
| `cpufreq` | **host-global 1 instance** | CPU hardware (P-state / C-state) 가 host 전체 공유 |
| `acpi-pm` | **host-global 1 instance** | ACPI table / DSDT 가 host 전체.  per-VM virtual DSDT 는 capsule 안의 partition |
| `psp-pm` | **host-global 1 instance** | PSP / PCH mailbox 가 host hardware |
| `rapl` | **host-global 1 instance** | RAPL counter MSR 이 host 전체.  per-VM energy budget tracker 는 capsule 안 partition |
| `wakeup` | **host-global 1 instance** | wake source hardware (NIC / GPE / IO-APIC) 가 host 전체.  per-VM wake source binding 은 capsule 안 partition |
| `lease-pm` | **per-VM 1 instance** | lease 별 secret state (XChaCha20 master key + segment key + epoch counter) — vmm_arch §2.6 의 `vmcb` capsule 의 per-vCPU 와 동일 패턴 |
| `power-orchestrator` | **host-global 1 instance** | trusted core, vmrun-orchestrator 와 짝 |

총 **6 host-global + 1 per-VM** — power 의 hardware sharing 본성 반영.
ARCH-II' 의 "per-VM 9 + per-vCPU 1 (vmcb)" 와 비대칭 (power 측은 hardware
가 lease 사이 share 됨, capsule 안 partition 으로 isolation).

### 2.8 Lease 발급 시 power capsule cap 분배 (H=a, vmm_arch §2.3 LeaseCap 확장)

vmm_arch.md §2.3 의 `LeaseCap` struct 확장 — power 측 cap 추가 row.
본 변경은 vmm_arch.md / amdv_safety.md 의 **v1.x patch** (§6.4 정합) —
4 doc frozen 변경 0:

```rust
struct LeaseCap {
    // ... 기존 vmm_arch §2.3 필드 (partition_id, key, nonce, ...)

    // ... ARCH-II' 의 11 capsule cap (vmm_arch §2.3 그대로)
    vmcb_caps:              Vec<Cap<VmcbCapsule>>,        // per-vCPU
    npt_cap:                Cap<NptCapsule>,
    msr_bitmap_cap:         Cap<MsrBitmapCapsule>,
    io_bitmap_cap:          Cap<IoBitmapCapsule>,
    cpuid_emul_cap:         Cap<CpuidEmulCapsule>,
    npf_handler_cap:        Cap<NpfHandlerCapsule>,
    firmware_approval_cap:  Cap<FirmwareApprovalCapsule>,
    nested_request_cap:     Cap<NestedRequestCapsule>,
    audit_cap:              Cap<AuditCapsule>,
    lifecycle_cap:          Cap<LifecycleCapsule>,

    // 신규 — power 측 cap (P-arch.2 H=a)
    pm_cpufreq_cap:         Cap<CpufreqCapsule>,          // host-global ref
    pm_acpi_cap:            Cap<AcpiPmCapsule>,           // host-global ref
    pm_psp_cap:             Cap<PspPmCapsule>,            // host-global ref
    pm_rapl_cap:            Cap<RaplCapsule>,             // host-global ref
                                                          //   (per-VM tracker
                                                          //    는 capsule 안 partition)
    pm_wakeup_cap:          Cap<WakeupCapsule>,           // host-global ref
    pm_lease_cap:           Cap<LeasePmCapsule>,          // per-VM instance
}
```

**의미:**
- host-global capsule 5 개 (cpufreq / acpi-pm / psp-pm / rapl / wakeup)
  의 cap 은 **모든 lease 가 동일 capsule reference 를 보유** — capsule
  자체가 partition 으로 lease 사이 isolation 책임 (예: rapl capsule 의
  per-VM energy tracker)
- per-VM capsule 1 개 (lease-pm) 만 lease 별 fresh instance — lease
  발급 시 신규 capsule instantiation, lease 회수 시 destroy
- cap revoke chain (S13) 에서 per-VM `pm_lease_cap` 만 destroy 진행,
  host-global cap 들은 capsule 안 partition cleanup (per-VM state
  zeroize) 만 진행

**Lease 발급 시 흐름:**
```
1. lease 발급 요청 → lifecycle capsule
2. host-global cap 5 (cpufreq/acpi-pm/psp-pm/rapl/wakeup) 의 reference 부여
3. host-global capsule 들이 per-VM partition 신설 (rapl tracker, wakeup
   binding, acpi-pm virtual DSDT 등)
4. per-VM lease-pm capsule instantiation (XChaCha20 master key + epoch
   counter)
5. LeaseCap struct 채움 → caller 에 반환
```

**Lease 회수 시 흐름:**
```
1. lifecycle capsule 의 LeaseRevoke (S13.3 master sequence)
2. lease-pm capsule destroy (XChaCha20 key zeroize, epoch erase)
3. host-global capsule 의 per-VM partition cleanup (state zeroize)
4. amdv 측 capsule chain (vmm_arch §2.4 + S13.1 reverse order) 진행
5. lease 의 모든 cap revoke
```

---

## 3. Lease 와 power 의 정합

### 3.1 Deep idle 진입 시 lease state — 4-tier secure storage (A=a)

`lease-pm` capsule 의 책임 — power_safety §S20.2 의 ISA-agnostic 4-tier
와 정합.  C-state ≥ C3 진입 시 (S20.1 trigger) seal/unseal 4-step flow:

```
1. (Suspend) lease 의 secret state (XChaCha20 master key + HKDF segment
   key + audit ring buffer key) 를 secure storage 로 sealing
   ├── Tier 1 (CPU hardware): PSP-protected SRAM / TXT / SEV-SNP / TDX /
   │       TZ Secure World / CCA Realm / PEF / IBM Z SE / PMP+Keystone
   │       / CoVE — fTPM 도 Tier 1 sub-case
   ├── Tier 1.5 (외장 dTPM 2.0): master key 를 PCR 0+1+2+3+7 binding seal,
   │       AEAD ciphertext 는 DRAM, session encryption (HMAC + AES-CFB)
   ├── Tier 2 (XChaCha20-Poly1305 sealed DRAM, universal fallback): boot-
   │       time secret + per-lease nonce HKDF-Expand 의 master key
   └── Tier 3 (suspend 거부): suspend 자체 발화 X
2. (Suspend atomicity) 모든 vCPU vmexit 후 진행 — S20.3 의 mid-vmrun race
   resolve (S13.2 패턴), retry 임계 10 회 = 10 ms
3. (Wake) C-state 이탈 시 wake source 도착 (wakeup capsule, S22) →
   power-orchestrator → lease-pm 의 wake hook
4. (Wake integrity check) Tier 별:
   - Tier 1: hardware-attested integrity 사용 (SEV-SNP RMP / TDX MKTME-i
     / SGX MAC tree / TZ secure memory / etc. — 추가 software MAC X)
   - Tier 1.5: TPM unseal (PCR match 시에만) + XChaCha20-Poly1305
     Poly1305 tag 검증
   - Tier 2: XChaCha20-Poly1305 Poly1305 tag 검증
   replay protection: AEAD additional data = epoch counter, lease-pm
   internal monotonic counter 와 비교 (S20.4.2)
   integrity / replay fail → lease revoke (lifecycle capsule chain)
```

cross-ref: AV21 (`tpm_pcr_consistency`) + AV33 (`wake_epoch_monotonic`).

### 3.2 Form-factor 별 suspend 정책 (B=a, Mobile merger 정합)

power_safety §2.1 의 4 default form-factor + mobile sub-mode + certified
overlay 정합:

| Form factor | sub-mode | deep idle 시 lease 거동 |
|---|---|---|
| **server-farm** | (sub-mode 없음) | suspend X (latency tail 차단) |
| **rack-mount** | (sub-mode 없음) | C1 까지만, suspend X |
| **mobile** | dock | C2 까지, suspend OFF (docked, throughput 우선) |
| **mobile** | portable (default) | C6 deep idle, **suspend ON** (lease state Tier 1/1.5/2 보호) |
| **mobile** | transportation | **§3.3 본문 — sudden power loss 강화** |
| **임베디드 SoC** | (platform-별) | always-on 또는 duty-cycle 의 idle 구간 suspend |
| **certified overlay** (모든 form-factor) | — | TPM 의무 (`Y4_PM_TPM_REQUIRED=on`) — Tier 1 또는 Tier 1.5 강제, Tier 2 fallback 시 boot fail.  voltage offset ±0 mV (S23.4) 강제 |

### 3.3 Transportation sub-mode 의 sudden power loss 대비 강화 (C=a)

차량 / 철도 / 항공 환경의 특수 위협 — 운전자 ignition off, 차량 사고,
충격 / 진동 으로 인한 sudden power loss + dTPM bus contact 신뢰성 ↓.

**강화 정책:**

| 항목 | 일반 mobile-portable | **transportation 강화** |
|---|---|---|
| Suspend latency budget | `Y4_PM_SUSPEND_LATENCY_NS` 10 ms | **5 ms** (절반, ignition off race 대비) |
| Mid-vmrun race retry 임계 | 10 회 × 1 ms | 5 회 × 1 ms (5 ms 한도) |
| TPM 우선순위 | fTPM = dTPM (Tier 1 / 1.5 동등) | **fTPM 우선 강제** (`Y4_PM_TPM_PREFER_FTPM = ON`) — vibration / 캐빈 온도 변동으로 dTPM bus contact / chip 신뢰성 ↓ |
| dTPM fallback 시 | 그대로 진행 | audit Warning `TpmInTransportationFallback` (M9) |
| Wake source | lid-open / power-button / nic / usb / rtc / battery | + **vehicle-bus / gps-movement / cellular** 추가 (M7) |
| Multi-tenant 우선순위 | normal | **LOW** (단일 사용자 — driver / 승객 가정) |
| Lease suspend 적극성 | C6 진입 시 ON | **모든 deep idle 시 ON 적극** + 진입 빈도 ↑ |

**Sudden power loss 차단 invariant — AV24 (`mode_invariant_holds`):**

```
defined("transportation") ⟹ transportation_sudden_power_loss_safe

where:
    transportation_sudden_power_loss_safe :=
        forall lease, t: (current_sub_mode(lease) == "transportation" ∧
                          power_event(t) == SuddenLoss) ⟹
            (suspend_completed_before(t, lease) ∨
             lease_revoked_before(t, lease))
```

power-orchestrator 가 transportation sub-mode 에서 deep idle 진입 시
lease-pm capsule 의 5 ms suspend latency 안에서 sealing 완료 보장 →
sudden power loss 가 capture 가능한 secret state plaintext 0.

cross-ref: AV24 (transportation 정의 시) + AV33 (wake_epoch_monotonic).

### 3.4 Cross-cluster integration (D=a)

`lease-pm` capsule 의 cross-cluster API 의존 (vmm_arch.md §2.4 의
sink-only 그래프 정합):

| amdv 측 capsule | API surface | trigger |
|---|---|---|
| **lifecycle** (cross-cluster) | `revoke_lease(lease_id)` | wake integrity fail (S20.4) / TPM PCR mismatch (S20.2.7) / replay fail (S20.4.2) — lease revoke chain (S13) |
| **audit** (cross-cluster) | `LeaseSuspend*` 8 op_tag entry (S20.10) | suspend 시작 / 완료 / deferred / timeout / wake 시작 / integrity fail / wake timeout / force-toggle 변경 |
| **firmware-approval** (cross-cluster) | TPM PCR mismatch 시 `revoke_all_leases()` trigger (S20.2.7 의 `TpmPcrMismatch` Critical) | dTPM 의 PCR mismatch — measured boot 변조 신호, host integrity 심대 |

cross-ref: AV23 (sub_mode_transition_atomicity) + AV32 (acpi_table_integrity
의 짝 — measured boot 변조 신호 처리 패턴).

### 3.5 Atomicity 책임 분담 (E=a)

suspend / wake 의 atomicity 가 capsule 사이 분담:

| 책임 | 주체 |
|---|---|
| Per-lease epoch counter 관리 (S20.4.2 monotonic increment) | `lease-pm` capsule (per-VM instance, 자체 internal state) |
| Sealed handle 관리 (Tier 별 backend 의 seal/unseal 호출) | `lease-pm` capsule + Tier 별 backend sub-module (S20.2.1) |
| 모든 vCPU vmexit 후만 suspend 진입 (S20.3) | `power-orchestrator` (vmrun-orchestrator 협력 — vCPU running state query) |
| Mid-vmrun race resolve (S20.3 retry 임계) | `lifecycle` capsule (cross-cluster — S13.2 패턴 재사용) |
| Suspend 의 secure_zero (메모리상 plaintext key 제거) | `lease-pm` capsule (Rust `secure_zero` + cache flush) |

**관련 invariant:**
- AV23 (`sub_mode_transition_atomicity`) — sub-mode 전환 시 모든 lease
  suspended 상태에서만 const 변경 적용
- AV33 (`wake_epoch_monotonic`) — `expected_wake_epoch` lease-pm internal
  monotonic counter, suspend 마다 1 증가 + handle.epoch 와 매치 검증

### 3.6 Tier dispatch flow (F=a, S20.2.2 cross-ref)

부팅 시점 platform 감지 → 가장 높은 tier 선택.  S20.2.2 의 `detect_
secure_storage_backend()` 흐름:

```
boot
  ├── (Priority 1) Tier 1 detection
  │     - PSP / TXT / SEV-SNP / TDX / TZ / CCA / PEF / SE / PMP+Keystone /
  │       CoVE / fTPM (PSP-fTPM, ME-fTPM/PTT, TZ-fTPM)
  │     - 감지 시 storage_psp.rs / storage_txt.rs / etc. backend 활성
  │     - audit `TpmDetected { tier: Tier1, mechanism }` (severity Info)
  │
  ├── (Priority 2) Tier 1.5 detection (외장 dTPM 2.0)
  │     - ACPI TPM2 table (TCG ACPI Specification) detection
  │     - device tree 의 /tpm@... node detection (ARM 등)
  │     - TPM CRB / FIFO interface vendor ID 검증
  │     - 감지 시 storage_tpm_aead.rs backend 활성 + session encryption
  │       setup (TQ.3 의무) + PCR policy (TQ.2: PCR 0+1+2+3+7)
  │     - audit `TpmDetected { tier: Tier1_5, mechanism: "dTPM 2.0" }`
  │
  ├── (Priority 3) Tier 2 fallback
  │     - storage_xchacha.rs backend 활성 (universal)
  │     - boot-time secret + per-lease nonce HKDF-Expand
  │     - audit `TpmAbsent { fallback: Tier2 }`
  │
  └── (Tier 3) Tier 1, 1.5, 2 모두 부재 시
        - storage_no_suspend backend 활성
        - lease-pm 의 suspend trigger 자체 발화 X
        - form-factor profile 에서 "no-suspend" 강제
        - audit `TpmAbsent { fallback: Tier3, suspend_disabled: true }`
```

cross-ref: AV21 (`tpm_pcr_consistency` — Tier 1.5 시).

### 3.7 Tier 결정 timing (G=a, boot 1 회 fix)

**Tier 결정 = boot 시점 1 회.**  lease 발급마다 새 tier 선택 X.

근거:
- Y4 measured boot 의 일관성 우선 — boot 시점 platform 감지가 host 의
  capability profile 결정, runtime 변동 X (security posture 일관)
- TPM PCR binding (Tier 1.5) 이 boot measurement 에 binding — boot 후
  변경 시 PCR mismatch 자동
- system-wide tier 가 audit chain 의 boot-time stamp 와 정합

**Lease 발급마다 fresh:**
- per-lease master key (XChaCha20-Poly1305) — `generate_xchacha20_key()`
  매 lease 발급 시 신선
- per-lease nonce (192-bit XChaCha20) — `fresh_nonce()` 매 seal 시 신선
- per-lease epoch counter — lease 발급 시 0 으로 초기화, suspend 마다
  1 증가

cross-ref: AV21 (Tier 1.5 시 boot-time PCR binding 의 일관성).

---

## 4. PR split 매트릭스 (6-column, vmm_arch §4 + power_safety §5 정합)

### 4.1 9 안전장치 + S20.2 sub-row 매핑

| 안전장치 | seL4 측 (D1a') | power-orchestrator | power capsule (본체) | cross-cluster amdv 측 capsule (의존) | sub-PR |
|---|:---:|:---:|---|---|:---:|
| **S15 cpufreq 격리** | ◎ P-state MSR 6 항목 차단 (msr-bitmap entry) | ◎ governor 결정 dispatch + dwell 검증 | **cpufreq** (본체) | msr-bitmap (P-state MSR 6) + cpuid-emul (frequency leaf 0x15/0x16) + audit (PStateChange) | **PR-5c** |
| **S16 C-state side-channel** | ◎ HLT/MWAIT intercept (S2 mandatory v1.x patch) + C-state residency MSR 8 항목 차단 | ◎ C-state 결정 hub | **cpufreq** (본체) + **lease-pm** (S20.8 trigger 짝) | msr-bitmap (residency MSR 8) + audit (CStateTransition) | **PR-5c** |
| **S17 RAPL 격리** | ◎ RAPL MSR 18 항목 차단 (PowerHammer / PLATYPUS mitigation) | △ thermal cap internal channel | **rapl** (본체) | msr-bitmap (RAPL MSR 18) + cpuid-emul (RAPL feature mask) + audit (RaplRead/EnergyBudget*) + lifecycle (vmrun gate) | **PR-5d** |
| **S18 ACPI 검증** | △ ACPI MSR mediation | △ Sx state forward | **acpi-pm** (본체 — AML interpreter mediation + per-VM virtual DSDT + ACPI table SHA-256) | firmware-approval (ACPI table tamper revoke) + audit (AcpiMethodEval 6 op_tag) + cpuid-emul | **PR-5d** |
| **S19 SMT power gating** | ◎ S5 정합 (lifecycle) + APIC 측 | ◎ pair grouping + 3-tier force-toggle | **cpufreq** (본체) + **lifecycle** (cross-cluster, S5 grouping) | lifecycle (S5 SMT-aware + offline re-pin) + audit (SmtPair* 6 op_tag) | **PR-5c** |
| **S20 deep idle lease suspend** | △ HLT path | ◎ entry/exit hub + sub-mode signal | **lease-pm** (본체 — 4-tier secure storage + XChaCha20-Poly1305 AEAD + replay protection) | wakeup (S20.8 wake hook) + lifecycle (lease revoke chain) + audit (LeaseSuspend* 8 op_tag) | **PR-5b** |
| **S20.2 Tier 1.5 dTPM** | — (TPM hardware 자체) | ◎ Tier dispatch (S20.2.2) | **lease-pm** (본체 — `storage_tpm_aead.rs` backend + tss-esapi crate cargo dep + PCR 0+1+2+3+7 binding + session encryption) | firmware-approval (TpmPcrMismatch revoke_all_leases) + audit (TpmDetected/Absent/PcrMismatch) | **PR-5b** |
| **S21 thermal throttle** | ◎ _TCC MSR mediation + IA32_THERM_* 7 항목 차단 | ◎ thermal mode signal hub (state machine) + 3-tier force-toggle | **acpi-pm** + **cpufreq** (state machine) + **lifecycle** (S21.8 hardlimit pause) | msr-bitmap (thermal MSR 7) + cpuid-emul (thermal feature mask 6) + audit (Thermal* 6 op_tag) | **PR-5d** |
| **S22 wake source** | ◎ GPE / IRQ remap | ◎ wake routing + 3-tier force-toggle | **wakeup** (본체 — 6-tier priority + Y4-defined cryptographic signed magic packet + virtio-rtc) | acpi-pm (_PRW mediation) + lease-pm (S20.8 wake hook source) + audit (Wake* 6 op_tag) | **PR-5d** |
| **S23 PSP/PCH mailbox** | ◎ voltage MSR 4 항목 차단 (`IA32_VR_MISC_CTL 0x150` 등 — Plundervolt mitigation) | ◎ S14 firmware-approval forward + 3-tier force-toggle | **psp-pm** (본체 — vendor 별 9-platform mailbox backend) | firmware-approval (`MailboxOperation` enum variant) + msr-bitmap (voltage MSR 4) + cpuid-emul (power feature mask) + npt (mailbox MMIO 거부) + io-bitmap (mailbox port 차단) + audit (Mailbox* 5 op_tag) | **PR-5d** |

표기:
- ◎ = 본체 (primary 책임)
- △ = sub (보조)
- ○ = wrapper (forward only)

### 4.2 Cross-cluster API surface 확장 (amdv 측 v1.x patch)

power_safety §7.6 의 cross-cluster API ledger — amdv 측 capsule 의 추가
surface 는 amdv_safety / vmm_arch 의 v1.x patch 로 진행:

| amdv 측 capsule | 추가 surface | sub-PR (power 측 짝) | 영향 amdv doc |
|---|---|:---:|---|
| `audit` (S12.2) | power op_tag 30+ enum 확장 (PState* / CState* / Rapl* / EnergyBudget* / AcpiMethodEval 6 / SmtPair* 6 / LeaseSuspend* 8 + Reason / Thermal* 6 / Wake* 6 / Mailbox* 5 / Tpm 3 / SubModeTransition / DeprecatedFormFactorAlias) | **PR-5a** (orchestrator + audit hook) | amdv_safety §S12.2 v1.x patch |
| `lifecycle` | `pause_all_vcpus()` / `revoke_lease(lease_id)` / sub-mode transition hook | **PR-5a** (sub-mode hub) + **PR-5b** (revoke chain) + **PR-5d** (S21.8 pause) | vmm_arch §2 v1.x patch (sink-only sink role 확장) |
| `firmware-approval` (S14) | `FirmwareOp::MailboxOperation { vendor, target, opcode, payload, class }` enum variant + `revoke_all_leases()` API | **PR-5d** (S23.8 + S20.2.7) | amdv_safety §S14 v1.x patch |
| `npt` (S3) | mailbox MMIO range 의 NPT mapping 거부 (`is_in_mailbox_mmio_range(host_pa)` predicate) | **PR-5d** (S23.5) | amdv_safety §S3 v1.x patch |
| `msr-bitmap` (S10.1) | mandatory entry 30+ 항목 추가 (P-state MSR 6 + C-state residency 8 + RAPL 18 + thermal 7 + voltage 4 + SMU vendor MSR) | **PR-5c** (S15/S16/S19) + **PR-5d** (S17/S21/S23) | amdv_safety §S10.1 v1.x patch |
| `io-bitmap` (S11) | mailbox port (PCH PMC port range) default-block 추가 | **PR-5d** (S23.5) | amdv_safety §S11.1 default-block 갱신 |
| `cpuid-emul` (S2 sub) | power feature bit 마스킹 (CPUID 0x6 thermal 6 / 0x15+0x16 frequency / 0x80000007 power feature / 0x80000008 RAPL2 등) | **PR-5c** (frequency) + **PR-5d** (RAPL/thermal/power) | amdv_safety §S2 CPUID emulation 정책 v1.x patch |

본 cross-cluster API extension PR 들은 **amdv 측 doc 의 v1.x patch
형태로 분리 진행** — 4 doc frozen 변경 0.

### 4.3 Phase D forward-compat row (F=a)

Phase D 진입 시 추가 작업 (vmm_arch §8.8 + power_safety §7.4 cross-ref):

| 항목 | seL4 측 | power-orchestrator | power capsule | cross-cluster | Status |
|---|:---:|:---:|---|---|:---:|
| **AV28-D body** (`wake_source_iommu_consistent`) | △ IOMMU programming | △ wake source dispatch | **wakeup** (forward-compat hook 활성화) | npt (Phase D IOMMU programming capsule) + (Phase D IOMMU + per-device BAR cap) | **Phase D** |
| **R-α/R-γ wake event 정합** | — | △ wake forward | **wakeup** (nested guest wake event 처리) | (Phase D R-α/R-γ infrastructure, vmm_arch §8.4) | **Phase D** |
| **PCIe device passthrough + power management** | ◎ IOMMU + per-device BAR | △ | **wakeup** (S22.7 USB wake passthrough 활성화) | (Phase D IOMMU programming capsule) | **Phase D** |
| **per-capsule restart policy** | — | △ fault dispatch | 6 power capsule 별 fallback (§2.6 Phase D 표) | lifecycle (per-capsule restart trigger, vmm_arch §8.4) | **Phase D** |
| **Disk-backed audit persistence** | — | — | — | audit (S12.4 forward-compat hook 활성화 — power op_tag 도 disk dump) | **Phase D** |
| **Hardware enclave 통합 (SGX/TZ Realm)** | △ enclave entry | △ | **psp-pm** (Plundervolt secondary mitigation 의 enclave fault detection 통합) | (Phase D enclave 도입 시) | **Phase D** |

Phase D 진입 시 Verus AV28-D body 채움 + 본 6 항목 v1.x patch 진입.

### 4.4 Sub-PR boundary 정합 검증 (G=a)

각 row 의 capsule 이 **정확히 1 sub-PR** 안에 들어가야 함 (cross-PR
span 0):

| Sub-PR | 책임 capsule | 책임 안전장치 |
|---|---|---|
| **PR-5a** | `power-orchestrator` + `audit` (cross-cluster) op_tag 확장 | (cluster 진입 + cross-cluster API extension hub) |
| **PR-5b** | `lease-pm` (per-VM, 본체) + `lease-pm/storage_*` 11 backend | **S20** + **S20.2** |
| **PR-5c** | `cpufreq` (본체) + `msr-bitmap-extension` (cross-cluster v1.x patch) | **S15** + **S16** + **S19** |
| **PR-5d** | `acpi-pm` + `psp-pm` + `rapl` + `wakeup` (4 host-global 본체) + cross-cluster `firmware-approval`/`npt`/`io-bitmap`/`cpuid-emul` v1.x patch | **S17** + **S18** + **S21** + **S22** + **S23** |

검증:
- power-orchestrator 의 IPC dispatch hub (`PmCapsuleMsg` 34 verb) 는
  PR-5a 진입 시 모든 verb 의 stub 작성, body 는 후속 PR 에서 채움
  (PR-5b~d 의 의존 reverse — orchestrator 가 first, capsule 이 의존)
- cross-cluster API extension 은 sub-PR 별 분산:
  - PR-5a: audit op_tag 30+ enum (모든 sub-PR 가 의존하는 sink)
  - PR-5b: lifecycle revoke_lease + firmware-approval revoke_all_leases
    (lease-pm 본체 의존)
  - PR-5c: msr-bitmap entry (P-state/C-state) + cpuid-emul frequency
  - PR-5d: msr-bitmap entry (RAPL/thermal/voltage) + cpuid-emul (power
    feature) + firmware-approval MailboxOperation + npt mailbox MMIO +
    io-bitmap mailbox port
- 각 sub-PR 가 의존 cycle 짧음 (DAG sink-first 진행)

### 4.5 Contribute-back ledger (E=a, power_safety §5.5 cross-ref)

PR-1 ~ PR-5 의 4-way contribute-back (vmm_arch §6.3 + amdv_safety §6.2
패턴 정합):

| 산출물 | 게시 plan |
|---|---|
| **C 패치 (D1a + D1a')** | seL4 mainline PR-1 (raw-SVM + power MSR/ACPI/SMI mediation 통합 단일 submission, sel4_fork_policy.md §6.3 numbering: `100-power-*.patch`) |
| **Rust capsule 코드** | Y4 워크스페이스 (Y4 GitHub) — PR-2 (vmrun cluster) + PR-5a~d (power cluster) |
| **Verus 명세 (AV1~AV40)** | Y4 frozen tag + paper artifact (PR-3 + PR-5 의 power 측 module — 단일 paper artifact 의 다른 module) |
| **Isabelle skeleton** | `y4-verus2isabelle` 도구 산출물 (PR-4 + power fixture round-trip — single tool, P1.4 §5.3 / P3.6 §3.2 정합) |

paper venue: vmm_arch §6.4 의 1 순위 SOSP workshop / PLOS, 2 순위 SOSP /
OSDI main track.

---

## 5. Repo 구조

### 5.1 Y4 워크스페이스 안 (vmm_arch §5.1 패턴 정합)

| 경로 | 형태 | 내용 |
|---|---|---|
| `Y4/proofs/verus/src/power/` | 신규 디렉터리 | `upper/` + `lower/` 안에 per-capsule + per-안전장치 파일 (§5.4 본문) |
| `Y4/capsules/pm-cpufreq/` | 신규 sub-crate | 본체 — P-state governor + DVFS dwell + C-state state machine + SMT pair sync (S15/S16/S19/S21 cap) |
| `Y4/capsules/pm-acpi-pm/` | 신규 sub-crate | 본체 — per-VM virtual DSDT + AML interpreter mediation + thermal threshold + ACPI table SHA-256 (S18/S21) |
| `Y4/capsules/pm-psp-pm/` | 신규 sub-crate | 본체 — vendor 별 9-platform mailbox backend + voltage range + 3-등급 분류 (S23) |
| `Y4/capsules/pm-rapl/` | 신규 sub-crate | 본체 — RAPL MSR mediation + virtio-rapl + per-VM energy budget (S17) |
| `Y4/capsules/pm-wakeup/` | 신규 sub-crate | 본체 — wake source + lease binding + Y4-defined cryptographic signed magic packet + virtio-rtc (S22) |
| `Y4/capsules/pm-lease-pm/` | 신규 sub-crate (per-VM instance) | 본체 — 4-tier secure storage + XChaCha20-Poly1305 AEAD + replay protection epoch (S20/S20.2) + 11 backend sub-module |
| `Y4/power-orchestrator/` | 신규 workspace member | Trusted core, 600 LoC budget (`Y4_PM_ORCHESTRATOR_LOC_BUDGET`) |

#### 5.1.1 Capsule sub-crate 트리 (per-capsule)

```
Y4/capsules/pm-cpufreq/                  (S15/S16/S19/S21 cap)
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── governor.rs                      # S15.4 governor 적용
    ├── dwell.rs                         # S15.5 dwell time 강제
    ├── cstate_state_machine.rs          # S16 + S21 state machine (Cool/Throttling/HardThrottling/Emergency)
    ├── smt_pair_sync.rs                 # S19.2 strict 동기 + IPI atomic
    └── pstate_apply.rs                  # P-state hardware apply

Y4/capsules/pm-acpi-pm/                  (S18/S21)
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── aml_interpreter.rs               # S18.1 server-side AML eval
    ├── virtual_dsdt.rs                  # S18.3 per-VM virtual DSDT
    ├── method_whitelist.rs              # S18.2
    ├── osi_handling.rs                  # S18.4
    ├── thermal_threshold.rs             # S18.5 / S21.4 / S21.5
    ├── sx_state.rs                      # S18.6
    └── table_integrity.rs               # S18.8 SHA-256

Y4/capsules/pm-psp-pm/                   (S23)
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── mailbox_amd_smu.rs               # S23.1 9-vendor backend
    ├── mailbox_amd_svi2.rs
    ├── mailbox_intel_pmc.rs
    ├── mailbox_intel_vr.rs
    ├── mailbox_arm_pmic_qcom.rs
    ├── mailbox_arm_pmic_nxp.rs
    ├── mailbox_power_occ.rs
    ├── mailbox_z_se.rs
    ├── mailbox_riscv_sbi.rs
    ├── mailbox_legacy.rs
    ├── voltage_range.rs                 # S23.4
    └── op_class.rs                      # S23.3 3-등급 분류

Y4/capsules/pm-rapl/                     (S17)
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── msr_mediation.rs                 # S17.1
    ├── virtio_rapl.rs                   # S17.2
    ├── energy_budget.rs                 # S17.8 per-VM tracker
    ├── noise_injection.rs               # S17.4 LSB mask
    └── thermal_internal_channel.rs      # S17.6

Y4/capsules/pm-wakeup/                   (S22 + M7 transportation)
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── source_whitelist.rs              # S22.2
    ├── lease_binding.rs                 # S22.3
    ├── spurious_detection.rs            # S22.4
    ├── priority_queue.rs                # S22.5 6-tier
    ├── magic_packet.rs                  # S22.6 Y4-defined cryptographic
    ├── usb_wake.rs                      # S22.7
    ├── button_routing.rs                # S22.8
    ├── virtio_rtc.rs                    # S22.9
    ├── vehicle_bus.rs                   # M7 transportation auto-detection
    └── force_toggle.rs                  # S22.10

Y4/capsules/pm-lease-pm/                 (S20 + S20.2 — per-VM instance)
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── secure_storage.rs                # SecureStorage trait + dispatch (S20.2.2)
    ├── storage_psp.rs                   # Tier 1 — x86-64 AMD PSP-protected SRAM (+ PSP-fTPM)
    ├── storage_txt.rs                   # Tier 1 — x86-64 Intel TXT (+ ME-fTPM/PTT)
    ├── storage_sev_snp.rs               # Tier 1 — AMD SEV-SNP secure memory
    ├── storage_tdx.rs                   # Tier 1 — Intel TDX secure memory
    ├── storage_tz.rs                    # Tier 1 — AArch64 TrustZone (+ TZ-fTPM)
    ├── storage_cca.rs                   # Tier 1 — AArch64 ARMv9 CCA Realm
    ├── storage_pef.rs                   # Tier 1 — POWER PEF Ultravisor
    ├── storage_se.rs                    # Tier 1 — IBM Z Secure Execution
    ├── storage_pmp.rs                   # Tier 1 — RISC-V PMP+Keystone
    ├── storage_cove.rs                  # Tier 1 — RISC-V CoVE
    ├── storage_tpm_aead.rs              # Tier 1.5 — 외장 dTPM 2.0 + AEAD master key seal (TQ.1, TQ.9 tss-esapi)
    ├── storage_xchacha.rs               # Tier 2 — XChaCha20-Poly1305 sealed DRAM (universal)
    ├── storage_no_suspend.rs            # Tier 3 — suspend 거부 backend
    ├── epoch_counter.rs                 # S20.4.2 monotonic counter
    └── form_factor_policy.rs            # S20.7 form-factor 별 suspend ON/OFF

Y4/power-orchestrator/                   (trusted core, 600 LoC budget)
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── main.rs                          # workspace member entry
    ├── ipc.rs                           # PmCapsuleMsg dispatch (§2.2.2 의 34 verb)
    ├── cstate_decision.rs               # vCPU HLT vmexit 응답
    ├── form_factor_profile.rs           # tools/power.rules.d/ overlay merge 적용
    ├── sub_mode_transition.rs           # §2.4 ModeSignal hub
    └── force_toggle_state.rs            # 5 force-toggle master state
```

ARCH-II' 의 **16 workspace member + 신규 7 (orchestrator + 6 capsule) =
23 workspace member**.

### 5.2 Workspace dependency 표

| Crate | 의존 |
|---|---|
| `y4-capsules-pm-cpufreq` | `y4-ipc` + `y4-alloc` + `y4-capsules` (test fixtures + 공통 trait) + cross-cluster amdv 측 (`y4-capsules-vmm-msr-bitmap` / `y4-capsules-vmm-cpuid-emul` / `y4-capsules-vmm-audit` / `y4-capsules-vmm-lifecycle` 의 client API) |
| `y4-capsules-pm-acpi-pm` | `y4-ipc` + `y4-alloc` + `y4-capsules` + `y4-capsules-vmm-firmware-approval` + `y4-capsules-vmm-audit` + `y4-capsules-vmm-cpuid-emul` |
| `y4-capsules-pm-psp-pm` | `y4-ipc` + `y4-alloc` + `y4-capsules` + `y4-capsules-vmm-firmware-approval` + `y4-capsules-vmm-msr-bitmap` + `y4-capsules-vmm-cpuid-emul` + `y4-capsules-vmm-npt` + `y4-capsules-vmm-io-bitmap` + `y4-capsules-vmm-audit` |
| `y4-capsules-pm-rapl` | `y4-ipc` + `y4-alloc` + `y4-capsules` + `y4-capsules-vmm-msr-bitmap` + `y4-capsules-vmm-cpuid-emul` + `y4-capsules-vmm-audit` + `y4-capsules-vmm-lifecycle` |
| `y4-capsules-pm-wakeup` | `y4-ipc` + `y4-alloc` + `y4-capsules` + `y4-capsules-pm-acpi-pm` (cross-power: _PRW mediation) + `y4-capsules-pm-lease-pm` (cross-power: S20.8 wake hook) + `y4-capsules-vmm-audit` |
| `y4-capsules-pm-lease-pm` | `y4-ipc` + `y4-alloc` + `y4-capsules` + **`tss-esapi`** (BSD-2, Tier 1.5 dTPM backend) + `y4-capsules-vmm-firmware-approval` + `y4-capsules-vmm-lifecycle` + `y4-capsules-vmm-audit` |
| `y4-power-orchestrator` | 위 6 power capsule sub-crate 모두 + `y4-ipc` + `y4-alloc` + `y4-capsules-vmm-audit` + `y4-capsules-vmm-lifecycle` + **`logicutils-core`** (BSD-2, `tools/power.rules.d/` overlay merge 적용) + seL4 raw-SVM cap binding (D1a' patch 측 syscall) |

기존 16 (vmm_arch §5.1.1 + power 추가 7) = 23 workspace member 의존
그래프 (DAG, §2.5 + §2.8 정합).

### 5.3 logicutils `tools/power.rules.d/` (D=a, power_safety §2.1.3 정합)

`tools/power.rules.d/` **디렉터리** (single file 아닌 numbered overlay
file 묶음) — power_safety §2.1.3 의 overlay merge:

```
Y4/tools/power.rules.d/
├── 00-default-server-farm.rules           # Y4 ship default
├── 00-default-rack-mount.rules
├── 00-default-mobile.rules
├── 00-default-mobile-dock.rules
├── 00-default-mobile-portable.rules
├── 00-default-mobile-transportation.rules
├── 00-default-soc.rules
├── 00-default-certified.rules
├── 00-default-aliases.rules               # M8 deprecated alias
├── 00-default-detection.rules             # power_safety §2.5
└── (user override / addition: NN > 00, lu-rule layering)
```

build-time `lu-rule` 가 본 디렉터리 모든 file 처리 → form-factor / sub-
mode / certified overlay 적용 → power-orchestrator + 6 capsule 의
build-time const 결정.

### 5.4 Proofs 디렉터리 layout (E=a, vmm_arch §5.1 2-축 정합)

```
Y4/proofs/verus/src/power/
├── lib.rs                                  # top-level imports
├── upper/                                  # cross-tenant / cross-CPU layer
│   ├── npt.rs
│   ├── sub_mode.rs                         # AV22
│   ├── mode_invariants.rs                  # AV24 (transportation 등)
│   ├── voltage_range.rs                    # AV25
│   ├── magic_packet.rs                     # AV26
│   ├── thermal_hardlimit.rs                # AV27
│   ├── wake_whitelist.rs                   # AV28
│   ├── wake_iommu.rs                       # AV28-D (Phase D forward-compat)
│   ├── rapl_budget.rs                      # AV29
│   ├── acpi_integrity.rs                   # AV32
│   ├── wake_epoch.rs                       # AV33
│   └── boot_fix.rs                         # AV35
└── lower/                                  # within-cluster / capsule cooperation
    ├── tpm_consistency.rs                  # AV21
    ├── sub_mode_transition.rs              # AV23
    ├── smt_sync.rs                         # AV30
    ├── dvfs_dwell.rs                       # AV31
    └── force_mask.rs                       # AV34
```

§4.2 catalog 의 Proof file column 과 1:1 정합 (AV21~AV35 + Phase D
AV28-D + reserved AV36~AV40).

### 5.5 외부 sibling repo cross-ref (F=a, vmm_arch §5.3 정합)

power 측 신규 sibling repo X — 기존 sibling repo 만 활용:

| Repo | 라이선스 | 본 doc 와의 정합 |
|---|---|---|
| **`/home/ybi/y4-verus2isabelle/`** (P3.6 §3.2) | Apache-2.0 single crate | power AV21~AV40 fixture 추가 (P3.4 / P3.6 의 Y4-scope 한정 정합) — `tests/fixtures/y4-power/` 의 snapshot copy |
| **`/home/ybi/y4-hypercall/`** (P1.4 §5.3) | Apache-2.0 사용자 CLI | power 측 CLI 명령 추가: `y4-hypercall power smt-force / suspend-force / thermal-force / wake-force / mailbox-force / energy show / energy set-cap / wake-source / mode-set / sub-mode / thermal-status / suspend-status / wake-status / mailbox-status` 등 (S15-S23 의 모든 force-toggle + status query CLI) |
| **`/home/ybi/y4-drivers/`** (이전 결정) | Apache-2.0 + GPL capsule mixed | M7 의 vehicle bus signal driver 후보 위치 (`y4-driver-can-bus` / `y4-driver-obd2` / `y4-driver-vehicle-ethernet-avb`) — Phase C 진입 후 결정 (power_safety §7.3 항목 3) |
| `/home/ybi/y4-upstream-refs/{linux,freebsd,dragonfly,trusted-firmware-a}/` | (각 upstream) | §1.1 참조 자료 통합 표의 read-only reference |

### 5.6 Cargo dep 라이선스 정합 (G=a)

power capsule cluster 의 cargo dep — Y4 single Apache-2.0 와 호환 검증:

| Crate | License | 호환 | 위치 |
|---|---|:---:|---|
| `tss-esapi` | BSD-2-Clause | ◎ (BSD-2 → Apache-2 단방향 호환) | `pm-lease-pm/storage_tpm_aead.rs` (TQ.9) |
| `logicutils-core` | BSD-2-Clause | ◎ | `power-orchestrator` (`tools/power.rules.d/` overlay merge) |
| `syn` / `quote` | Apache-2 OR MIT | ◎ | (필요 시 build script) |
| `serde` / `bincode` | Apache-2 OR MIT | ◎ | (cross-cluster IPC payload encoding 후보, v1.x patch) |
| `zeroize` | Apache-2 OR MIT | ◎ | `pm-lease-pm/secure_storage.rs` (secure_zero) |
| `subtle` | BSD-3-Clause | ◎ | constant-time comparison (XChaCha20-Poly1305 tag) |

본 cargo dep 들은 NOTICE 의 reuse manifest 에 추가 — power capsule
cluster 의 attribution 보존 (Apache-2 §4 (d) 정합).

기타 cargo dep 미정 항목 → v1.x patch 시 추가 + NOTICE 갱신.

---

## 6. 차별점 (vmm_arch §6 패턴 정합 — 학술적 / 산업적 / contribute-back)

### 6.1 학술적 차별점

다음 7 항목에서 **공개된 prior art 의 부재** (2026-05-07 시점, §6.7
prior art ledger 기준):

1. **seL4 + Y4 capsule pattern + VeriSMo 검증 기법 + power-mgr 통합**
   — vmm_arch §6.1.1 의 통합 사례 위에 power domain (capsule cluster
   + 9 안전장치 S15~S23 + AV21~AV40 invariant) 을 추가.  공개 prior art
   부재 (SOSP '25 Atmosphere, AWS Nitro Isolation Engine 모두 power
   domain 미진입).
2. **VMM (vmm_arch) + power-mgr 의 통합 capsule cluster** — vCPU idle
   ↔ host C-state 의 정합이 verified, cross-cluster API (audit /
   lifecycle / firmware-approval / npt / msr-bitmap / io-bitmap /
   cpuid-emul) 가 v1.x patch path 로 일관 통합.
3. **Hertzbleed / DVFS side-channel mitigation 의 capsule-level invariant
   + Verus property** — DVFS 의 secret-dependent timing 차단을 verified
   property 로 박은 첫 사례.  prior art (Hertzbleed IEEE S&P 2025 update
   기준): mitigation = disable boost only, verified hypervisor capsule-
   level mitigation **부재 confirmed**.
4. **PLATYPUS / Plundervolt mitigation 의 verified hypervisor + Verus**
   — RAPL energy counter 격리 (4-bit LSB noise + virtio-rapl + per-VM
   energy budget) + voltage range 한도 (±50 mV bound) 를 capsule-level
   invariant (AV25 / AV29) 로 박은 첫 사례.  prior art (PLATYPUS USENIX
   '21 / Plundervolt USENIX '20 기준): Linux RAPL driver 권한 제약 +
   Intel microcode 패치만, **verified + Verus property 박은 사례 부재
   confirmed**.
5. **ISA-agnostic 4-tier secure storage + lease-aware deep idle suspend**
   — Tier 1 (PSP / TXT / SEV-SNP / TDX / TZ / CCA / PEF / SE / PMP+
   Keystone / CoVE) + Tier 1.5 (외장 dTPM 2.0 + XChaCha20-Poly1305
   master key seal) + Tier 2 (universal software fallback) + Tier 3
   (suspend 거부) 가 verified hypervisor 의 lease integration (S20 +
   S20.2 + AV21 + AV33) 과 결합한 첫 사례.  POWER PEF / IBM Z Secure
   Execution / RISC-V CoVE / ARM CCA 등 차세대 ISA confidential VM
   인프라까지 cover.  prior art (CoVE / ACE-RISCV arXiv 2505.12995
   기준): 단일 ISA 의 confidential computing 만 다룸, hypervisor lease
   + 4-tier 통합 **부재 confirmed**.
6. **Universal customizability + generic formal invariant** — Y4 ship
   의 form-factor / sub-mode 도 default definitions (built-in 아님,
   override / removal 가능, `tools/power.rules.d/` overlay merge),
   formal invariant `mode_invariant_holds` (AV24) 가 named sub-mode
   의 정의 여부에 따라 자동 활성/비활성.  configuration 의 formal
   invariant 처리가 일반화된 사례 **부재 confirmed**.
7. **Convertible + transportation sub-mode 의 verified hypervisor
   power management** — 자동차 / 철도 / 항공 환경의 sudden power loss
   대비 (5 ms suspend latency + fTPM 우선 + vehicle bus / GPS / cellular
   wake) + AV24 의 `transportation_sudden_power_loss_safe` invariant.
   prior art (automotive hypervisor 시장 — VOSySmcs / Synopsys VDK
   CES 2026 / 시장 $3.2B by 2030 기준): ISO-26262 functional safety
   인증 수준에 머물고, **formal verification + Verus invariant +
   transportation lease integration 부재 confirmed**.
8. **ACLP-driven build orchestration (logicutils) 통합 verified
   hypervisor** — Y4 가 build infrastructure 측에서 logicutils
   (`/home/ybi/logicutils/`, BSD-2-Clause) 의 핵심 도구를 적극 활용.

   #### 8.1 logicutils 의 KB 언어 — Abductive-Deductive Logic + Constraint Logic 의 결합

   `lu-query` 의 logic engine 은 KB 언어 (`docs/man/lu-kb.5` 의 line 6
   본문 + `docs/learning/{ko,en}/logicutils.typ` 의 §"귀추 한 문단" /
   "Abduction in one paragraph" 본문) 평가 — 표준 Prolog / Datalog 와
   본질적으로 다른 reasoning paradigm:

   > "KB has facts and rules like Prolog, plus **three additional ideas
   > you will not find in standard Prolog: abduction, constraints, and
   > type relations**." (`logicutils.typ` line 304-306)

   > "A build system is, **fundamentally, a logical theory**: targets
   > exist, files depend on each other, recipes build files, freshness
   > implies skipping the recipe." (`logicutils.typ` line 335-336)

   본 framing — **빌드 시스템 = 논리 이론** — 은 logicutils 의 학술적
   기여 핵심.  Y4 의 build infrastructure 는 본 framing 위에 verified
   hypervisor 의 모든 reproducibility / customizability / supply chain
   integrity 를 박는다.

   **(체계 1) Abductive-Deductive Logic Programming (ALP) — 귀추 논리 +
   연역 논리의 결합**.  Kakas / Kowalski / Toni "Abductive Logic
   Programming" (1992) 의 표준 정의:

   - **Deductive (연역)** — `fact` + `rule` 로 forward chaining.
     "rules and facts → conclusions" 방향.
   - **Abductive (귀추)** — `abduce` + `explain` 으로 역방향: "given a
     conclusion you would like to be true and the rules of the world,
     propose facts that — if true — would explain the conclusion"
     (`logicutils.typ` line 310-312).  build 맥락에서 핵심 query =
     "what would I need to do to make this target up-to-date?"
     ```
     abduce missing_source(File):
         depends(Target, File)
         not exists(File)
         explain "source file may need generation"
     ```
   - **Abductive ↔ Deductive 의 결합 (사용자 강조점)** — 두 reasoning
     direction 이 단일 coherent reasoning system 안에서 interleave.
     abductive hypothesis 가 deductive 추론 chain 의 missing premise 를
     채우면서 동시에 consistency 유지.  ALP 의 핵심 = 두 directions 의
     함께 사용 (단순 deductive datalog 와 본질적으로 다른 paradigm).

   **(체계 2) Constraint Logic Programming (CLP)** — Jaffar / Lassez
   "Constraint Logic Programming" (POPL 1987) 의 표준 정의:
   - **Constraint blocks** (`constraint` + typed parameter) — watched
     constraint, "as soon as a watched variable is determined and the
     condition is violated, the search backtracks" (`logicutils.typ`
     line 326-328).  build 측: 형상별 resource / 시간 / 메모리 budget
     constraint 를 KB 안에서 표현 + solver 가 build schedule 결정.

   **(결합 체계) ACLP = ALP + CLP** — Kakas / Michael / Mourlas "ACLP:
   Integrating Abduction and Constraint Solving" (1999, ScienceDirect /
   arXiv cs/0003020).  abductive hypothesis 와 constraint solving 의
   cooperative interleaving.

   **(보조 체계 3) Type Relations** — KB 의 셋째 idea, ALP / CLP 와
   별개로 logicutils 가 도입.  Prolog / Datalog 에 없는 typed parameter
   + nested instance 의 logic-level type system.  Y4 측 적용: form-factor
   / sub-mode / certified overlay 의 type-safe customizability.

   #### 8.2 빌드 = 논리 이론 framing 의 query 3 가지 (`logicutils.typ` line 339-342 직접 인용)

   logicutils 의 learning material 이 명시한 3 가지 query 는 ALP +
   build orchestration 의 결합 가치를 직접 보여준다:

   1. **"Which targets are stale?"** — 표준 deductive forward chaining
      (어떤 build system 도 수행).
   2. **"What is the smallest set of files I would need to regenerate
      to make target T up-to-date?"** — **abductive minimal explanation**.
      ALP 의 minimal hypothesis 추론 — 본 query 는 conventional build
      system (make / cargo / bazel / nix) 가 답할 수 없음.  "make T"
      이 모든 deps 를 regenerate 하는 것과 본질적으로 다른 의미.
   3. **"Why is target T being rebuilt? What did the engine deduce?"**
      — **abductive explanation chain** + deductive trace.  reviewer /
      auditor / debugger 가 build 결정의 root cause 를 logic engine
      안에서 직접 query 가능 — paper artifact reviewer 친화 + supply
      chain integrity audit 의 forensic 가치.

   인용: "Every classical build system answers a fixed subset of these
   questions.  With `lu-query` you ask whichever question fits today."
   (`logicutils.typ` line 344-345)

   #### 8.3 ALP 의 application 영역 + build 측 prior art 부재

   ALP 의 전통적 application 영역 (2026-05-07 검색 ledger 기준): diagnosis
   / planning / formal verification / multi-agent systems / normative
   reasoning / ontology reasoning / commitments / web service choreographies.
   CLP 의 application: scheduling / resource allocation / configuration /
   circuit verification.  ACLP application 영역 = 두 영역의 교집합 + 통합.

   **build orchestration 측 ALP / ACLP application 공개 prior art 부재
   confirmed** (검색 결과: BioMake 가 가장 가까운 영감 source — Prolog
   logic programming 도입했으나 abductive minimal explanation 의 본격
   build 측 활용 미명시.  Modus 등 datalog-based build = deductive 만).

   #### 8.4 logicutils 의 핵심 도구 (Y4 활용)

   - **`freshcheck`** — content-based freshness (BLAKE3 / SHA3 / CRC32),
     mtime 기반 X.  Verus statement-only frozen 의 immutability + paper
     artifact 의 long-term reproducibility 보장
   - **`stamp`** — file signature 기록/조회/diff.  Verus + Isabelle
     skeleton emission 의 reproducibility audit chain
   - **`lu-rule`** — pattern-rule selection with goal backtracking.
     `tools/power.rules.d/` 의 default + user override file 들의 overlay
     merge 가 본 도구 기반
   - **`lu-par`** — DAG-aware parallel runner with **transactional
     rollback**.  task fail 시 stamp signature 를 atomic rollback →
     paper artifact 의 build atomicity 보장
   - **`lu-query`** — ACLP solver (§8.1).  security policy 의 stale
     invariant 식별 + abductive explanation + constraint-aware schedule
   - **`lu-queue`** — local + SLURM / SGE / PBS cluster engine 단일
     인터페이스.  HPC 환경 (server-farm 형상) 의 multi-node Verus
     verification 분산 가능
   - **CLI protocol** (semver-versioned, `--protocol-version`) — alternative
     engines + reimplementations interoperable

   #### 8.5 본 통합의 학술적 차별점 (이중 prior art 부재)

   **(a) ACLP 의 build orchestration 측 application — prior art 부재**:
   - ACLP (Abductive Constraint Logic Programming) 의 전통적 application
     영역은 diagnosis / planning / formal verification / multi-agent
     systems / ontology reasoning — **build orchestration 측 application
     공개 prior art 부재 confirmed** (2026-05-07 검색)
   - Datalog 기반 build (Modus 등) 는 deductive 만 — abductive +
     constraint 측 X
   - BioMake (logicutils 영감 source, BSD-3) 는 Prolog logic programming
     를 build 에 도입했으나 abductive / constraint 측 본격 활용 미명시,
     bioinformatics 영역만 진입
   - logicutils 가 **ACLP 를 generic build orchestration 으로 확장한 첫
     toolkit** (Tier-1 build mode + CLI protocol + 11 utility crate
     workspace + KB language)

   **(b) verified hypervisor 의 build infrastructure 측 적용 — prior art
   부재**:
   - 기존 verified hypervisor (Atmosphere SOSP '25 / AWS Nitro Isolation
     Engine / Lightweight Hypervisor Verification HOTOS '25 / CoVE-ACE
     arXiv 2505.12995) 는 모두 conventional build orchestration (cargo /
     cmake / make / bazel / nix) 사용 — ACLP-enhanced build 통합 사례 부재
   - Y4 가 logicutils (ACLP-driven) 를 verified hypervisor 의 build
     infrastructure 로 활용한 **첫 사례**

   #### 8.6 통합 결과의 paper-level 강점

   - paper artifact 의 USENIX / ACM / IEEE artifact badge 의 **Reproducible
     자격이 build orchestration 측에서 hash-driven + transactional 로
     강제** — conventional reproducible builds (mtime + best-effort
     determinism) 와 차별화
   - **abductive explanation** 으로 verification fail / build mismatch
     의 root cause analysis 자동화 — paper 의 reviewer 가 "왜 이
     invariant 가 깨졌는가" 를 KB query 로 직접 추론 가능 (artifact
     badge 의 evaluation 친화)
   - **constraint logic** 으로 form-factor / sub-mode 의 customizability
     boundary 가 KB 안에 명시 — universal customizability (§6.1 항목 6)
     의 formal foundation 보강

### 6.2 산업 차별점

paper / 사용자 의 "Why does this matter outside the academy" 답변
(vmm_arch §6.2 + power-specific):

- **5 form-factor cross-portable host OS** — 단일 verified base 가
  server-farm / rack-mount / mobile (laptop+handheld 통합) / SoC 모두
  cover, certified overlay 로 의료/항공/금융 트랙도 포함
- **WaveTensor 가속기 통합** — HIU lease capability 가 hypervisor
  primitive, power-mgr 의 lease suspend 와 정합
- **Lease-based multi-tenancy + per-VM energy budget** — 시간/자원/전력
  격리 단위가 OS 의 first-class concept (S17.8)
- **Apache-2.0 patent grant** — 상업 도입 시 patent retaliation 우려 0
- **Strictly Additive Fork policy** (sel4_fork_policy.md) — Y4 도입 변경
  이 upstream seL4 회귀 0 보장
- **KDE Plasma 패턴 force-toggle 5 개** — SMT / suspend / thermal /
  wake / mailbox 의 사용자측 명시 override.  사용자 UX 가 KDE Plasma
  의 "screen lock / sleep inhibit" toggle 패턴과 정합
- **ISA-agnostic single codebase** — datacenter 부터 임베디드까지 단일
  Y4 codebase 가 cover, ISA-별 backend sub-module 로 platform
  diversity 흡수
- **automotive 시장 진입 가능성** — formal verification + ISO-26262
  certified profile overlay 의 결합으로 자동차 / 철도 / 항공 transportation
  workflow cover (VOSySmcs / Synopsys VDK 와 보완 가능)
- **logicutils 기반 build infrastructure** — BSD-2-Clause toolkit 의 활용
  으로 OEM / SI / cluster 운영자 측 build 인프라 자연 통합:
  * `lu-queue` 가 SLURM / SGE / PBS 자동 dispatch — HPC datacenter / 자동차
    OEM 의 기존 build 클러스터 그대로 재사용
  * `freshcheck` + `stamp` 의 content-based freshness — supply chain
    integrity (Y4 frozen tag 의 byte-level reproducibility) audit 가능
  * `tools/power.rules.d/` 의 user override mechanism — form-factor /
    sub-mode 의 customization 이 build orchestration layer 에서 일관 처리,
    Y4 codebase 변경 0 으로 OEM-specific power profile 도입 가능
  * Tier-1 build mode (`--no-default-features`) — 임베디드 SoC 의 cross-
    build host 환경에서도 동일 toolkit 활용

### 6.3 contribute-back 경로 (timeline + 의존, vmm_arch §6.3 정합)

| 산출물 | 게시 plan | timeline | 의존 |
|---|---|---|---|
| **C 패치 (D1a + D1a')** | seL4 mainline PR-1 (raw-SVM + power MSR/ACPI/SMI mediation 통합 단일 submission) | Phase C 진입 직후 | sel4_fork_policy.md frozen |
| **Rust capsule 코드 (vmrun + power-orchestrator + 16 capsule)** | Y4 워크스페이스 — PR-2 + PR-5a~d | Phase C 중반 | PR-1 머지 또는 review 진입 |
| **Verus 명세 (AV1~AV40)** | Y4 frozen tag + paper artifact | Phase C 종반 | PR-2 + PR-5a~d 머지 |
| **Isabelle skeleton (AV1~AV40)** | `y4-verus2isabelle` 도구 산출물 | Phase C 종반 + Verus 명세 짝 | `y4-verus2isabelle` v1.0 + power fixture round-trip |

power 측 산출물은 vmm_arch 측 산출물의 **augmentation** — 별도 paper
아닌 단일 paper artifact 의 다른 module (power_safety §5.5 정합).

### 6.4 paper venue fit 분석 (vmm_arch §6.4 정합 + power 측)

| Venue | fit 평가 | power 측 evidence |
|---|---|---|
| **SOSP workshop (PLOS)** | ◎ 가장 자연 fit | "verified hypervisor on a verified microkernel + power-mgr" 가 PLOS workshop 트랙 매칭.  **1 순위** |
| **SOSP main track** | ◎ 강한 fit (2027) | Atmosphere (SOSP '25) 의 후속 — verified kernel + power 도입 첫 사례.  Phase C 종반 결과 강도 + 차별점 (§6.1) 의 evidence 충분 시 시도 |
| **OSDI main track** | ○ 강한 fit (2027) | systems-evaluation 비중 ↑, microbench 산출물 (power_safety §7.3) 충분 시 시도 |
| **IEEE S&P (Oakland)** | ◎ security framing 강함 | Hertzbleed / PLATYPUS / Plundervolt mitigation 의 verified hypervisor 측 진입 첫 사례 framing — **2 순위 (security venue 의 main track 자격)** |
| **USENIX Security** | ○ side-channel 트랙 강 fit | Hertzbleed / PLATYPUS / Plundervolt 의 mitigation 사례 → S&P 와 양립 후보 |
| **EuroSys** | ○ systems track | systems-heavy 가 더 적합한 경우 후보 |
| **ASPLOS** | △ HW 측면 약함 | WaveTensor 통합 paper 별도 시 후보 |
| **HOTOS** | △ workshop short paper | 초기 idea paper 형식 적합, full paper 부적합 |

**Phase C 종반 시점 1 순위 = SOSP workshop (PLOS), 2 순위 = IEEE S&P
(Oakland), 3 순위 = SOSP / OSDI main track 시도.**  Hertzbleed /
PLATYPUS / Plundervolt mitigation 의 security framing 강함 → S&P 자격
↑.

### 6.5 paper artifact 형식 (vmm_arch §6.5 정합)

**USENIX / ACM / IEEE S&P artifact badge 자격 충족 목표** (Available +
Functional + Reproducible).

artifact 묶음 (vmm 측 + power 측 통합):

| # | 산출물 | 형태 |
|---|---|---|
| (i) | Y4 GitHub repo 의 v1.0 frozen tag | git tag, immutable |
| (ii) | Verus 증명 산출물 (AV1~AV40) | `proofs/verus/{amdv,power}/` 트리, `just verus` 1-command 재실행 |
| (iii) | qemu reproducibility script | `qemu-smoke` + capsule cluster boot + power workflow simulation |
| (iv) | Isabelle skeleton | `y4-verus2isabelle` 도구로 자동 생성, `.thy` 파일 묶음 (amdv + power) |
| (v) | **power microbench 산출물** | S15.5 dwell / S20.5 latency / S22 spurious threshold (Phase C 종반 측정) — paper 의 evaluation data |
| (vi) | **TPM-based reproducibility** | Tier 1.5 backend 의 dTPM 2.0 simulator (qemu + swtpm) 또는 hardware loaner |
| (vii) | **logicutils-driven artifact verification** | `freshcheck --method=hash` (BLAKE3) + `stamp record` 로 모든 산출물의 content-based freshness 강제 + `lu-par --transaction` 으로 build atomicity 보장.  artifact tarball 에 `.lu-store/` (BLAKE3 nested hashtable) 동봉 — reviewer 가 hash mismatch 확인 가능 |

### 6.6 재현성 패키지 위치 (vmm_arch §6.6 정합)

**`/home/ybi/y4-paper-artifact/`** (sibling repo, paper draft 시점에
Y4 의 frozen tag 에서 cherry-pick 으로 생성).  vmm + power 측 산출물
모두 본 sibling repo 안에 통합.

paper 게시 후 GitHub release 에 동일 artifact mirror.

### 6.7 Prior art ledger (2026-05-07 검색 ledger, §6.1 의 evidence)

본 ledger 는 §6.1 의 7 학술적 차별점이 **공개된 prior art 부재**
주장의 evidence:

| Prior art | Venue | 본 doc 와 비교 | 결론 |
|---|---|---|---|
| **Atmosphere** | SOSP '25 (mars-research) | Verus + Rust full-featured microkernel, 7.5:1 proof-to-code ratio.  power management 측면 미진입 | power domain prior art 부재 |
| **AWS Nitro Isolation Engine** | re:Invent 2025 (AWS) | Graviton5 EC2 의 Isabelle/HOL 기반 verified isolation.  power management 미명시 | power domain prior art 부재 |
| **Lightweight Hypervisor Verification** | HOTOS '25 (EPFL) | top-down lock-step verification, power 측면 미진입 | 본 doc 의 capsule-level + power 통합 prior art 부재 |
| **CoVE / ACE-RISCV** | arXiv 2505.12995 (IBM, 2025) | RISC-V confidential computing, formal spec embedded + post-quantum crypto.  단일 ISA, power management 미명시 | 본 doc 의 ISA-agnostic 4-tier + lease integration prior art 부재 |
| **Hertzbleed update** | IEEE S&P 2025 | modern Intel processors 5x faster Hertzbleed, mitigation = disable boost only.  Intel/AMD 측 microcode patch 없음 (2026-05-07 시점) | verified hypervisor capsule-level mitigation prior art 부재 |
| **PLATYPUS** | USENIX Sec '21 + 후속 | RAPL driver 권한 제약 (Linux) + Intel microcode 만, hypervisor 측 verified 격리 부재 | verified RAPL isolation prior art 부재 |
| **Plundervolt** | USENIX Sec '20 + Intel firmware patch | hypervisor 가 MSR write 차단 standard practice, formal verification + Verus property 박은 사례 부재 | verified voltage range bound prior art 부재 |
| **automotive hypervisor 시장** | VOSySmcs (ISO-26262) / Synopsys VDK (CES 2026) / 시장 $3.2B by 2030 | functional safety 인증 수준, formal verification + transportation lease + Verus invariant 부재 | verified transportation power management prior art 부재 |
| **Windows Server 2025 HVCI / TPM 2.0** | Microsoft 2025 | hypervisor + TPM 의 single-platform integration | ISA-agnostic 4-tier + universal customizability prior art 부재 |
| **BioMake** | github.com/evoldoers/biomake (BSD-3) | bioinformatics pipeline 용 make 확장.  multi-wildcard + md5 signature + cluster engine + Prolog logic programming | bioinformatics 영역만, verified system / hypervisor 영역 진입 0.  logicutils 가 본 영감을 generic ACLP-driven build 로 확장 |
| **Modus (Datalog dialect for container images)** | Datalog-driven build | container image build 만, deductive datalog 만 (abductive/constraint X), kernel verification 측 X | logic-enhanced + ACLP + verified hypervisor 통합 prior art 부재 |
| **Reproducible Builds (Vienna 2025 summit)** | reproducible-builds.org | mtime + best-effort determinism 기반 framework | content-based hash-driven + transactional rollback + ACLP + verified hypervisor 통합 부재 |
| **OSS Rebuild (Google 2025)** | open source package ecosystem | upstream artifact reproducibility automation | verified hypervisor 의 form-factor customizability 통합 부재 |
| **Verifying Datalog Reasoning with Lean (ITP 2025)** | dagstuhl.de | datalog reasoning 의 formal verification | build orchestration 통합 X.  logicutils 와 보완 가능 (Verus → Lean translation 후속) |
| **ACLP (Abductive Constraint Logic Programming)** | Kakas/Michael/Mourlas 1999 + ScienceDirect / arXiv cs/0003020 | ALP (Kakas/Kowalski/Toni 1992) + CLP (Jaffar/Lassez POPL 1987) 의 통합 framework.  application = diagnosis / planning / formal verification / multi-agent / ontology reasoning | **build orchestration 측 application 공개 prior art 부재** — 본 영역에서 logicutils 가 첫 ACLP-driven build toolkit |
| **ALP (Abductive Logic Programming)** | doc.ic.ac.uk + Wikipedia | hypothetical reasoning, abducible predicates | software build dependency analysis 측 prior art 부재 (2025-2026 검색 confirm) |
| **SCIFF framework** | (IFF abductive framework derived) | verifiable agent interaction in ALP, multi-agent verification | agent verification 영역, hypervisor build orchestration 측 미진입 |

ledger 의 search 출처는 `.claude-notes/trackers/power-prior-art-ledger.md`
(Phase C 진입 후 신설) 에 보관 — 새 CVE / 논문 발견 시 본 표 갱신
+ §6.1 의 차별점 영향 재평가.

Sources:
- [Atmosphere: Towards Practical Verified Kernels in Rust](https://dl.acm.org/doi/10.1145/3625275.3625401)
- [Atmosphere Verified Operating System (Mars Research)](https://mars-research.github.io/projects/atmo/)
- [SOSP 2025 accepted papers](https://sigops.org/s/conferences/sosp/2025/accepted.html)
- [AWS Nitro Isolation Engine — TYPES announcement](https://lists.seas.upenn.edu/pipermail/types-announce/2025/012222.html)
- [Isabelle/HOL behind Nitro Isolation Engine](https://nwquantum.uw.edu/2026/04/17/isabelle-hol-the-proof-assistant-behind-the-nitro-isolation-engine/)
- [Lightweight Hypervisor Verification (HOTOS '25)](https://infoscience.epfl.ch/server/api/core/bitstreams/1ca2e9be-ff34-4da3-a74f-7dcf306e806d/content)
- [CoVE: Towards Confidential Computing on RISC-V Platforms](https://arxiv.org/abs/2304.06167)
- [ACE: Confidential Computing for Embedded RISC-V Systems](https://arxiv.org/html/2505.12995v1)
- [IBM/ACE-RISCV repository](https://github.com/IBM/ACE-RISCV)
- [Hertzbleed Attack](https://www.hertzbleed.com/)
- [PLATYPUS: Software-based Power Side-Channel Attacks on x86](https://platypusattack.com/platypus.pdf)
- [Plundervolt: Software-based Fault Injection Attacks](https://plundervolt.com/doc/plundervolt.pdf)
- [Automotive Hypervisor Strategic Industry Research Report 2024-2030](https://finance.yahoo.com/news/automotive-hypervisor-strategic-industry-research-135700513.html)
- [Verus: A Practical Foundation for Systems Verification (SOSP '24)](https://dl.acm.org/doi/10.1145/3694715.3695952)
- [State-of-the-art virtualisation technologies for centralised automotive E/E architecture](https://www.frontiersin.org/journals/future-transportation/articles/10.3389/ffutr.2025.1519390/full)
- [VOSySmcs](http://www.virtualopensystems.com/en/products/vosysmcs/)
- [BioMake (logicutils 영감 source)](https://github.com/evoldoers/biomake)
- [Modus: Datalog dialect for building container images](https://www.researchgate.net/publication/365270486_Modus_a_Datalog_dialect_for_building_container_images)
- [Reproducible Builds (Vienna 2025 summit)](https://reproducible-builds.org/)
- [Verifying Datalog Reasoning with Lean (ITP 2025)](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ITP.2025.36)
- [BLAKE3 paper specifications](https://github.com/BLAKE3-team/BLAKE3-specs)
- [Reproducible Builds in 2025](https://reproducible-builds.org/reports/2025-04/)
- [ACLP: Abductive Constraint Logic Programming (Kakas/Michael, ScienceDirect)](https://www.sciencedirect.com/science/article/pii/S0743106699000758)
- [ACLP: Integrating Abduction and Constraint Solving (arXiv cs/0003020)](https://arxiv.org/abs/cs/0003020)
- [Abductive Logic Programming (Kakas, Imperial College)](https://www.doc.ic.ac.uk/~rak/papers/abdsurv.pdf)
- [Abductive logic programming — Wikipedia](https://en.wikipedia.org/wiki/Abductive_logic_programming)
- [An efficient propositional system for ALP (Springer 2024)](https://link.springer.com/article/10.1007/s10462-024-10928-7)
- [Exploiting Logic Programming for Runtime Verification (Springer 2023)](https://link.springer.com/chapter/10.1007/978-3-031-35254-6_25)

---

## 7. 동결 정책 (frozen / sign-off)

본 doc 은 v0 design draft.  `v1.0 frozen` 마킹 조건 (power_safety §6 mirror):

### 7.1 sign-off 조건

- **§1 핵심 결정 (12 axis)** 사용자 sign-off — Base / Power-mgr 위치 /
  Threat model / 검증 도구 (AV21~AV40) / 검증 기법 / 형상별 정책 /
  Customizability / Secure storage (4-tier) / Cross-cluster capsule
  reuse / seL4 인터페이스 (D1a' patch series) / 라이선스 / 학술적
  차별점 cross-ref
- **§1.1 참조 자료 통합 표** 사용자 sign-off — Linux power drivers
  (GPL reference only) / FreeBSD cpufreq+acpi (BSD-2 port) / DragonFly
  powerd (BSD-3 port) / ARM TF-A (BSD-3, PSCI/TBBR 영감) / tss-esapi
  crate (BSD-2 cargo dep) / Plundervolt + PLATYPUS + Hertzbleed paper
  인용
- **§2 capsule 분해** 사용자 sign-off — 6 capsule + 1 orchestrator +
  6-column matrix (책임 / 안전장치 / cross-cluster amdv 측 capsule /
  AV / proof file) + LoC budget 600 + IPC interface (`PmCapsuleMsg`
  34 verb) + Trust model + DAG 의존 그래프 + Capsule fault recovery +
  Capsule cluster scope (host-global / per-VM) + LeaseCap 확장
- **§3 lease integration** 사용자 sign-off — 4-step seal/unseal flow +
  form-factor 별 suspend 정책 + transportation sub-mode sudden power
  loss 강화 + cross-cluster integration + atomicity 책임 분담 + Tier
  dispatch flow + Tier 결정 timing
- **§4 PR split 매트릭스** 사용자 sign-off — 6-column matrix + cross-
  cluster API surface 확장 + Phase D forward-compat row + sub-PR boundary
  정합 검증 + contribute-back ledger
- **§5 repo 구조** 사용자 sign-off — 23 workspace member + capsule
  sub-crate 트리 + workspace dependency 표 + `tools/power.rules.d/`
  layout + proofs 디렉터리 layout + 외부 sibling repo cross-ref +
  cargo dep 라이선스 정합
- **§6 차별점 (8 학술 + 9 산업) + paper venue + artifact + prior art
  ledger** 사용자 sign-off
- 짝 doc **`docs/power_safety.md` v1.0 frozen** 과 짝 (§7.2)

### 7.2 짝 doc 일괄 frozen 의존 (power_safety §6.2 mirror)

본 doc 의 v1.0 frozen 은 **`docs/power_safety.md` v1.0 frozen 과 짝으로만
발화** — power domain 의 별도 v1.0 cycle (vmm_arch.md / amdv_safety.md
/ sel4_fork_policy.md / verus_to_isabelle.md 4 doc 의 v1.0 frozen 과
**별도 cycle**, 단 cross-cluster capsule API 의존):

#### Cross-cluster API 의존 (vmm_arch / amdv_safety 측 v1.x patch 형태)

| amdv 측 capsule | 의존 surface | 영향 amdv doc |
|---|---|---|
| `audit` (S12.2) | power op_tag 30+ enum 확장 (PState* / CState* / Rapl* / EnergyBudget* / AcpiMethod* 6 / SmtPair* 6 / LeaseSuspend* 8 / Thermal* 6 / Wake* 6 / Mailbox* 5 / Tpm 3 / SubModeTransition / DeprecatedFormFactorAlias) | amdv_safety §S12.2 v1.x patch |
| `lifecycle` | `pause_all_vcpus()` / `revoke_lease(lease_id)` / sub-mode transition hook | vmm_arch §2 v1.x patch |
| `firmware-approval` (S14) | `FirmwareOp::MailboxOperation` enum variant + `revoke_all_leases()` API | amdv_safety §S14 v1.x patch |
| `npt` (S3) | mailbox MMIO range 의 NPT mapping 거부 (`is_in_mailbox_mmio_range(host_pa)`) | amdv_safety §S3 v1.x patch |
| `msr-bitmap` (S10.1) | mandatory entry 30+ 항목 (P-state 6 + C-state residency 8 + RAPL 18 + thermal 7 + voltage 4 + SMU vendor MSR) | amdv_safety §S10.1 v1.x patch |
| `io-bitmap` (S11) | mailbox port (PCH PMC port range) default-block | amdv_safety §S11 v1.x patch |
| `cpuid-emul` (S2 sub) | power feature bit 마스킹 (CPUID 0x6 thermal 6 + 0x15+0x16 frequency + 0x80000007 power feature + 0x80000008 RAPL2) | amdv_safety §S2 CPUID emulation v1.x patch |

본 cross-cluster surface extension PR 들은 **amdv 측 doc 의 v1.x patch
형태로 분리 진행** — 4 doc frozen 변경 0.

### 7.3 P-arch sub-decision sign-off ledger

P-arch.1~P-arch.6 의 각 sub-decision 채택 record (amdv_safety §7.3 /
power_safety §6.3 패턴):

| Section | sub-decision 묶음 | 채택 |
|---|---|---|
| **P-arch.1** §1 핵심 결정 + §1.1 참조 자료 | (a)~(i) capsule 명 inline / Threat catalog cross-ref / AV21~AV40 / D1a' patch series / Customizability default definitions / 4-tier secure storage / Cross-cluster reuse / 학술적 차별점 §6 cross-ref / 참조 자료 통합 표 (Linux GPL reference + FreeBSD/DragonFly/TF-A/tss-esapi + Plundervolt/PLATYPUS/Hertzbleed paper) | 2026-05-05 |
| **P-arch.2** §2 capsule 분해 | (A)~(I) 6-column matrix + AV column / LoC budget CI 검사 / `PmCapsuleMsg` 34 verb IPC / cross-cluster trust 의존 boundary check / DAG ASCII 갱신 / Phase D fallback 표 / host-global vs per-VM scope / LeaseCap 확장 | 2026-05-05 |
| **P-arch.3** §3 lease integration | (A)~(H) §3.1 4-step + 4-tier flow / §3.2 Mobile merger 정합 form-factor 표 / §3.3 transportation sudden power loss 강화 + AV24 invariant / §3.4 cross-cluster integration / §3.5 atomicity 책임 분담 / §3.6 Tier dispatch flow / §3.7 Tier 결정 timing boot 1 회 / AV cross-ref | 2026-05-05 |
| **P-arch.4** §4 PR split | (A)~(G) 6-column matrix + S20.2 sub-row + cross-cluster API row + Phase D forward-compat row + ◎/△/○ 정밀 / contribute-back ledger / sub-PR boundary 정합 검증 | 2026-05-05 |
| **P-arch.5** §5 repo 구조 | (A)~(G) workspace member 트리 정확화 + 6 capsule sub-crate src/ layout + dependency 표 + `tools/power.rules.d/` 위치 + proofs 디렉터리 layout 2-축 + 외부 sibling repo cross-ref + cargo dep 라이선스 정합 | 2026-05-05 |
| **P-arch.6** §6 차별점 | (A)~(I) 학술적 / 산업적 분리 + 학술 8 항목 (logicutils 통합 ALP+CLP+Type relations 추가) + 산업 9 항목 + prior art 부재 표현 정합 + Hertzbleed/PLATYPUS/Plundervolt mitigation prior art 부재 confirmed + ISA-agnostic 4-tier + Universal customizability + Convertible+transportation + §6.2 산업 + §6.3 timeline + §6.4 venue + §6.5 artifact + §6.7 prior art ledger 14 row | 2026-05-07 |

### 7.4 v1.x patch / v2 의 정의 (power_safety §6.4 mirror + power_arch-specific)

| 분류 | 정의 |
|---|---|
| **v1.x patch (backwards-compatible)** | (i) §1 12 axis 의 값 갱신 = patch (mechanism 변경 X).<br>(ii) capsule 6 → N 추가 = patch (vmm_arch §2 패턴 정합).<br>(iii) cross-cluster API 의존 surface 변경 = patch (amdv 측 v1.x patch 와 짝).<br>(iv) `PmCapsuleMsg` enum 의 새 verb 추가 = patch.<br>(v) `tools/power.rules.d/` 의 default file 갱신 또는 user override 추가 = patch (mechanism 변경 X).<br>(vi) §4 PR split matrix 의 row 추가 (S24+ 안전장치 또는 새 form-factor) = patch.<br>(vii) §6.7 prior art ledger 갱신 = patch (사실 갱신).<br>(viii) Phase D forward-compat row 의 body 채움 (AV28-D 등) = patch. |
| **v2 (incompatible)** | capsule 갯수 감소 (`PmCapsuleMsg` verb 제거 등) 또는 6 capsule 의 책임 재구성 또는 power-orchestrator 의 trust model 변경 (cluster-scope-trusted boundary 변경) 또는 `PmCapsuleMsg` enum 의 verb 제거 또는 cross-cluster API 의 backward-compat 깨는 변경. |

frozen 후 v1.x patch 는 PR review + paper artifact 갱신, v2 는 별도
재검토 cycle (P-arch.1~6 + power_safety §6 + ARCH-II' 측 호환성 재검토
+ paper revision).

### 7.5 frozen 후 진입 가능 작업 (power_safety §6.6 mirror + 본 doc-specific)

본 doc + 짝 doc `power_safety.md` 양쪽 v1.0 frozen → **PR-5 진입 차단
해제** (phase_plan.md §C 의 8 단계 중 8 번째):

1. ✅ §7.1 sign-off 조건 모두 만족
2. ✅ `power_safety.md` v1.0 frozen 짝 (§7.2)
3. (열림) **PR-5a** — `power-orchestrator` + `audit` capsule power
   op_tag 확장 진입 (§4.4)
4. (열림) **PR-5b** — `lease-pm` capsule + 4-tier secure storage + 11
   ISA backend 진입 (§3.6)
5. (열림) **PR-5c** — `cpufreq` + `msr-bitmap-extension` 진입
6. (열림) **PR-5d** — `acpi-pm` / `rapl` / `wakeup` / `psp-pm` 진입
7. (열림) `tools/power.rules.d/` 의 user override 작성 가능 (build-time,
   §5.3)
8. (열림) Verus AV21~AV40 proof body 채움 (PR-3 짝, §4.5)
9. (열림) `y4-verus2isabelle` 의 power fixture round-trip 검증 (PR-4
   짝, §4.5)
10. (열림) **paper venue draft 시작** — §6.4 의 1 순위 SOSP workshop /
    PLOS 또는 2 순위 IEEE S&P 의 deadline 추적
11. (열림) **`.claude-notes/trackers/power-prior-art-ledger.md` 신설** + Phase
    C 진입 후 새 CVE / 학술 논문 발견 시 §6.7 ledger 갱신 path

§4.5 의 4-way contribute-back (C 패치 / Rust capsule / Verus AV1~AV40 /
Isabelle skeleton) 진입.

---

## 8. 미해결 / 추가 결정 필요

power_safety §7 패턴 정합.

### 8.1 닫힘 ledger (sign-off 또는 sub-decision 으로 해결됨)

| # | 항목 | 닫힘 사유 |
|---|---|---|
| 1 | ARCH-II' 매핑 — 6 capsule + power-orchestrator 의 책임 분담 | P-arch.2 sign-off (2026-05-05) — 6-column matrix + AV column + cross-cluster amdv 측 capsule client API 정합 |
| 2 | 12 axis 핵심 결정 (Base / Power-mgr 위치 / Threat model / 검증 도구 / 검증 기법 / 형상별 정책 / Customizability / Secure storage 4-tier / Cross-cluster reuse / seL4 인터페이스 D1a' patch series / 라이선스 / 학술적 차별점) | P-arch.1 sign-off (2026-05-05) |
| 3 | 참조 자료 통합 표 (Linux GPL reference + FreeBSD/DragonFly/TF-A/tss-esapi BSD-2/3 + Plundervolt/PLATYPUS/Hertzbleed paper) | P-arch.1 §1.1 sign-off + NOTICE 갱신 |
| 4 | LoC budget CI 검사 + IPC interface (`PmCapsuleMsg` 34 verb) | P-arch.2 §2.2.1 + §2.2.2 sign-off |
| 5 | Cross-cluster trust 의존 + boundary check Verus invariant | P-arch.2 §2.4 sign-off |
| 6 | DAG 의존 그래프 + cross-cluster sink 갱신 | P-arch.2 §2.5 sign-off |
| 7 | Phase D fallback 표 (cpufreq conservative / acpi-pm safe-method-only / psp-pm dispatch disable / rapl audit only / wakeup inhibit-all / lease-pm non-recoverable) | P-arch.2 §2.6 sign-off |
| 8 | host-global vs per-VM scope 표 (6 host-global + 1 per-VM) + LeaseCap 확장 | P-arch.2 §2.7~§2.8 sign-off |
| 9 | 4-step seal/unseal flow (4-tier secure storage) + transportation sub-mode sudden power loss 강화 (5 ms suspend latency + fTPM 우선 + AV24 invariant) | P-arch.3 §3.1~§3.3 sign-off |
| 10 | cross-cluster integration 표 (lifecycle / audit / firmware-approval) + atomicity 책임 분담 | P-arch.3 §3.4~§3.5 sign-off |
| 11 | Tier dispatch flow + Tier 결정 timing (boot 1 회 fix) | P-arch.3 §3.6~§3.7 sign-off |
| 12 | 6-column PR split matrix + cross-cluster API surface 확장 표 + Phase D forward-compat row + sub-PR boundary 정합 검증 | P-arch.4 sign-off |
| 13 | 23 workspace member + capsule sub-crate 트리 (per-capsule src/ layout) + dependency 표 + cargo dep 라이선스 정합 | P-arch.5 sign-off |
| 14 | 학술적/산업적 분리 + 학술 8 항목 + 산업 9 항목 + paper venue + artifact 형식 + prior art ledger 14 row | P-arch.6 sign-off (2026-05-07) |
| 15 | **logicutils 의 KB 언어 차별점** — Abductive-Deductive Logic (ALP) + Constraint Logic (CLP) 의 결합 + Type Relations + 빌드 시스템 = 논리 이론 framing.  사용자 강조점 정확 반영 (2026-05-07) | P-arch.6 §6.1.8 sign-off |

### 8.2 v1.x patch 미해결 ledger

(현 시점 비어 있음 — frozen 시 추가될 항목 모두 §7.4 의 v1.x patch 분류
로 편입.)

### 8.3 Phase C 진입 후 신규 unresolved

PR-5a~d 진입 직후 결정:

1. **`PmCapsuleMsg` 의 concrete msgport msg type 또는 scheme verb
   encoding** — §2.2.2 의 추상 형태에서 concrete 화.  vmm_arch §8.1
   (`CapsuleMsg`) 의 결정 패턴 정합 (msgport enum primary + scheme
   verb fallback debug-only).  PR-5a 진입 시 결정.
2. **`tools/power.rules.d/` 의 lu-rule syntax 표준** — overlay merge
   의 정확한 grammar 결정 (TOML / INI / 자체 lu-rule syntax 의 어느
   것).  power_safety §7.3 항목 2 와 짝, logicutils 측 결정 의존
   (`boot/x86_64-debug.rules` 의 기존 syntax 와 정합 검토).
3. **vehicle bus signal driver 의 `y4-drivers` repo 통합 위치** —
   M7 의 CAN / OBD-II / vehicle Ethernet AVB driver 가 `y4-drivers` 에
   추가될지, `wakeup` capsule 안에 통합될지.  power_safety §7.3 항목
   3 와 짝 — 주류 옵션은 y4-drivers sibling repo (transportation
   form-factor 의 기본 driver set) + wakeup capsule 의 abstract signal
   API 분리.
4. **Microbench 산출물 publication 형식** — paper artifact §6.5 의
   (v) 항목, `qemu-smoke` + capsule cluster boot + power workflow
   simulation 의 결과 형식 (CSV / JSON / TSV) + reviewer 친화 visualization
   (matplotlib / plotly / d3.js 의 어느 것).  Phase C 종반 microbench
   측정 시점 결정.
5. **logicutils 측 ACLP query 의 build 측 활용 example** —
   `tools/power.rules.d/` 의 abductive query 의 구체 예시 작성 (예:
   ```
   abduce why_lease_revoked(L):
       lease_revoked(L)
       not (sub_mode_transition_atomic(L) and audit_consistent(L))
       explain "either sub-mode transition or audit was not consistent"
   ```
   ).  paper artifact 의 reviewer 친화 evidence 의 일부.

### 8.4 Phase D 진입 시 검토 영역 (power_safety §7.4 mirror + power_arch-specific)

vmm_arch §8.8 + power_safety §7.4 + 본 doc 의 forward-compat hook —
Phase D 진입 시 spec patch 로 추가:

1. **AV28-D body** — IOMMU programming capsule + per-device BAR cap
   도입 후 `wake_source_iommu_consistent` invariant body 채움 (forward-
   compat hook 활성화, §7.4 v1.x patch).
2. **per-capsule restart policy** (§2.6 Phase D 표) — power capsule
   별 fallback (cpufreq conservative / acpi-pm safe-method-only /
   psp-pm dispatch disable / rapl audit only / wakeup inhibit-all /
   lease-pm non-recoverable).
3. **disk-backed audit persistence** (S12.4 forward-compat hook) —
   power op_tag (30+, §7.2 cross-cluster) 도 disk dump 대상.
4. **R-α / R-γ wake source 정합** — Phase D 의 KVM ioctl 프록시 /
   paravirt agent 가 nested guest 의 wake event 처리.  guest-안-VM
   의 wake source 가 host wake-from-deep-idle 과 정합.
5. **PCIe device passthrough + power management** — Phase D 의 IOMMU +
   per-device BAR 가 own driver guest 의 wake source / power state 와
   정합 (S22.7 USB wake / device wake 의 Phase D 활성화 path).
6. **Hardware enclave (SGX / TZ Realm) 도입 시 Plundervolt secondary
   mitigation** — enclave 측 fault detection + Y4 측 voltage range
   bound (S23.4) 의 보안 layer 통합.

### 8.5 v2 (incompatible) 후보 (power_safety §7.5 mirror)

frozen 후 v2 (incompatible) 단계에서 검토할 변경:

1. **ModeSignal namespace 변경** — string-keyed → typed enum (universal
   customizability 약화 시 검토).  paper review 또는 산업 도입 피드백
   에서 string-keyed 의 type-safety 부족이 issue 될 시 재검토.
2. **`tools/power.rules.d/` overlay merge mechanism 변경** — numbered
   file overlay → JSON / TOML database 등의 structured config.  logicutils
   진화 또는 별도 config tool 도입 시.
3. **4-tier secure storage trust model 변경** — Tier 분리 책임 재구성
   (post-quantum primitive 진화 — XChaCha20-Poly1305 → post-quantum
   AEAD 등 — 시 재검토).

### 8.6 logicutils 측 prior art ledger 갱신 path

§6.7 prior art ledger 의 logicutils 측 항목 (BioMake / Modus / ACLP /
ALP / SCIFF / 등) 의 갱신 path:

- 신규 ACLP / ALP 의 build orchestration 측 application 발견 시 (학술
  논문 또는 산업 도입) — `.claude-notes/trackers/power-prior-art-ledger.md`
  (Phase C 진입 후 신설) 에 row 추가
- logicutils 자체의 진화 (새 utility / KB 언어 확장) 시 — `power_arch.md`
  §6.1.8 의 evidence 갱신 + cross-ref 검토
- §6 학술적 차별점 8 항목 의 영향 재평가 — prior art 부재 주장이
  여전히 holds 하는지 검증

### 8.7 paper venue deadline 추적 ledger

§6.4 의 venue 후보들의 deadline 추적:

- **`.claude-notes/trackers/power-paper-venue-tracker.md`** (Phase C 종반 paper
  draft 시점에 신설)
- 추적 항목: SOSP workshop (PLOS) submission deadline / IEEE S&P
  Oakland deadline / SOSP main track / OSDI main track / USENIX Security
  / EuroSys 등 venue 별 cycle 추적 + 본 doc 의 v1.0 frozen → paper
  draft → submission 의 timeline 정합

### 8.8 `y4-paper-artifact/` sibling repo 생성 시점

§6.6 의 `/home/ybi/y4-paper-artifact/` sibling repo 생성:

- **시점**: Phase C 종반 paper draft 시점 (vmm_arch §8.7 / power_safety
  §8.3 항목 4 정합)
- **메커니즘**: Y4 의 v1.0 frozen tag 에서 cherry-pick 으로 생성 —
  vmm 측 + power 측 산출물 모두 통합
- 산출물 묶음 (§6.5 의 7 항목): Y4 frozen tag + Verus AV1~AV40 + qemu
  reproducibility script + Isabelle skeleton + power microbench + TPM-
  based reproducibility + logicutils-driven artifact verification
