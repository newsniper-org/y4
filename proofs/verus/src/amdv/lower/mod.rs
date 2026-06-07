// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Cluster 1 — amdv lower (12 AV).
//!
//! Per `av-proof-body-tracker.md` §2 + `amdv_safety.md` §5:
//!
//! | AV | file | source S |
//! |---|---|---|
//! | AV1 | `intercept_floor.rs` | S2 (16-bit mandatory mask) ← R7.11 first emit |
//! | AV3 | `deadline.rs` | S4 |
//! | AV6 | `gif.rs` (microkernel 측 본체) | S7 |
//! | AV7 | `tsc.rs` | S8 |
//! | AV8 | `nested.rs` | S9 |
//! | AV11+12+13 | `audit.rs` (shared) | S12 / S12.8 / S12.5 |
//! | AV16 | `vmcb_whitelist.rs` | §4 |
//! | AV18 | `cluster_dep.rs` | §2.4 / §8.2 |
//! | AV19 | `boundary.rs` | §8.3 |
//! | AV20 | `dispatch.rs` | §8.1 |
//!
//! 의존 graph: AV6 → AV1 (verus_to_isabelle.md line 138).  topological
//! order = AV1 → AV6 → 나머지 (lu-par DAG-aware, R3.5).

pub mod intercept_floor; // AV1 — R7.11 first emit milestone
// pub mod deadline;     // AV3
// pub mod gif;          // AV6 (microkernel 측 본체)
// pub mod tsc;          // AV7
// pub mod nested;       // AV8
// pub mod audit;        // AV11+12+13 (shared)
// pub mod vmcb_whitelist; // AV16
// pub mod cluster_dep;  // AV18
// pub mod boundary;     // AV19
// pub mod dispatch;     // AV20
