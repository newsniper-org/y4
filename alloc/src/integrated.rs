// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! `IntegratedAllocator` — `DragonFly` SLAB front-end + hardened backend
//! composition.  Realises the `X1–X3` boundary contracts from
//! `proofs/verus/src/alloc/boundary.rs`:
//!
//!   * `X1` SLAB pages ⊆ scudo pages — every chunk the SLAB hands out
//!     came from a backend reservation served by [`HardenedBackend`].
//!   * `X2` error propagation        — backend errors bubble up
//!     unchanged because the SLAB only wraps backend calls in the
//!     refill path.
//!   * `X3` composed no-overlap      — follows from `B1 + S1` because
//!     SLAB chunks live inside disjoint backend reservations.
//!
//! The integrated allocator is what `kernel/` will use as Y4's primary
//! `GlobalAlloc`-equivalent (Phase B step 5+).  A trait-objectish
//! abstraction is intentionally avoided: callers pick their CPU and
//! the layout, the integrated allocator does the rest.

use crate::error::Y4Error;
use crate::hardened::HardenedBackend;
use crate::page_backend::PageBackend;
use crate::slab::{CpuId, SlabAllocator, class_for};
use crate::types::{Allocation, Layout};

/// SLAB + hardened backend combo.
pub struct IntegratedAllocator {
    /// SLAB front-end — owns per-(cpu, class) zones.
    pub slab: SlabAllocator,
    /// Hardened backend — vends pages to the SLAB and handles large
    /// allocations directly (bypassing the SLAB ladder).
    pub hardened: HardenedBackend,
}

impl IntegratedAllocator {
    /// Build a fresh integrated allocator given a base [`PageBackend`]
    /// and an entropy seed for the hardened layer's randomization.
    ///
    /// # Errors
    /// Forwards the hardened backend's reservation failure (the SLAB
    /// itself does not reserve at construction).
    pub fn new<B: PageBackend>(
        backend: &mut B,
        hardened_bytes: u64,
        seed: u32,
    ) -> Result<Self, Y4Error> {
        let hardened = HardenedBackend::new(backend, hardened_bytes, seed)?;
        Ok(Self {
            slab: SlabAllocator::new(),
            hardened,
        })
    }

    /// Allocate.  Routes the request to either:
    /// - the SLAB front-end (small classes, hot path), refilled from
    ///   the hardened backend on miss;
    /// - the hardened backend directly (large allocations bypassing
    ///   the SLAB ladder).
    ///
    /// # Errors
    /// Forwards layer-specific failures unchanged (X2).
    pub fn alloc(&mut self, cpu: CpuId, layout: Layout) -> Result<Allocation, Y4Error> {
        if class_for(layout).is_some() {
            self.slab.alloc(
                cpu,
                layout,
                &mut HardenedAdapter {
                    inner: &mut self.hardened,
                },
            )
        } else {
            self.hardened.alloc(layout)
        }
    }

    /// Free an allocation.  Small allocations go back to the SLAB
    /// zone; large allocations go straight to the hardened quarantine.
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] if `alloc` is not currently live in either
    /// layer.
    pub fn free(&mut self, cpu: CpuId, alloc: Allocation) -> Result<(), Y4Error> {
        // Try the SLAB owner table first.  On BadCap we assume the
        // allocation came from the hardened backend (large bypass).
        match self.slab.free(cpu, alloc) {
            Ok(()) => Ok(()),
            Err(Y4Error::BadCap) => self.hardened.free(alloc),
            Err(e) => Err(e),
        }
    }
}

/// Adapter so the SLAB's `PageBackend` argument type-checks against
/// the hardened backend.  The hardened backend is not itself a
/// `PageBackend` (it serves user allocations, not pages); this adapter
/// reserves an integral number of pages by issuing a hardened
/// allocation of the requested byte count.
struct HardenedAdapter<'a> {
    inner: &'a mut HardenedBackend,
}

impl PageBackend for HardenedAdapter<'_> {
    fn page_size(&self) -> u64 {
        4096
    }

    fn reserve(&mut self, bytes: u64) -> Result<crate::types::Range, Y4Error> {
        // Round up to page granularity then issue one hardened alloc
        // of that size.  Use a maximally-permissive layout (page-aligned).
        let rounded = bytes.div_ceil(4096) * 4096;
        let layout = Layout::new(rounded, 4096).ok_or(Y4Error::InvalidArg)?;
        let alloc = self.inner.alloc(layout)?;
        Ok(alloc.range)
    }

    fn release(&mut self, _range: crate::types::Range) -> Result<(), Y4Error> {
        // Page-grain release through the hardened layer requires
        // reconstructing the Allocation; for now the SLAB owns its
        // refill regions until subsystem teardown.  Phase B step 5+
        // SLAB drain wires this up.
        Err(Y4Error::InvalidArg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page_backend::MockPageBackend;

    fn build() -> IntegratedAllocator {
        let mut base = MockPageBackend::new(0x10_0000, 4096, 1 << 20);
        IntegratedAllocator::new(&mut base, 1 << 20, 0xDEAD_BEEF).unwrap()
    }

    #[test]
    fn small_alloc_via_slab() {
        let mut a = build();
        let l = Layout::new(64, 8).unwrap();
        let r = a.alloc(0, l).unwrap();
        assert_eq!(r.range.len(), 64);
        a.free(0, r).unwrap();
    }

    #[test]
    fn large_alloc_via_hardened() {
        let mut a = build();
        let l = Layout::new(8192, 8).unwrap();
        let r = a.alloc(0, l).unwrap();
        assert_eq!(r.range.len(), 8192);
        assert!(r.is_guarded);
        a.free(0, r).unwrap();
    }

    #[test]
    fn x1_chunks_inside_backend_region() {
        // X1: every SLAB chunk lives inside a backend reservation.
        // Operationally: the SLAB chunk's range is fully inside the
        // hardened backend's owned region.
        let mut a = build();
        let l = Layout::new(64, 8).unwrap();
        let r = a.alloc(0, l).unwrap();
        let h = &a.hardened;
        assert!(r.range.start >= h.region_start());
        assert!(r.range.end <= h.region_end());
    }

    #[test]
    fn x3_pressure_no_overlap() {
        // X3: under sustained alloc pressure no two live ranges overlap.
        let mut a = build();
        let l = Layout::new(64, 8).unwrap();
        let mut acc: heapless::Vec<Allocation, 32> = heapless::Vec::new();
        for _ in 0..32 {
            acc.push(a.alloc(0, l).unwrap()).unwrap();
        }
        for i in 0..acc.len() {
            for j in (i + 1)..acc.len() {
                assert!(acc[i].range.disjoint(&acc[j].range));
            }
        }
    }
}
