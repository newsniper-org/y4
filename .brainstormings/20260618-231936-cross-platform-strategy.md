<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Cross-platform 대응 전략 — multi-ISA (AArch64 / RISC-V / POWER / ARCv3)
created: 2026-06-18T23:19:36+09:00   # KST (UTC+9)
status: brainstorming (결정 X — 옵션 탐색).  rev 2026-06-19 — bhyve/nvmm reference 확보 + virt port 가능성 (질문 2)
scope: Y4 전체의 multi-ISA 포팅 전략.  x86_64 (현 first) 외 4 ISA.
       virtualization / MMU / boot / endianness / verified base / tooling
target_isas:
  - AArch64 (little-endian 한정)
  - RISC-V RV64GC
  - IBM POWER / PowerPC 64-bit (little + big endian 둘 다)
  - ARCv3 64-bit (Synopsys ARC HS6x)
refs:
  - CLAUDE.md §1 (5 form-factor) / §8 D2 (x86_64 first)
  - docs/cpu_virt_compat.md (vendor-neutrality AMD-V↔VT-x — ISA 확장 토대)
  - docs/amdv_safety.md (S1~S14 + AV1~AV20, AMD-V 특화 → arch-neutral 화)
  - docs/vmm_arch.md §8.1 (CapsuleMsg ABI — 추상 boundary)
  - third_party/sel4 (seL4 multi-arch verified base)
---

# Cross-platform 대응 전략 (multi-ISA)

## 0. 발제 (사용자 2026-06-18)

x86_64 (현 first, D2) 외 4 ISA 대응:
- **AArch64** (little-endian 한정)
- **RISC-V RV64GC**
- **IBM POWER / PowerPC 64-bit** (LE + BE 둘 다)
- **ARCv3 64-bit** (Synopsys ARC HS6x)

## 1. 현 상태 + 왜 multi-ISA

- 현: **x86_64 first** (D2, CLAUDE.md §8) — AMD-V (SVM) + Intel VT-x
  vendor-neutral (cpu_virt_compat.md).  다른 arch 는 "form factor 작업
  시 추가".
- **왜**: 5 form-factor (CLAUDE.md §1) 가 ISA 다양:

| form-factor | 자연 ISA |
|---|---|
| server-farm host | x86_64 (AMD/Intel) + AArch64 (ARM server) + POWER (IBM) |
| special-purpose laptop | x86_64 + AArch64 |
| rack-mount node | x86_64 + AArch64 + POWER |
| handheld + dock | AArch64 + RISC-V |
| embedded SoC/SoM | RISC-V + ARCv3 + AArch64 (저전력) |

## 2. ISA-independent vs ISA-dependent 분해 (★ 전략의 토대)

Y4 의 가치 대부분이 **ISA-independent** (Rust + arch-neutral) — 이게
multi-ISA 비용을 낮추는 구조적 이점:

| 영역 | ISA 의존성 |
|---|---|
| IPC (scheme + msgport) | **independent** (Rust, arch-neutral) |
| allocator front-end (SLUB/SLOB-style) | **independent** (단 atomic-free §0.5 가 arch memory model 과 무관해짐 — 이점) |
| capsule typing | **independent** |
| lease capability schema | **independent** (단 HIU MMIO byte order = endian, §4C) |
| Verus invariant statement | **대개 independent** (abstract, cpu_virt_compat F=a) |
| **CPU virtualization** | **dependent** (SVM/VMX/EL2/H-ext/PowerVM) |
| **MMU / page table** | **dependent** (NPT/EPT/stage-2/G-stage/radix) |
| **boot** | **dependent** (Limine/U-Boot/OPAL) |
| **interrupt controller** | **dependent** (APIC/GIC/PLIC/XIVE) |
| **atomic / memory model** | dependent — **단 §0.5 atomic-free 가 이를 회피!** |
| **endianness** | dependent (POWER BE, §4C) |

