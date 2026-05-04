# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
#
# CMake initial-cache for seL4 x86_64 debug build.
# Consumed via `cmake -C boot/x86_64-debug.cmake ...` from boot/sel4.rules.
#
# Reference: third_party/sel4/configs/CMakeCache* and seL4 manual.
#
# Phase B step 2 first-pass values. Some -D entries are left commented
# until their actual upstream key is verified against the seL4 15.0.0
# release; the first `just sel4-build` PR closes those holes.

# ---- Architecture / platform ----
set(KernelArch       "x86"     CACHE STRING "")
set(KernelSel4Arch   "x86_64"  CACHE STRING "")
set(KernelPlatform   "pc99"    CACHE STRING "")

# ---- Build mode ----
set(CMAKE_BUILD_TYPE "Debug"   CACHE STRING "")

# ---- Cross-compiler ----
# x86_64 host-on-x86_64 build → no cross-compiler prefix needed.
set(CROSS_COMPILER_PREFIX "" CACHE STRING "")

# ---- Verification-aware features (Phase B has no verified Y4 layer
# above seL4 yet, but we keep the seL4 verification-friendly defaults).
set(KernelVerificationBuild OFF CACHE BOOL "")

# ---- Y4-specific knobs (placeholders — closed in first build PR) ----
# set(KernelMaxNumNodes  "1" CACHE STRING "")  # SMP nodes
# set(KernelDebugBuild   ON  CACHE BOOL   "")  # debug printk
# set(KernelPrinting     ON  CACHE BOOL   "")  # serial output
# set(KernelInvocationReportErrorIPC OFF CACHE BOOL "")
