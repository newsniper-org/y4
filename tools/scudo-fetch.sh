#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
#
# Materialise `third_party/scudo/standalone/` from the commit pinned in
# `third_party/scudo/PIN.toml`.  Designed to be called by `just
# scudo-fetch`; safe to run standalone.
#
# Footprint after a successful run: ~1 MB on disk, no `.git` artifacts.

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
pin_file="${repo_root}/third_party/scudo/PIN.toml"
out_dir="${repo_root}/third_party/scudo/standalone"

[[ -f "$pin_file" ]] || { echo "missing: $pin_file"; exit 1; }

# Tiny TOML parser — these three keys are simple `key = "value"` lines.
read_toml() {
    local key="$1"
    grep -E "^${key}[[:space:]]*=" "$pin_file" \
        | head -1 \
        | sed -E 's/^[^=]*=[[:space:]]*"([^"]*)".*$/\1/'
}

repo="$(read_toml repo)"
commit="$(read_toml commit)"
subpath="$(read_toml subpath)"

[[ -n "$repo" && -n "$commit" && -n "$subpath" ]] \
    || { echo "PIN.toml missing repo / commit / subpath"; exit 1; }

# Idempotency check: if standalone/ already matches the pin, exit clean.
stamp_file="${out_dir}/.PIN_SHA"
if [[ -f "$stamp_file" ]] && [[ "$(cat "$stamp_file")" == "$commit" ]]; then
    echo "[scudo-fetch] up-to-date at $commit"
    exit 0
fi

echo "[scudo-fetch] repo    = $repo"
echo "[scudo-fetch] commit  = $commit"
echo "[scudo-fetch] subpath = $subpath"

# Stage in a sibling tmpdir so a half-fetched run never half-replaces the
# existing standalone/ tree.
tmp_dir="${out_dir}.tmp"
rm -rf "$tmp_dir"
mkdir -p "$tmp_dir"

(
    cd "$tmp_dir"
    git init -q
    git remote add origin "$repo"
    git config core.sparseCheckoutCone true
    git sparse-checkout init --cone --sparse-index
    git sparse-checkout set "$subpath"
    # GitHub allows fetching arbitrary SHAs by name; --filter=blob:none keeps
    # the partial-clone footprint small.
    git fetch --depth=1 --filter=blob:none origin "$commit"
    git checkout -q FETCH_HEAD
)

# Move only the pinned subdir into the canonical location.  Drop the
# .git tree entirely — the PIN.toml is the only persistent record of
# the upstream coordinate.
rm -rf "$out_dir"
mkdir -p "$(dirname "$out_dir")"
mv "${tmp_dir}/${subpath}" "$out_dir"
rm -rf "$tmp_dir"

echo "$commit" > "$stamp_file"

du -sh "$out_dir"
echo "[scudo-fetch] OK"