**핵심 관찰**: allocator 의 §0.5 (atomic-free) 가 여기서 *또* 이득 —
arch 별 memory model (x86 TSO vs ARM/RISC-V/POWER weak ordering) 의
차이가 atomic 동기화에서 가장 까다로운데, atomic 을 안 쓰면 그 arch 차이
가 allocator 에서 사라짐.  per-CPU exclusive + critical section 은
arch-neutral.

## 3. 4 ISA 별 특수성

| ISA | virtualization ext | seL4 verified | Rust target | endian | bootloader | **virt reference (질문 2, 2026-06-19)** |
|---|---|---|---|---|---|---|
| **AArch64** | ARMv8 Virt Ext (EL2 + VHE) / stage-2 | **verified** (2024+ 진전) | tier 1 (`aarch64-unknown-none`) | LE (한정) | Limine ✅ / U-Boot | **bhyve `sys/arm64/vmm/` ✅** (EL2 + VHE + nVHE, hyp.S) |
| **RISC-V RV64GC** | H-extension (VS/G-stage) | **verified** (seL4 RV64) | tier 2 (`riscv64gc-unknown-none-elf`) | LE | Limine ✅ / U-Boot / OpenSBI | **bhyve `sys/riscv/vmm/` ✅** (H-ext, SBI, APLIC, vtimer) |
| **IBM POWER 64** | PowerVM / PAPR / KVM-HV / radix MMU / XIVE | **미지원** (seL4 POWER 없음) | tier 2 (`powerpc64le` + `powerpc64` BE) | **LE + BE** | OPAL / petitboot | **부재** (bhyve/nvmm 둘 다 powerpc vmm 없음) → spec 기반 자체 / KVM-PPC 은 GPL 불가 |
| **ARCv3 64** | 제한적/불확실 (HS6x virt?) | **미지원** | tier 3 / 미지원 (Rust ARCv3-64 거의 X) | LE | U-Boot | **부재** (greenfield) |

**성숙도 순**: AArch64 (성숙) > RISC-V (성숙, open) ≫ POWER (seL4 포팅
선결 + endian) ≫ ARCv3 (seL4 + Rust + virt 모두 부재, 거의 greenfield).

### 3.1 질문 2 — 다른 arch 의 virt 코드도 bhyve/NVMM port 가능? (★ 답: 부분적으로 YES)

x86 SVM 을 bhyve/NVMM 알고리즘 port 하기로 결정 (vmm_arch §1.1).  사용자
질문: 다른 arch 도?  **bhyve/nvmm 을 `~/y4-upstream-refs/` 에 확보
(2026-06-19) 후 sparse-checkout 으로 확인한 결과**:

| arch | bhyve | nvmm | 결론 |
|---|---|---|---|
| x86 (AMD SVM + Intel VMX) | `sys/amd64/vmm/{amd,intel}/` ✅ | `sys/dev/nvmm/x86/{svm,vmx}` ✅ | **2개 reference** (이미 결정) |
| **AArch64** | `sys/arm64/vmm/` ✅ (vmm_hyp_el2.S / vhe / nvhe) | 부재 (x86 only) | **bhyve 단독 port 가능** |
| **RISC-V** | `sys/riscv/vmm/` ✅ (vmm_riscv.c / vmm_sbi.c) | 부재 | **bhyve 단독 port 가능** |
| **POWER** | 부재 (powerpc vmm 없음) | 부재 | **BSD reference 부재** — spec 기반 자체 (KVM-PPC 은 GPL → 직접 port 불가) |
| **ARCv3** | 부재 | 부재 | **부재** (greenfield) |

> **⚠️ NVMM ≠ NetBSD portability (확인 2026-06-19)**: NetBSD *OS* 는
> 60+ arch portable (가장 portable BSD) 이지만, **NVMM (`sys/dev/nvmm/`,
> NetBSD 의 type-2 hypervisor) 은 `x86/` 만** (svm + vmx).  `git ls-tree`
> 로 확인 — nvmm 하위 디렉터리는 `x86/` 하나뿐.  `sys/arch/{aarch64,
> sparc64}/include/hypervisor.h` 는 NetBSD 가 *게스트* 로서 firmware
> hypervisor (SPARC sun4v 등) 와 인터페이스하는 헤더 — host-side virt
> backend 아님.  "arch 위에서 OS 가 돈다 (portability)" ≠ "그 arch 의
> HW 가상화로 VM 을 돌린다 (NVMM)".  → AArch64/RISC-V virt reference 는
> **bhyve 단독**, nvmm 기여 0.

