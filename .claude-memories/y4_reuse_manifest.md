---
name: Y4 reuse manifest + bootloader exclusions
description: What Y4 reuses from upstream OSes; Limine is the 1st-priority bootloader; systemd-boot and rEFInd are explicitly excluded.
type: project
---

**Reuse layers (what Y4 does not write itself):**

| Layer | Origin | License | Mode |
|-------|--------|---------|------|
| Microkernel | seL4 | BSD-2-Clause | Binary as-is (formally verified) |
| Driver isolation | Tock | MIT/Apache-2 | Capsule pattern + crate reuse |
| IPC | DragonFlyBSD LWKT + Redox scheme | BSD-3 / MIT | Source port, fused |
| Allocator | Linux SLUB + DragonFly lock-free SLAB + OpenBSD mmap-only | mixed | Algorithmic fusion |
| Verification | Verus, Coq, Kani | MIT / LGPL-2.1 / Apache-2/MIT | Build-time only |

**Bootloader priority (chain-loaded only — Y4 never directly links any of them):**

1. **Limine** — BSD-2-Clause, modern, kernel-dev friendly, plaintext `limine.conf`, Y4 image build pipeline writes config directly. UEFI x86_64 / ARM64 / RISC-V all supported.
2. **GRUB2 (BLS)** — GPLv3, industry-standard, multi-arch coverage, BLS entry write supported. GPL transitivity avoided through chain-load isolation (no direct linkage from Y4).
3. **U-Boot** — GPLv2, ARM SoC / embedded standard. SPL → U-Boot proper → Y4 boot chain.
4. **coreboot + LinuxBoot/Heads payload** — GPLv2+, Phase 4 certification track only. Firmware-level decision, late phase.

**Excluded:**
- **systemd-boot** — EFI binary itself runs without systemd at runtime, but the boot-entry management tool chain (`bootctl`, `kernel-install`, `sdbootutil`) is part of the systemd project. Y4 does not ship systemd, so its build/maintenance pipeline would permanently depend on an external systemd environment. Misaligned with Y4's BSD/Redox/Tock + Rust ecosystem identity. **Note:** systemd-boot is still suitable for systemd-based Linux on the same machine (e.g., MicroOS on a co-resident NVMe disk) — but not as Y4's own bootloader.
- **rEFInd** — BSD-3-Clause license is fine, but no transactional-update hooks, and existing tooling (refind-btrfs) assumes snapper RW snapshot model. Maintenance overhead too high for the value delivered.

**How to apply:**
- When choosing reuse for a new layer, prefer permissive (BSD/MIT/Apache) sources first.
- For driver development, the tier order is: DragonFlyBSD BSD driver (1st) → NetBSD via rump kernel (2nd) → Linux GPL driver isolated as a separate capsule binary (3rd, last resort).
- When discussing bootloader options, do not propose systemd-boot or rEFInd for Y4. Limine is the default answer unless the form factor (ARM SoC) forces 3rd-tier U-Boot or the certification track (Phase 4) forces 4th-tier coreboot.
