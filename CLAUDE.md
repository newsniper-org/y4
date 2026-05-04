<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 — Project Context for Claude Code

## 1. What Y4 is

**Y4** is a Type-1 Rust hypervisor that serves as the common base OS layer
for every commercial form factor that hosts a WaveTensor accelerator
(server-farm host, special-purpose laptop, rack-mount node, handheld+dock,
embedded SoC/SoM).

Y4 is a sibling project to **WaveTensor** (`/home/ybi/WaveTensor/`).
WaveTensor is the accelerator (RTL + assembler + cocotb tests + Vivado
synth flow); Y4 is the host hypervisor that exposes the accelerator's
hardware capabilities (HIU, partitioned TLB, shadow regions, XChaCha20
masking, lease) as OS-level capabilities.

The original Y4 design memo was drafted inside the WaveTensor repo as
`WaveTensor/.claude-memos/host_a_custom_hypervisor.md`. **The canonical
design document now lives here at `docs/architecture.md`.** When Y4
design changes, update Y4's copy first; only mirror to WaveTensor's memo
if a change crosses into WaveTensor's RTL ABI.

## 2. Status

**Scaffold + Phase A docs.** Y4 가 WaveTensor RTL 의 실제 운용 전에
먼저 개발된다 — 운용 데이터 대신 **HIU ABI 명세**(`docs/hiu_abi.md`)가
capability 스키마의 입력. 모든 하드웨어 의존 코드는 mock 뒤로 격리.

Phase A → Phase B 진입 트리거: `docs/hiu_abi.md` 가 `v1.0 frozen` 으로
표시되고 (Y4 + WaveTensor 양측 sign-off), `HIU_ABI_VERSION` 레지스터
값이 `0x0001_0000` 으로 고정.

전체 Phase A → Phase E 진행은 `docs/phase_plan.md` 참조.

## 3. License

**Single-license Apache-2.0** for everything Y4 itself writes.

This is intentionally simpler than WaveTensor's multi-license layout
(SHL-2.1 / CERN-OHL-W-2.0 for HW, LGPL-2.1+ / BSD-2 / Apache-2 for SW,
CC-BY-4.0 for docs). Y4 is single-license because:

- Patent grant matters (Apache-2.0 §3).
- Compatible with every base Y4 reuses (seL4 BSD-2, Tock MIT/Apache-2,
  DragonFlyBSD BSD-3, Redox MIT, Limine BSD-2).
- Permissive enough for guest workloads (users layer their own distro
  on top with whatever proprietary apps they want).

**SPDX header policy** (every source file Y4 ships):

```rust
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
```

Documentation:
```markdown
<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->
```

For ported code, **preserve the upstream SPDX line** and add Y4's line
below it. See `CONTRIBUTING.md` §3 for full rules.

GPL'd Linux drivers, when ported as 3rd-tier-fallback capsules, stay
**GPLv2 in their own capsule binary** with no direct linkage from the
Y4 main tree. See `docs/licensing.md` §2.

## 4. Reuse manifest

| Layer | Origin | License | Reuse mode |
|-------|--------|---------|-----------|
| Microkernel | seL4 | BSD-2-Clause | Binary as-is (formally verified) |
| Driver isolation | Tock | MIT/Apache-2 | Capsule pattern + crate reuse |
| IPC | DragonFlyBSD LWKT + Redox scheme | BSD-3 / MIT | Source port (fused) |
| Allocator | DragonFlyBSD lock-free SLAB + LLVM scudo | BSD-3 + Apache-2.0 | Algorithmic fusion (replaces original SLUB+OpenBSD-malloc plan; see `docs/architecture.md` §Memory allocator) |
| Bootloader | Limine (1st), GRUB2-BLS (2nd), U-Boot (3rd), coreboot (4th) | mixed | Chain-loaded only, never linked |
| Verification | Verus, Coq, Kani | MIT / LGPL-2.1 / Apache-2/MIT | Build-time only |

