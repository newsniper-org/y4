<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Contributing to Y4

Thanks for your interest in Y4. This document covers the rules every
contribution must satisfy.

## 1. Developer Certificate of Origin (DCO)

Every commit must be signed off under the
[Developer Certificate of Origin v1.1](https://developercertificate.org/).
This is the same mechanism the Linux kernel uses; signing off means **you
have the right to submit the work under the project license**.

Sign your commits with `-s`:

```
git commit -s -m "kernel: add foo"
```

This appends a trailer:

```
Signed-off-by: Your Name <you@example.com>
```

**Pull requests without a DCO sign-off on every commit will not be merged.**

A separate Contributor License Agreement (CLA) is **not** required at this
time. The CLA decision is deferred until the first external contribution
arrives — see `docs/architecture.md` §"외부 기여 / 거버넌스".

## 2. License of contributions

By submitting a contribution, you license your work under
**Apache License, Version 2.0** — the same license under which Y4 is
distributed. See [`LICENSE`](./LICENSE).

Apache-2.0 §3 ("Grant of Patent License") binds you with respect to any
patent claims you control that read on your contribution. Do not contribute
code you cannot grant under those terms.

### Linux-driver capsules (special case)

If you are porting a GPL-licensed Linux driver as a 3rd-tier fallback
capsule (see `docs/licensing.md` §"Linux driver tier"), the **capsule
binary stays GPLv2** and is distributed separately. The Y4 main tree must
not directly link your capsule's source — only its external ABI is
permitted. Submit such ports as a sibling repository (or `capsules/gpl/`
sub-tree) clearly tagged with the original GPL header.

## 3. SPDX headers

Every source file Y4 ships must begin with a machine-readable SPDX header.

For Y4-original code:

```rust
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
```

```c
/* SPDX-License-Identifier: Apache-2.0 */
/* SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors */
```

```toml
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
```

For ported code, **preserve the original SPDX line** and add a second line
indicating the porter:

```rust
// SPDX-License-Identifier: BSD-2-Clause
// SPDX-FileCopyrightText: <upstream copyright as in original file>
// SPDX-FileCopyrightText: 2026 Y4 contributors (port and Rust capsule wrap)
```

For documentation:

```markdown
<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->
```

## 4. Code style

Phase 0 / Phase 1 internal-only — style guide will be promoted to a
proper file when external contribution begins. Until then:

- Rust: `cargo fmt` + `cargo clippy -- -D warnings` clean.
- C / kernel-adjacent: `clang-format` with the kernel preset for whatever
  base you are porting from (seL4 / Linux / NetBSD).
- Commit messages: subsystem-prefixed, e.g. `hiu: bind XChaCha20 nonce on
  context_switch raise`. Imperative mood, body explains *why*.

## 5. Verification expectations

Y4 follows **formal-first**: specifications and proofs land **before** the
implementation they describe. PRs that introduce new privileged code paths
or capability primitives must include the corresponding Verus / Coq
artifacts, not as follow-up.

Exceptions: bug fixes that do not alter specified behavior, ergonomics of
build tooling, documentation, capsule ports of code that already has its
own upstream verification trail.

## 6. Security disclosure

Until a dedicated security email is registered, report vulnerabilities
directly to the project maintainer (`yeun0908@gmail.com`) with `[Y4
SECURITY]` in the subject. Do not file public issues for unfixed
vulnerabilities.

## 7. Discussion

Until a public forum is set up, design discussion happens in the issue
tracker of this repository.
