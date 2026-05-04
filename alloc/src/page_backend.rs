// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! `PageBackend` — abstraction over the seL4 kernel calls the alloc
//! subsystem invokes.
//!
//! The trusted boundary (C4) is `seL4_X86_Page_Map`, `seL4_X86_Page_Unmap`,
//! `seL4_X86_PageTable_Map`, and `seL4_Untyped_Retype`.  This trait
//! collapses them into the three operations the alloc layer actually
//! needs: reserve a page-sized backing region, release it, and query
//! the page size.  The real seL4-backed implementor lives in `kernel/`
//! once that subsystem lands.
//!
//! `MockPageBackend` is provided for unit tests and for bringing up
//! the bump allocator without seL4 in the loop.

use crate::error::Y4Error;
use crate::types::{Range, VAddr};

/// Page-grain backend the alloc subsystem requires.  Implementations
/// must guarantee that every successful `reserve` returns a range
/// whose `start` is page-aligned to `page_size()` and whose length is
/// an exact multiple of it.
pub trait PageBackend {
    /// Page granularity in bytes.  Constant for the lifetime of the
    /// backend (typically 4 KiB on `x86_64`).
    fn page_size(&self) -> u64;

    /// Reserve a contiguous region whose length is the smallest
    /// multiple of `page_size()` that is `>= bytes`.  Returns
    /// [`Y4Error::NoMemory`] when no such region is available.
    ///
    /// # Errors
    /// Returns [`Y4Error::NoMemory`] if the backend has no remaining
    /// capacity, or [`Y4Error::InvalidArg`] if `bytes == 0`.
    fn reserve(&mut self, bytes: u64) -> Result<Range, Y4Error>;

    /// Release a region previously returned by [`Self::reserve`].
    ///
    /// # Errors
    /// Returns [`Y4Error::BadCap`] if `range` was not vended by this
    /// backend or has already been released.
    fn release(&mut self, range: Range) -> Result<(), Y4Error>;
}

/// In-memory mock backend used by unit tests.  Vends consecutive
/// page-aligned ranges starting from a configurable base.  Tracks
/// outstanding reservations so `release` can detect double-free.
#[derive(Debug)]
pub struct MockPageBackend {
    base: VAddr,
    page: u64,
    capacity: u64,
    next: VAddr,
    live: heapless::Vec<Range, 64>,
}

impl MockPageBackend {
    /// Construct a backend with the given base address, page size,
    /// and total capacity in bytes.  `base` and `capacity` are
    /// treated as already-aligned to `page` (caller's responsibility).
    #[must_use]
    pub fn new(base: VAddr, page: u64, capacity: u64) -> Self {
        Self {
            base,
            page,
            capacity,
            next: base,
            live: heapless::Vec::new(),
        }
    }

    /// Outstanding live reservations (test introspection only).
    #[must_use]
    pub fn live_count(&self) -> usize {
        self.live.len()
    }
}

impl PageBackend for MockPageBackend {
    fn page_size(&self) -> u64 {
        self.page
    }

    fn reserve(&mut self, bytes: u64) -> Result<Range, Y4Error> {
        if bytes == 0 {
            return Err(Y4Error::InvalidArg);
        }
        let rounded = bytes.div_ceil(self.page) * self.page;
        let end = self.next.checked_add(rounded).ok_or(Y4Error::NoMemory)?;
        if end - self.base > self.capacity {
            return Err(Y4Error::NoMemory);
        }
        let range = Range::new(self.next, end).ok_or(Y4Error::InvalidArg)?;
        self.live.push(range).map_err(|_| Y4Error::NoMemory)?;
        self.next = end;
        Ok(range)
    }

    fn release(&mut self, range: Range) -> Result<(), Y4Error> {
        let pos = self
            .live
            .iter()
            .position(|r| *r == range)
            .ok_or(Y4Error::BadCap)?;
        self.live.swap_remove(pos);
        Ok(())
    }
}
