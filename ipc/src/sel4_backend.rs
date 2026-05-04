// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! `Sel4Backend` — abstraction over the seL4 syscalls the IPC subsystem
//! invokes.
//!
//! Trusted boundary (C4): `seL4_Send / Recv / Call / Reply`,
//! `seL4_Signal / Wait`, `seL4_CNode_Copy / Move / Mint`,
//! `seL4_Untyped_Retype` (rare — endpoint cap birth), `seL4_TCB_*`
//! (rare — LWKT thread migration).  This trait collapses them into the
//! handful of operations the upper layers actually invoke.

use crate::error::Y4Error;
use crate::types::{Endpoint, Msg};

/// seL4 surface required by the IPC subsystem.
pub trait Sel4Backend {
    /// Mint a fresh endpoint capability.  Used by `scheme_open` and
    /// by msgport to bootstrap a new port.
    ///
    /// # Errors
    /// Returns [`Y4Error::NoMemory`] when no untyped capacity remains.
    fn mint_endpoint(&mut self) -> Result<Endpoint, Y4Error>;

    /// Revoke an endpoint capability.  Idempotent on a non-live cap.
    ///
    /// # Errors
    /// Returns [`Y4Error::BadCap`] if the cap was never minted.
    fn revoke_endpoint(&mut self, ep: Endpoint) -> Result<(), Y4Error>;

    /// Send a message to `ep`.  Caller retains ownership of the `msg`
    /// struct (LWKT pattern).
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] if `ep` is invalid; [`Y4Error::Timeout`] if
    /// no receiver pulls within the backend's bound.
    fn send(&mut self, ep: Endpoint, msg: &Msg) -> Result<(), Y4Error>;

    /// Receive a message from `ep`.  Returns `None` on timeout when
    /// the backend supports non-blocking receive.
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] if `ep` is invalid.
    fn recv(&mut self, ep: Endpoint) -> Result<Option<Msg>, Y4Error>;
}

/// In-memory mock backend.  Maintains FIFO queues per endpoint.  Used
/// by unit tests and bring-up of upper layers without seL4 in the loop.
#[derive(Debug, Default)]
pub struct MockSel4Backend {
    next_ep: Endpoint,
    queues: heapless::LinearMap<Endpoint, heapless::Vec<Msg, 16>, 16>,
}

impl MockSel4Backend {
    /// Empty backend.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of currently live endpoints (test introspection only).
    #[must_use]
    pub fn endpoint_count(&self) -> usize {
        self.queues.len()
    }
}

impl Sel4Backend for MockSel4Backend {
    fn mint_endpoint(&mut self) -> Result<Endpoint, Y4Error> {
        let ep = self.next_ep;
        self.next_ep = self.next_ep.checked_add(1).ok_or(Y4Error::NoMemory)?;
        self.queues
            .insert(ep, heapless::Vec::new())
            .map_err(|_| Y4Error::NoMemory)?;
        Ok(ep)
    }

    fn revoke_endpoint(&mut self, ep: Endpoint) -> Result<(), Y4Error> {
        self.queues.remove(&ep).ok_or(Y4Error::BadCap).map(|_| ())
    }

    fn send(&mut self, ep: Endpoint, msg: &Msg) -> Result<(), Y4Error> {
        let q = self.queues.get_mut(&ep).ok_or(Y4Error::BadCap)?;
        q.push(*msg).map_err(|_| Y4Error::Timeout)
    }

    fn recv(&mut self, ep: Endpoint) -> Result<Option<Msg>, Y4Error> {
        let q = self.queues.get_mut(&ep).ok_or(Y4Error::BadCap)?;
        if q.is_empty() {
            Ok(None)
        } else {
            // FIFO: drain from the front.
            let m = q.remove(0);
            Ok(Some(m))
        }
    }
}