**핵심 결론**: bhyve 가 *이미 multi-arch VMM* (amd64 + arm64 + riscv) →
AArch64/RISC-V 의 virt backend 를 **x86 SVM 과 동일한 bhyve reference
에서 일관되게 algorithm port** 가능.  cpu_virt_compat 의 `<topic>_<arch>.rs`
backend (§4.B) 가 모두 *같은 출처* (bhyve) 에서 나옴 → 일관성 + BSD-2
attribution 통일.  단 **POWER/ARC 는 BSD reference 마저 부재** — virt
backend 를 spec (PAPR / ARCv3 PRM) 기반 자체 구현 (KVM 은 GPL 이라 직접
port 불가, license 격리 필요).

이것이 §5 우선순위를 강화: AArch64/RISC-V = (seL4 verified + Rust +
**bhyve reference**) 3박자.  POWER/ARC = (seL4 부재 + reference 부재 +
[ARC 는 Rust 까지 부재]) → 비용 차원이 다름.

### 3.2 bhyve/nvmm 외 non-GPL virt reference 대안 (사용자 질문 2026-06-19)

POWER/ARC (bhyve/nvmm 부재) + 일반 대안.  **경로 3가지**:

**경로 A — 다른 permissive hypervisor**:
| 출처 | license | arch | 비고 |
|---|---|---|---|
| OpenBSD `vmm`/`vmd` | ISC | x86 (VMX+SVM) | permissive, x86 only |
| **Hafnium** (Google) | BSD-3 | **AArch64** | type-1, FF-A secure partition — ARM 보조 reference |
| ACRN | BSD-3 | x86 (Intel) | embedded hypervisor |
| **libsel4vm** (seL4 community) | BSD-2 | **AArch64 + x86** | ★ **seL4 위 VMM** — Y4 가 seL4 위라 가장 정합 (단 user-level VMM 구조 차이 검토) |
| Bareflank | MIT | x86 | hypervisor SDK |

**경로 B — spec 기반 clean-room (license 원천 무관)** ★ 근본 해법:
- ISA virtualization spec 은 **모두 공개 문서** → GPL 코드 안 보고 자체
  구현 (clean-room) → license 문제 원천 차단:
  - AMD SVM = AMD APM Vol.2 / Intel VMX = Intel SDM Vol.3
  - ARM = ARM ARM (EL2/VHE) / RISC-V = RISC-V Priv **H-ext spec**
  - **POWER = PAPR + Power ISA** (공개) / **ARC = ARCv3 PRM**
- 비용 ↑ (reference 코드 없이 spec 만) 이나 **formal-first 와 직결** —
  spec → Verus invariant → 구현 (Y4 의 spec-우선 워크플로 그 자체).
- **POWER/ARC 의 현실적 답**: bhyve/nvmm 부재 + KVM-PPC GPL 불가 →
  PAPR / ARCv3 PRM **clean-room**.  skiboot/OPAL (Apache-2) 는 POWER
  firmware/부팅 참고 (가상화 backend 아님, 단 HW 인터페이스).

**경로 C — rust-vmm 생태계** (Apache-2 / BSD-3):
- `vm-memory` / `vm-superio` / virtio crate — device model / virtio
  **device 측 재사용** (Rust, Y4 정합).  단 **hypervisor extension
  backend 는 아님** (KVM API wrapper).  device 횡단 재사용만.

**license 매트릭스**:
- ✅ Apache-2 호환: BSD-2/3, MIT, ISC — attribution 보존만
- ⚠️ 회피: CDDL (illumos bhyve) — Apache-2 와 file-level 비호환 우려
- ❌ 불가: GPL-2 (KVM / Xen / Jailhouse / Xvisor) — Y4 single-license
  Apache-2 비호환

