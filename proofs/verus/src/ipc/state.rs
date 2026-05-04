// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Abstract state model used by the ipc subsystem invariants.
//!
//! Spec-only. Real implementation lands in Phase B step 4 onwards and
//! must establish refinement against this model.

use vstd::prelude::*;

verus! {

/// Identifiers — all opaque `nat`s at the spec layer.
pub type CpuId      = nat;
pub type ThreadId   = nat;
pub type Priority   = nat;
pub type EndpointCap = nat;   // index into the Y4 cap-table
pub type SchemeId   = nat;
pub type HandleId   = nat;
pub type MsgId      = nat;
pub type Time       = nat;    // monotonic logical time

/// A single in-flight message.  Modelled as plain data; the payload
/// itself is opaque (`payload: nat`) since this v0 spec is the
/// send-by-copy form (zero-copy split into a separate primitive per I-P3).
pub struct Msg {
    pub id:           MsgId,
    pub origin:       ThreadId,
    pub target_port:  EndpointCap,
    pub reply_port:   EndpointCap,
    pub priority:     Priority,
    pub payload:      nat,
}

/// Lifecycle states of a message.  M1 / M5 / liveness invariants reason
/// over transitions on this enum.
#[derive(PartialEq, Eq, Structural)]
pub enum MsgState {
    Pending,    // queued, not yet observed by a receiver
    Delivered,  // dequeued, awaiting reply
    Replied,    // replied — caller has reaped it
    Aborted,    // aborted by owner
    TimedOut,   // exceeded caller-supplied deadline
}

// ------------------------------------------------------------------------
// LWKT msgport state
// ------------------------------------------------------------------------

/// Per-CPU msgport queue.  Lock-free ordering on `Seq` reflects the
/// FIFO pop semantic of `lwkt_waitport`.
pub struct MsgportQueue {
    pub messages: Seq<Msg>,
}

/// Whole msgport subsystem state.
pub struct MsgportState {
    /// One queue per CPU (per-CPU isolation per M4).
    pub queues: Map<CpuId, MsgportQueue>,
    /// Endpoint cap → owning CPU at the time of bind.
    pub binding: Map<EndpointCap, CpuId>,
    /// Per-message lifecycle state.
    pub state:   Map<MsgId, MsgState>,
    /// `priority_holder[ep]` records the highest-priority blocked
    /// thread on `ep` (PI / M5 mechanism).  `None` means no waiter.
    pub priority_holder: Map<EndpointCap, Priority>,
}

// ------------------------------------------------------------------------
// scheme registry state (Redox-style)
// ------------------------------------------------------------------------

/// A handle vended by `scheme_open()`.  Closed once a single
/// `scheme_close()` runs; closure is reflected in `live_handles`.
pub struct Handle {
    pub id:        HandleId,
    pub scheme:    SchemeId,
    pub endpoint:  EndpointCap,    // K3 bijection target
    pub owner:     ThreadId,
}

/// Scheme registry: SchemeId → SchemeRoot (where SchemeRoot's identity
/// is opaque).  `root_seq` increases on each register; close-then-
/// register reuses the *next* unused id, never recycling within an
/// epoch (SC4 semantics).
pub struct SchemeState {
    pub registry:    Map<SchemeId, EndpointCap>,
    pub live_handles: Set<HandleId>,
    pub handle_meta: Map<HandleId, Handle>,
    pub epoch:       nat,
}

// ------------------------------------------------------------------------
// Composed (ipc-public) view
// ------------------------------------------------------------------------

pub struct IpcState {
    pub mp:     MsgportState,
    pub scheme: SchemeState,
    pub now:    Time,
}

impl IpcState {
    /// Set of currently alive endpoint caps — present in the binding
    /// table or referenced by a live handle.
    pub open spec fn live_endpoints(self) -> Set<EndpointCap> {
        Set::<EndpointCap>::new(|ep: EndpointCap|
            self.mp.binding.dom().contains(ep)
        )
    }
}

} // verus!
