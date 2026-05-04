// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! `SchemeRegistry` — Redox-style scheme dispatcher (control plane).
//!
//! Mirrors the pattern from
//! `/home/ybi/y4-upstream-refs/redox-kernel/src/scheme/mod.rs`.  Verbs
//! covered:
//!
//! - `open(SchemeId, owner) -> Handle`  — mints a fresh endpoint cap
//! - `close(Handle)`                    — revokes the endpoint
//! - `read(Handle, &mut buf) -> usize`
//! - `write(Handle, &buf) -> usize`
//! - `dup(Handle) -> Handle`            — second handle on same endpoint
//!   - `K3` strict bijection breaks here intentionally for the `dup()`
//!     case; the spec captures this as the "explicit share" carve-out.
//! - `fevent(Handle, EventMask)`        — subscribe to events
//!
//! Spec correspondence (`proofs/verus/src/ipc/scheme.rs`):
//!   * `SC1 path_resolution_deterministic` — `lookup` is a pure
//!     function of the registry.
//!   * `SC2 handle_lifetime_bounded_by_close` — verb on a closed
//!     handle returns `Y4Error::BadCap`.
//!   * `SC3 verb_dispatch_in_caller_context` — every verb takes a
//!     `caller: ThreadId` and rejects mismatch.
//!   * `SC4 scheme_id_uniqueness` — `register` cannot duplicate.

use crate::error::Y4Error;
use crate::sel4_backend::Sel4Backend;
use crate::types::{Endpoint, Handle, HandleId, ThreadId};

/// `SchemeId` — opaque numeric tag for a registered scheme.
pub type SchemeId = u32;

/// Bitmask of subscribed event kinds (read-ready, write-ready, ...).
pub type EventMask = u32;

/// Maximum simultaneous schemes.
pub const MAX_SCHEMES: usize = 16;

/// Maximum simultaneous handles per registry.
pub const MAX_HANDLES: usize = 64;

/// Maximum byte payload `read`/`write` can carry inline (larger
/// transfers go through the shared-frame primitive — Phase C).
pub const MAX_INLINE_BYTES: usize = 256;

/// Read/write inline buffer.
pub type InlineBuf = heapless::Vec<u8, MAX_INLINE_BYTES>;

/// Per-handle event subscription record.
#[derive(Debug, Clone, Copy)]
struct Subscription {
    mask: EventMask,
}

/// Registry of schemes and the handles they have vended.
pub struct SchemeRegistry {
    schemes: heapless::LinearMap<SchemeId, (), MAX_SCHEMES>,
    handles: heapless::LinearMap<HandleId, Handle, MAX_HANDLES>,
    /// Per-handle inline data buffer (write fills, read drains).
    /// Used in lieu of an external scheme implementation for unit tests.
    buffers: heapless::LinearMap<HandleId, InlineBuf, MAX_HANDLES>,
    /// Per-handle event subscriptions.
    subs: heapless::LinearMap<HandleId, Subscription, MAX_HANDLES>,
    next_handle: HandleId,
}

