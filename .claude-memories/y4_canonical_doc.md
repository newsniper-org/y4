---
name: Y4 design canonical doc location
description: docs/architecture.md is the source-of-truth Y4 design document; the WaveTensor-side memo is historical context only.
type: project
---

**Canonical Y4 design document:** `/home/ybi/Y4/docs/architecture.md` (Apache-2.0).

**Historical context only:** `/home/ybi/WaveTensor/.claude-memos/host_a_custom_hypervisor.md` (CC-BY-4.0). The original 초안 was drafted there before Y4 split into its own repo on 2026-05-04. WaveTensor's copy is preserved for archival/cross-reference but **must not** be the place where new Y4 design decisions are recorded.

**How to apply:**
- New Y4 design changes → edit `Y4/docs/architecture.md` first.
- Mirror back to the WaveTensor memo **only when** the change crosses into the WaveTensor RTL ABI (HIU lease ABI shape, TRNG output format, capability-binding nonce semantics).
- For purely Y4-internal changes (IPC fusion details, allocator layout, capsule typing, formal-proof obligations) — do not touch the WaveTensor memo at all.

This split exists to prevent design drift between the two copies and to keep the WaveTensor repo's commit history focused on RTL/assembler work.
