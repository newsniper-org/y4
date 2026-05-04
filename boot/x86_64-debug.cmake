# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
#
# CMake initial-cache for seL4 x86_64 debug build.
# Consumed via `cmake -C boot/x86_64-debug.cmake ...` from boot/sel4.rules.
#
# Reference: third_party/sel4/configs/X64_verified.cmake (verified config)
# and seL4 manual.  Every value here is set EXPLICITLY (rather than relying
# on seL4's defaults) so the build is reproducible across future seL4 bumps.

# ---- Architecture / platform ----
set(KernelArch       "x86"     CACHE STRING "")
set(KernelSel4Arch   "x86_64"  CACHE STRING "")
set(KernelPlatform   "pc99"    CACHE STRING "")

# ---- Build mode ----
set(CMAKE_BUILD_TYPE "Debug"   CACHE STRING "")

# ---- Cross-compiler ----
# x86_64 host-on-x86_64 build → no cross-compiler prefix needed.
set(CROSS_COMPILER_PREFIX "" CACHE STRING "")

# ---- Verification flag ----
# OFF for Phase B — the Y4 specialization layer above seL4 is not yet
# Verus-proven end-to-end, and we want fast-path code paths enabled.
# Phase E's certification track will flip this ON for the verified build.
set(KernelVerificationBuild OFF CACHE BOOL "")

# ---- Kernel feature set ----
set(KernelDebugBuild        ON      CACHE BOOL   "")  # debug printk available
set(KernelPrinting          ON      CACHE BOOL   "")  # serial output enabled — required for "Hello, Y4"
set(KernelFastpath          ON      CACHE BOOL   "")  # fast-path syscalls
set(KernelMaxNumNodes       "1"     CACHE STRING "")  # uniprocessor for now (Phase B step 2)
set(KernelNumDomains        "1"     CACHE STRING "")  # single scheduling domain (lease scheduler comes later)
set(KernelOptimisation      "-O2"   CACHE STRING "")
set(KernelBenchmarks        "none"  CACHE STRING "")
set(KernelRetypeFanOutLimit "256"   CACHE STRING "")

# ---- CPU feature dependencies ----
# Disabled for the Phase B step 2 "Hello, Y4" milestone so the kernel
# boots on QEMU's emulated CPU without needing -cpu host (which requires
# KVM) or specific micro-architectures.  Real hardware (and Phase D's
# WaveTensor host) has both — re-enable when promoting the platform
# config beyond QEMU.
set(KernelSupportPCID       OFF     CACHE BOOL   "")
set(KernelHugePage          OFF     CACHE BOOL   "")
set(KernelFSGSBase          "msr"   CACHE STRING "")
