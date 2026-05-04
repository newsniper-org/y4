<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Licensing Policy

## 1. Y4 main tree — Apache License 2.0

Every line of code that Y4 itself writes is licensed under
**Apache License, Version 2.0** (`Apache-2.0`). The choice was made for
four concrete reasons:

1. **Patent grant.** Apache-2.0 §3 carries an explicit patent grant — a
   non-trivial defense for accelerator IP / hypervisor patent claims.
2. **Compatibility with every base Y4 reuses.** seL4 is BSD-2-Clause,
   Tock is MIT/Apache-2.0 dual, DragonFlyBSD is BSD-3-Clause, Redox is
   MIT. All are forward-compatible into Apache-2.0.
3. **Permissive for guest workloads.** Vendors and end users may layer
   proprietary guest OSes / proprietary applications on top of Y4 —
   required for the form factors Y4 targets (a user installs whatever
   distro they want as a guest).
4. **Friction-free integration with the broader Rust ecosystem.** Most
   Rust crates Y4 will pull in are MIT/Apache-2.0 dual; staying on
   Apache-2.0 avoids re-licensing churn at every dependency boundary.

GPLv2 / GPLv3 / AGPL were rejected: GPLv2 conflicts with seL4's BSD-2
combination at the linkage boundary (resulting binary would have to be
GPLv2), and GPLv3/AGPL would break embedded / firmware-signing scenarios
(tivoization clauses) that some Y4 form factors require.

## 2. Linux driver tier (3rd-tier fallback)

When neither (a) a DragonFlyBSD BSD-licensed driver nor (b) a NetBSD
driver via rump kernel covers a device Y4 needs, the last resort is to
port the equivalent Linux driver. **GPLv2 transitivity must not infect
the Y4 main tree.**

Isolation pattern (modeled on Linux kernel's loadable-module +
stable-ABI separation):

```
+-------------------------+   external ABI    +-----------------------------+
|  Y4 main tree           |  ------------->   |  GPL capsule binary         |
|  (Apache-2.0)           |  user-space rpc   |  (GPLv2, distributed apart) |
|  - never links GPL      |  <-------------   |  - ported Linux driver      |
+-------------------------+                   +-----------------------------+
```

Rules:

- The GPL capsule is **a separate binary** distributed under GPLv2 with
  its own SPDX header preserved.
- The Y4 main tree calls into the capsule **only through the external
  capsule ABI** — no direct symbol linkage, no inline header inclusion.
- A user-space helper translates between the GPL capsule and the
  Apache-2.0 main tree. The helper is GPLv2 if it uses GPL headers,
  Apache-2.0 if it stays on the public ABI side only.
- When a GPL capsule's underlying device has open hardware documentation,
  schedule a long-term replacement that re-implements the driver from
  the spec under Apache-2.0 — eliminates the GPL dependency entirely.

This pattern is the same one Linux uses for its own loadable modules and
has well-established legal precedent.

## 3. seL4 base

The seL4 microkernel ships under **BSD-2-Clause** (BSD 2-Clause license).
Apache-2.0 ⊃ BSD-2-Clause: combining BSD-2 source with Apache-2.0 source
is permitted, the resulting binary distributes under Apache-2.0, and the
seL4 BSD-2 attribution and notice are preserved in `NOTICE`. seL4's
formal proofs themselves carry their own license (currently GPL-2.0 for
some artifacts) — Y4 redistributes only the **proof-derived microkernel
binary / source** under BSD-2, not the proof scripts themselves, so no
GPL transitivity into Y4.

## 4. Tock capsule reuse

Tock OS is dual-licensed **MIT OR Apache-2.0**. When Y4 ports Tock
capsule code, we elect the **Apache-2.0** option (simpler — same license
as Y4 main, no dual-license dance per file). The original Tock SPDX
header is preserved alongside Y4's, per `CONTRIBUTING.md` §3.

## 5. DragonFlyBSD / Redox / NetBSD reuse

| Base | License | Y4 reuse |
|------|---------|----------|
| DragonFlyBSD | BSD-3-Clause | Source port (LWKT, vkernel, lock-free SLAB) |
| Redox OS     | MIT          | Crate-level (scheme IPC, relibc, kernel core) |
| NetBSD       | BSD-2-Clause | Driver port via rump kernel |

All three are permissive and forward-compatible with Apache-2.0.
Original copyright lines and license headers are preserved per
`CONTRIBUTING.md` §3.

## 6. Bootloader interaction

Bootloaders are **chain-loaded, never linked**, so their licenses
(BSD-2 for Limine, GPLv2 for U-Boot, GPLv3 for GRUB2, GPLv2+ for
coreboot) do not propagate into Y4. The pattern matches how Linux is
loaded by GRUB without the kernel becoming GPLv3.

| Bootloader | License | Why no transitivity |
|-----------|---------|---------------------|
| Limine     | BSD-2-Clause | Permissive — no transitivity even if linked. |
| GRUB2-BLS  | GPLv3 | Chain-loaded only; Y4 never links GRUB code or its public headers. |
| U-Boot     | GPLv2 | Chain-loaded only; same boundary. |
| coreboot   | GPLv2+ | Chain-loaded only; coreboot payload is the loader, Y4 is the loaded image. |

`systemd-boot` (LGPL-2.1+) and `rEFInd` (BSD-3-Clause) are excluded for
non-license reasons — see `architecture.md` §Bootloader.

## 7. Trademarks

`Apache-2.0` §6 explicitly excludes trademark grants. The strings "Y4",
"WaveTensor", and "WT64v1" are reserved for the project's official
distribution. Forks of the source code under Apache-2.0 are permitted
without trademark grant — which means a fork must use a different name.

## 8. Signed releases

Phase 4 release artifacts will carry a detached PGP signature plus an
SBOM (`SPDX-2.3` JSON) covering every reused base, every capsule, and
the reproducibility hash of the build inputs. Form factors with a Secure
Boot trust chain receive bootloader-signed payloads via the chosen
bootloader's signing toolchain (see `architecture.md` §Bootloader).

Until Phase 4, releases are unsigned development snapshots and must not
be deployed to production.
