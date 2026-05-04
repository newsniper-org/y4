// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Abstract state model for the Tock-style capsule subsystem.
//!
//! Capsules are isolated driver units; each holds a typed capability
//! set granted at boot and may not mint new capabilities.  HIU is
//! intentionally absent from this set — HIU work is blocked until
//! `docs/hiu_abi.md` v1.0 frozen.

use vstd::prelude::*;

verus! {

/// Identifiers — opaque `nat`s.
pub type CapsuleId   = nat;
pub type CapTokenId  = nat;     // a Tock-style capability token
pub type ResourceId  = nat;     // an MMIO range, IRQ line, DMA channel ID
pub type BusAddr     = nat;     // PCIe (bus, device, function) packed
pub type IrqLine     = nat;

/// Kind of resource a capsule may hold.
#[derive(PartialEq, Eq, Structural)]
pub enum ResourceKind {
    Mmio,         // memory-mapped IO range
    Irq,          // interrupt line
    DmaChannel,
    BusEnumerator,// PCIe / USB / CXL bus-walking permission
}

/// Capability token granted to one capsule.
pub struct CapToken {
    pub id:       CapTokenId,
    pub holder:   CapsuleId,
    pub resource: ResourceId,
    pub kind:     ResourceKind,
}

/// One capsule's view of itself.  No mutable kernel state — capsules
/// receive their cap-set at boot.
pub struct Capsule {
    pub id:        CapsuleId,
    pub cap_set:   Set<CapTokenId>,
}

/// Whole capsule subsystem state.
pub struct CapsulesState {
    pub capsules:  Map<CapsuleId, Capsule>,
    pub tokens:    Map<CapTokenId, CapToken>,
}

impl CapsulesState {
    /// Every token recorded in `tokens` belongs to a capsule whose
    /// `cap_set` contains it.  Foundational well-formedness.
    pub open spec fn tokens_well_formed(self) -> bool {
        forall|t: CapTokenId| #![trigger self.tokens.dom().contains(t)]
            self.tokens.dom().contains(t)
                ==> self.capsules.dom().contains(self.tokens[t].holder)
                    && self.capsules[self.tokens[t].holder].cap_set.contains(t)
    }
}

// ------------------------------------------------------------------------
// PCIe enumeration model
// ------------------------------------------------------------------------

/// One PCIe device discovered by an enumeration scan.
pub struct PcieDevice {
    pub addr:     BusAddr,        // (bus, dev, fn) packed
    pub vendor:   nat,            // 16-bit vendor id
    pub device:   nat,            // 16-bit device id
    pub class:    nat,            // 24-bit class code
}

/// Result of one enumeration call.
pub struct PcieEnumResult {
    pub devices:  Seq<PcieDevice>,
}

// ------------------------------------------------------------------------
// USB / CXL — stubs (Phase B step 4 covers PCIe in depth; USB+CXL get
// scaffolding only, fleshed out when their first driver lands).
// ------------------------------------------------------------------------

/// USB device handle (stub).
pub struct UsbDevice { pub addr: nat, pub vid: nat, pub pid: nat }

/// CXL device handle (stub).
pub struct CxlDevice { pub addr: nat, pub region_id: nat }

} // verus!
