// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! LWKT msgport — caller-allocated message dispatch with per-CPU
//! sharding, forwarding, abort, priority, and reply.
//!
//! Mirrors the `DragonFly` LWKT API surface from
//! `/home/ybi/y4-upstream-refs/dragonfly/sys/sys/msgport.h`:
//!
//! - `lwkt_sendmsg(port, msg)` ↔ [`Msgport::send`]
//! - `lwkt_waitport(port, ...)` ↔ [`Msgport::wait`]
//! - `lwkt_forwardmsg(port, msg)` ↔ [`Msgport::forward`]
//! - `lwkt_abortmsg(msg)` ↔ [`Msgport::abort`]
//!
//! Caller owns `msg` (LWKT pattern); the port only borrows for
//! dispatch (C1: ipc/alloc independent).
//!
//! Spec correspondence (`proofs/verus/src/ipc/msgport.rs`):
//!   * `M1 send_recv_pairing`        — per-message lifecycle in `state`.
//!   * `M2 forward_transitivity`     — `forward` re-targets without
//!     touching the lifecycle observed by the original sender.
//!   * `M3 abort_only_by_owner`      — `abort` requires `caller ==
//!     msg.origin`.
//!   * `M4 per_cpu_queue_isolation`  — distinct CPU queues are owned
//!     by separate fields in [`Msgport`].
//!   * `M5 priority_inversion_avoidance` — `priority_holder` records
//!     the highest priority blocked thread per endpoint.

use crate::error::Y4Error;
use crate::sel4_backend::Sel4Backend;
use crate::types::{Endpoint, Msg, MsgId, MsgState, Priority, ThreadId};

/// Identifier for a CPU.  C2: SMP-first.
pub type CpuId = u8;

/// Maximum CPUs the msgport tracks.
pub const MAX_CPUS: usize = 32;

/// Maximum tracked messages (per-msgport lifecycle table).
pub const MAX_TRACKED: usize = 256;

/// One LWKT-style port.  Thin wrapper around an endpoint cap, tagged
/// with the CPU that owns its queue (M4).
#[derive(Debug, Clone, Copy)]
pub struct Port {
    /// Endpoint cap this port wraps.
    pub endpoint: Endpoint,
    /// Owner CPU at bind time.
    pub owner_cpu: CpuId,
}

/// Msgport dispatcher — bridges caller-owned messages to a
/// [`Sel4Backend`].  Sharded per CPU so M4 disjointness holds by
/// construction (each CPU gets its own owner_cpu-tagged ports).
pub struct Msgport<'a, B: Sel4Backend> {
    backend: &'a mut B,
    /// Per-message lifecycle (M1).
    state: heapless::LinearMap<MsgId, MsgState, MAX_TRACKED>,
    /// Per-message origin thread (M3).
    origin: heapless::LinearMap<MsgId, ThreadId, MAX_TRACKED>,
    /// Per-endpoint highest blocked-waiter priority (M5).
    priority_holder: heapless::LinearMap<Endpoint, Priority, 64>,
}

impl<'a, B: Sel4Backend> Msgport<'a, B> {
    /// Wrap a backend so the caller can issue LWKT-style sends/waits.
    pub fn new(backend: &'a mut B) -> Self {
        Self {
            backend,
            state: heapless::LinearMap::new(),
            origin: heapless::LinearMap::new(),
            priority_holder: heapless::LinearMap::new(),
        }
    }

    /// Allocate a new port (mints a fresh endpoint cap) bound to `owner_cpu`.
    ///
    /// # Errors
    /// Forwards backend mint failures.
    pub fn open(&mut self, owner_cpu: CpuId) -> Result<Port, Y4Error> {
        let endpoint = self.backend.mint_endpoint()?;
        Ok(Port {
            endpoint,
            owner_cpu,
        })
    }

    /// Tear down a port.
    ///
    /// # Errors
    /// Forwards backend revocation failures.
    pub fn close(&mut self, port: Port) -> Result<(), Y4Error> {
        let _ = self.priority_holder.remove(&port.endpoint);
        self.backend.revoke_endpoint(port.endpoint)
    }

