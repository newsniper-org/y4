// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! DragonFly LWKT msgport specifications.
//!
//! Reference (read-only):
//!   `/home/ybi/y4-upstream-refs/dragonfly/sys/kern/lwkt_msgport.c`
//!   `/home/ybi/y4-upstream-refs/dragonfly/sys/sys/msgport.h`
//!
//! LWKT 의 핵심 가치는 *per-CPU lock-free fast-path* + *caller-allocated
//! message* 패턴.  Y4 는 이를 데이터평면 fast-path 로 노출 (I-P2 hybrid).
//! 메시지 객체는 caller 가 alloc 에서 받아 ipc 에 넘김 (C1: ipc/alloc 독립).
//!
//! v0 invariant catalog (placeholders, bodies deferred):
//!   M1 — send_recv_pairing
//!   M2 — forward_transitivity
//!   M3 — abort_only_by_owner
//!   M4 — per_cpu_queue_isolation
//!   M5 — priority_inversion_avoidance   (I-P3: priority-inversion v0 IN)

use vstd::prelude::*;

verus! {

/// **M1 — send/recv pairing.**
///
/// For every `lwkt_sendmsg(port, msg)` that returns success, exactly
/// one matching `lwkt_waitport(port, ...)` returns `msg`.  No message
/// is delivered twice; no message is silently dropped.
pub proof fn send_recv_pairing()
    ensures true,
{
}

/// **M2 — forward transitivity.**
///
/// `lwkt_forwardmsg(port_b, msg)` invoked from inside `port_a`'s
/// handler is observationally equivalent to `lwkt_sendmsg(port_b,
/// msg)` directly from the original sender.  Forwarding chains do
/// not duplicate replies.
pub proof fn forward_transitivity()
    ensures true,
{
}

/// **M3 — abort only by owner.**
///
/// `lwkt_abortmsg(msg)` is a no-op unless the caller is the message's
/// originator (`msg.ms_reply_port`'s owner thread).  Prevents a
/// listening peer from cancelling messages it didn't send.
pub proof fn abort_only_by_owner()
    ensures true,
{
}

/// **M4 — per-CPU queue isolation.**
///
/// CPU `i`'s msgport queue and CPU `j`'s queue (i ≠ j) are disjoint
/// memory regions.  The lock-free push/pop on either queue cannot
/// race against the other.  (SMP-first per C2.)
pub proof fn per_cpu_queue_isolation()
    ensures true,
{
}

/// **M5 — priority-inversion avoidance (I-P3 v0 IN).**
///
/// A high-priority thread blocked on `lwkt_waitport` does not stall
/// indefinitely because a lower-priority thread holds an msgport
/// resource.  Mechanism (priority inheritance vs ceiling) is selected
/// in the implementation PR; the spec only states the liveness
/// property.
pub proof fn priority_inversion_avoidance()
    ensures true,
{
}

} // verus!
