<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Phase Plan

Y4 development progresses through five phases. Each phase has an explicit
**entry trigger**; no phase begins until its predecessor's deliverables
are validated.

## Phase 0 — Linux + Rust daemon (current)

Active **inside the WaveTensor repository**, not in this repo. The Y4
team writes nothing here yet. Deliverables:

- Accelerator daemon on a stock Linux host with Rust SDK.
- Lease lifecycle exercised over real workloads.
- Operational telemetry on lease patterns, failure modes, and security
  asks.

Phase 0 → Phase 1 entry trigger: WaveTensor RTL has cleared FPGA
synthesis with timing closure on the target board, AND Phase 0 daemon
has accumulated enough operational data to inform the seL4 capability
schema.

## Phase 1 — seL4 + accelerator daemon (server-farm A only)

First Y4 commits land in **this repository**.

| Subsystem | Deliverable |
|-----------|-------------|
| Microkernel | seL4 brought up on x86_64 host A. |
| Daemon | Existing daemon ported on top of seL4 capabilities. |
| HIU integration | Lease capability raises `context_switch`, partitioned TLB and shadow regions are wired through capability typing. |
| Bootloader | Limine config + Y4 first-stage handoff (Limine boot protocol). |
| Coverage | x86_64 host A only. Other form factors deferred. |
| Hosting | **No guest OS hosting yet.** Linux on bare-metal A and Y4 on bare-metal A coexist on separate boots. |

Phase 1 → Phase 2 entry trigger: server-farm A is running production
loads on Y4 and the accelerator team treats Y4 as a stable substrate
(no more daily microkernel-side firefights).

## Phase 2 — Fused IPC, Tock capsules, fused allocator

| Subsystem | Deliverable |
|-----------|-------------|
| IPC | DragonFlyBSD LWKT + Redox scheme fusion shipped (per `architecture.md`). |
| Driver isolation | Tock capsule pattern adopted for PCIe / USB / CXL / HIU drivers. |
| Allocator | SLUB + lock-free SLAB + mmap-only fusion. |
| Verification | Verus / Coq toolchain in CI; **every new privileged path lands with its proof** (formal-first). |

Phase 2 → Phase 3 entry trigger: every Phase-2 component has a
machine-checked invariant set + a soak-test record.

## Phase 3 — Type-1 hypervisor + guest OS hosting

| Subsystem | Deliverable |
|-----------|-------------|
| Paravirt | Linux distros + DragonFlyBSD hosted as guests with a paravirt interface (Xen-PV / virtio / Y4-native — decision deferred). |
| Form factors | Special-purpose laptop, rack-mount, handheld+dock, embedded SoC/SoM all booting the same Y4 base. |
| GPU passthrough | Display GPU passes through to the guest (Y4 itself is headless). |

Phase 3 → Phase 4 entry trigger: at least one external user with a
security-sensitive workload is running on Y4 and the certification ROI
is justified.

## Phase 4 — Formal-verified certification track + external release

| Track | Standard |
|-------|----------|
| Medical | FDA 510(k) |
| Aviation | DO-178C |
| Financial | FIPS 140-3 |

`formal-first` lets all three tracks proceed in parallel; priority order
will be set by external demand at Phase 3 close. Public release of the
certification dossier and signed release artifacts.

## Cross-phase notes

- **Cumulative validation.** Each phase boundary requires both
  operational data and proof artifacts; no phase advances on time
  alone.
- **Driver tier order is unchanged across phases:** DragonFlyBSD (1st)
  → NetBSD via rump (2nd) → Linux GPL capsule (3rd, isolated). See
  `licensing.md` §"Linux driver tier".
- **Bootloader stays Limine** across all phases unless a form factor's
  trust-chain requirement (Phase 4 certification) forces 4th-tier
  coreboot+payload. Switches lower in the priority list (GRUB2-BLS,
  U-Boot) happen only when target hardware mandates them (ARM SoC →
  U-Boot).
