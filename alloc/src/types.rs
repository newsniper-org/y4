// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Allocator types — Rust mirror of `proofs/verus/src/alloc/state.rs`.
//!
//! The two definitions must stay in lock-step.  When refining the
//! implementation against the Verus spec (Phase B step 4 onwards),
//! these structs are the bridge.

/// Virtual address used by the allocator.
pub type VAddr = u64;

/// Half-open virtual address range `[start, end)`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Range {
    /// Inclusive lower bound.
    pub start: VAddr,
    /// Exclusive upper bound.
    pub end: VAddr,
}

impl Range {
    /// Create a range, returning `None` if `end <= start`.
    #[must_use]
    pub fn new(start: VAddr, end: VAddr) -> Option<Self> {
        if start < end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// `addr` falls inside the half-open range.
    #[must_use]
    pub fn contains(&self, addr: VAddr) -> bool {
        self.start <= addr && addr < self.end
    }

    /// Length in bytes.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.end - self.start
    }

    /// `true` when the range is empty (length zero).  Always `false`
    /// for ranges constructed via [`Self::new`], which rejects empty
    /// inputs; provided to satisfy clippy's `len_without_is_empty`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// `true` when this range and `other` share no address.
    #[must_use]
    pub fn disjoint(&self, other: &Self) -> bool {
        self.end <= other.start || other.end <= self.start
    }
}

/// Allocation request layout: size + power-of-two alignment.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Layout {
    /// Number of bytes the allocation must cover.
    pub size: u64,
    /// Alignment requirement (must be a power of two).
    pub align: u64,
}

impl Layout {
    /// Construct a layout, validating that `align` is a power of two
    /// and `size > 0`.  Returns `None` otherwise.
    #[must_use]
    pub fn new(size: u64, align: u64) -> Option<Self> {
        if size > 0 && align > 0 && align.is_power_of_two() {
            Some(Self { size, align })
        } else {
            None
        }
    }
}

/// One live allocation.  `is_guarded` reflects scudo's guard-page
/// installation (B4 in the spec).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Allocation {
    /// Virtual address range covered.
    pub range: Range,
    /// Layout this allocation satisfies.
    pub layout: Layout,
    /// NUMA node hint.
    pub numa: u32,
    /// Whether the allocation has guard pages installed (B4).
    pub is_guarded: bool,
}

impl Allocation {
    /// `true` if `range.start` honours `layout.align`.
    #[must_use]
    pub fn aligned(&self) -> bool {
        self.range.start % self.layout.align == 0
    }
}
