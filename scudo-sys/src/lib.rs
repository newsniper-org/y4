// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Hand-written FFI declarations for the LLVM scudo standalone
//! allocator's C API (`wrappers_c.cpp`).
//!
//! Bindgen is intentionally avoided — the public C surface scudo
//! exports is six functions wide; hand-writing them keeps the build
//! fast (no libclang dep) and the bindings auditable.
//!
//! All functions are `unsafe extern "C"` matching scudo's signatures.
//! Safe wrappers live in `y4-alloc::scudo_ffi` (added in a follow-up
//! patch on this PR).

#![no_std]

use core::ffi::c_void;

/// Mirror of C's `size_t`.  scudo's headers use the platform `size_t`,
/// which on every Y4 target equals `usize`.
#[allow(non_camel_case_types)]
pub type size_t = usize;

unsafe extern "C" {
    /// Allocate `size` bytes.  Returns a null pointer on OOM.
    /// scudo provides this from `wrappers_c.cpp` as `scudo_malloc`.
    pub fn scudo_malloc(size: size_t) -> *mut c_void;

    /// Allocate `size` bytes aligned to `alignment` (must be a power of
    /// two and a multiple of `sizeof(void*)`).  Returns null on OOM or
    /// invalid alignment.
    pub fn scudo_aligned_alloc(alignment: size_t, size: size_t) -> *mut c_void;

    /// Allocate `nmemb * size` bytes, zero-initialised.
    pub fn scudo_calloc(nmemb: size_t, size: size_t) -> *mut c_void;

    /// Resize an allocation.  `ptr == null` is equivalent to malloc;
    /// `size == 0` is equivalent to free.
    pub fn scudo_realloc(ptr: *mut c_void, size: size_t) -> *mut c_void;

    /// Free a previously-allocated pointer.  Null is a no-op.
    pub fn scudo_free(ptr: *mut c_void);

    /// Query the usable size of an allocation.
    pub fn scudo_malloc_usable_size(ptr: *const c_void) -> size_t;
}
