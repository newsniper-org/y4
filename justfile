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
# it via `stamp record` and shortcuts via `freshcheck`. See
# /home/ybi/logicutils/docs/man/{freshcheck.1,stamp.1,lu-par.1}.

set shell := ["bash", "-cu"]
set dotenv-load := false

# --- Defaults / discovery -------------------------------------------------

# Set Y4_STAMP_STORE in your shell to relocate the stamp database.
stamp_store := env_var_or_default("Y4_STAMP_STORE", justfile_directory() + "/.cache/stamps")

# Number of parallel slots for lu-par. Override with -j on the CLI:
#     just par-build j=16
j := env_var_or_default("Y4_J", num_cpus())

# --- Default target -------------------------------------------------------

default:
    @just --list --justfile {{justfile()}}

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

# --- Verification (formal-first gate) ------------------------------------

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

# Boot the ISO under QEMU. Phase B step 2 success: "Hello, Y4" on serial.
qemu-boot target="x86_64-debug":
    cd boot && just qemu-boot {{target}}

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
# Combines fmt-check + lint + test + proofs and skips any of them whose
# inputs have not changed since the last green run.  Each step records
# its signature on success; the next invocation short-circuits with
# `freshcheck`.

ci:
    @mkdir -p {{stamp_store}}
    @# Each gate has a sentinel file in the stamp store; freshcheck
    @# compares the sentinel + recorded dep hashes to current state.
    @# If anything changed, the gate runs and re-records.
    @# We use `find` (not `git ls-files`) so untracked-but-staged-locally
    @# files are still picked up.
    @just _gate fmt-check "$(find . -type f \( -name 'Cargo.toml' -o -name '*.rs' \) -not -path './target/*' -not -path './.cache/*' -not -path './third_party/*' -not -path './proofs/verus/*' 2>/dev/null | sort)"
    @just _gate lint      "$(find . -type f \( -name 'Cargo.toml' -o -name '*.rs' \) -not -path './target/*' -not -path './.cache/*' -not -path './third_party/*' -not -path './proofs/verus/*' 2>/dev/null | sort)"
    @just _gate test      "$(find . -type f \( -name 'Cargo.toml' -o -name '*.rs' \) -not -path './target/*' -not -path './.cache/*' -not -path './third_party/*' -not -path './proofs/verus/*' 2>/dev/null | sort)"
    @just _gate proofs    "$(find proofs -type f \( -name '*.rs' -o -name '*.v' -o -name 'Cargo.toml' -o -name '_CoqProject' -o -name 'justfile' \) 2>/dev/null | sort)"

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
    @command -v freshcheck >/dev/null || { echo "missing: logicutils/freshcheck — see /home/ybi/logicutils/README.md"; exit 1; }
    @command -v stamp      >/dev/null || { echo "missing: logicutils/stamp"; exit 1; }
    @command -v lu-par     >/dev/null || { echo "missing: logicutils/lu-par"; exit 1; }
    @echo "core tools OK (cargo, just, logicutils)"
    @command -v verus      >/dev/null || echo "warn:  verus not installed — Verus proofs will fail. See proofs/verus/README.md"
    @command -v coqc       >/dev/null || echo "warn:  coqc not installed — Coq proofs will fail. See proofs/coq/README.md"
