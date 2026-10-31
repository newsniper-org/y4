# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
#
# Top-level Y4 build orchestration.
#
# This justfile delegates to per-subtree justfiles where they exist.
# Cross-subtree concerns (workspace lint, hash-driven freshness across
# the whole tree, Verus + Coq verification gates) live here.
#
# Hash-driven incremental: each recipe that produces an artifact stamps
# it via `stamp record` and shortcuts via `freshcheck`.
#
# logicutils tools are absorbed into adsmt (P5 option 5, 2026-06-01).
# Install path:
#   Arch:    pacman -S adsmt-testing      # archlinux PKGBUILD (system-wide)
#   cargo:   cargo install --git https://github.com/newsniper-org/adsmt --branch testing \
#                          freshcheck stamp lu-par lu-rule
# Man pages (post-install):
#   /usr/share/man/man1/{freshcheck,stamp,lu-par,lu-rule}.1
# Or in-tree adsmt repo:
#   ~/AD1/state/logicutils-frozen/docs/man/  +  ~/AD1/docs/man/

set shell := ["bash", "-cu"]
set dotenv-load := false

# --- Defaults / discovery -------------------------------------------------

# Set Y4_STAMP_STORE in your shell to relocate the stamp database.
stamp_store := env_var_or_default("Y4_STAMP_STORE", justfile_directory() + "/.cache/stamps")

# MSRV toolchain = workspace `rust-version`, padded to a full release
# ("1.94" → "1.94.0") so it names an installable toolchain.
msrv_toolchain := `v=$(sed -n 's/^rust-version *= *"\(.*\)"/\1/p' Cargo.toml | head -1); case "$v" in *.*.*) echo "$v";; *) echo "$v.0";; esac`

# Number of parallel slots for lu-par. Override with -j on the CLI:
#     just par-build j=16
j := env_var_or_default("Y4_J", num_cpus())

# --- Default target -------------------------------------------------------

default:
    @just --list --justfile {{justfile()}}

# --- git via dev env ------------------------------------------------------

# Run the git CLI with `.env.dev-git` applied, then forward all args verbatim.
# `.env.dev-git` (gitignored) points SSH_ASKPASS at ksshaskpass so SSH
# pushes/fetches prompt via the KDE dialog instead of failing on the absent
# /usr/lib/ssh/ssh-askpass.  If the env file is missing, git still runs.
#
# Usage:  just git <git-args...>            e.g.  just git push origin main
#         flags that `just` swallows need `--`:
#                 just git -- push --no-recurse-submodules origin main
git *ARGS:
    #!/usr/bin/env bash
    set -uo pipefail
    envf="{{justfile_directory()}}/.env.dev-git"
    if [ -f "$envf" ]; then set -a; . "$envf"; set +a; fi
    exec git {{ARGS}}

# --- Workspace-wide Rust hygiene -----------------------------------------

# `cargo fmt --check` across the workspace.  Skipped until first member.
fmt-check:
    @if grep -q 'members = \[\]' Cargo.toml; then \
        echo "[fmt-check] workspace has no members yet — skip"; \
    else \
        cargo fmt --all -- --check; \
    fi

# Auto-format the workspace.
fmt:
    @if grep -q 'members = \[\]' Cargo.toml; then \
        echo "[fmt] workspace has no members yet — skip"; \
    else \
        cargo fmt --all; \
    fi

# `cargo clippy -- -D warnings` across the workspace, including tests.
lint:
    @if grep -q 'members = \[\]' Cargo.toml; then \
        echo "[lint] workspace has no members yet — skip"; \
    else \
        cargo clippy --workspace --all-targets --locked -- -D warnings; \
    fi

# Build everything in the workspace.
build:
    @if grep -q 'members = \[\]' Cargo.toml; then \
        echo "[build] workspace has no members yet — skip"; \
    else \
        cargo build --workspace --locked; \
    fi

# Run all unit + integration tests.
test:
    @if grep -q 'members = \[\]' Cargo.toml; then \
        echo "[test] workspace has no members yet — skip"; \
    else \
        cargo test --workspace --locked; \
    fi

# --- Cross / bare-metal / MSRV gates ------------------------------------

