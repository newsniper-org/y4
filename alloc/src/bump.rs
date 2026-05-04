// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! `BumpAllocator` — minimum-correct allocator over a [`PageBackend`].
//!
//! Hands out aligned ranges by linearly bumping a watermark.  No free,
//! no fragmentation, no NUMA awareness, no quarantine — but it is the
//! simplest implementation that satisfies the no-overlap invariant
//! (`alloc/mod.rs` `alloc_no_overlap`).  Every later allocator we ship
//! must do at least this well; the unit tests here are the lower
//! bound contract.

use crate::error::Y4Error;
use crate::page_backend::PageBackend;
use crate::types::{Allocation, Layout, Range};

/// A bump allocator that pulls one large region from a [`PageBackend`]
/// at construction time and serves layout-honouring slices from it.
#[derive(Debug)]
pub struct BumpAllocator {
    region: Range,
    next: u64,
    /// Mirrors scudo's `is_guarded` — bump never installs guards;
    /// the field is propagated so callers see the spec-shape directly.
    is_guarded: bool,
}

impl BumpAllocator {
    /// Construct an allocator that owns one region reserved from
    /// `backend`.  `bytes` is rounded up to a page boundary by the
    /// backend.
    ///
    /// # Errors
    /// Forwards backend reservation failures.
    pub fn new<B: PageBackend>(backend: &mut B, bytes: u64) -> Result<Self, Y4Error> {
        let region = backend.reserve(bytes)?;
        Ok(Self {
            region,
            next: region.start,
            is_guarded: false,
        })
    }

    /// Allocate a [`Layout`].
    ///
    /// # Errors
    /// Returns [`Y4Error::NoMemory`] when the remaining region cannot
    /// fit the request after honouring `layout.align`.
    pub fn alloc(&mut self, layout: Layout) -> Result<Allocation, Y4Error> {
        let aligned = align_up(self.next, layout.align).ok_or(Y4Error::NoMemory)?;
        let end = aligned.checked_add(layout.size).ok_or(Y4Error::NoMemory)?;
        if end > self.region.end {
            return Err(Y4Error::NoMemory);
        }
        self.next = end;
        let range = Range::new(aligned, end).ok_or(Y4Error::InvalidArg)?;
        Ok(Allocation {
            range,
            layout,
            numa: 0,
            is_guarded: self.is_guarded,
        })
    }

    /// Number of bytes still available for future allocations.
    #[must_use]
    pub fn remaining(&self) -> u64 {
        self.region.end - self.next
    }
}

/// Round `addr` up to the next multiple of `align` (which must be a
/// non-zero power of two).  Returns `None` on overflow.
fn align_up(addr: u64, align: u64) -> Option<u64> {
    debug_assert!(align.is_power_of_two() && align > 0);
    let mask = align - 1;
    addr.checked_add(mask).map(|x| x & !mask)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page_backend::MockPageBackend;

    fn fresh() -> (MockPageBackend, BumpAllocator) {
        let mut backend = MockPageBackend::new(0x1_0000, 4096, 64 * 4096);
        let bump = BumpAllocator::new(&mut backend, 64 * 4096).unwrap();
        (backend, bump)
    }

    #[test]
    fn alignment_honoured() {
        let (_b, mut a) = fresh();
        let l = Layout::new(100, 64).unwrap();
        let one = a.alloc(l).unwrap();
        let two = a.alloc(l).unwrap();
        assert!(one.aligned());
        assert!(two.aligned());
        assert!(one.range.disjoint(&two.range));
    }

    #[test]
    fn no_overlap_under_pressure() {
        let (_b, mut a) = fresh();
        let l = Layout::new(8, 8).unwrap();
        let mut prev: Option<Range> = None;
        for _ in 0..100 {
            let cur = a.alloc(l).unwrap().range;
            if let Some(p) = prev {
                assert!(p.disjoint(&cur));
            }
            prev = Some(cur);
        }
    }

    #[test]
    fn out_of_memory() {
        let mut backend = MockPageBackend::new(0x1_0000, 4096, 4096);
        let mut bump = BumpAllocator::new(&mut backend, 4096).unwrap();
        let l = Layout::new(2048, 8).unwrap();
        bump.alloc(l).unwrap();
        bump.alloc(l).unwrap();
        assert_eq!(bump.alloc(l), Err(Y4Error::NoMemory));
    }
}
