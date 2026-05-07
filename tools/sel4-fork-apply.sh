#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
#
# Apply Y4 seL4 fork patch series on top of the upstream pin.
# Reference: docs/sel4_fork_policy.md §6.4.
#
# Usage:
#   tools/sel4-fork-apply.sh
#   just sel4-fork-apply
#
# Effect:
#   - cd third_party/sel4
#   - checkout the commit pinned in third_party/sel4-pin.txt
#   - apply third_party/sel4-patches/[0-9]*.patch in sequence via
#     `git am --3way --keep-cr`
#   - leaves the working tree of third_party/sel4 with the patches
#     applied; the submodule pointer is NOT updated (sel4_fork_policy
#     §6.4 H decision — the Y4 root commit only references the patch
#     series).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SEL4_DIR="$ROOT/third_party/sel4"
PATCHES_DIR="$ROOT/third_party/sel4-patches"
PIN_FILE="$ROOT/third_party/sel4-pin.txt"

if [ ! -d "$SEL4_DIR/.git" ] && [ ! -f "$SEL4_DIR/.git" ]; then
    echo "[sel4-fork-apply] $SEL4_DIR is not a git repo / submodule." >&2
    echo "[sel4-fork-apply] Run 'git submodule update --init third_party/sel4' first." >&2
    exit 2
fi

if [ ! -f "$PIN_FILE" ]; then
    echo "[sel4-fork-apply] $PIN_FILE missing." >&2
    exit 2
fi

# pin file format: "<hash> # <comment>" — strip comment + whitespace
PIN_HASH="$(awk '{print $1; exit}' "$PIN_FILE")"
if [ -z "$PIN_HASH" ]; then
    echo "[sel4-fork-apply] could not parse pin from $PIN_FILE" >&2
    exit 2
fi

echo "[sel4-fork-apply] checking out upstream pin $PIN_HASH"
cd "$SEL4_DIR"
git -c advice.detachedHead=false checkout "$PIN_HASH"

# discover patches: NNN-<topic>.patch in lexical order
shopt -s nullglob
PATCHES=("$PATCHES_DIR"/[0-9]*.patch)
shopt -u nullglob

if [ "${#PATCHES[@]}" -eq 0 ]; then
    echo "[sel4-fork-apply] no patches in $PATCHES_DIR — submodule is at upstream pin"
    exit 0
fi

echo "[sel4-fork-apply] applying ${#PATCHES[@]} patch(es)"
for p in "${PATCHES[@]}"; do
    echo "  - $(basename "$p")"
    git am --3way --keep-cr "$p"
done

echo "[sel4-fork-apply] done."
echo "[sel4-fork-apply] note: third_party/sel4 working tree has the patches applied,"
echo "[sel4-fork-apply]       but the submodule pointer in the Y4 root repo is not changed."
