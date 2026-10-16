---
name: y4-basics
description: "Y4 is the Type-1 Rust hypervisor (seL4-based) that hosts WaveTensor accelerators across all commercial form factors; single-license Apache-2.0; Phase B complete (mock HIU), hiu/ + lease runtime blocked on hiu_abi.md v1.0 frozen."
metadata:
  node_type: memory
  type: project
  originSessionId: 38a83459-9d91-4355-910c-062288dd88aa
  modified: 2026-10-01T04:57:43.764Z
---

**Y4** = Type-1 Rust hypervisor for WaveTensor accelerator hosts, built on seL4. Form factors: server-farm host A, special-purpose laptop, rack-mount node, handheld+dock unit, embedded SoC/SoM. All five share the same Y4 base.

**Status (2026-10-01):** Phase B complete.
- `proofs/` (Verus/Rocq/Isabelle), `boot/` (Limine v12.6.1 + seL4 16.0.0, qemu-smoke PASS), `ipc/`, `alloc/`, `capsules/` (PCIe enum) and `kernel/` (root task prints `Hello, Y4`) are all green.
- FIP crates `fip/{core,macros,x86_64,aarch64,riscv64}` live on topic branch `fip/isa-subcrates`.
- Only `hiu/` is missing. It waits for `docs/hiu_abi.md` v1.0 frozen, which needs WaveTensor + Y4 sign-off and a non-zero `HIU_ABI_VERSION` (`0xFFC`) value set by the §3.10 agreement. `0x0001_0000` (`(major<<16)|minor`) is agreed (Y4 proposal, WaveTensor agreed 2026-10-05, landed in canonical 6a25359, kept through e88f6f4). As of 2026-10-16 hiu_abi.md is v1.0-rc1 (freeze candidate at WaveTensor e88f6f4), waiting for both sign-offs. The non-zero value enters the device only in a separate commit after both sides sign off; until then the device reads 0.
- Phase model: `docs/phase_plan.md` (A → E). The old Phase 0–4 numbering (Linux + Rust daemon first) is historical (`docs/architecture.md` §Phase).

**License:** single Apache-2.0. Intentionally simpler than WaveTensor's multi-license split (SHL-2.1/CERN-OHL-W for HW, LGPL/BSD-2/Apache-2 for SW, CC-BY-4.0 for docs).

**SPDX header pattern** for new files:
- Rust/C: `// SPDX-License-Identifier: Apache-2.0` + `// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors`
- Markdown: same with `<!-- ... -->`
- Ported code: preserve upstream SPDX line, add Y4's below.

**Phase gating:**
- HIU-touching code (`hiu/`, the lease runtime) waits for the hiu_abi freeze.
- Everything else is built against mocks.
- The original trigger ("WaveTensor RTL clears FPGA timing on the target board + Phase 0 daemon data") was replaced by "HIU ABI spec as input" (`docs/phase_plan.md` intro).

**How to apply:**
- Do not create `hiu/` as a placeholder.
- When a task touches the accelerator surface, check the hiu_abi status and the WaveTensor canonical guide first ([[wavetensor-terms]], [[y4-wavetensor-relation]]).