# Bare-metal build gate: every no_std workspace crate for each seL4 arch
# Y4 keeps in the workspace (x86_64 first; AArch64 and RV64GC kept by user
# decision 2026-10-31), plus the root task.  Targets are listed in
# rust-toolchain.toml so rustup installs them.
bare-metal:
    cargo build --locked --release --target x86_64-unknown-none -p y4-fip-core -p y4-fip-x86_64 -p y4-ipc -p y4-alloc -p y4-capsules
    cargo build --locked --release --target aarch64-unknown-none -p y4-fip-core -p y4-fip-aarch64 -p y4-ipc -p y4-alloc -p y4-capsules
    cargo build --locked --release --target riscv64gc-unknown-none-elf -p y4-fip-core -p y4-fip-riscv64 -p y4-ipc -p y4-alloc -p y4-capsules
    cd kernel && cargo build --locked --release

# MSRV gate: the workspace must build with the declared rust-version.
msrv:
    @rustup toolchain list | grep -q '^{{msrv_toolchain}}-' \
        || { echo "[msrv] toolchain {{msrv_toolchain}} missing — rustup toolchain install {{msrv_toolchain}} --profile minimal"; exit 1; }
    cargo +{{msrv_toolchain}} check --workspace --all-targets --locked

# AArch64 / RV64GC unit + doc tests of the fip crates under qemu-user.
# Runs at idle priority (nice 19 + idle I/O class) so it never competes
# with interactive work (user decision 2026-10-31).  Static musl binaries
# linked by rust-lld — no cross gcc needed.
fip-qemu:
    #!/usr/bin/env bash
    set -euo pipefail
    for q in qemu-aarch64 qemu-riscv64; do
        command -v "$q" >/dev/null || { echo "[fip-qemu] $q missing (qemu-user)"; exit 1; }
    done
    idle="nice -n 19 ionice -c 3"
    static="-C target-feature=+crt-static -C link-self-contained=yes"
    echo "[fip-qemu] aarch64-unknown-linux-musl (qemu-aarch64, idle)"
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="$static" \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUNNER="$idle qemu-aarch64" \
    RUSTDOCFLAGS="$static -C linker=rust-lld" \
        $idle cargo test --locked --target aarch64-unknown-linux-musl -p y4-fip-core -p y4-fip-aarch64
    echo "[fip-qemu] riscv64gc-unknown-linux-musl (qemu-riscv64, idle)"
    CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_MUSL_LINKER=rust-lld \
    CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_MUSL_RUSTFLAGS="$static" \
    CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_MUSL_RUNNER="$idle qemu-riscv64" \
    RUSTDOCFLAGS="$static -C linker=rust-lld" \
        $idle cargo test --locked --target riscv64gc-unknown-linux-musl -p y4-fip-core -p y4-fip-riscv64

# --- Verification (formal-first gate) ------------------------------------

# --- Phase B step 5: kernel/ root task ----------------------------------

# Build the Y4 root task ELF (bare-metal x86_64-unknown-none).  Excluded
# from the workspace — its own .cargo/config.toml selects the target.
roottask-build:
    cd kernel && cargo build --release
    @ls -lh kernel/target/x86_64-unknown-none/release/y4-roottask

# --- Phase B step 2: boot subtree ---------------------------------------

# Verify host build prerequisites for the boot chain (cmake/ninja/gcc/...).
boot-deps:
    cd boot && just deps-check

# Build seL4 kernel.elf via boot/justfile (logicutils-driven cmake call).
sel4-build target="x86_64-debug":
    cd boot && just sel4-build {{target}}

# Print the expanded cmake recipe without running it.
sel4-dry-run target="x86_64-debug":
    cd boot && just sel4-dry-run {{target}}

# Build Limine bootloader binaries.
limine-build:
    cd boot && just limine-build

# Pack kernel.elf + Limine + config into a hybrid ISO.
iso-build target="x86_64-debug":
    cd boot && just iso-build {{target}}

# Boot the ISO under QEMU interactively (Ctrl-A x to exit).
qemu-boot target="x86_64-debug":
    cd boot && just qemu-boot {{target}}

# Phase B step 2 milestone gate: seL4 boot output reached under QEMU.
qemu-smoke target="x86_64-debug":
    cd boot && just qemu-smoke {{target}}

# --- Verification (formal-first gate) ------------------------------------

# Verify Verus specs.  Errors helpfully if `verus` is not installed.
verus:
    cd proofs/verus && just

# Verify Rocq (formerly Coq) theories.  Errors helpfully if `rocq`/`coqc` missing.
coq:
    cd proofs/coq && just

# Run the full proof gate — this is what CI gates merges on.
proofs: verus coq

