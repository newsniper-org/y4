<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Power Management 아키텍처 — capsule-decomposed

> **상태:** v0 design draft (2026-05-05 진입).  ARCH-II' (vmm_arch.md)
> 의 capsule 분해 패턴을 power domain 에 적용.  6 capsule + 1
> orchestrator.  짝 doc = `docs/power_safety.md` (안전장치 catalog).

본 문서는 Y4 의 power management capsule cluster 디자인.  `power_safety.md`
의 S15+ 안전장치들이 *어디 (어느 capsule) 에* 구현되며 *어떻게* 격리되는지
정의.

---

## 1. 핵심 결정

| 축 | 결정 |
|---|---|
| Base | seL4 + ARCH-II' (vmm_arch.md v1.0 frozen) — 변경 없음 |
| **Power-mgr 위치** | **Y4 의 capsules 패턴 안 — 6 capsule + 1 orchestrator** |
| Threat model | power-mgr core (orchestrator) trusted, intercept handler 들 capsule 격리.  vmm_arch.md §2.3 의 trust model 패턴 재사용 |
| 검증 도구 | Verus (AV21~AV30+) — vmm_arch.md 와 같은 chain |
| 검증 기법 | VeriSMo 2-layer concurrency 패턴 (vmm_arch.md §3.1 정합) |
| 형상별 정책 | logicutils `tools/power.rules` (`power_safety.md` §2.3) |
| seL4 인터페이스 | 필요 시 D1a' 패치 — power MSR mediation, ACPI eval mediation (`sel4_fork_policy.md` Strictly Additive Fork 정합) |
| 라이선스 | Apache-2.0 (Y4 single-license).  코드 reference (linux power-mgr / FreeBSD acpi / DragonFly powerd) 알고리즘 port 시 BSD attribution 보존 |

---

## 2. Capsule 분해 — 6 개 capsule + 1 orchestrator

### 2.1 Capsule 목록

| # | Capsule | 책임 | 안전장치 |
|---|---------|------|---------|
| 1 | **cpufreq** | P-state governor (performance / ondemand / powersave / userspace) + DVFS 정책 + cross-tenant frequency observable 격리 | S15, S19 |
| 2 | **acpi-pm** | ACPI _PSx / _CST / _PSV / _TCC method 평가 + guest 의 ACPI eval mediation | S18, S21 |
| 3 | **psp-pm** | AMD PSP / Intel PCH 의 power mailbox + S14 firmware-approval 정합 | S23 |
| 4 | **rapl** | RAPL energy counter (MSR 0x611 / 0x619 / 0x639 등) 격리 + cross-tenant audit | S17 |
| 5 | **wakeup** | wake source routing (interrupt / GPE / device wake) + spurious wake 차단 | S22 |
| 6 | **lease-pm** | lease 의 deep idle / hibernate suspend / resume semantics + atomicity | S20 |

### 2.2 Power Orchestrator (trusted core)

`y4-power-orchestrator` — workspace member, 가장 작은 trusted 구성 요소.

책임:
- C-state 진입/이탈 결정 (어느 CPU 가 어떤 C-state)
- form-factor profile 적용 (logicutils `tools/power.rules` 의
  `Y4_PM_DEEP_C_STATE_MAX` 등)
