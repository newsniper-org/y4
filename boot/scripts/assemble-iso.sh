#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
#
# Pack a Y4 boot ISO from kernel.elf + Limine binaries + limine.conf.
#
# Usage: assemble-iso.sh <target>
#   target = form-factor identifier (e.g. x86_64-debug)
#
# Invoked from boot/limine.rules.  Lives in scripts/ rather than inline
# because xorriso invocations are long and need careful flag ordering;
# inlining them in the lu-rule rule file would hurt readability without
# adding orchestration value.
#
# This script is intentionally minimal in Phase B step 2.  ISO layout
# choices (UEFI vs BIOS hybrid, EFI partition size, label) lock in with
# the first successful boot and are documented in boot/README.md then.

set -euo pipefail

target="${1:?usage: assemble-iso.sh <target>}"

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
kernel="${repo_root}/build/sel4/${target}/kernel.elf"
limine_dir="${repo_root}/third_party/limine"
limine_conf="${repo_root}/boot/limine.conf"
iso_root="${repo_root}/build/iso/${target}-staging"
iso_out="${repo_root}/build/iso/y4-${target}.iso"

[[ -f "$kernel"      ]] || { echo "missing: $kernel  (run 'just sel4-build ${target}' first)";  exit 1; }
[[ -f "$limine_conf" ]] || { echo "missing: $limine_conf"; exit 1; }
[[ -d "$limine_dir"  ]] || { echo "missing: third_party/limine submodule"; exit 1; }
command -v xorriso >/dev/null || { echo "missing: xorriso (Arch: pacman -S libisoburn)"; exit 1; }

mkdir -p "${iso_root}/boot/limine"
cp "$kernel"      "${iso_root}/boot/kernel.elf"
cp "$limine_conf" "${iso_root}/boot/limine/limine.conf"

# Limine 12.x ships the BIOS payloads as build artifacts; copy whatever
# is present in the Limine build output.  The exact list locks in with
# the first successful build run.
for f in limine-bios.sys limine-bios-cd.bin limine-uefi-cd.bin BOOTX64.EFI; do
    [[ -f "${repo_root}/build/limine/host/${f}" ]] && \
        cp "${repo_root}/build/limine/host/${f}" "${iso_root}/boot/limine/"
done

mkdir -p "$(dirname "$iso_out")"
xorriso -as mkisofs \
    -b boot/limine/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --efi-boot boot/limine/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image --protective-msdos-label \
    "${iso_root}" -o "${iso_out}"

echo "iso: ${iso_out}"