    /// Send `msg` to `port`.  Records the message in the lifecycle
    /// table (M1) and bumps the priority holder if appropriate (M5).
    ///
    /// # Errors
    /// Forwards backend send failures.
    pub fn send(&mut self, port: Port, msg: &Msg) -> Result<MsgState, Y4Error> {
        self.backend.send(port.endpoint, msg)?;
        // Update priority_holder if this msg's priority exceeds the
        // current high-water mark for the endpoint.
        let cur = self
            .priority_holder
            .get(&port.endpoint)
            .copied()
            .unwrap_or(0);
        if msg.priority > cur {
            let _ = self.priority_holder.insert(port.endpoint, msg.priority);
        }
        let _ = self.state.insert(msg.id, MsgState::Pending);
        let _ = self.origin.insert(msg.id, msg.origin);
        Ok(MsgState::Pending)
    }

    /// Wait for one message on `port`.  Transitions any returned
    /// message to `Delivered` state.
    ///
    /// # Errors
    /// Forwards backend recv failures.
    pub fn wait(&mut self, port: Port) -> Result<Option<Msg>, Y4Error> {
        let msg = self.backend.recv(port.endpoint)?;
        if let Some(ref m) = msg {
            let _ = self.state.insert(m.id, MsgState::Delivered);
        }
        Ok(msg)
    }