- 6 capsule cluster 의 dispatch hub
- vmrun-orchestrator (ARCH-II') 와의 협력 — vCPU 의 deep idle 진입은
  power-orchestrator 에 위임

#### 2.2.1 LoC budget

`Y4_PM_ORCHESTRATOR_LOC_BUDGET` build-time const, **default 600 LoC**.
CI 자동 검사 (vmm_arch.md §2.2.1 패턴 정합).  vmrun-orchestrator (800
LoC) 와 별도 budget — 두 trusted core 가 분리.

### 2.3 vmrun-orchestrator 와의 협력

- vCPU 가 `HLT` 발화 (S2 의 INTERCEPT_HLT 가 mandatory) → vmexit →
  vmrun-orchestrator 가 power-orchestrator 에 IPC ("vCPU N idle, suggest
  deep idle")
- power-orchestrator 가 form-factor profile 에 따라 결정 — laptop 형상
  은 deep C-state 진입 권고, server-farm 은 C1 max
- deep idle 진입 시 lease-pm capsule 이 lease state suspend (S20)

### 2.4 Capsule trust model

vmm_arch.md §2.3 정합:
- power-orchestrator = fully trusted (TCB 보조)
- 6 capsule = cluster-scope-trusted
- guest = untrusted (ACPI eval / MSR access 모두 capsule 측 input
  validation)

### 2.5 Capsule 의존 그래프 — DAG 보장

```
                power-orchestrator
                     │  (single source)
       ┌────┬────┬────┼────┬────┐
       ▼    ▼    ▼    ▼    ▼    ▼
    cpufreq acpi-pm psp-pm rapl wakeup lease-pm
       │    │    │    │    │    │
       └────┴────┴────┴────┴────┴────┐
                                      ▼
                            ┌──────────────────┐
                            │  audit (vmm)     │  (cross-cluster sink — 기존 capsule)
                            │  lifecycle (vmm) │  (cross-cluster sink — 기존 capsule)
                            └──────────────────┘
```

audit / lifecycle capsule 은 **vmm_arch.md 의 기존 capsule 재활용** —
power 측이 별도 신설 X.  AuditEntry schema (S12.2) 에 power 관련
op_tag 추가 (v1.x patch 형태로 amdv_safety.md §7.4 정합).

### 2.6 Capsule fault 시 거동

vmm_arch.md §2.5 패턴 정합 — capsule fault → cluster 전체 lease revoke.
Phase D 의 per-capsule restart policy 에 power capsule 도 포함.

---

## 3. Lease 와 power 의 정합

### 3.1 Deep idle / hibernate 진입 시 lease state

`lease-pm` capsule 의 책임:
- deep C-state (≥ C3) 진입 시 lease 의 secret state (XChaCha20 key /
  HKDF segment key) 를 secure storage (PSP-protected SRAM 또는
  in-memory encrypted) 로 이동
- 진입 atomicity 보장 — 진입 중 다른 CPU 가 lease 사용 시도 0
- 이탈 (wake) 시 state restore + integrity check (HMAC) 실패 시 lease
  revoke

### 3.2 Form-factor 별 suspend 정책

| Form factor | deep idle 시 lease 거동 |
|---|---|
| server-farm | suspend X (latency tail 차단) |
| rack-mount | C1 까지만, suspend X |
| laptop | C6 deep idle 시 suspend ON (lease state SRAM 보호) |
| handheld + 독 | dual-mode — battery 모드 suspend ON, 독 모드 OFF |
| SoC | always-on 또는 duty-cycle 의 idle 구간 suspend |

---

## 4. PR split 매트릭스 (안전장치 ↔ seL4 / orchestrator / capsule)

| 안전장치 | seL4 측 (D1a') | orchestrator | capsule |
|---|:---:|:---:|---|
| S15 cpufreq 격리 | △ MSR mediation | ◎ governor 결정 dispatch | **cpufreq** |
| S16 C-state side-channel | ◎ HLT/MWAIT intercept | ◎ C-state 결정 | **cpufreq** + **lease-pm** (S20 짝) |
| S17 RAPL 격리 | ◎ MSR 0x611 etc bitmap | △ | **rapl** + **audit** (cross-cluster sink) |
| S18 ACPI 검증 | △ ACPI MSR mediation | △ | **acpi-pm** |
| S19 SMT power gating | ◎ S5 정합 (lifecycle) + APIC 측 | ◎ pair grouping | **cpufreq** + **lifecycle** (cross-cluster) |
| S20 deep idle lease suspend | △ HLT path | ◎ entry/exit hub | **lease-pm** + **vmcb** (cross-cluster, vmcb metadata) |
| S21 thermal throttle | ◎ _TCC MSR mediation | △ | **acpi-pm** |
| S22 wake source | ◎ GPE / IRQ remap | ◎ wake routing | **wakeup** |
| S23 PSP/PCH mailbox | ◎ MSR mediation | ◎ S14 firmware-approval forward | **psp-pm** + **firmware-approval** (cross-cluster) |

3-column 매트릭스 (vmm_arch.md §4 패턴 정합).

---

## 5. Repo 구조

### 5.1 Y4 워크스페이스 안 (신규)

| 경로 | 형태 | 내용 |
|---|---|---|
| `Y4/proofs/verus/src/power/` | 신규 디렉터리 | `upper/` + `lower/` 안에 per-capsule + per-안전장치 파일 |
| `Y4/capsules/pm-{cpufreq,acpi-pm,psp-pm,rapl,wakeup,lease-pm}/` | 신규 sub-crate **6 개** | 각 capsule = 독립 workspace member |
| `Y4/power-orchestrator/` | 신규 workspace member | Trusted core, ~600 LoC budget |

ARCH-II' 의 16 workspace member + 신규 7 (orchestrator + 6 capsule) =
**23 workspace member**.

### 5.2 logicutils 통합

`tools/power.rules` (lu-rule 형식) — `power_safety.md` §2.3 cross-ref.

### 5.3 ISO 동봉

vmm_arch.md §5.2 패턴 정합 — 동적 로드.  power capsule 도 lease 발급 시
필요한 것만 load.

---

## 6. 학술적 차별점

1. **5 form factor cross-portable verified power-mgr** — 단일 verified
   base + form-factor 별 logicutils rule
2. **Hertzbleed / DVFS side-channel 의 capsule-level 격리 + Verus
   invariant** — DVFS 의 secret-dependent timing 차단을 verified property
   로 박은 첫 사례 (공개된 prior art 부재)
3. **VMM (vmm_arch.md) + power-mgr 의 통합 capsule cluster** — vCPU
   idle ↔ host C-state 의 정합이 verified
4. **lease-aware deep idle suspend** — XChaCha20 key 의 SRAM 보호
   + HMAC integrity check

---

## 7. 동결 정책

본 문서는 v0 design draft.  `v1.0 frozen` 조건:
- §1 핵심 결정 (8 axis) sign-off
- §2 capsule 분해 (6 capsule + orchestrator) sign-off
- §3 lease integration sign-off
- §4 PR split sign-off
- §5 repo 구조 sign-off
- 짝 doc `docs/power_safety.md` v1.0 frozen 과 짝

vmm_arch.md / amdv_safety.md / sel4_fork_policy.md / verus_to_isabelle.md
의 v1.0 frozen 과는 짝 X — 본 doc 은 power domain 의 별도 v1.0 cycle.

---

## 8. 미해결 / 추가 결정 필요

(sign-off cycle 진행 중 채움)
