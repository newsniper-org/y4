// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Redox scheme verb specifications.
//!
//! Reference (read-only):
//!   `/home/ybi/y4-upstream-refs/redox-kernel/src/scheme/`
//!
//! Y4 ports the scheme dispatch model — `RwLock<HashMap<SchemeId,
//! KernelScheme>>` + per-verb trait dispatch — but adapts it to seL4
//! capability invocation underneath (Redox uses its own microkernel
//! syscall model).
//!
//! Scheme verbs covered: `open`, `read`, `write`, `close`, plus the
//! Redox-extended `dup`, `fevent`, `fmap`, `fpath`, `fstat`.
//!
//! v0 invariant catalog (placeholders, bodies deferred):
//!   SC1 — path_resolution_deterministic
//!   SC2 — handle_lifetime_bounded_by_close
//!   SC3 — verb_dispatch_in_caller_context
//!   SC4 — scheme_id_uniqueness

use vstd::prelude::*;

verus! {

/// **SC1 — path resolution determinism.**
///
/// `scheme_lookup(path)` returning `(scheme_id, sub_path)` is a pure
/// function of the global scheme registry state at lookup time.
/// Same path + same registry ⇒ same `(scheme_id, sub_path)`.
pub proof fn path_resolution_deterministic()
    ensures true,
{
}

/// **SC2 — handle lifetime bounded by close.**
///
/// A handle returned by `open()` is valid until exactly one `close()`
/// is called on it.  Any `read/write/dup/...` on a closed handle
/// returns `Y4Error::BadCap`.
pub proof fn handle_lifetime_bounded_by_close()
    ensures true,
{
}

/// **SC3 — verb dispatch in caller context.**
///
/// Scheme verb handlers run on the caller's thread (no scheduler
/// hop), so cap checks see the caller's CSpace, not the scheme
/// implementer's.  Crucial for capability-based isolation.
pub proof fn verb_dispatch_in_caller_context()
    ensures true,
{
}

/// **SC4 — SchemeId uniqueness.**
///
/// At any moment, the scheme registry maps each `SchemeId` to at most
/// one `KernelScheme`.  Re-using an ID after `unregister` requires a
/// monotone bump (no recycling within an epoch).
pub proof fn scheme_id_uniqueness()
    ensures true,
{
}

} // verus!