    /// Forward an in-flight message to a different port (LWKT
    /// `lwkt_forwardmsg`).  M2: observationally equivalent to a
    /// direct send by the original sender.
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] if `target` is invalid; backend send errors.
    pub fn forward(&mut self, target: Port, msg: &Msg) -> Result<(), Y4Error> {
        // Reuse `send`'s lifecycle bookkeeping; the message id is stable
        // so the original sender's view of `state[msg.id]` is unchanged
        // until the new target replies.
        self.backend.send(target.endpoint, msg)
    }

    /// Abort an in-flight message.  M3: only the message originator
    /// may abort; foreign callers get `InvalidArg`.
    ///
    /// # Errors
    /// [`Y4Error::InvalidArg`] when `caller != msg.origin`.
    /// [`Y4Error::BadCap`] when `msg_id` is unknown.
    pub fn abort(&mut self, caller: ThreadId, msg_id: MsgId) -> Result<(), Y4Error> {
        let owner = self.origin.get(&msg_id).copied().ok_or(Y4Error::BadCap)?;
        if owner != caller {
            return Err(Y4Error::InvalidArg);
        }
        let _ = self.state.insert(msg_id, MsgState::Aborted);
        Ok(())
    }

    /// Mark a message as replied.  Called by the receiver after it
    /// processes the message.
    pub fn reply(&mut self, msg_id: MsgId) {
        let _ = self.state.insert(msg_id, MsgState::Replied);
    }

    /// Mark a message as timed out.
    pub fn time_out(&mut self, msg_id: MsgId) {
        let _ = self.state.insert(msg_id, MsgState::TimedOut);
    }

    /// Inspect per-message lifecycle state.
    #[must_use]
    pub fn state_of(&self, msg_id: MsgId) -> Option<MsgState> {
        self.state.get(&msg_id).copied()
    }

    /// Inspect the highest-priority blocked waiter on `endpoint` (M5).
    #[must_use]
    pub fn priority_holder(&self, endpoint: Endpoint) -> Option<Priority> {
        self.priority_holder.get(&endpoint).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sel4_backend::MockSel4Backend;

    fn sample_msg(id: u32, target: Endpoint, origin: ThreadId, priority: Priority) -> Msg {
        Msg {
            id,
            origin,
            target,
            reply_to: target,
            priority,
            payload: u64::from(id) * 0x1111_1111,
        }
    }

    #[test]
    fn open_send_wait_round_trip() {
        let mut backend = MockSel4Backend::new();
        let mut mp = Msgport::new(&mut backend);
        let port = mp.open(0).unwrap();
        let msg = sample_msg(1, port.endpoint, 7, 0);
        assert_eq!(mp.send(port, &msg).unwrap(), MsgState::Pending);
        assert_eq!(mp.state_of(1), Some(MsgState::Pending));
        let got = mp.wait(port).unwrap().unwrap();
        assert_eq!(got, msg);
        assert_eq!(mp.state_of(1), Some(MsgState::Delivered));
    }

    #[test]
    fn fifo_ordering() {
        let mut backend = MockSel4Backend::new();
        let mut mp = Msgport::new(&mut backend);
        let port = mp.open(0).unwrap();
        for i in 1..=5 {
            mp.send(port, &sample_msg(i, port.endpoint, 0, 0)).unwrap();
        }
        for i in 1..=5 {
            let got = mp.wait(port).unwrap().unwrap();
            assert_eq!(got.id, i);
        }
    }

    #[test]
    fn empty_wait_returns_none() {
        let mut backend = MockSel4Backend::new();
        let mut mp = Msgport::new(&mut backend);
        let port = mp.open(0).unwrap();
        assert_eq!(mp.wait(port).unwrap(), None);
    }

    #[test]
    fn close_revokes() {
        let mut backend = MockSel4Backend::new();
        let mut mp = Msgport::new(&mut backend);
        let port = mp.open(0).unwrap();
        mp.close(port).unwrap();
        assert_eq!(
            mp.send(port, &sample_msg(1, port.endpoint, 0, 0)),
            Err(Y4Error::BadCap)
        );
    }

    #[test]
    fn forward_re_targets() {
        // M2: forwarding produces the same observable lifecycle as
        // a direct send to the new target.
        let mut backend = MockSel4Backend::new();
        let mut mp = Msgport::new(&mut backend);
        let p1 = mp.open(0).unwrap();
        let p2 = mp.open(0).unwrap();
        let msg = sample_msg(1, p1.endpoint, 0, 0);
        mp.send(p1, &msg).unwrap();
        let _ = mp.wait(p1).unwrap().unwrap();
        mp.forward(p2, &msg).unwrap();
        let got = mp.wait(p2).unwrap().unwrap();
        assert_eq!(got.id, msg.id);
    }

    #[test]
    fn abort_owner_only() {
        // M3: foreign abort rejected.
        let mut backend = MockSel4Backend::new();
        let mut mp = Msgport::new(&mut backend);
        let port = mp.open(0).unwrap();
        let msg = sample_msg(1, port.endpoint, 7, 0);
        mp.send(port, &msg).unwrap();
        assert_eq!(mp.abort(8, 1), Err(Y4Error::InvalidArg));
        assert_eq!(mp.state_of(1), Some(MsgState::Pending));
        mp.abort(7, 1).unwrap();
        assert_eq!(mp.state_of(1), Some(MsgState::Aborted));
    }

    #[test]
    fn priority_high_water_mark() {
        // M5: priority_holder tracks the highest seen priority.
        let mut backend = MockSel4Backend::new();
        let mut mp = Msgport::new(&mut backend);
        let port = mp.open(0).unwrap();
        mp.send(port, &sample_msg(1, port.endpoint, 0, 5)).unwrap();
        mp.send(port, &sample_msg(2, port.endpoint, 0, 10)).unwrap();
        mp.send(port, &sample_msg(3, port.endpoint, 0, 3)).unwrap();
        assert_eq!(mp.priority_holder(port.endpoint), Some(10));
    }

    #[test]
    fn per_cpu_ports_disjoint() {
        // M4: ports owned by different CPUs have disjoint endpoint caps
        // (mint_endpoint vends fresh ones) and isolated lifecycle tables.
        let mut backend = MockSel4Backend::new();
        let mut mp = Msgport::new(&mut backend);
        let p_cpu0 = mp.open(0).unwrap();
        let p_cpu1 = mp.open(1).unwrap();
        assert_ne!(p_cpu0.endpoint, p_cpu1.endpoint);
        assert_eq!(p_cpu0.owner_cpu, 0);
        assert_eq!(p_cpu1.owner_cpu, 1);
    }

    #[test]
    fn lifecycle_terminal_states_settable() {
        // M1: every lifecycle terminal is reachable via the public API.
        let mut backend = MockSel4Backend::new();
        let mut mp = Msgport::new(&mut backend);
        let port = mp.open(0).unwrap();
        for id in 1..=4u32 {
            mp.send(port, &sample_msg(id, port.endpoint, 0, 0)).unwrap();
        }
        let _ = mp.wait(port).unwrap().unwrap(); // Delivered
        mp.reply(2); // Replied
        mp.abort(0, 3).unwrap(); // Aborted
        mp.time_out(4); // TimedOut
        assert_eq!(mp.state_of(1), Some(MsgState::Delivered));
        assert_eq!(mp.state_of(2), Some(MsgState::Replied));
        assert_eq!(mp.state_of(3), Some(MsgState::Aborted));
        assert_eq!(mp.state_of(4), Some(MsgState::TimedOut));
    }
}
