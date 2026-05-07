#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
#
# G1~G7 regression gates for the Y4 seL4 fork patch series.
# Reference: docs/sel4_fork_policy.md §2 (G1~G5) + §3.6 (G6 timing-equal
# optional) + power_safety.md §5.4 (G7 power-related timing-equal).
#
# Usage:
#   tools/sel4-fork-check.sh             # run all gates
#   tools/sel4-fork-check.sh G5          # run only G5
#   just sel4-fork-check
#
# v0 status: gates are STUB (echo + exit 0).  Real implementations land
# alongside the patch series — each gate's logic depends on the patches
# present.  Intentional layering: this scaffold lets `just sel4-fork-check`
# work today, and gates light up incrementally as patches are added.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

usage() {
    echo "usage: $0 [G1|G2|G3|G4|G5|G6|G7|all]"
    echo "       all (default) runs every gate"
    exit 64
}

# ---- G1: sel4test passes ----
g1_sel4test() {
    echo "[G1] sel4test regression — STUB (v0)"
    echo "[G1] real impl: clone sel4test, build with the patched sel4 submodule,"
    echo "[G1]            run via QEMU, expect 0 fail."
    return 0
}

# ---- G2: seL4 unit tests pass ----
g2_unit_tests() {
    echo "[G2] seL4 unit tests — STUB (v0)"
    echo "[G2] real impl: cmake/cocotb-driven unit tests inside third_party/sel4."
    return 0
}

# ---- G3: y4-roottask boots unchanged ("Hello, Y4") ----
g3_y4_roottask_boot() {
    echo "[G3] y4-roottask 'Hello, Y4' boot — STUB (v0)"
    echo "[G3] real impl: just qemu-smoke (existing recipe), expect serial 'Hello, Y4'."
    return 0
}

# ---- G4: Verus 50+ invariants still verified ----
g4_verus_unchanged() {
    echo "[G4] Verus 50+ invariants — STUB (v0)"
    echo "[G4] real impl: just verus, expect '51 verified, 0 errors' (or grow)."
    return 0
}

# ---- G5: diff audit (byte-equal when CONFIG_Y4_AMDV=OFF) ----
g5_diff_audit() {
    echo "[G5] diff audit / byte-equal (CONFIG_Y4_AMDV=OFF) — STUB (v0)"
    echo "[G5] real impl: build sel4 twice (upstream pin vs patched-with-AMDV-OFF),"
    echo "[G5]            diff -r build/sel4-upstream build/sel4-y4-fork-off,"
    echo "[G5]            expect every .o / .elf byte-equal (sel4_fork_policy §3.2.1)."
    return 0
}

# ---- G6: timing-equal (KernelDebugBuild=ON, ±5%) ---- (optional in v1.0)
g6_timing_equal() {
    echo "[G6] timing-equal (sel4_fork_policy §3.6) — STUB (v0, optional)"
    echo "[G6] real impl: sel4bench-equivalent latency trace with CONFIG_Y4_AMDV=OFF,"
    echo "[G6]            compare to upstream pin, expect ±5%."
    return 0
}

# ---- G7: power-related syscall timing-equal ---- (power_safety §5.4)
g7_power_timing() {
    echo "[G7] power-related syscall timing-equal (power_safety §5.4) — STUB (v0)"
    echo "[G7] real impl: HLT / MWAIT / SMI / ACPI eval / wake event handling"
    echo "[G7]            latency trace with CONFIG_Y4_AMDV=OFF vs upstream, ±5%."
    return 0
}

run_gate() {
    case "$1" in
        G1) g1_sel4test ;;
        G2) g2_unit_tests ;;
        G3) g3_y4_roottask_boot ;;
        G4) g4_verus_unchanged ;;
        G5) g5_diff_audit ;;
        G6) g6_timing_equal ;;
        G7) g7_power_timing ;;
        *) usage ;;
    esac
}

if [ $# -eq 0 ] || [ "${1:-all}" = "all" ]; then
    for g in G1 G2 G3 G4 G5 G6 G7; do
        run_gate "$g"
    done
    echo "[sel4-fork-check] all gates dispatched (v0 stubs)."
elif [ $# -eq 1 ]; then
    run_gate "$1"
else
    usage
fi