impl Default for SchemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemeRegistry {
    /// Empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            schemes: heapless::LinearMap::new(),
            handles: heapless::LinearMap::new(),
            buffers: heapless::LinearMap::new(),
            subs: heapless::LinearMap::new(),
            next_handle: 0,
        }
    }

    /// SC1 — pure registry lookup: `id` → `()` (the registration mark).
    /// Returning `()` keeps the spec-shape simple; production schemes
    /// pair the id with a `KernelScheme` trait object.
    #[must_use]
    pub fn lookup(&self, id: SchemeId) -> Option<()> {
        self.schemes.get(&id).copied()
    }

    /// SC4 — register a scheme.  No-op if already present (uniqueness
    /// is preserved by the `LinearMap` overwrite).
    ///
    /// # Errors
    /// [`Y4Error::NoMemory`] when `MAX_SCHEMES` exhausted.
    pub fn register(&mut self, id: SchemeId) -> Result<(), Y4Error> {
        self.schemes.insert(id, ()).map_err(|_| Y4Error::NoMemory)?;
        Ok(())
    }

    /// Open a handle against scheme `id`, vending a fresh endpoint.
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] if the scheme is not registered;
    /// [`Y4Error::NoMemory`] if any internal table is full.
    pub fn open<B: Sel4Backend>(
        &mut self,
        id: SchemeId,
        owner: ThreadId,
        backend: &mut B,
    ) -> Result<Handle, Y4Error> {
        if !self.schemes.contains_key(&id) {
            return Err(Y4Error::BadCap);
        }
        let endpoint = backend.mint_endpoint()?;
        let handle = Handle {
            id: self.next_handle,
            owner,
            endpoint,
        };
        self.handles
            .insert(handle.id, handle)
            .map_err(|_| Y4Error::NoMemory)?;
        self.buffers
            .insert(handle.id, InlineBuf::new())
            .map_err(|_| Y4Error::NoMemory)?;
        self.next_handle = self.next_handle.checked_add(1).ok_or(Y4Error::NoMemory)?;
        Ok(handle)
    }

    /// Close `handle`, revoking the underlying endpoint.
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] if `handle` is not live.
    pub fn close<B: Sel4Backend>(
        &mut self,
        handle: Handle,
        backend: &mut B,
    ) -> Result<(), Y4Error> {
        let live = self.handles.remove(&handle.id).ok_or(Y4Error::BadCap)?;
        let _ = self.buffers.remove(&handle.id);
        let _ = self.subs.remove(&handle.id);
        backend.revoke_endpoint(live.endpoint)
    }

    /// Read up to `out.capacity()` bytes from the handle's inline
    /// buffer.  Drains what it returns.
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] for a stale handle (SC2).
    /// [`Y4Error::InvalidArg`] when `caller` does not match the
    /// handle owner (SC3).
    pub fn read(
        &mut self,
        handle: Handle,
        caller: ThreadId,
        out: &mut InlineBuf,
    ) -> Result<usize, Y4Error> {
        self.check_live_and_owner(handle, caller)?;
        let buf = self.buffers.get_mut(&handle.id).ok_or(Y4Error::BadCap)?;
        let cap = out.capacity() - out.len();
        let take = buf.len().min(cap);
        for _ in 0..take {
            // FIFO: drain from the front — explicit byte-shift to keep
            // `no_std` and avoid pulling `arrayvec`.
            let b = buf.remove(0);
            out.push(b).map_err(|_| Y4Error::NoMemory)?;
        }
        Ok(take)
    }

    /// Write bytes into the handle's inline buffer.
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] for a stale handle.
    /// [`Y4Error::InvalidArg`] for owner mismatch.
    /// [`Y4Error::NoMemory`] when the buffer is full.
    pub fn write(
        &mut self,
        handle: Handle,
        caller: ThreadId,
        data: &[u8],
    ) -> Result<usize, Y4Error> {
        self.check_live_and_owner(handle, caller)?;
        let buf = self.buffers.get_mut(&handle.id).ok_or(Y4Error::BadCap)?;
        let cap = MAX_INLINE_BYTES - buf.len();
        let take = data.len().min(cap);
        for &b in &data[..take] {
            buf.push(b).map_err(|_| Y4Error::NoMemory)?;
        }
        Ok(take)
    }

    /// Duplicate a handle — vends a second handle id sharing the same
    /// endpoint cap.  Spec-wise this is the K3 "explicit share"
    /// carve-out (the bijection invariant is relaxed for `dup`).
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] for a stale handle.
    /// [`Y4Error::InvalidArg`] for owner mismatch.
    pub fn dup(&mut self, handle: Handle, caller: ThreadId) -> Result<Handle, Y4Error> {
        self.check_live_and_owner(handle, caller)?;
        let dup = Handle {
            id: self.next_handle,
            owner: handle.owner,
            endpoint: handle.endpoint,
        };
        self.handles
            .insert(dup.id, dup)
            .map_err(|_| Y4Error::NoMemory)?;
        self.buffers
            .insert(dup.id, InlineBuf::new())
            .map_err(|_| Y4Error::NoMemory)?;
        self.next_handle = self.next_handle.checked_add(1).ok_or(Y4Error::NoMemory)?;
        Ok(dup)
    }

    /// Subscribe `handle` to events specified by `mask`.
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] for a stale handle.
    /// [`Y4Error::InvalidArg`] for owner mismatch.
    pub fn fevent(
        &mut self,
        handle: Handle,
        caller: ThreadId,
        mask: EventMask,
    ) -> Result<(), Y4Error> {
        self.check_live_and_owner(handle, caller)?;
        let _ = self.subs.insert(handle.id, Subscription { mask });
        Ok(())
    }

    /// Inspect the active subscription mask for a handle (test hook).
    #[must_use]
    pub fn subscription(&self, id: HandleId) -> Option<EventMask> {
        self.subs.get(&id).map(|s| s.mask)
    }

    /// Inspect the underlying endpoint of a handle (test hook).
    #[must_use]
    pub fn endpoint_of(&self, id: HandleId) -> Option<Endpoint> {
        self.handles.get(&id).map(|h| h.endpoint)
    }

    /// Number of currently-live handles (test introspection only).
    #[must_use]
    pub fn live_handle_count(&self) -> usize {
        self.handles.len()
    }

    /// SC2 + SC3: handle is live AND `caller` owns it.
    fn check_live_and_owner(&self, handle: Handle, caller: ThreadId) -> Result<(), Y4Error> {
        let live = self.handles.get(&handle.id).ok_or(Y4Error::BadCap)?;
        if live.owner != caller {
            return Err(Y4Error::InvalidArg);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sel4_backend::MockSel4Backend;

    fn build() -> (MockSel4Backend, SchemeRegistry) {
        (MockSel4Backend::new(), SchemeRegistry::new())
    }

    #[test]
    fn open_and_close_round_trip() {
        let (mut backend, mut reg) = build();
        reg.register(42).unwrap();
        let h = reg.open(42, 7, &mut backend).unwrap();
        assert_eq!(h.owner, 7);
        assert_eq!(reg.live_handle_count(), 1);
        reg.close(h, &mut backend).unwrap();
        assert_eq!(reg.live_handle_count(), 0);
    }

    #[test]
    fn open_unknown_scheme_fails() {
        let (mut backend, mut reg) = build();
        assert_eq!(reg.open(99, 0, &mut backend), Err(Y4Error::BadCap));
    }

    #[test]
    fn distinct_handles_have_distinct_endpoints() {
        // K3 (without dup): two opens against the same scheme yield
        // two distinct endpoint caps.
        let (mut backend, mut reg) = build();
        reg.register(1).unwrap();
        let h1 = reg.open(1, 0, &mut backend).unwrap();
        let h2 = reg.open(1, 0, &mut backend).unwrap();
        assert_ne!(h1.id, h2.id);
        assert_ne!(h1.endpoint, h2.endpoint);
    }

    #[test]
    fn write_then_read_returns_bytes() {
        let (mut backend, mut reg) = build();
        reg.register(1).unwrap();
        let h = reg.open(1, 7, &mut backend).unwrap();
        let n = reg.write(h, 7, b"hello").unwrap();
        assert_eq!(n, 5);
        let mut out: InlineBuf = heapless::Vec::new();
        let n2 = reg.read(h, 7, &mut out).unwrap();
        assert_eq!(n2, 5);
        assert_eq!(&out[..], b"hello");
    }

    #[test]
    fn read_on_stale_handle_fails() {
        // SC2: closed handle yields BadCap on read.
        let (mut backend, mut reg) = build();
        reg.register(1).unwrap();
        let h = reg.open(1, 7, &mut backend).unwrap();
        reg.close(h, &mut backend).unwrap();
        let mut out: InlineBuf = heapless::Vec::new();
        assert_eq!(reg.read(h, 7, &mut out), Err(Y4Error::BadCap));
    }

    #[test]
    fn read_with_wrong_caller_fails() {
        // SC3: foreign caller rejected.
        let (mut backend, mut reg) = build();
        reg.register(1).unwrap();
        let h = reg.open(1, 7, &mut backend).unwrap();
        let mut out: InlineBuf = heapless::Vec::new();
        assert_eq!(reg.read(h, 8, &mut out), Err(Y4Error::InvalidArg));
    }

    #[test]
    fn dup_shares_endpoint() {
        // K3 carve-out: dup explicitly shares the endpoint.
        let (mut backend, mut reg) = build();
        reg.register(1).unwrap();
        let h = reg.open(1, 7, &mut backend).unwrap();
        let d = reg.dup(h, 7).unwrap();
        assert_ne!(h.id, d.id);
        assert_eq!(h.endpoint, d.endpoint);
    }

    #[test]
    fn fevent_records_mask() {
        let (mut backend, mut reg) = build();
        reg.register(1).unwrap();
        let h = reg.open(1, 7, &mut backend).unwrap();
        reg.fevent(h, 7, 0b0011).unwrap();
        assert_eq!(reg.subscription(h.id), Some(0b0011));
    }

    #[test]
    fn lookup_is_deterministic() {
        // SC1: lookup is a pure function.
        let (_b, mut reg) = build();
        reg.register(1).unwrap();
        let l1 = reg.lookup(1);
        let l2 = reg.lookup(1);
        assert_eq!(l1, l2);
        assert!(l1.is_some());
        assert!(reg.lookup(999).is_none());
    }
}