**Excluded from Y4:** `systemd-boot` (boot-entry tool chain is
systemd-tied; misaligned with Y4's BSD/Redox/Tock+Rust ecosystem),
`rEFInd` (no transactional-update hooks). See `docs/architecture.md`
§Bootloader for the full priority table and rationale.

## 5. Repository layout

```
Y4/
├── LICENSE              Apache-2.0 (full text)
├── NOTICE               attributions + reuse manifest
├── README.md            project overview
├── CONTRIBUTING.md      DCO sign-off + SPDX header rules
├── CLAUDE.md            this file (Claude Code project context)
├── docs/
│   ├── architecture.md  canonical design memo (Apache-2.0 inbound from WaveTensor's CC-BY-4.0)
│   ├── licensing.md     Apache-2.0 main + GPL-capsule isolation
│   └── phase_plan.md    Phase 0 → Phase 4 progression
└── third_party/         seL4, Tock, etc. (added as git submodules in Phase 1)
```

When Phase 1 begins, these top-level dirs will be added — they are
**deliberately omitted from the scaffold** so empty placeholders don't
accumulate:

```
kernel/    Y4 specialization layer above seL4
capsules/  Tock-style driver capsules (PCIe / USB / CXL / HIU)
ipc/       fused LWKT + Redox-scheme IPC implementation
alloc/     fused DragonFly lock-free SLAB + LLVM scudo allocator
hiu/       HIU integration & lease capability runtime
boot/      Limine config + Y4 first-stage handoff
proofs/    Verus and Coq specifications
```

## 6. Engineering principles

These are non-negotiable; they govern every Y4 decision:

1. **TCB minimization.** Tenant data must not traverse a Linux stack.
   Every layer above the seL4 base must justify its inclusion in the
   trust boundary.
2. **Capability-based isolation.** Lease, MMIO access, memory regions
   — every cross-tenant primitive is a capability, never an ad-hoc
   user-space token.
3. **Rust-first.** New code is Rust unless the base it ports from is
   in C (DragonFlyBSD LWKT) — in which case wrap behind a Rust
   capsule ABI.
4. **Direct hardware access.** PCIe / USB / CXL / HIU paths minimize
   hops between hardware and the accelerator daemon.
5. **Verified base + specialization-only authoring.** Don't rewrite
   what's already verified. Reuse seL4's microkernel, Tock's capsule
   typing, DragonFly's LWKT, Redox's scheme. Author only the
   WaveTensor-specific layer.
6. **Formal-first verification.** Specifications and proofs land
   **before** the code they describe. Verus is the 1st-tier tool
   (Rust-native), Coq is reserved for high-level invariants Verus
   cannot express. PRs that introduce new privileged paths without
   their proofs do not merge — see `CONTRIBUTING.md` §5.

## 7. Cross-project relationships

| Project | Path | Relationship |
|---------|------|-------------|
| **WaveTensor** | `/home/ybi/WaveTensor/` | The accelerator. Y4 hosts it. WaveTensor's RTL exposes the HIU primitives Y4 maps to capabilities. **WaveTensor side keeps `host_a_custom_hypervisor.md` as historical context only — the canonical Y4 design lives here.** |
| imads-hpo / imads-src | `/home/ybi/imads-*/` | HPO infrastructure that will eventually run as guest workloads on Y4. |

When working in this repo, do **not** edit WaveTensor source files
unless the change crosses into the WaveTensor RTL ABI (HIU lease ABI,
TRNG output format, etc.). Y4 design changes stay here.

## 8. How to work in this repo (Claude Code conventions)

- **Branch model:** `main` is the integration branch. Phase-gated
  feature work goes on topic branches (`p1/sel4-bootstrap`, etc.).
- **Build system:** single Cargo workspace (`Cargo.toml` at repo root).
  Top-level `justfile` orchestrates incremental and DAG builds via
  [`logicutils`](/home/ybi/logicutils) — `freshcheck` for hash-driven
  freshness, `stamp` for signature recording, `lu-par` for DAG-aware
  parallel execution. Sub-trees may have their own `justfile`.
- **Target architecture:** **x86_64 first** (Phase B). Other arches
  (aarch64 for handheld/SoC) added when their form factor work begins.
- **External dependencies:** **hybrid** — non-Rust upstream
  (seL4, Limine) as `git submodule` under `third_party/`; Rust crates
  reused from upstream (Tock parts, etc.) via cargo `[patch.crates-io]`
  + git deps in workspace `Cargo.toml`. Submodule pins:
  **seL4 = 15.x stable**, **Limine = 12.x stable**.
- **CMake invocation wrapping (`boot/`, future capsule build):**
  **logicutils-only**. Per-form-factor cmake `-D` flags live in
  `boot/<subsystem>.rules` (consumed by `lu-rule`); matrix execution
  via `lu-par`; freshness via `freshcheck`/`stamp`. No xtask, no
  cargo-make, no CMakePresets — single source-of-truth in the
  logicutils rule files.
- **Phase B implementation order:** `proofs/` build harness first
  (Verus + Coq + CI gate) → `boot/` (Limine → seL4 QEMU) → `ipc/` and
  `alloc/` in parallel → `capsules/` (non-HIU). HIU-touching work
  (`hiu/`, lease runtime) defers until `docs/hiu_abi.md` is `v1.0
  frozen`.
- **Git hooks:** `tools/git-hooks/` is committed. Fresh clones must run
  `just install-hooks` once to wire `core.hooksPath`. The pre-commit
  hook mirrors Claude Code memory into `.claude-memories/` (also
  triggered by the Claude Code Stop hook in `.claude/settings.json`)
  and stages the diff so the in-repo backup ships with each commit.
- **Commits:** DCO sign-off mandatory (`git commit -s`). PRs without
  sign-off do not merge.
- **Style:** `cargo fmt` + `cargo clippy -- -D warnings` clean. C
  ports follow the upstream's `clang-format` preset. Imperative-mood
  commit subjects, subsystem-prefixed (`hiu: bind XChaCha20 nonce on
  context_switch raise`).
- **Verification gating:** new privileged code paths require their
  Verus/Coq artifacts in the same PR. Bug fixes that don't change
  specified behavior are exempt.
- **Memory:** project memory is at
  `/home/ybi/.claude/projects/-home-ybi-Y4/memory/` — preserve
  cross-project knowledge there (especially the WaveTensor relationship
  and the bootloader exclusion list).
- **External name:** "Y4" is the official project name and is
  reserved per Apache-2.0 §6 (no trademark grant). Forks must rename.

## 9. Quick links

- Canonical design: `docs/architecture.md`
- License policy details: `docs/licensing.md`
- Phase plan + entry triggers: `docs/phase_plan.md`
- Contribution rules: `CONTRIBUTING.md`
- Reuse attributions: `NOTICE`
