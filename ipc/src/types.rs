// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! IPC types — Rust mirror of `proofs/verus/src/ipc/state.rs`.

/// Endpoint capability — opaque index into the Y4 cap-table.
pub type Endpoint = u32;

/// Scheme handle vended by `scheme_open`.
pub type HandleId = u32;

/// Message id within the msgport subsystem.
pub type MsgId = u32;

/// Thread id (uniprocessor for now; SMP-first per C2 once kernel/ lands).
pub type ThreadId = u32;

/// Caller-supplied priority for a message.
pub type Priority = u8;

/// LWKT message — caller-allocated, dispatched by [`crate::Msgport`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Msg {
    /// Stable identifier the caller assigns.
    pub id: MsgId,
    /// Originating thread (used by `m3` abort-only-by-owner).
    pub origin: ThreadId,
    /// Endpoint cap to deliver against.
    pub target: Endpoint,
    /// Where replies arrive.
    pub reply_to: Endpoint,
    /// Caller priority.
    pub priority: Priority,
    /// Opaque payload.  Real implementations carry an `MR_LEN`-bounded
    /// inline buffer; spec uses a single `u64` for now.
    pub payload: u64,
}

/// Lifecycle states from `proofs/verus/src/ipc/state.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgState {
    /// Queued, not yet observed by a receiver.
    Pending,
    /// Dequeued by a receiver, awaiting reply.
    Delivered,
    /// Replied — caller has reaped it.
    Replied,
    /// Aborted by the originating thread.
    Aborted,
    /// Exceeded the caller-supplied deadline.
    TimedOut,
}

/// Scheme handle with everything the dispatcher needs to route a verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handle {
    /// Stable id within the scheme registry.
    pub id: HandleId,
    /// Owning thread (SC3 verb-dispatch-in-caller-context).
    pub owner: ThreadId,
    /// Underlying endpoint cap (K3 handle ↔ endpoint bijection).
    pub endpoint: Endpoint,
}
