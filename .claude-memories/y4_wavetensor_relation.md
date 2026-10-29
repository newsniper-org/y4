---
name: y4-wavetensor-relation
description: "Y4 (/home/ybi/Y4) and WaveTensor (/home/ybi/WaveTensor) are separate local repos with distinct license stacks; WaveTensor may be read (read-only) but not edited from Y4 unless an ABI crosses; ABI = WaveTensor canonical guide's MMIO configuration surface (§3.10 agreement)."
metadata:
  node_type: memory
  type: reference
  originSessionId: 38a83459-9d91-4355-910c-062288dd88aa
  modified: 2026-10-01T04:57:35.876Z
---

**Layout:**
- `/home/ybi/Y4/` — this repo. Type-1 hypervisor. Single Apache-2.0.
- `/home/ybi/WaveTensor/` — the accelerator project.
  - Multi-license: SHL-2.1 OR CERN-OHL-W-2.0 for hardware, LGPL-2.1+ OR BSD-2 OR Apache-2.0 for software, CC-BY-4.0 for docs/specs.
  - Since 2026-08-20 it is a **clean-room RTL reimplementation**: SpinalHDL main + SystemVerilog verification layer under `rtl/`.
  - The process is pre-formal TLA+ → RTL → post-formal sby.
  - The sole canonical spec is `docs/wt64v1-architecture-guide/`.
  - Remote (since 2026-10-26): public GitHub `newsniper-org/WaveTensor` — the old `Honey-Be/WaveTensor-private` URL redirects there (WT local origin still names the old URL). Remote `main` = `d1e026e` at the 2026-10-29 push (version-value commit `40de875` landed 2026-10-27; fact commits W9–W11 since); freeze tag `wt64v1-s3.10-v1.0-frozen` → `b16fc07`; Y4 checks reachability with `git ls-remote https://github.com/newsniper-org/WaveTensor` (read-only).
  - Repo-root `*.v` files (`HIU.v`, `Pod.v`, `Cluster.v`, …) are the **legacy** reference implementation and must not be used as an ABI source.

**ABI crossings (only places Y4 work touches WaveTensor):**
- **HIU configuration surface (MMIO)** — the canonical guide App. E (E.1 map, E.2 translation entries, E.3 miss fill, E.7 page ownership, E.9 masking keys / lease rotation, E.6 completion notification).
  - Freezing it requires both sides to agree (canonical §3.10 「호스트 ABI 경계」).
  - The Y4-side document is `docs/hiu_abi.md`.
- **Lease semantics** — Y4 expresses leases as capabilities. The device has no lease object; it knows only per-partition surfaces — the `0xC00` translation-entry array (partition in `addr[8:7]`), the `0xE00` flush, and the `0xE80` masking window (MASK_PART / MASK_REKEY / MASK_STATUS). There is no global `context_switch`.
- **Masking key/nonce obligations** — the host generates keys, one per lease. Keeping the `(key, nonce)` pair unique is Y4's duty.
- **Entropy health** — `FAULT_STATUS[17]` only. There is no host-readable TRNG value channel.

**Channels:**
- Y4 → WaveTensor change requests: `Y4/.local-requests-to/WaveTensor/`, mirrored to `WaveTensor/.local-requests-from/Y4/`.
- WaveTensor → Y4 requests arrive in `Y4/.local-requests-from/WaveTensor/`.
- Replies go to `.local-replies-to/<peer>/` and are mirrored to the peer's `.local-replies-from/`.
- All four directories are gitignored on both sides.
- WaveTensor's sign-off response to hiu_abi v0 is at `WaveTensor/.claude-memos/y4_hiu_abi_signoff_response.md` (non-canonical memo; holds rows H1–H4, HP-1..7, G1–G7).

**Read access:** read-only is allowed under the six conditions in [[wavetensor-read-policy]]. **Outside the ABI crossings, never edit WaveTensor from a Y4 session.** Y4 design decisions live in `Y4/docs/architecture.md`. `WaveTensor/.claude-memos/host_a_custom_hypervisor.md` is historical context only.

**Other sibling projects in `/home/ybi/`:**
- `imads-hpo` / `imads-src` — HPO infrastructure that will run as a guest workload on Y4. Out of scope until Phase C (first guest hosting, `docs/phase_plan.md`).
- `aiworks` — separate workspace; not directly related to Y4.

**How to apply:**
- For any "look at WaveTensor" task, first decide whether the change is Y4-internal (do not propagate) or an ABI crossing (propagate via the request channel, not by editing). If in doubt, ask the user.
- Cite WaveTensor as `file:line` + commit sha + verbatim phrase. Use absolute paths (`/home/ybi/WaveTensor/...`), because the repos are siblings.
- Term cache: [[wavetensor-terms]]; full dictionary: `Y4/docs/glossary.md`.
