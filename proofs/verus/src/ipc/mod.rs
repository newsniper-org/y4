// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Y4 IPC specifications — `Redox scheme` (제어평면) + `DragonFly LWKT
//! msgport` (데이터평면) hybrid (decision I-P2).
//!
//! Layered structure:
//!
//! ```text
//! callers (제어평면)            callers (데이터평면)
//!   │                              │
//!   ▼                              ▼
//! [scheme verbs]                [LWKT msgport]
//!   open/read/write/close          send/recv/forward/abort
//!   │                              │
//!   └──────────┬───────────────────┘
//!              ▼
//! seL4 kernel API:
//!   seL4_Send / Recv / Call / Reply (Endpoint)
//!   seL4_Signal / Wait               (Notification)
//!   seL4_CNode_Copy / Move / Mint    (cap shuffling)
//!   seL4_Untyped_Retype              (endpoint cap 생성, 드물게)
//!   seL4_TCB_*                       (LWKT 마이그레이션, 드물게)
//!                                    (C4 trusted boundary)
//! ```
//!
//! ipc 와 alloc 은 서로 독립 (C1) — 메시지 객체는 caller 가 alloc 에서
//! 받아 ipc 에 넘기는 패턴 (DragonFly LWKT의 caller-allocated msg 와 동일).
//!
//! See `proofs/verus/src/ipc/README.md` for the v0 invariant catalog.

use vstd::prelude::*;

pub mod scheme;
pub mod msgport;
pub mod consistency;

verus! {

/// Top-level IPC liveness invariant: every send eventually completes
/// (delivered, replied, aborted, or times out — never wedged forever).
///
/// ⚠ TODO Phase B step 3 — body intentionally empty, awaiting fairness
///       model selection (lockstep vs weak fairness vs scheduler-aware).
pub proof fn ipc_send_eventually_completes()
    ensures true,
{
}

} // verus!