# --- Hash-stamped composite gate -----------------------------------------
#
# Combines fmt-check + lint + test + bare-metal + msrv + proofs and skips
# any of them whose inputs have not changed since the last green run.
# Each step records its signature on success; the next invocation
# short-circuits with `freshcheck`.
#
# Inputs of the Rust gates = every Cargo.toml / *.rs outside build
# outputs, third_party/, proofs/verus/ and verus-fork/ (not workspace
# members), plus rust-toolchain.toml, Cargo.lock, kernel/Cargo.lock, this
# justfile and a toolchain fingerprint (`rustc -Vv` · `cargo -V` · the
# MSRV toolchain) — so a `stable` bump re-runs the gates even when no
# source changed.

ci:
    @mkdir -p {{stamp_store}}
    @just _toolchain-fingerprint
    @just _gate fmt-check  "$(just _rust-deps)"
    @just _gate lint       "$(just _rust-deps)"
    @just _gate test       "$(just _rust-deps)"
    @just _gate bare-metal "$(just _rust-deps)"
    @just _gate msrv       "$(just _rust-deps)"
    @just _gate proofs     "$(find proofs -type f \( -name '*.rs' -o -name '*.v' -o -name 'Cargo.toml' -o -name '_CoqProject' -o -name 'justfile' \) 2>/dev/null | sort) {{stamp_store}}/toolchain.txt"

# What the pre-push hook runs: `ci` plus the AArch64 / RV64GC fip tests
# under qemu-user at idle priority (hash-gated like the rest).
ci-push: ci
    @just _gate fip-qemu "$(just _rust-deps)"

# Dependency list of the Rust gates (see `ci`).
_rust-deps:
    @{ find . -type f \( -name 'Cargo.toml' -o -name '*.rs' \) -not -path '*/target/*' -not -path './.cache/*' -not -path './third_party/*' -not -path './proofs/verus/*' -not -path './verus-fork/*' 2>/dev/null; printf '%s\n' ./rust-toolchain.toml ./Cargo.lock ./kernel/Cargo.lock ./justfile {{stamp_store}}/toolchain.txt; } | sort

# Record the active toolchains so a toolchain change invalidates the gates.
_toolchain-fingerprint:
    @mkdir -p {{stamp_store}}
    @{ rustc -Vv; cargo -V; echo "msrv: {{msrv_toolchain}}"; rustc +{{msrv_toolchain}} -V 2>/dev/null || echo "msrv toolchain missing"; } > {{stamp_store}}/toolchain.txt.new
    @cmp -s {{stamp_store}}/toolchain.txt.new {{stamp_store}}/toolchain.txt 2>/dev/null \
        && rm {{stamp_store}}/toolchain.txt.new \
        || mv {{stamp_store}}/toolchain.txt.new {{stamp_store}}/toolchain.txt

# Internal helper: hash-stamped invocation of a single gate.
# $1 = recipe name (also used as sentinel filename).
# $2 = whitespace-separated dep file list (may be empty).
_gate name deps:
    @sentinel={{stamp_store}}/{{name}}.ok; \
    deps="{{deps}}"; \
    if [ -f "$sentinel" ] && [ -n "$deps" ] \
       && freshcheck --method=hash --store={{stamp_store}} "$sentinel" $deps >/dev/null 2>&1; then \
        echo "[{{name}}] up-to-date (skipped)"; \
    else \
        just {{name}} \
        && touch "$sentinel" \
        && { [ -z "$deps" ] || stamp record --method=hash --store={{stamp_store}} "$sentinel" $deps >/dev/null; }; \
    fi

# --- scudo upstream mirror ----------------------------------------------
#
# `third_party/scudo/PIN.toml` records the exact LLVM commit + subpath
# that y4-scudo-sys vendors.  `scudo-fetch` materialises that pin into
# `third_party/scudo/standalone/` (gitignored) without leaving a
# 175k-file submodule index behind.

# Fetch the pinned scudo standalone subtree into third_party/scudo/standalone.
# Idempotent: re-running with the same PIN.toml is a no-op (clean exit).
scudo-fetch:
    @bash tools/scudo-fetch.sh

# Drop the materialised standalone tree.  Next `scudo-fetch` re-clones.
scudo-fetch-clean:
    rm -rf third_party/scudo/standalone

# --- Memory mirror -------------------------------------------------------
#
# Claude Code's memory store lives outside the repo at
#   /home/ybi/.claude/projects/-home-ybi-Y4/memory/
# so it survives `rm -rf Y4/` but is lost if the home dir is wiped.
# `mirror-memory` keeps an in-repo copy at .claude-memories/ as a
# safety net; commit it like any other doc.
#
# Wiring options (none enforced — user decides):
#   - run before each commit:  add `just mirror-memory` to a pre-commit hook
#   - run on a schedule:       systemd-user timer or cron calling
#                              `just -f /home/ybi/Y4/justfile mirror-memory`
#   - run on Claude exit:      add to settings.json `Stop` hook

