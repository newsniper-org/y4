<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# y4-scudo-sys — LLVM scudo standalone FFI

Hand-written `unsafe extern "C"` declarations for the LLVM scudo
standalone allocator's C API (`wrappers_c.cpp` —
`scudo_malloc` / `scudo_free` / ...).  Used by
[`y4-alloc`](../alloc/) when the `scudo` cargo feature is enabled.

The C++ source itself is **not committed** — it lives at the pinned
SHA recorded in [`/third_party/scudo/PIN.toml`](../third_party/scudo/PIN.toml)
and is materialised on demand by `just scudo-fetch`.  See
[`/third_party/scudo/README.md`](../third_party/scudo/README.md) for
the rationale (avoiding a 175k-file llvm-project submodule index).

## Build

`build.rs` compiles the standalone subdirectory with `cc` (in C++17,
`-fno-exceptions`, `-fno-rtti`, `-fvisibility=hidden`) and links the
result as a static library.  The C wrapper symbols are prefixed with
`scudo_` via `-DSCUDO_PREFIX_NAME=scudo_` so they don't clash with
libc's `malloc` / `free`.

Bindgen is intentionally avoided — the public C surface is six
functions wide; hand-writing them keeps the build fast (no libclang
dependency) and the bindings auditable.

## Tests

```sh
just scudo-fetch              # one-time per fresh clone (~1 MB on disk)
cargo test -p y4-scudo-sys    # 4 tests
```

- `malloc_then_free` — basic round-trip.
- `aligned_alloc_honours_alignment` — `scudo_aligned_alloc(128, 256)`
  returns a 128-byte-aligned pointer.
- `usable_size_at_least_request` — `scudo_malloc_usable_size` >=
  requested.
- `many_allocations_disjoint` — operational B1: 32 distinct live
  pointers are pairwise unequal.

## Hosted Linux only

This crate targets *hosted Linux* builds.  The seL4 / `no_std` Y4
build cannot link scudo directly until `kernel/` provides the platform
shims (mmap, pthread, etc.) scudo expects.  Until then the production
path on bare-metal Y4 stays on
[`y4-alloc::hardened::HardenedBackend`](../alloc/src/hardened.rs),
which satisfies the same B1–B6 contract in pure Rust.
