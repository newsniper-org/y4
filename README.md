<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4

**Y4** is a Type-1 Rust hypervisor that serves as the common base OS layer
for every commercial form factor that hosts a WaveTensor accelerator —
server-farm hosts, special-purpose laptops, rack-mount nodes,
handheld+dock units, and embedded SoC/SoM.

This repository is the canonical home of Y4. The original design rationale
was drafted inside the WaveTensor project as
`WaveTensor/.claude-memos/host_a_custom_hypervisor.md`; the canonical copy
in this repo lives at [`docs/architecture.md`](./docs/architecture.md).

> **Status: scaffold.** No code yet. Phase 0 (Linux + Rust daemon) is still
> active inside the WaveTensor repo. Phase 1 (seL4 + accelerator daemon
> only, server-farm A) is the trigger for actual development to begin
> here. See [`docs/phase_plan.md`](./docs/phase_plan.md).

---

## Why Y4

WaveTensor's HIU already exposes hardware features (`context_switch`,
partitioned TLB, shadow regions, XChaCha20 capability binding,
flush-on-context-switch) whose security potential is wasted under a
general-purpose OS. Y4 maps each of those primitives to a hypervisor-level
capability, giving:

- **Tiny TCB** — tenant data never traverses a Linux stack.
- **Deterministic latency** — wave-aligned lease scheduling, no cgroup /
  preempt jitter on the dispatch path.
- **OS-level lease isolation** — capability-typed; user-space bugs cannot
  leak across tenants.
- **Rust everywhere** — same ecosystem as the WaveTensor SDK / imads stack.
- **Formal-first verification** — specs and proofs land before
  implementation, on top of a verified seL4 base.

## What Y4 reuses (and what it writes)

Y4 is **not from-scratch.** Verified components are pulled in as-is and
only the WaveTensor-specific specialization layer is written by the Y4
team.

| Layer | Reuse | Origin |
|-------|-------|--------|
| Microkernel | binary as-is | seL4 (BSD-2-Clause, formally verified) |
| Driver / device isolation | model + crate reuse | Tock capsule (MIT/Apache-2.0) |
| IPC | source port (fused) | DragonFlyBSD LWKT + Redox scheme |
| Memory allocator | algorithm fusion | SLUB + DragonFlyBSD lock-free SLAB + OpenBSD mmap-only malloc |
| Bootloader | upstream as-is, chain-loaded | **Limine** (1st), GRUB2-BLS (2nd), U-Boot (3rd), coreboot (4th) — `systemd-boot` and `rEFInd` are explicitly excluded for Y4. See `docs/architecture.md` §Bootloader. |
| Verification toolchain | build-time | Verus (Rust), Coq (high-level invariants) |

Self-written by Y4:

- HIU integration (context switch, partitioned TLB, shadow regions,
  XChaCha20 capability binding)
- Lease scheduler (wave-aligned preemption)
- Accelerator daemon RPC / local IPC unified under one namespace API
- Guest-OS hosting paravirt for Linux distros and DragonFlyBSD

## Repository layout

```
Y4/
├── LICENSE              Apache-2.0 (full text)
├── NOTICE               attributions + reuse manifest
├── README.md            this file
├── CLAUDE.md            project context auto-loaded by Claude Code
├── CONTRIBUTING.md      DCO, SPDX header policy, signing
├── docs/
│   ├── architecture.md     canonical design memo
│   ├── glossary.md         WaveTensor terms (HIU/lease/TRNG/...) extracted from RTL
│   ├── hiu_abi.md          Y4 ↔ HIU ABI v0 (MMIO map, timing contracts, freeze policy)
│   ├── lease_capability.md lease capability schema v0 (invariants, lifecycle)
│   ├── licensing.md        Apache-2.0 main + GPL-capsule isolation
│   └── phase_plan.md       Phase A → Phase E progression + entry triggers
└── third_party/         seL4, Tock, etc. (added as git submodules during Phase 1)
```

Phase B has begun. The following top-level dirs land in order
(`proofs/` → `boot/` → `ipc/` & `alloc/` → `capsules/`); each appears
once its first PR ships:

```
proofs/    Verus + Rocq specifications + CI gate          (✅ scaffolded)
boot/      Limine config + seL4 cmake rules               (✅ scaffolded)
kernel/    Y4 specialization layer above seL4             (⏳ pending)
ipc/       fused LWKT + Redox-scheme IPC implementation   (⏳ pending)
alloc/     fused SLUB + lock-free SLAB + mmap-only        (⏳ pending)
capsules/  Tock-style driver capsules (non-HIU)           (⏳ pending)
hiu/       HIU integration & lease capability runtime     (⛔ blocked: hiu_abi v1.0)
```

## License

Apache License 2.0. See [`LICENSE`](./LICENSE),
[`docs/licensing.md`](./docs/licensing.md), and [`NOTICE`](./NOTICE).

The Apache-2.0 choice is permissive with an explicit patent grant, and is
license-compatible with every base Y4 reuses (BSD-2 / MIT / Apache-2). It
also lets vendors layer proprietary guests on top of Y4 — necessary for
the user form-factor (any Linux distro / DragonFlyBSD running in a
guest VM with whatever the user installs there).

GPL'd Linux drivers, when used as a last-resort fallback, are isolated as
separate capsule binaries through an external ABI; the Y4 main tree never
directly links GPL code.

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md). Until external contributors
arrive, internal development uses the Developer Certificate of Origin
(DCO); a CLA decision is deferred until the first external contribution.

## Related projects

- [WaveTensor](https://github.com/...) — the accelerator that Y4 is
  designed to host. The Y4 design memo originated there.
