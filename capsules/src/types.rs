// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Capsule subsystem types — Rust mirror of
//! `proofs/verus/src/capsules/state.rs`.

/// Identifier for a capsule (Tock-style).
pub type CapsuleId = u16;

/// Identifier for a capability token granted to a capsule.
pub type CapTokenId = u32;

/// Identifier for a resource (MMIO range, IRQ line, DMA channel).
pub type ResourceId = u32;

/// Kind of resource a capability token gates access to.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ResourceKind {
    /// Memory-mapped IO range.
    Mmio,
    /// Interrupt line.
    Irq,
    /// DMA channel.
    DmaChannel,
    /// `PCIe` / USB / CXL bus enumeration permission.
    BusEnumerator,
}

/// One capability token.  Issued by the boot kernel into a single
/// capsule's `cap_set`; the capsule cannot mint further tokens.
#[derive(Debug, Clone, Copy)]
pub struct CapToken {
    /// The capsule that holds this token (C2 mint-blocker invariant).
    pub holder: CapsuleId,
    /// Resource the token authorises access to.
    pub resource: ResourceId,
    /// Kind of resource.
    pub kind: ResourceKind,
}

/// One capsule.  Holds a set of token ids granted at boot.
#[derive(Debug, Clone, Default)]
pub struct Capsule {
    /// Token ids this capsule holds.  Bound by `MAX_TOKENS_PER_CAPSULE`.
    pub cap_set: heapless::Vec<CapTokenId, 16>,
}

impl Capsule {
    /// Empty capsule — no tokens granted yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `true` when this capsule's `cap_set` contains `token`.
    #[must_use]
    pub fn holds(&self, token: CapTokenId) -> bool {
        self.cap_set.contains(&token)
    }
}
