// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Y4 memory allocator.
//!
//! Architecture: `DragonFly` lock-free SLAB front-end + LLVM scudo backend
//! (decision A-P1).  This crate currently ships:
//!
//! - **Public types** mirroring `proofs/verus/src/alloc/state.rs`
//!   ([`Range`], [`Layout`], [`Allocation`]).
//! - **`PageBackend` trait** abstracting the seL4 page-management calls
//!   (`seL4_X86_Page_Map`, `seL4_X86_Page_Unmap`, `seL4_Untyped_Retype`)
//!   that the eventual real allocator invokes.  See [`PageBackend`].
//! - **A bump allocator** (`bump`) — the simplest correct backend that
//!   satisfies the no-overlap invariant.  Used by the unit tests and as
//!   a stepping-stone for the SLAB front-end work.
//!
//! Real SLAB (`DragonFly` port) and scudo binding land in dependent PRs.
//! Until then this crate is a runnable skeleton with a useful test
//! surface — `cargo test -p y4-alloc` exercises the bump allocator end-
//! to-end against a mock `PageBackend`.
//!
//! Refinement against the Verus spec at `proofs/verus/src/alloc/` is a
//! separate workflow (formal-first per `CLAUDE.md` §6.6) and lands as
//! `assume()` discharges in dependent PRs.

#![no_std]
#![allow(clippy::module_name_repetitions)]

pub mod bump;
pub mod error;
pub mod hardened;
pub mod integrated;
pub mod page_backend;
#[cfg(feature = "scudo")]
pub mod scudo_ffi;
pub mod slab;
pub mod types;

pub use error::Y4Error;
pub use page_backend::{MockPageBackend, PageBackend};
pub use types::{Allocation, Layout, Range};