memory_src := "/home/ybi/.claude/projects/-home-ybi-Y4/memory"
memory_dst := justfile_directory() + "/.claude-memories"

# Mirror live memory → .claude-memories/ (exact copy, deletions propagated).
# Safe to re-run; rsync only moves changed bytes.
mirror-memory:
    @mkdir -p {{memory_dst}}
    @rsync -av --delete --itemize-changes \
        {{memory_src}}/ {{memory_dst}}/

# Show what mirror-memory WOULD change without writing anything.
mirror-memory-dry:
    @rsync -avn --delete --itemize-changes \
        {{memory_src}}/ {{memory_dst}}/

# --- Git hook installer --------------------------------------------------
#
# Y4 ships its git hooks in `tools/git-hooks/` (committed) so every
# clone gets the same checks.  This recipe wires git to use them by
# setting `core.hooksPath` — a one-time, repo-local config change.
install-hooks:
    git config core.hooksPath tools/git-hooks
    @echo "[install-hooks] core.hooksPath = tools/git-hooks"
    @ls -l tools/git-hooks/

# --- Claude Code session resume -----------------------------------------
#
# `.claude-recent-session-id` is written by the SessionStart hook in
# `.claude/settings.json` whenever a Claude Code session starts in this
# repo.  After [Ctrl+C] x2 ends a session, run `just claude-resume` (or
# `make claude-resume`) to re-enter that session.
claude-resume:
    @if [ ! -s .claude-recent-session-id ]; then \
        echo "[claude-resume] .claude-recent-session-id is empty or missing."; \
        echo "[claude-resume] Has a Claude Code session been started in this repo?"; \
        exit 1; \
    fi
    @sid=$(cat .claude-recent-session-id); \
        echo "[claude-resume] resuming session $sid"; \
        exec claude --resume "$sid"

# --- seL4 fork patch series ---------------------------------------------
#
# Y4 maintains its seL4 fork as an overlay patch directory
# (`third_party/sel4-patches/`) — see `docs/sel4_fork_policy.md` §6.
# These recipes drive the apply / regression-check / mainline-export
# workflow.  `tools/sel4-{fork-apply,fork-check,mainline-export}.sh`
# carry the actual logic.

# Apply Y4 fork patch series on top of the upstream pin.
sel4-fork-apply:
    @tools/sel4-fork-apply.sh

# Run G1~G7 regression gates (sel4_fork_policy §2 + §3.6 + power_safety §5.4).
# v0 status: gates are STUB; real logic lands alongside the patch series.
sel4-fork-check gate="all":
    @tools/sel4-fork-check.sh {{gate}}

# Convert the Y4 fork patch series into a seL4 mainline submission format
# (sel4_fork_policy §6.8).  Strips Y4-specific naming.
sel4-mainline-export output_dir:
    @tools/sel4-mainline-export.sh {{output_dir}}

# --- Stamp store maintenance ---------------------------------------------

# Drop the freshness store; next `just ci` re-runs every gate.
stamps-clear:
    rm -rf {{stamp_store}}

# Show what is currently considered up-to-date.
stamps-list:
    @[ -d {{stamp_store}} ] && stamp list --store={{stamp_store}} || echo "(empty stamp store)"

# --- Tool checks ---------------------------------------------------------

# Verify that the toolchain we depend on is present.  Used by CI bootstrap.
tools-check:
    @command -v cargo      >/dev/null || { echo "missing: cargo"; exit 1; }
    @command -v just       >/dev/null || { echo "missing: just"; exit 1; }
    @command -v freshcheck >/dev/null || { echo "missing: freshcheck — install via 'pacman -S adsmt-testing' (Arch) or 'cargo install --git https://github.com/newsniper-org/adsmt --branch testing freshcheck'"; exit 1; }
    @command -v stamp      >/dev/null || { echo "missing: stamp — same install path as freshcheck"; exit 1; }
    @command -v lu-par     >/dev/null || { echo "missing: lu-par — same install path as freshcheck"; exit 1; }
    @echo "core tools OK (cargo, just, logicutils — adsmt absorbed)"
    @[ -x verus-fork/source/target-verus/release/verus ] || echo "warn:  verus-fork submodule binary 없음 (verus-fork/source/target-verus/release/verus) — Verus proofs will fail. Build: git submodule update --init verus-fork && cd verus-fork/source && ./tools/get-z3.sh && source ../tools/activate.fish && vargo build --release"
    @command -v coqc       >/dev/null || echo "warn:  coqc not installed — Coq proofs will fail. See proofs/coq/README.md"
