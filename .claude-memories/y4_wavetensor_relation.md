---
name: Y4 ↔ WaveTensor sibling repo relationship
description: Y4 (/home/ybi/Y4) and WaveTensor (/home/ybi/WaveTensor) are separate local repos with distinct license stacks; do not edit WaveTensor sources from Y4 unless an ABI crosses.
type: reference
---

**Layout:**
- `/home/ybi/Y4/` — this repo. Type-1 hypervisor. Single Apache-2.0.
- `/home/ybi/WaveTensor/` — the accelerator project. Multi-license: SHL-2.1 OR CERN-OHL-W-2.0 for hardware (Verilog), LGPL-2.1+ OR BSD-2 OR Apache-2.0 for software, CC-BY-4.0 for docs/specs.

**ABI crossings (only places Y4 work touches WaveTensor sources):**
- HIU lease ABI shape — Y4 expresses leases as capabilities; WaveTensor's `HIU.v` raises `context_switch` and exposes partitioned-TLB / shadow-region / XChaCha20 primitives. If lease semantics change in Y4, mirror to WaveTensor's RTL.
- TRNG output format — Y4 may consume `rng_word` / `rng_word_valid` from HIU; format changes require synchronized updates.
- Capability-binding nonce semantics — XChaCha20 keying scheme.

**Outside those crossings, do not edit WaveTensor sources from a Y4 session.** Y4 design decisions live in `Y4/docs/architecture.md`, not in `WaveTensor/.claude-memos/host_a_custom_hypervisor.md` (which is now historical context only).

**Other sibling projects in `/home/ybi/`:**
- `imads-hpo` / `imads-src` — HPO infrastructure that will eventually run as a guest workload on Y4. Out of scope until Phase 3 (guest-OS hosting).
- `aiworks` — separate workspace; not directly related to Y4.

**How to apply:**
- For any task that says "look at WaveTensor", confirm whether the change is a Y4-internal design issue (do not propagate to WaveTensor) or an ABI crossing (must propagate). If in doubt, ask the user.
- When generating cross-references in Y4 docs, use absolute paths to `/home/ybi/WaveTensor/...` rather than relative paths — the two repos are siblings, not parent/child, and relative paths are fragile.