**Y4 권고**:
- x86/AArch64/RISC-V: **bhyve** (확보) 1차 + AArch64 보조 = **Hafnium
  (BSD-3)** / **libsel4vm (BSD-2, seL4 정합)**
- POWER/ARC: **spec clean-room (경로 B)** — reference 부재 + GPL 회피 +
  formal-first 정합 (비용 ↑ 수용)
- device/virtio: **rust-vmm (경로 C)** 횡단 재사용
- 즉 **virt backend reference 부재 = 막힘 아님** — 경로 B (spec clean-
  room) 가 모든 arch 에 항상 열려 있음 (공개 spec).  bhyve 같은 코드
  reference 는 *비용 절감* 이지 *필수* 가 아님.  Y4 의 formal-first 가
  오히려 spec clean-room 과 자연 정합 (spec → invariant → 구현).

## 4. cross-platform 전략 축

### 4.A ISA-independent core 최대화 (HAL trait)
- arch-dependent 를 **trait 뒤로 격리** (PageBackend 패턴 — `page_backend.rs`
  가 이미 seL4 호출 추상화).  arch-specific = trait impl, core = trait
  소비.
- arch 별 `arch/<isa>/` crate + 공통 HAL trait.  cfg(target_arch) 는
  최소 (trait dispatch 우선).
- Y4 가치 (§2 independent) 는 **1회 작성 + 1회 Verus 증명** → 모든 arch
  재사용.  arch-dependent 만 per-arch.

### 4.B virtualization 추상화 (cpu_virt_compat 의 ISA 확장)
- cpu_virt_compat 의 패턴 (**abstract AV invariant + per-backend file
  `<topic>_<vendor>.rs`**) 을 **arch 축으로 확장**:
  ```
  vmm-vmcb/src/
  ├── vcb_amd.rs / vcb_intel.rs        # x86 (현)
  ├── vcb_arm_el2.rs                   # AArch64 (VMCB→vCPU sysreg context)
  ├── vcb_riscv_h.rs                   # RISC-V (hgatp / VS-mode CSR)
  ├── vcb_power_pvm.rs                 # POWER (partition context)
  └── intercept_floor.rs               # arch-neutral (이미 abstract)
  ```
- **AV1~AV20 statement 가 이미 abstract** (cpu_virt_compat F=a) → arch-
  neutral 그대로, per-arch refinement 만 추가.  vmm-npt = NPT/EPT/stage-2
  /G-stage/radix backend.
- arch 별 hypervisor primitive 매핑:
  - intercept floor (S2): AMD intercept_words / Intel VMCS controls /
    ARM HCR_EL2 / RISC-V hideleg+hedeleg / POWER LPCR
  - nested page (S3): NPT / EPT / stage-2 / G-stage / radix partition-scoped
  - deadline (S4, AV3): arch timer (x86 TSC / ARM CNTV / RISC-V time CSR
    / POWER DEC) — vmrun_terminates_within 은 arch-neutral
- `Y4_AMDV` master flag → `Y4_VIRT` arch-neutral flag (runtime arch+vendor
  감지).
- **★ soft-MMU / soft-virt fallback 명시적 배제** (vkernel drop 교훈,
  `20260620-153653-dragonfly-vkernel-y4-reinterpretation.md` §3.2):
  하드웨어 virt 부재 환경에서 *소프트웨어 nested page* (DragonFly
  vkernel 의 `MAP_VPAGETABLE` 류) 로 production 격리를 흉내내는 fallback
  은 **채택 X**.  근거 = vkernel 이 정확히 그 이유로 6.0 에서 drop
  (소프트웨어 MMU 가 VM 코어 강결합 → 유지보수 부담 + verification
  복잡).  Y4 는 모든 arch 에서 **하드웨어 nested page (NPT/EPT/stage-2/
  G-stage/radix) 전제**.  HW virt 부재 환경 = production 대상 아님
  (오직 dev/test mode → Y4 dev mode 문서, soft backend 는 test 전용).
- AArch64/RISC-V/ARCv3 = LE.  **POWER 만 LE + BE 둘 다** (POWER8+ LE
  주류, BE legacy/특정 workload).
