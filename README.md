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

> **Status: Phase B complete.** All five Phase B steps green —
> `proofs/` harness, `boot/` (Limine → seL4 in QEMU), `ipc/` + `alloc/`
> Rust crates with their Verus specs, `capsules/` with the first
> concrete PCIe enum capsule, and `kernel/` printing **`Hello, Y4`**
> on the QEMU serial.  HIU-touching work (`hiu/`, lease runtime)
> remains blocked until [`docs/hiu_abi.md`](./docs/hiu_abi.md) is
> `v1.0 frozen` (joint Y4 + WaveTensor sign-off).
> See [`docs/phase_plan.md`](./docs/phase_plan.md).

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
| Microkernel | binary as-is | seL4 15.0.0 (BSD-2-Clause, formally verified) — `third_party/sel4/` |
| Bootloader | upstream as-is, chain-loaded | **Limine v12.1.0** (1st), GRUB2-BLS (2nd), U-Boot (3rd), coreboot (4th) — `systemd-boot` and `rEFInd` are explicitly excluded for Y4. See [`docs/architecture.md`](./docs/architecture.md) §Bootloader. |
| IPC (control plane) | source port + Rust adapter | Redox scheme — `ipc/src/scheme.rs` |
| IPC (data plane) | source port + Rust adapter | DragonFlyBSD LWKT msgport — `ipc/src/msgport.rs` |
| Memory allocator (front) | algorithm port | DragonFlyBSD lock-free SLAB — `alloc/src/slab.rs` |
| Memory allocator (back) | C++ via FFI **+** Rust contract twin | LLVM scudo (Apache-2.0, vendored on demand from `third_party/scudo/PIN.toml`) — `scudo-sys/`, `alloc/src/scudo_ffi.rs`, `alloc/src/hardened.rs` |
| Driver / device isolation | Tock-style capability typing | `capsules/src/isolation.rs` |
| Verification toolchain | build-time | Verus (Rust-native, AUR `verus-bin`), Rocq 9.x (formerly Coq) |

Self-written by Y4 (Phase B already shipped):

