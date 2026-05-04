---
name: Y4 project basics
description: Y4 is the Type-1 Rust hypervisor that hosts WaveTensor accelerators across all commercial form factors; single-license Apache-2.0; currently scaffold-only.
type: project
---

**Y4** = Type-1 Rust hypervisor for WaveTensor accelerator hosts.

Form factors covered by Y4: server-farm host A, special-purpose laptop, rack-mount node, handheld+dock unit, embedded SoC/SoM. All five share the same Y4 base.

**Status (as of 2026-05-04):** scaffold only. No code, no first commit yet. Phase 0 (Linux + Rust daemon) is still active inside the WaveTensor repo. Y4 itself has no source files beyond the scaffold (LICENSE / NOTICE / README / CONTRIBUTING / claude.md / docs/*).

**License:** single Apache-2.0. Intentionally simpler than WaveTensor's multi-license split (SHL-2.1/CERN-OHL-W for HW, LGPL/BSD-2/Apache-2 for SW, CC-BY-4.0 for docs).

**SPDX header pattern** for new files:
- Rust/C: `// SPDX-License-Identifier: Apache-2.0` + `// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors`
- Markdown: same with `<!-- ... -->`
- Ported code: preserve upstream SPDX line, add Y4's below.

**Phase trigger to start actual code:** WaveTensor RTL clears FPGA synthesis with timing closure on the target board AND the Phase 0 daemon has produced enough operational data to inform the seL4 capability schema.

**How to apply:** When a Y4 task lands, check whether Phase 1 has been triggered. If not, the task is probably a docs/scaffold change rather than code. If yes, the dirs `kernel/`, `capsules/`, `ipc/`, `alloc/`, `hiu/`, `boot/`, `proofs/` will be created at that moment — until then they should not exist as empty placeholders.