- Y4 = **LE 우선** (x86/ARM/RISC-V 다 LE), POWER BE 는 특수 처리.
- endian 영향: capability token / IPC message serialization / HIU MMIO
  byte order / network.
- **Rust type-safe byte order**: 명시적 `to_le_bytes`/`to_be_bytes`,
  `#[repr(C)]` + endian-aware accessor.  raw `transmute` 금지.  byteorder
  류 패턴.  endian 을 **type 으로** (예: `Le<u64>` / `Be<u64>` newtype)
  → 컴파일 시점 byte order 강제 (capability §contract-by-construction
  정신).
- **HIU MMIO**: WaveTensor RTL 의 endianness 고정 (LE 추정) → BE host
  에서 MMIO accessor 가 swap.  HIU ABI 에 endian 명시 (⏳ hiu_abi).
- Verus: endian round-trip invariant (`from_le(to_le(x))==x`) — 자명.

### 4.D verified base 차등 (seL4 가 전략을 끌고 감) ★
- Y4 = seL4 위 (verified base 재사용, CLAUDE.md §6.5).  → **seL4 가
  지원/verified 한 arch 만 자연스럽다**.
- **AArch64 / RISC-V = seL4 verified** → Y4 의 formal-first + hard
  real-time (real-time §1) 보장이 그대로.  1·2순위.
- **POWER / ARCv3 = seL4 미지원** → seL4 를 그 arch 로 **포팅 선결**
  (막대 — microkernel 포팅 + verification 재작업).  Y4 의 verified base
  이점 상실 → port 비용 ≫.  후순위.
- 즉 "verified base 재사용" 원칙이 arch 우선순위를 **강제** — AArch64/
  RISC-V 먼저, POWER/ARC 는 seL4 측 진전 대기.

### 4.E tooling
- **Rust target**: AArch64 tier1 / RISC-V tier2 / POWER tier2 (le+be) /
  ARCv3 미지원 (Rust 측 ARCv3-64 port 선결).
- **bootloader**: Limine (x86/ARM/RISC-V ✅) / POWER OPAL / ARC U-Boot.
  reuse manifest (README) 의 Limine 1st 가 ARM/RISC-V 까지 커버, POWER/
  ARC 는 별도.
- **build matrix**: logicutils per-form-factor rules (`boot/<ff>.rules`)
  를 per-(form-factor, ISA) 로 확장.  D1 (logicutils) 정합.

### 4.F form-factor ↔ ISA 매핑 (§1 표) → build target matrix
- 각 form-factor 의 ISA 집합 → CI build matrix.  embedded = RISC-V/ARCv3
  (SLOB-variant, real-time §) / server = x86/AArch64/POWER (SLUB-style) /
  handheld = AArch64/RISC-V.
- allocator pluggable (pluggable §) 과 정합 — form-factor 가 (ISA,
  front-end) 둘 다 선택.

## 5. 우선순위 (전략 결론)

| 순위 | ISA | 근거 (3박자: seL4 verified + Rust + virt reference) | 비용 |
|---|---|---|---|
| 1 | **AArch64** (LE) | seL4 verified + Rust tier1 + EL2/VHE + **bhyve arm64 reference ✅** + 광범 form-factor | 中 (adapter 수준) |
| 2 | **RISC-V RV64GC** | seL4 verified + Rust tier2 + H-ext + **bhyve riscv reference ✅** + open | 中 (adapter 수준) |
| 3 | **POWER 64** | server/rack 수요, 단 **seL4 포팅 선결 + virt reference 부재 + endian (BE)** | ≫ (seL4 port + 자체 virt + BE) |
| 4 | **ARCv3 64** | **seL4 + Rust + virt reference 모두 부재** | ≫≫ (거의 greenfield) |

→ 1·2 순위 (AArch64/RISC-V) 는 **3박자 완비** (verified base + Rust +
bhyve reference) → adapter 수준.  3·4 순위는 base/reference 부터 부재 →
차원이 다른 비용.  bhyve/nvmm 확보 (2026-06-19) 가 이 3박자의 reference
축을 1·2 순위에 대해 확정.

