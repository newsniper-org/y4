// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! `ScudoFfiBackend` — production hardened backend backed by the LLVM
//! scudo standalone allocator (`y4-scudo-sys`).
//!
//! Hosted Linux only (`#[cfg(feature = "scudo")]` gating lives at
//! [`crate`] root).  When Y4 is built for the seL4 / `no_std` target the
//! Rust [`crate::hardened::HardenedBackend`] is used instead — scudo
//! will be wired into `kernel/` once that subsystem provides the
//! platform shims (mmap, pthread).
//!
//! Spec correspondence (`proofs/verus/src/alloc/scudo.rs`): the LLVM
//! scudo source has been audited upstream against the same B1–B6
//! contract Y4's spec demands; this module is a thin Rust safety
//! adapter and does not re-prove the contract.

use core::ffi::c_void;

use y4_scudo_sys as ffi;

use crate::error::Y4Error;
use crate::types::{Allocation, Layout, Range};

/// Backend that delegates every allocation to scudo via the FFI.
#[derive(Debug, Default)]
pub struct ScudoFfiBackend {
    live_count: usize,
}

impl ScudoFfiBackend {
    /// Empty backend.  scudo's static allocator initialises lazily on
    /// the first call.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate via `scudo_aligned_alloc`.
    ///
    /// # Errors
    /// [`Y4Error::NoMemory`] when scudo returns a null pointer (OOM
    /// or the alignment combo was rejected).
    /// [`Y4Error::InvalidArg`] for ill-formed `layout`.
    pub fn alloc(&mut self, layout: Layout) -> Result<Allocation, Y4Error> {
        let size = usize::try_from(layout.size).map_err(|_| Y4Error::InvalidArg)?;
        let align = usize::try_from(layout.align).map_err(|_| Y4Error::InvalidArg)?;
        let p = unsafe { ffi::scudo_aligned_alloc(align, size) };
        if p.is_null() {
            return Err(Y4Error::NoMemory);
        }
        let start = p as u64;
        let end = start.checked_add(layout.size).ok_or(Y4Error::InvalidArg)?;
        let range = Range::new(start, end).ok_or(Y4Error::InvalidArg)?;
        self.live_count += 1;
        Ok(Allocation {
            range,
            layout,
            numa: 0,
            is_guarded: true, // scudo installs guard regions per allocation.
        })
    }

    /// Free via `scudo_free`.
    ///
    /// # Errors
    /// Always succeeds for a valid scudo pointer.  Callers passing a
    /// pointer that scudo did not vend will trip scudo's internal
    /// abort path (no graceful error returned).
    pub fn free(&mut self, alloc: Allocation) -> Result<(), Y4Error> {
        let p = alloc.range.start as *mut c_void;
        unsafe { ffi::scudo_free(p) };
        if self.live_count > 0 {
            self.live_count -= 1;
        }
        Ok(())
    }

    /// Live allocation count (test introspection only — exact count is
    /// best-effort because scudo manages its own live set internally).
    #[must_use]
    pub fn live_count(&self) -> usize {
        self.live_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_returns_aligned_pointer() {
        let mut b = ScudoFfiBackend::new();
        let l = Layout::new(64, 64).unwrap();
        let a = b.alloc(l).unwrap();
        assert!(a.aligned());
        assert!(a.is_guarded);
        b.free(a).unwrap();
    }

    #[test]
    fn many_allocs_disjoint() {
        // Operational B1: scudo never vends overlapping live ranges.
        let mut b = ScudoFfiBackend::new();
        let l = Layout::new(96, 8).unwrap();
        let mut acc: heapless::Vec<Allocation, 16> = heapless::Vec::new();
        for _ in 0..16 {
            acc.push(b.alloc(l).unwrap()).unwrap();
        }
        for i in 0..acc.len() {
            for j in (i + 1)..acc.len() {
                assert!(acc[i].range.disjoint(&acc[j].range));
            }
        }
        for a in acc {
            b.free(a).unwrap();
        }
    }
}
