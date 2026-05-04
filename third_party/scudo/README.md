<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# `third_party/scudo` — LLVM scudo standalone (vendored on demand)

Y4 uses LLVM scudo as the production hardened backend for `y4-alloc`
(decision A-P1).  Bringing scudo in as a normal git submodule pulls
the entire LLVM index (~175k files) into Y4's git state, which makes
VSCode and other repo-walking tools sluggish even after sparse-checkout.

To dodge that, scudo is **vendored on demand** instead of as a submodule:

- [`PIN.toml`](./PIN.toml) records the exact upstream coordinate
  (repo + commit SHA + subpath).  This file IS committed.
- [`standalone/`](./standalone/) holds the materialised source.
  This directory is **gitignored** — `just scudo-fetch` rebuilds it
  from the pin, idempotently.
- The `y4-scudo-sys` build script reads from `standalone/` and emits
  a clear error if it's missing.

## Bringing scudo into a fresh clone

```sh
just scudo-fetch
cargo test -p y4-scudo-sys     # smoke test against the C++ allocator
```

Total disk footprint after fetch: ~1 MB.

## Bumping the pin

1. Edit `commit` in `PIN.toml` to the new SHA.
2. `rm -rf third_party/scudo/standalone`  (or `just scudo-fetch-clean`).
3. `just scudo-fetch`.
4. `cargo test -p y4-scudo-sys` to confirm the new revision still links.
5. Commit `PIN.toml` only.

## Why not a submodule

Past tries with `git submodule add` of `llvm-project` produced an
unmanageable 175k-entry index even after `git sparse-checkout` — VSCode
read it as 175k pending changes and ground to a halt.  Sparse-index
helped the on-disk index size (21 KB) but did not fix the file count
external tools see via `git ls-files`.  The vendored-on-demand pattern
sidesteps the problem entirely while preserving exact reproducibility
through the pinned SHA.
