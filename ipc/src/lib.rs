// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Y4 IPC subsystem.
//!
//! Architecture: `Redox` scheme verbs (제어평면) + `DragonFly` LWKT
//! msgport (데이터평면) hybrid (decision I-P2).  This crate ships:
//!
//! - **Public types** mirroring `proofs/verus/src/ipc/state.rs`
//!   ([`Msg`], [`MsgState`], [`Endpoint`], [`Handle`]).
//! - **`Sel4Backend` trait** abstracting the seL4 endpoint and
//!   notification syscalls.  `MockSel4Backend` for tests.
//! - **`Msgport`** — caller-allocated-message dispatch (LWKT pattern).
//! - **`SchemeRegistry`** — scheme-id → endpoint dispatcher.
//!
//! Refinement against the Verus spec at `proofs/verus/src/ipc/` is a
//! separate workflow and lands as `assume()` discharges in dependent
//! PRs.

#![no_std]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod msgport;
pub mod scheme;
pub mod sel4_backend;
pub mod types;

pub use error::Y4Error;
pub use msgport::{Msgport, Port};
pub use scheme::SchemeRegistry;
pub use sel4_backend::{MockSel4Backend, Sel4Backend};
pub use types::{Endpoint, Handle, Msg, MsgState};
