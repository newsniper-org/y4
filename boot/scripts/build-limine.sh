#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
#
# Build Limine 12.x: bootstrap → configure → make → stage outputs.
#
# Invoked from boot/limine.rules.  Lives in scripts/ rather than inline
# because the bootstrap step downloads tarballs and the output binary
# list is non-trivial; inlining would hurt readability.

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
src_dir="${repo_root}/third_party/limine"
out_dir="${repo_root}/build/limine/host"

[[ -f "${src_dir}/configure.ac" ]] || { echo "missing: third_party/limine submodule"; exit 1; }
command -v autoconf >/dev/null || { echo "missing: autoconf"; exit 1; }
command -v nasm     >/dev/null || { echo "missing: nasm"; exit 1; }
command -v mtools   >/dev/null || { echo "missing: mtools (Arch: pacman -S mtools)"; exit 1; }

cd "${src_dir}"

# bootstrap downloads dependency tarballs declared in 3RDPARTY.md and runs
# autoreconf.  Skip if its output already exists (idempotent).
if [[ ! -x ./configure ]]; then
    ./bootstrap
fi

# Configure for x86_64 BIOS + UEFI ports (Phase B step 2 = pc99).
./configure \
    --enable-bios \
    --enable-bios-cd \
    --enable-uefi-x86-64 \
    --enable-uefi-cd

make -j"$(nproc)"

mkdir -p "${out_dir}"
# Stage every binary the ISO assembly step might need.  Missing files
# are silently skipped (e.g. when a port is disabled) — the ISO script
# handles per-file presence.
for f in \
    bin/limine \
    bin/limine-bios.sys \
    bin/limine-bios-cd.bin \
    bin/limine-uefi-cd.bin \
    bin/BOOTX64.EFI ; do
    [[ -f "$f" ]] && cp "$f" "${out_dir}/" || true
done

ls -la "${out_dir}/"