**핵심 전략**: §2 (ISA-independent core) + §4.B (virtualization 추상,
cpu_virt_compat 의 arch 확장) + §4.D (seL4 verified base) 3개가 척추.
"Y4 가치 대부분이 arch-neutral + cpu_virt_compat 패턴이 ISA 확장 자연 +
seL4 가 verified arch 한정" → AArch64/RISC-V 는 *adapter 수준* 비용,
POWER/ARC 는 *base 포팅* 비용 (차원이 다름).

## 6. 결정 / 미결

### 방향 (잠정)
- ISA-independent core (IPC/allocator/capsule/lease/verus) 1회 + per-arch
  HAL trait impl (4.A)
- virtualization = cpu_virt_compat 패턴의 arch 확장 (`<topic>_<arch>.rs`
  + abstract AV invariant, 4.B)
- endian = LE 우선 + POWER BE type-safe newtype (4.C)
- 우선순위 AArch64 → RISC-V → POWER → ARCv3 (4.D verified base 가 강제)

### 미결
- **§0.5 atomic-free 가 arch memory model 차이를 얼마나 흡수하나** —
  per-CPU exclusive + critical section 이 ARM/RISC-V/POWER weak ordering
  에서도 정말 arch-neutral 인지 (critical section = preempt/IRQ disable
  의 arch 별 구현은 dependent, 단 그 위 로직은 neutral)
- **seL4 AArch64 verification 상태** — verified 완료 vs in-progress (Y4
  의 formal 보장 등급에 직결)
- **POWER seL4 포팅 + virt 자체 구현** — 자체 프로젝트 (Y4 scope 밖?)
  vs Y4 가 추진.  seL4 port + **virt reference 부재** (bhyve/nvmm 둘 다
  powerpc 없음 확인 2026-06-19, KVM-PPC 은 GPL 불가) → PAPR spec 기반
  자체 virt + endian (BE) + radix MMU + XIVE 전반
- **ARCv3 Rust port** — Rust 측 ARCv3-64 target 부재 → LLVM ARCv3
  backend + Rust target 선결.  virt reference 도 부재.  Y4 가 감당 vs 포기
- **HIU MMIO endian** (⏳ hiu_abi) — WaveTensor RTL byte order 고정값 +
  BE host swap 정책
- **amdv_safety / power_safety 의 arch-neutral 화 범위** — S1~S23 +
  AV1~AV40 중 arch-specific (TSC, MSR 등) 의 per-arch 매핑 분량

### virt reference 확보 (2026-06-19)
- `~/y4-upstream-refs/` 에 **bhyve** (24M, amd64+arm64+riscv vmm) +
  **nvmm** (33M, x86 only) blobless sparse clone 추가 (dragonfly/redox
  와 동일 방식).
- 질문 2 (§3.1): bhyve multi-arch → AArch64/RISC-V virt = bhyve port.
  NVMM = x86 only (NetBSD portability ≠ NVMM virt).
- 질문 3 (§3.2): bhyve/nvmm 외 non-GPL 경로 — A(permissive hv: Hafnium/
  libsel4vm/OpenBSD vmm) / **B(spec clean-room, 모든 arch 항상 열림)** /
  C(rust-vmm device).  POWER/ARC = 경로 B.

### 후속 / 연결
- 본 발제가 cpu_virt_compat.md (vendor-neutral) 를 **ISA-neutral 로 일반화**
  — cpu_virt_compat 의 §8 미해결 (Hygon/TDX/AArch64/RISC-V 등 §8 (1)(3)
  이미 언급) 의 본격화.
- allocator §0.5 atomic-free 가 또 이득 (arch memory model 회피) — 3번째
  소급 의미 (보안 → Verus-only → real-time → cross-platform).
- pluggable allocator (§) 의 form-factor 선택이 (ISA, front-end) 2축으로
  확장 — embedded RISC-V/ARC = SLOB/SLUB-tiny, server x86/ARM/POWER =
  SLUB-style.
- ⏳ seL4 POWER/ARC 포팅 + Rust ARCv3 + hiu_abi endian 차단 부분 외에는
  AArch64/RISC-V 까지 지금 설계 가능.
