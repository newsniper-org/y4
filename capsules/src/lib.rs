// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Y4 capsule subsystem.
//!
//! Tock-style isolated drivers — each capsule holds a typed capability
//! set granted at boot and may not mint new capabilities.  This first
//! crate ships:
//!
//! - **Public types** mirroring `proofs/verus/src/capsules/state.rs`
//!   ([`Capsule`], [`CapToken`], [`ResourceKind`], `CapsulesState`).
//! - **Isolation invariants** (C1 token-unique-owner, C2 no-mint, C3
//!   resource-disjoint-or-share) as runtime predicates +
//!   [`CapsulesState::well_formed`].
//! - **`ConfigSpace` trait** abstracting `PCIe` (bus, dev, fn) config
//!   reads — the only side-channel a capsule needs from the host
//!   chipset.  `MockConfigSpace` drives the unit tests.
//! - **`PcieEnumerator`** — the first real capsule, satisfying P1
//!   determinism, P2 unique addresses, P3 cap-required.
//!
//! HIU capsules are intentionally absent until `docs/hiu_abi.md` is
//! v1.0 frozen (per `MEMORY/y4_basics.md`).

#![no_std]
#![allow(clippy::module_name_repetitions)]

pub mod config_space;
pub mod error;
pub mod isolation;
pub mod pcie;
pub mod types;

pub use config_space::{ConfigSpace, MockConfigSpace, MockDevice};
pub use error::Y4Error;
pub use isolation::{CapsulesState, c1_holds, c2_holds, c3_holds};
pub use pcie::{PcieDevice, PcieEnumerator};
pub use types::{CapToken, CapTokenId, Capsule, CapsuleId, ResourceId, ResourceKind};
