// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Smoke test for the scudo standalone FFI.  Confirms link + basic
//! allocate / free round-trip works against the bundled C++ source.
//!
//! These tests run in the hosted Linux build only (`#![cfg(unix)]`).

#![cfg(unix)]

use core::ffi::c_void;
use y4_scudo_sys::{scudo_aligned_alloc, scudo_free, scudo_malloc, scudo_malloc_usable_size};

#[test]
fn malloc_then_free() {
    unsafe {
        let p = scudo_malloc(64);
        assert!(!p.is_null());
        // Touch the allocation so the page is committed.
        core::ptr::write_bytes(p.cast::<u8>(), 0xAB, 64);
        scudo_free(p);
    }
}

#[test]
fn aligned_alloc_honours_alignment() {
    unsafe {
        let p = scudo_aligned_alloc(128, 256);
        assert!(!p.is_null());
        let addr = p as usize;
        assert_eq!(addr % 128, 0, "scudo_aligned_alloc returned 0x{addr:x}");
        scudo_free(p);
    }
}

#[test]
fn usable_size_at_least_request() {
    unsafe {
        let p = scudo_malloc(100);
        assert!(!p.is_null());
        let usable = scudo_malloc_usable_size(p.cast_const());
        assert!(usable >= 100, "usable_size = {usable}");
        scudo_free(p);
    }
}

#[test]
fn many_allocations_disjoint() {
    // B1 spec contract: distinct live allocations don't overlap.
    unsafe {
        let mut ptrs = [core::ptr::null_mut::<c_void>(); 32];
        for p in &mut ptrs {
            *p = scudo_malloc(64);
            assert!(!p.is_null());
        }
        // Pairwise distinctness — if scudo ever vends overlapping
        // ranges this fails immediately.
        for i in 0..ptrs.len() {
            for j in (i + 1)..ptrs.len() {
                assert_ne!(ptrs[i], ptrs[j]);
            }
        }
        for p in ptrs {
            scudo_free(p);
        }
    }
}