- `proofs/verus/` — 50 machine-checked invariants across error /
  ipc / alloc / capsules; 10 of them discharged constructively
  (refinement proofs in each module's `refinement.rs`).
- `proofs/coq/` — Rocq theories for high-level invariants Verus
  cannot express.
- `boot/` — Limine + seL4 chain harness (`logicutils`-driven cmake
  invocation per [`MEMORY/y4_build_decisions.md`](./.claude-memories/y4_build_decisions.md)).
- `kernel/` — root task (`Hello, Y4` milestone).

Self-written by Y4 (Phase C onwards):

- HIU integration (`context_switch`, partitioned TLB, shadow regions,
  XChaCha20 capability binding) — blocked until `docs/hiu_abi.md` v1.0.
- Lease scheduler (wave-aligned preemption).
- Accelerator daemon RPC / local IPC unified under one namespace API.
- Guest-OS hosting paravirt for Linux distros and DragonFlyBSD.

## Repository layout

```
Y4/
├── LICENSE                       Apache-2.0 (full text)
├── NOTICE                        attributions + reuse manifest
├── README.md                     this file
├── CLAUDE.md                     project context auto-loaded by Claude Code
├── CONTRIBUTING.md               DCO, SPDX header policy, signing
├── Cargo.toml                    cargo workspace root (alloc, capsules, ipc, scudo-sys)
├── rust-toolchain.toml           pins channel = stable (MSRV 1.94)
├── justfile                      top-level recipes (ci, scudo-fetch, mirror-memory, ...)
├── docs/
│   ├── architecture.md           canonical design memo
│   ├── glossary.md               WaveTensor terms (HIU/lease/TRNG/...) extracted from RTL
│   ├── hiu_abi.md                Y4 ↔ HIU ABI v0 (MMIO map, timing contracts, freeze policy)
│   ├── lease_capability.md       lease capability schema v0 (invariants, lifecycle)
│   ├── licensing.md              Apache-2.0 main + GPL-capsule isolation
│   └── phase_plan.md             Phase A → Phase E progression + entry triggers
├── alloc/                        y4-alloc — DragonFly SLAB + hardened backend (22 tests, +2 with `--features scudo`)
├── capsules/                     y4-capsules — Tock-style isolation + PCIe enum (16 tests)
├── ipc/                          y4-ipc — Redox scheme + LWKT msgport hybrid (18 tests)
├── kernel/                       y4-roottask — bare-metal x86_64-unknown-none ("Hello, Y4")
├── scudo-sys/                    y4-scudo-sys — LLVM scudo C++ FFI (4 link-smoke tests)
├── boot/                         Limine config + seL4 cmake rules (logicutils-driven)
├── proofs/
│   ├── verus/                    Verus specs (50 verified) — alloc / capsules / error / ipc
│   └── coq/                      Rocq theories
├── tools/
│   ├── git-hooks/                pre-commit (memory mirror)
│   └── scudo-fetch.sh            materialise scudo source from PIN.toml
├── third_party/
│   ├── sel4/                     seL4 15.0.0 (git submodule)
│   ├── limine/                   Limine v12.1.0 (git submodule)
│   └── scudo/                    pinned LLVM scudo standalone
│       ├── PIN.toml              upstream coordinate (commit SHA)
│       ├── README.md             vendoring policy
│       └── standalone/           materialised by `just scudo-fetch` (gitignored)
├── .claude-memories/             read-only mirror of Claude Code's project memory
└── .vscode/settings.json         editor exclusions for vendored sources
```

Subsystem status:

| Subsystem | Status | First milestone |
|-----------|--------|-----------------|
| `proofs/` | ✅ harness green, 50 invariants verified | placeholder + scaffold |
| `boot/`   | ✅ qemu-smoke PASS                       | Limine → seL4 → "Boot config" |
| `ipc/`    | ✅ 18 tests, Verus refinement proofs     | scheme verbs + LWKT msgport hybrid |
| `alloc/`  | ✅ 22+2 tests, Verus refinement proofs   | DragonFly SLAB + hardened backend |
| `capsules/` | ✅ 16 tests                            | PCIe enum (P1/P2/P3) |
| `kernel/` | ✅ qemu-smoke PASS                       | "Hello, Y4" on serial |
| `hiu/`    | ⛔ blocked                               | needs `docs/hiu_abi.md` v1.0 frozen |

## Build / test / verify

Set up a fresh clone:

```sh
git clone <repo> Y4 && cd Y4
git submodule update --init --recursive    # sel4, limine
just install-hooks                         # wire core.hooksPath = tools/git-hooks
just scudo-fetch                           # materialise pinned scudo source (~1 MB)
```

Run the workspace gate (cargo + Verus + Rocq, all hash-stamped via
`logicutils`):

```sh
just ci
```

Boot the assembled ISO under QEMU and assert the "Hello, Y4" milestone:

```sh
just sel4-build           # seL4 15.0.0 → kernel.elf
just limine-build         # Limine 12.1.0 host-side binaries
just roottask-build       # y4-roottask ELF
just iso-build            # xorriso ISO with kernel + roottask + Limine
just qemu-smoke           # PASS = root task greeted on serial
just qemu-boot            # interactive (Ctrl-A x to exit)
```

Per-crate tests:

```sh
cargo test -p y4-alloc            # 22 tests (24 with --features scudo)
cargo test -p y4-capsules         # 16 tests
cargo test -p y4-ipc              # 18 tests
cargo test -p y4-scudo-sys        # 4 C++ link-smoke tests
just verus                        # 50 verified, 0 errors
just coq                          # Rocq theories
```

## Tooling decisions (durable)

Recorded in [`.claude-memories/y4_build_decisions.md`](./.claude-memories/y4_build_decisions.md):

- **D1** — Cargo workspace + per-subtree `justfile`s, orchestration via
  [`logicutils`](/home/ybi/logicutils) (`freshcheck` / `stamp` / `lu-par`).
- **D2** — x86_64 first; other architectures added when their form factor
  work begins.
- **D3** — hybrid: non-Rust upstream (seL4, Limine) as `git submodule`;
  Rust crates via cargo `[patch.crates-io]` + git deps; LLVM scudo as a
  pin-file vendored on demand (separate from submodules — see
  [`third_party/scudo/README.md`](./third_party/scudo/README.md)).
- **D4** — Phase B order: `proofs/` → `boot/` → `ipc/` & `alloc/` →
  `capsules/` → `kernel/`.  All ✅.
- **CMake invocation** — logicutils-only (rules files, no xtask /
  cargo-make / CMakePresets).

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
directly links GPL code.  See [`docs/licensing.md`](./docs/licensing.md)
§"Linux driver tier" for the routing policy (DragonFlyBSD 1st → NetBSD
via rump 2nd → Linux GPL capsule 3rd).

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md). Until external contributors
arrive, internal development uses the Developer Certificate of Origin
(DCO); a CLA decision is deferred until the first external contribution.

Pre-commit hook (installed via `just install-hooks`) mirrors Claude
Code's project memory into `.claude-memories/` and stages it with each
commit, so the in-repo backup tracks the live state without manual sync.

## Related projects

- [WaveTensor](https://github.com/...) — the accelerator that Y4 is
  designed to host. The Y4 design memo originated there. Local sibling
  at `/home/ybi/WaveTensor/`.
- imads-hpo / imads-src — HPO infrastructure that will eventually run as
  guest workloads on Y4.
</content>
