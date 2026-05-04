// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Y4 Verus specifications.
//!
//! Module layout (Phase B step 3 onwards):
//!
//! ```text
//! src/
//! ├── lib.rs        ← this file (declares the module hierarchy)
//! ├── error.rs      ← common Y4Error enum (C3)
//! ├── ipc/          ← scheme + LWKT msgport hybrid (I-P2)
//! │   ├── mod.rs
//! │   ├── scheme.rs
//! │   ├── msgport.rs
//! │   ├── consistency.rs
//! │   └── README.md
//! ├── alloc/        ← DragonFly SLAB + LLVM scudo (A-P1)
//! │   ├── mod.rs
//! │   ├── state.rs
//! │   ├── slab.rs
//! │   ├── scudo.rs
//! │   ├── boundary.rs
//! │   └── README.md
//! ├── capsules/     ← Tock-style isolation + PCIe/USB/CXL (Phase B step 4)
//! │   ├── mod.rs
//! │   ├── state.rs
//! │   ├── isolation.rs
//! │   ├── pcie.rs
//! │   ├── usb.rs
//! │   ├── cxl.rs
//! │   └── README.md
//! ├── kernel/       ← seL4 root-task + cap bootstrap (Phase B step 5+)
//! └── lease/        ← LeaseCap I1–I6 (deferred — needs hiu_abi v1.0)
//! ```
//!
//! All `proof fn ... ensures true {}` are placeholders that establish
//! the invariant catalog as machine-checked names.  Bodies are filled
//! in by subsequent PRs (formal-first per CLAUDE.md §6.6).

use vstd::prelude::*;

pub mod error;
pub mod alloc;
pub mod ipc;
pub mod capsules;

verus! {

/// Trivial smoke proof retained from the harness scaffolding.
proof fn placeholder_trivial()
    ensures 1nat + 1nat == 2nat
{
}

} // verus!
