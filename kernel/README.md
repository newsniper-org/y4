<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# y4-roottask — Y4 root task

The first userland program seL4 hands control to.  Phase B step 5
milestone: print **`Hello, Y4`** on the serial console.

This crate is excluded from the main cargo workspace (`Cargo.toml` of
the workspace root, `exclude = ["proofs/verus", "kernel"]`) because it
builds for `x86_64-unknown-none` (bare metal, no `std`) — which would
derail `cargo build --workspace` if it were a member.

## Layout

```
kernel/
├── Cargo.toml                # bare-metal bin, panic=abort
├── .cargo/config.toml        # selects x86_64-unknown-none target + linker flags
├── linker.ld                 # link at 0x40_0000, entry _start
└── src/main.rs               # _start (naked) → rust_main → debug_put_string + pause loop
```

## Build / boot

```sh
just roottask-build           # cargo build --release inside kernel/
just iso-build                # repacks ISO with the roottask as a multiboot module
just qemu-smoke               # QEMU + 'Hello, Y4' assertion (Phase B step 5 gate)
just qemu-boot                # interactive QEMU
```

## seL4 ABI surface used

- **`SysDebugPutChar = -9`** — single byte to serial via `syscall`
  instruction.  Available because `boot/x86_64-debug.cmake` enables
  `KernelDebugBuild` + `KernelPrinting`.

That's the only kernel surface the root task touches today.  The lease
scheduler / capsule init / scheme bootstrap layers (Phase B step 6+)
extend the syscall set to `seL4_TCB_*`, `seL4_CNode_*`, etc. per the
C4 trusted-boundary table in `MEMORY/y4_ipc_alloc_preflight.md`.

## Known follow-ups

- After `Hello, Y4`, seL4 reports a `cap fault in send phase` — the
  root task has no fault handler endpoint registered.  Phase B step 6
  bootstraps the cap-table to install one.
- `_start` initialises a fixed 16 KiB stack in `.bss`; the lease
  scheduler PR replaces this with a proper TCB-managed stack.
- `seL4_BootInfo` (passed in `rdi` per seL4's x86_64 ABI) is currently
  ignored; Phase B step 6 walks it to discover the cap-set the kernel
  granted us at boot.
