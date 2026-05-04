// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! DragonFly-style slab front-end (Rust port).
//!
//! Reference (read-only): `/home/ybi/y4-upstream-refs/dragonfly/sys/kern/kern_slaballoc.c`
//!
//! Design summary:
//! - **Size classes** map any allocation request to one of `NUM_CLASSES`
//!   chunk sizes (powers-of-two ladder).
//! - **Per-CPU zones** hold free chunks of a single class; allocation
//!   pops from a CPU's zone with no global lock.
//! - **Cross-CPU free** is the only path that needs synchronisation.
//!   In a Y4 kernel build that maps to an seL4 IPI; in this port we
//!   emulate it as a simple "owner CPU" check that returns
//!   [`Y4Error::InvalidArg`] on a foreign free (caller must route).
//! - **Refill** requests pages from a [`PageBackend`] when a zone is
//!   exhausted.
//!
//! Spec correspondence (`proofs/verus/src/alloc/slab.rs`):
//!   * `S1 magazine_per_cpu_disjoint`  — held by `cpu_zones[cpu]`'s exclusive
//!     ownership of its `Zone` (no shared mutable refs).
//!   * `S2 zone_cache_size_bound`     — held by `Z_MAX` constant (zones
//!     never exceed it).
//!   * `S3 alloc_returns_aligned`     — held by zone chunks being multiples
//!     of the class size (which is a power of two ≥ alignment).

use crate::error::Y4Error;
use crate::page_backend::PageBackend;
use crate::types::{Allocation, Layout, Range};

/// Identifier for a CPU in the SLAB front-end.  Spec C2: SMP-first.
pub type CpuId = u8;

/// Maximum CPUs the SLAB tracks.  Sufficient for Phase B; a build-time
/// const widens it for larger form factors.
pub const MAX_CPUS: usize = 32;

/// Number of size classes the slab serves.  Larger requests bypass the
/// slab and go straight to the backend (`alloc_large`).
pub const NUM_CLASSES: usize = 9;

/// Per-zone cache size bound (S2).  Counted in chunks.
pub const Z_MAX: usize = 64;

/// Slab class table: class `i` covers chunks of `CLASS_SIZES[i]` bytes.
/// Powers of two from 8 B up to 2 KiB inclusive.
pub const CLASS_SIZES: [u64; NUM_CLASSES] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048];

/// Map a [`Layout`] to its slab class.  Returns `None` when the layout
/// is too large for the slab (`alloc_large` path).
#[must_use]
pub fn class_for(layout: Layout) -> Option<usize> {
    let need = layout.size.max(layout.align);
    CLASS_SIZES.iter().position(|&s| s >= need)
}

/// One zone — a vec of free chunk addresses for a specific (cpu, class).
#[derive(Debug, Default)]
struct Zone {
    free: heapless::Vec<u64, Z_MAX>,
    /// Page-grain backing region (returned to backend on drain).
    region: Option<Range>,
}

impl Zone {
    fn pop(&mut self) -> Option<u64> {
        self.free.pop()
    }

    fn push(&mut self, chunk: u64) -> Result<(), Y4Error> {
        self.free.push(chunk).map_err(|_| Y4Error::NoMemory)
    }

    fn len(&self) -> usize {
        self.free.len()
    }
}

/// Slab front-end.  Owns one zone per (cpu, class).  Refills from a
/// [`PageBackend`] on miss.
pub struct SlabAllocator {
    zones: [[Zone; NUM_CLASSES]; MAX_CPUS],
    /// Per-chunk owner (cpu, class) for cross-CPU free routing.
    /// Indexed by chunk start address modulo a small table — kept tiny
    /// for the spec-shape checker; production hashing would use a
    /// page-frame-keyed map.
    /// Maps a chunk start address to its owning (cpu, class).  Modest
    /// capacity — large allocations bypass this (and bypass the SLAB
    /// entirely).  Phase B step 4+ refinement may swap this for a
    /// page-frame-keyed structure.
    owners: heapless::LinearMap<u64, (CpuId, u8), 256>,
}

impl Default for SlabAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl SlabAllocator {
    /// Empty allocator with no pre-warmed zones.
    #[must_use]
    pub fn new() -> Self {
        Self {
            zones: core::array::from_fn(|_| core::array::from_fn(|_| Zone::default())),
            owners: heapless::LinearMap::new(),
        }
    }

    /// Allocate on behalf of `cpu`.  Path:
    /// 1. classify
    /// 2. pop from local zone if any
    /// 3. on miss, refill the zone from `backend` and retry
    ///
    /// # Errors
    /// [`Y4Error::InvalidArg`] for ill-formed `cpu` or `layout`;
    /// [`Y4Error::NoMemory`] when the backend cannot serve a refill.
    pub fn alloc<B: PageBackend>(
        &mut self,
        cpu: CpuId,
        layout: Layout,
        backend: &mut B,
    ) -> Result<Allocation, Y4Error> {
        if (cpu as usize) >= MAX_CPUS {
            return Err(Y4Error::InvalidArg);
        }
        let Some(class) = class_for(layout) else {
            // Large allocation — bypass slab.
            return alloc_large(layout, backend);
        };
        let chunk_size = CLASS_SIZES[class];
        loop {
            if let Some(addr) = self.zones[cpu as usize][class].pop() {
                let range = Range::new(addr, addr + chunk_size).ok_or(Y4Error::InvalidArg)?;
                return Ok(Allocation {
                    range,
                    layout,
                    numa: u32::from(cpu),
                    is_guarded: false, // SLAB does not install guards; backend does.
                });
            }
            self.refill(cpu, class, backend)?;
        }
    }

