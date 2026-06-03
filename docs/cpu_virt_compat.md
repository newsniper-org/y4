<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 CPU Virtualization vendor-neutrality (AMD-V ↔ Intel VT-x)

> **상태:** v0 declaration (2026-05-07).  ARCH-II' (`docs/vmm_arch.md`)
> + AMD-V 안전장치 catalog (`docs/amdv_safety.md`) 의 vendor-neutrality
> 정책 명시 doc.  본격 Intel VT-x backend code 는 v1.x patch 로
> deferred — 본 doc 은 정책 declaration 만.

본 doc 은 Y4 의 CPU 가상화 layer (vmrun-orchestrator + 11 capsule
cluster) 가 **AMD-V (SVM) 와 Intel VT-x 사이 vendor-neutral 추상**
이라는 사실을 명시.  ARCH-II' v1.0 frozen 의 capsule cluster 는 AMD
VMCB / Intel VMCS / AMD NPT / Intel EPT / 등의 vendor-specific hardware
detail 을 backend file 안에 격리하고, capsule cap ABI 와 Verus invariant
는 vendor-neutral 추상 위에 박혀 있음.

---

## 1. 핵심 declaration

| 축 | 결정 |
|---|---|
| capsule cluster scope | AMD-V (SVM) + Intel VT-x **양방향 vendor-neutral 추상** |
| 추상 boundary | **capsule cap ABI** (orchestrator → capsule 의 `CapsuleMsg` enum, `vmm_arch.md` §8.1) + **Verus AV1~AV20 invariant statement** (vendor-specific detail 미명시, abstract) |
| Vendor-specific 영역 | **capsule sub-crate 안의 backend file** (`<topic>_amd.rs` / `<topic>_intel.rs` 분리) — 자세한 file convention §3 |
| seL4 측 D1a 패치 | **`Y4_AMDV` master flag** (sel4_fork_policy.md §3.3) 가 `KernelSVM` (Y4 신설) + `KernelVTX` (mainline 기존) 둘 다 enable — single flag, runtime vendor 자동 감지 |
| 본격 Intel VT-x backend code | **v1.x patch 로 deferred** (현 frozen 4 doc 변경 0).  본 doc 은 declaration 만 |
| 라이선스 | Apache-2.0 (Y4 single-license).  Intel VT-x 측 reference: bhyve 의 VT-x backend (BSD-2) — 현 vmm_arch.md §1.1 의 reuse manifest 와 정합 |

---

## 2. capsule sub-crate naming — vendor-neutral 재해석 (D=a)

ARCH-II' v1.0 frozen 의 capsule sub-crate 명칭 (`vmm-vmcb` / `vmm-npt`
등) 은 **historical 잔재** — semantic 은 vendor-neutral.  rename X
(현 frozen doc 변경 0):

| capsule | 현 명칭 | semantic (vendor-neutral) | AMD backend | Intel backend |
|---|---|---|---|---|
| `vmm-vmcb` | (현 명칭 그대로) | **vCPU control block** | AMD VMCB | Intel VMCS |
| `vmm-npt` | (현 명칭 그대로) | **nested page tables** | AMD NPT | Intel EPT |
| `vmm-msr-bitmap` | (현 명칭 그대로) | MSR permission bitmap | (vendor-neutral, 동일 표 layout) | (동일) |
| `vmm-io-bitmap` | (현 명칭 그대로) | IO permission bitmap | (vendor-neutral) | (동일) |
| `vmm-cpuid-emul` | (현 명칭 그대로) | CPUID emulation table | (vendor-neutral) | (동일) |
| `vmm-firmware-approval` | (현 명칭 그대로) | firmware mutation pending queue | AMD PSP / SMU | Intel ME / CSME / Intel TXT |
| `vmm-npf-handler` | (현 명칭 그대로) | nested page fault handler | AMD NPF | Intel EPT violation |
| `vmm-audit` | (현 명칭 그대로) | audit ring buffer | (vendor-neutral) | (동일) |
| `vmm-nested-request` | (현 명칭 그대로) | nested SVM/VTX 시도 audit + Phase D R-α/R-γ hook | AMD nested SVM | Intel nested VT-x |
| `vmm-lifecycle` | (현 명칭 그대로) | VCPU lifetime + parent group | (vendor-neutral) | (동일) |

`vmrun-orchestrator` 의 7-step atomic sequence (vmm_arch.md §2.2) 는
vendor-specific:
- AMD-V: clgi → vmsave → vmrun → vmload → stgi
- Intel VT-x: VMXON → VMPTRLD → VMLAUNCH or VMRESUME → VMXOFF

Orchestrator 의 dispatch hub 는 **vendor 감지 후 backend 호출** —
single orchestrator + 2 backend file (`atomic_sequence_amd.rs` + `atomic_sequence_intel.rs`).

---

## 3. Backend file naming convention (E=a)

S20.2 의 11 secure storage backend 패턴 정합.  vendor-specific backend
file 의 naming = `<topic>_<vendor>.rs`:

```
Y4/capsules/vmm-vmcb/src/
├── lib.rs
├── vmcb_amd.rs                  # AMD VMCB layout + raw 필드 access
├── vmcb_intel.rs                # Intel VMCS layout + VMREAD/VMWRITE
├── intercept_floor.rs           # vendor-neutral S2 mandatory mask 검증
└── metadata.rs                  # vendor-neutral L3 deadline ceiling 등 (D-S4)

Y4/capsules/vmm-npt/src/
├── lib.rs
├── npt_amd.rs                   # AMD NPT
├── ept_intel.rs                 # Intel EPT
├── cap_derived.rs               # vendor-neutral S3.1 cap derivation
└── huge_page.rs                 # vendor-neutral S3.2 alignment

Y4/capsules/vmm-firmware-approval/src/
├── lib.rs
├── mailbox_amd_psp.rs           # AMD PSP / SMU
├── mailbox_amd_svi2.rs
├── mailbox_intel_me.rs          # Intel ME / CSME
├── mailbox_intel_txt.rs         # Intel TXT
├── mailbox_intel_vr.rs
└── ... (8 vendor backend, S23.1 패턴 정합)

Y4/vmrun-orchestrator/src/
├── lib.rs
├── main.rs
├── ipc.rs                       # CapsuleMsg dispatch (vendor-neutral)
├── atomic_sequence_amd.rs       # clgi/stgi/vmsave/vmload/vmrun (S7.1)
├── atomic_sequence_intel.rs     # VMXON/VMPTRLD/VMLAUNCH/VMXOFF
├── vendor_detect.rs             # boot 시점 vendor 감지 (CPUID 0 의 vendor string)
└── ... (그 외 vendor-neutral file)
```

vendor 감지 = boot 시점 `CPUID 0` 의 vendor string 검사 (`AuthenticAMD`
vs `GenuineIntel` vs `HygonGenuine` vs ...).  power_safety §S20.2.2 의
`detect_secure_storage_backend()` 패턴 정합.

---

## 4. Verus AV1~AV20 의 vendor-neutrality (F=a)

`amdv_safety.md` §5.2 의 AV1~AV20 + AV2-D + reserved 5 의 모든 statement
는 **이미 vendor-neutral abstract** — `forall vcpu, ...` 같은 quantifier
+ vendor-specific hardware detail 미명시:

```verus
proof fn intercept_floor_holds(vcpu: VCPU)
    ensures s2_mandatory_mask_holds(vcpu)
{ ... }
```

`VCPU` type 자체가 abstract — backend 가 AMD VMCB / Intel VMCS 의 어느
것이든 statement 동일 적용.  vendor-specific 부분은 **backend implementation
의 책임** (Verus 의 `proof fn` body 가 backend 의 specific case 를 cover).

따라서 v1.x patch 의 Intel VT-x backend 추가 시:
- AV1~AV20 statement **변경 0**
- 각 invariant 의 proof body 가 AMD case + Intel case 둘 다 cover —
  case-split (`if backend == Amd { ... } else { ... }`) 또는 backend
  trait 의 generic proof

---

## 5. seL4 측 D1a 패치 — single `Y4_AMDV` flag dispatch (G=a)

`sel4_fork_policy.md` §3.3 의 `Y4_AMDV` master flag 가 vendor-neutral
의미로 재해석:

```cmake
config_option(
    Y4AMDVEnabled
    Y4_AMDV
    "Enable Y4's CPU virtualization safety extensions (AMD-V SVM + Intel VT-x)."
    DEFAULT OFF
    DEPENDS "KernelSel4ArchX86_64"
)
```

`Y4_AMDV=ON` 시:
- seL4 측 `KernelSVM` (Y4 신설) + `KernelVTX` (mainline 기존) 둘 다 enable
- runtime vendor 감지 후 vendor-specific dispatch
- 동일 cap 종류 (`SVMVCPU` / `SVMNPT` / `SVMMsrBitmap` / `SVMIoBitmap`)
  — naming 은 historical 잔재 (D 정책), semantic vendor-neutral

flag 명칭 변경 X (현 frozen `sel4_fork_policy.md` §3.3 변경 0).  v1.x
patch 시 description 의 갱신 (`SVM` → `CPU virtualization`) 만, flag
이름 자체는 보존.

---

## 6. v1.x patch 진행 path

본 doc v0 frozen 후 (5 doc 짝 — H 정책) Intel VT-x 본격 backend 진입:

| 단계 | 작업 | 분류 |
|---|---|---|
| (1) | `vmm-vmcb` capsule 의 `vmcb_intel.rs` backend 작성 (Intel VMCS layout + VMREAD/VMWRITE) | v1.x patch (vmm_arch §7.4) |
| (2) | `vmm-npt` capsule 의 `ept_intel.rs` backend 작성 (Intel EPT) | v1.x patch |
| (3) | `vmrun-orchestrator` 의 `atomic_sequence_intel.rs` 작성 (VMXON/VMPTRLD/VMLAUNCH/VMXOFF) + `vendor_detect.rs` | v1.x patch |
| (4) | seL4 측 D1a 패치의 `KernelVTX` enable 적용 (mainline 기존 코드 사용) — patch 형식: 현 D1a patch series 의 후속 numbered file (`002-vtx-enable.patch` 같은) | sel4_fork_policy.md §6.3 numbering 정합 |
| (5) | Verus AV1~AV20 의 proof body 의 Intel case 추가 (statement 변경 0) | v1.x patch |
| (6) | bhyve 의 VT-x backend 알고리즘 reference (BSD-2 attribution) — `~/y4-upstream-refs/bhyve/sys/amd64/vmm/intel/` | NOTICE 갱신 |
| (7) | qemu-smoke 의 Intel host 측 boot 검증 + microbench 측정 (G7 timing-equal 갱신) | sel4_fork_policy §3.6 |
| (8) | paper artifact 의 Intel VT-x reproducibility (qemu + Intel CPU emulation) | vmm_arch §6.5 / power_arch §6.5 |

진행 시점: ARCH-II' frozen 마킹 + Phase 4-power frozen 마킹 후 Phase
C 진입 단계에서 PR-2 와 같은 cycle 또는 후속 PR.

---

## 7. 동결 정책

본 doc 은 v0 declaration.  `v1.0 frozen` 마킹 조건:
- §1 핵심 declaration 7 row sign-off
- §2 capsule naming vendor-neutral 재해석 표 sign-off
- §3 backend file naming convention sign-off
- §4 Verus AV1~AV20 vendor-neutrality 정책 sign-off
- §5 seL4 D1a flag single-dispatch 정책 sign-off
- §6 v1.x patch 진행 path sign-off
- 짝 doc — `vmm_arch.md` / `amdv_safety.md` / `sel4_fork_policy.md` /
  `verus_to_isabelle.md` 의 v1.0 frozen 과 **짝 frozen** (5 doc 묶음)
  — 단 본 doc 은 v0 신설이라 frozen 마킹은 짝의 *다음 cycle* (현
  cycle 의 Phase 4-power 마킹과 동시 또는 별도)

frozen 후 변경: vmm_arch §7.4 / amdv_safety §7.4 의 v1.x patch / v2 분류
정합.

---

## 8. 미해결 / 추가 결정

1. **Hygon (AMD 호환 중국 vendor) + Centaur (VIA) + Zhaoxin (VIA 호환)
   등의 비-AMD/Intel x86-64 vendor** — vendor-neutrality 를 본격 확장 시
   추가 backend 필요.  Hygon 은 AMD-V (SVM) clone 이라 `vmcb_amd.rs` 그대로
   적용 가능 가능성 ↑ — 단 vendor string 차이 (`HygonGenuine` vs
   `AuthenticAMD`).  Phase D 또는 v1.x patch 검토.
2. **Intel TDX (Trust Domain Extensions, Sapphire Rapids+) 측 backend** —
   Intel 의 confidential VM, AMD SEV-SNP 등가물.  power_safety §S20.2
   의 Tier 1 secure storage 의 `storage_tdx.rs` 와 짝 — 본 cpu_virt_compat
   doc 은 hypervisor 측 VMCS 만 다룸, TDX 의 confidential VM 측은 별도
   spec 영역
3. **AArch64 + RISC-V + POWER + IBM Z + SPARC + MIPS64 의 hypervisor
   primitive** — power_safety §S20.2 의 4-tier secure storage 와 정합
   하지만 hypervisor 측은 별도 spec.  Y4 의 multi-arch 목표 (CLAUDE.md
   §8) 에 따라 Phase D / v2 단계에서 검토
4. **Lean4 / OxiLean 측 verification backend** — adsmt 측 `prover_emit/
   lean` 의 OxiLean path 가 adsmt v1.1.x 도달 시점에 활성화 예정.
   2026-06-01 추가 진척: leo4 v1.0.0-rc.1 ~ rc.4 hot patch chain (typed-
   enum lowering 3 조건 모두 satisfied), L1/L2/L3 unblocked, adsmt-lean-
   binding pin = `v1.0.0-rc.4` — L4 mslean4 path 의 `feat/mslean4-lecq-
   lecr-ipcs` branch 완료만 대기 (mainline Lean 4 path 는 v1.2.x post-RC).
   Y4 측 verification workflow 재설계 (`feedback_adsmt_v1_verification_
   redesign.md`) 의 R1=(a') 결정으로 현 시점 deferred — adsmt v1.1.x
   도달 시 retrofit trigger 활성: P-redesign.3 R3.10 (`av-proof-body-
   tracker.md` §1 + §8) 에 따라 AV proof body 의 `.lean.rs` 자동 emission
   (adsmt-emit-lean wrapper 정의 + `verus_to_isabelle.md` §3.2 file path
   표에 `.lean.rs` 매핑 추가 + cluster sub-PR retrofit).  관련 tracker:
   `.claude-notes/trackers/adsmt-integration-tracker.md` §10.5 +
   `av-proof-body-tracker.md` §8.
