#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
#
# Convert the Y4 fork patch series into a seL4 mainline submission format.
# Reference: docs/sel4_fork_policy.md §6.8.
#
# Effect:
#   - takes third_party/sel4-patches/[0-9]*.patch
#   - strips the Y4-specific naming (Y4_AMDV → SVM, Y4AMDVEnabled → KernelSVM,
#     leading '/* Y4-fork: ' comment marker)
#   - writes the converted patches to <output_dir>
#   - the converted series is suitable for `git format-patch`-style review
#     by seL4 mainline maintainers.
#
# Usage:
#   tools/sel4-mainline-export.sh <output_dir>
#   just sel4-mainline-export <output_dir>

set -euo pipefail

if [ $# -ne 1 ]; then
    echo "usage: $0 <output_dir>" >&2
    exit 64
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PATCHES_DIR="$ROOT/third_party/sel4-patches"
OUT_DIR="$1"

mkdir -p "$OUT_DIR"

shopt -s nullglob
PATCHES=("$PATCHES_DIR"/[0-9]*.patch)
shopt -u nullglob

if [ "${#PATCHES[@]}" -eq 0 ]; then
    echo "[sel4-mainline-export] no patches in $PATCHES_DIR — nothing to export"
    exit 0
fi

echo "[sel4-mainline-export] exporting ${#PATCHES[@]} patch(es) → $OUT_DIR"

for p in "${PATCHES[@]}"; do
    name="$(basename "$p")"
    out="$OUT_DIR/$name"
    sed -e 's/Y4_AMDV/SVM/g' \
        -e 's/Y4AMDVEnabled/KernelSVM/g' \
        -e 's|/\* Y4-fork: |/* |g' \
        -e 's|// Y4-fork: |// |g' \
        "$p" > "$out"
    echo "  - $name"
done

echo "[sel4-mainline-export] done.  Review the output before submission."
echo "[sel4-mainline-export] note: Y4-internal references (e.g., docs/amdv_safety.md"
echo "[sel4-mainline-export]       cross-refs in commit messages) survive the rename"
echo "[sel4-mainline-export]       and should be hand-edited or stripped per maintainer"
echo "[sel4-mainline-export]       review feedback."