    /// Free a chunk previously vended by [`Self::alloc`].
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] if the chunk start is not tracked.
    /// [`Y4Error::InvalidArg`] if the freeing CPU does not own it
    /// (caller must route via cross-CPU IPI / async free).
    pub fn free(&mut self, cpu: CpuId, alloc: Allocation) -> Result<(), Y4Error> {
        let (owner_cpu, class) = self
            .owners
            .get(&alloc.range.start)
            .copied()
            .ok_or(Y4Error::BadCap)?;
        if owner_cpu != cpu {
            return Err(Y4Error::InvalidArg);
        }
        self.zones[cpu as usize][class as usize].push(alloc.range.start)
    }

    /// Push enough chunks of `class` into `cpu`'s zone to satisfy the
    /// next allocation.  Uses one backend page (rounded up).
    fn refill<B: PageBackend>(
        &mut self,
        cpu: CpuId,
        class: usize,
        backend: &mut B,
    ) -> Result<(), Y4Error> {
        let chunk = CLASS_SIZES[class];
        let region = backend.reserve(chunk * 16)?;
        // Carve up to Z_MAX chunks (or however many fit in the region).
        let region_chunks = usize::try_from(region.len() / chunk).unwrap_or(usize::MAX);
        let max_chunks = region_chunks.min(Z_MAX);
        let class_u8 = u8::try_from(class).unwrap_or(u8::MAX);
        for i in 0..max_chunks {
            let addr = region.start + (i as u64) * chunk;
            // Owner table entry (skip if at capacity — caller may free
            // chunks before fresh refill saturates the table).
            let _ = self.owners.insert(addr, (cpu, class_u8));
            self.zones[cpu as usize][class].push(addr)?;
        }
        // Stash region for accounting; release path lives in `drain`.
        self.zones[cpu as usize][class].region = Some(region);
        Ok(())
    }

    /// Live chunk count in (cpu, class).  S2 invariant test hook.
    #[must_use]
    pub fn zone_len(&self, cpu: CpuId, class: usize) -> usize {
        self.zones[cpu as usize][class].len()
    }
}

/// Allocations that bypass the slab (≥ largest class).  Direct backend
/// reserve.  Caller takes the range as a single allocation.
fn alloc_large<B: PageBackend>(layout: Layout, backend: &mut B) -> Result<Allocation, Y4Error> {
    let region = backend.reserve(layout.size)?;
    Ok(Allocation {
        range: region,
        layout,
        numa: 0,
        is_guarded: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page_backend::MockPageBackend;

    fn fresh_backend() -> MockPageBackend {
        MockPageBackend::new(0x10_0000, 4096, 1 << 20) // 1 MiB
    }

    #[test]
    fn class_lookup() {
        assert_eq!(class_for(Layout::new(8, 8).unwrap()), Some(0));
        assert_eq!(class_for(Layout::new(9, 8).unwrap()), Some(1));
        assert_eq!(class_for(Layout::new(2048, 8).unwrap()), Some(8));
        assert_eq!(class_for(Layout::new(4096, 8).unwrap()), None);
    }

    #[test]
    fn small_alloc_round_trip() {
        let mut backend = fresh_backend();
        let mut slab = SlabAllocator::new();
        let l = Layout::new(64, 8).unwrap();
        let a = slab.alloc(0, l, &mut backend).unwrap();
        assert!(a.aligned());
        assert_eq!(a.range.len(), 64);
        slab.free(0, a).unwrap();
    }

    #[test]
    fn per_cpu_zones_disjoint() {
        // S1: CPU 0's zone and CPU 1's zone never share an address.
        let mut backend = fresh_backend();
        let mut slab = SlabAllocator::new();
        let l = Layout::new(32, 8).unwrap();
        let a0 = slab.alloc(0, l, &mut backend).unwrap();
        let a1 = slab.alloc(1, l, &mut backend).unwrap();
        assert!(a0.range.disjoint(&a1.range));
    }

    #[test]
    fn cross_cpu_free_rejected() {
        // S1 corollary: free must come from owning CPU.
        let mut backend = fresh_backend();
        let mut slab = SlabAllocator::new();
        let l = Layout::new(64, 8).unwrap();
        let a = slab.alloc(0, l, &mut backend).unwrap();
        assert_eq!(slab.free(1, a), Err(Y4Error::InvalidArg));
        slab.free(0, a).unwrap();
    }

    #[test]
    fn large_alloc_bypasses_slab() {
        let mut backend = fresh_backend();
        let mut slab = SlabAllocator::new();
        let l = Layout::new(8192, 8).unwrap();
        let a = slab.alloc(0, l, &mut backend).unwrap();
        // Owner table NOT updated for large allocs — confirm by
        // attempting free returns BadCap (signals caller to use the
        // direct-release path).
        assert_eq!(slab.free(0, a), Err(Y4Error::BadCap));
    }

    #[test]
    fn zone_size_bounded_by_z_max() {
        // S2: post refill, zone len <= Z_MAX.  Backend rounds reservation
        // up to its page size, so the carve fills the zone to Z_MAX.
        let mut backend = fresh_backend();
        let mut slab = SlabAllocator::new();
        let l = Layout::new(8, 8).unwrap();
        let _ = slab.alloc(0, l, &mut backend).unwrap();
        let after_refill = slab.zone_len(0, 0);
        // The test bound is the spec invariant — Z_MAX is the cap.
        assert!(after_refill <= Z_MAX);
    }
}
