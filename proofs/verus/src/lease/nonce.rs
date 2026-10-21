// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! `FAULT_STATUS[12]` origin attribution (`docs/hiu_abi.md` §2.6,
//! `docs/lease_capability.md` I3).
//!
//! WaveTensor canonical `b16fc07` `e-defined.typ:345` (`0xEBC` MASK_REKEY)
//! fixes the domain of the nonce-reuse check: when a rekey that *started*
//! targets partition p, `[12]` rises iff some *other* partition q ≠ p whose
//! mask is valid carries a nonce equal to p's *new* nonce — p itself is not
//! compared.  `e-defined.typ:347` (`0xEC4` MASK_DUP_CAPTURE) reports the
//! same domain: `[7:0]` = p, `[15:8]` the bitmap of those q (bit p always
//! 0), `[16]` capture valid, and the invariant `[16]` ⇔ `[12]`.
//!
//! Y4 never reads nonces back (they are WO), so it attributes `[12]` by
//! comparing the capture against its own nonce ledger, in the order of
//! `docs/hiu_abi.md` §2.6:
//!
//! 1. capture malformed — `[16]` = 0, p not the partition Y4 rekeyed, empty
//!    bitmap, bit p set, or a bit at or above the real partition count
//!    (`0xFF8`) — is a device contract violation: device side.
//! 2. ledger collision — a ledger-mask-valid q ≠ p holds the nonce Y4 just
//!    wrote for p, or two ledger-mask-valid partitions share a nonce — is a
//!    Y4 intent defect (`NonceAllocator` / ledger).
//! 3. a captured q whose key and nonce Y4 did not write this boot is a Y4
//!    procedure defect (boot neutralisation, §2.5 step 4, skipped or not
//!    recorded).  The ledger's write boot is set by any key/nonce write —
//!    the §2.5 step 4 neutralisation write or a rekey (I4 step 3) — so a q
//!    neutralised at boot and never rekeyed counts as written.
//! 4. otherwise the device side (write path, storage or comparator) —
//!    including a captured q that the ledger holds mask-invalid: the
//!    device then claims a mask validity Y4 never granted.
//!
//! During a rekey the ledger holds p as (written nonce, this boot, mask
//! valid = false), so clause (ii) of step 2 alone never compares p's new
//! nonce.  Clause (i) closes that: without it a live q colliding with p's
//! new nonce would fall through to step 4 and be blamed on the device.

use vstd::prelude::*;

use crate::error::{FaultOrigin, Y4Error, fault_disposition};

verus! {

/// One partition's entry in Y4's nonce ledger (`docs/lease_capability.md`
/// I3 `(현재 기입 논스, 기입 부팅, 마스크 유효)`): the nonce Y4 last wrote
/// to the device, whether the ledger holds the mask valid, and whether the
/// write boot is this boot — Y4 wrote this partition's key and nonce
/// during this boot, by the §2.5 step 4 neutralisation write or by a rekey
/// (I4 step 3).
pub struct NonceEntry {
    pub nonce: nat,
    pub mask_valid: bool,
    pub written_this_boot: bool,
}

/// One read of `0xEC4` MASK_DUP_CAPTURE: `[16]` valid, `[7:0]` p, and the
/// `[15:8]` bitmap as the set of its set bit positions.
pub struct DupCapture {
    pub valid: bool,
    pub p: int,
    pub bitmap: Set<int>,
}

/// Step 1: the capture is what a conforming v1.0 device reports for a
/// rekey of `p` on a device with `parts` real partitions.
pub open spec fn capture_well_formed(cap: DupCapture, p: int, parts: nat) -> bool {
    &&& cap.valid
    &&& cap.p == p
    &&& cap.bitmap != Set::<int>::empty()
    &&& !cap.bitmap.contains(p)
    &&& forall|q: int| #[trigger] cap.bitmap.contains(q) ==> 0 <= q < parts
}

/// Clause (i) of step 2: a ledger-mask-valid q ≠ p holds `new_nonce`.
pub open spec fn new_nonce_collides(ledger: Seq<NonceEntry>, p: int, new_nonce: nat) -> bool {
    exists|q: int|
        0 <= q < ledger.len() && q != p && (#[trigger] ledger[q]).mask_valid
            && ledger[q].nonce == new_nonce
}

/// Clause (ii) of step 2: two ledger-mask-valid partitions share a nonce.
pub open spec fn valid_pair_collides(ledger: Seq<NonceEntry>) -> bool {
    exists|i: int, j: int|
        #![trigger ledger[i], ledger[j]]
        0 <= i < ledger.len() && 0 <= j < ledger.len() && i != j && ledger[i].mask_valid
            && ledger[j].mask_valid && ledger[i].nonce == ledger[j].nonce
}

/// Step 2: the ledger itself shows a nonce collision.
pub open spec fn ledger_collision(ledger: Seq<NonceEntry>, p: int, new_nonce: nat) -> bool {
    new_nonce_collides(ledger, p, new_nonce) || valid_pair_collides(ledger)
}

/// The ledger records `q`'s key and nonce as written by Y4 this boot.  A
/// position the ledger does not cover was never recorded, so it counts as
/// not written.
pub open spec fn written_in_ledger(ledger: Seq<NonceEntry>, q: int) -> bool {
    0 <= q < ledger.len() && ledger[q].written_this_boot
}

/// Step 3: some captured q had no key/nonce written by Y4 this boot.
pub open spec fn captured_not_written(ledger: Seq<NonceEntry>, cap: DupCapture) -> bool {
    exists|q: int| #[trigger] cap.bitmap.contains(q) && !written_in_ledger(ledger, q)
}

/// Origin of a `[12]` raised by Y4's rekey of `p` with `new_nonce` (the
/// value Y4 wrote to MASK_NONCE before MASK_REKEY), on a device with
/// `parts` real partitions (`0xFF8`).  `docs/hiu_abi.md` §2.6 steps 1–4.
pub open spec fn nonce_dup_origin(
    ledger: Seq<NonceEntry>,
    cap: DupCapture,
    p: int,
    new_nonce: nat,
    parts: nat,
) -> FaultOrigin {
    if !capture_well_formed(cap, p, parts) {
        FaultOrigin::DeviceSide
    } else if ledger_collision(ledger, p, new_nonce) {
        FaultOrigin::Y4Intent
    } else if captured_not_written(ledger, cap) {
        FaultOrigin::Y4Procedure
    } else {
        FaultOrigin::DeviceSide
    }
}

/// `[12]` is always attributable on a v1.0 device: the origin is one of
/// Y4 intent, Y4 procedure or device side, never `Unattributed`.
pub proof fn nonce_dup_origin_total(
    ledger: Seq<NonceEntry>,
    cap: DupCapture,
    p: int,
    new_nonce: nat,
    parts: nat,
)
    ensures
        ({
            let o = nonce_dup_origin(ledger, cap, p, new_nonce, parts);
            &&& o == FaultOrigin::Y4Intent || o == FaultOrigin::Y4Procedure
                || o == FaultOrigin::DeviceSide
            &&& o != FaultOrigin::Unattributed
        }),
{
}

/// A malformed capture — including `[12]` with `[16]` = 0, which the
/// canonical invariant `[16]` ⇔ `[12]` makes unreachable — is the device's.
pub proof fn malformed_capture_is_device_side(
    ledger: Seq<NonceEntry>,
    cap: DupCapture,
    p: int,
    new_nonce: nat,
    parts: nat,
)
    requires
        !capture_well_formed(cap, p, parts),
    ensures
        nonce_dup_origin(ledger, cap, p, new_nonce, parts) == FaultOrigin::DeviceSide,
        fault_disposition(nonce_dup_origin(ledger, cap, p, new_nonce, parts))
            == Y4Error::DeviceFault,
{
}

/// A collision visible in Y4's own ledger is never blamed on the device:
/// with a well-formed capture it is a Y4 intent defect and panics the
/// hypervisor.
pub proof fn y4_collision_never_blamed_on_device(
    ledger: Seq<NonceEntry>,
    cap: DupCapture,
    p: int,
    new_nonce: nat,
    parts: nat,
)
    requires
        capture_well_formed(cap, p, parts),
        ledger_collision(ledger, p, new_nonce),
    ensures
        nonce_dup_origin(ledger, cap, p, new_nonce, parts) == FaultOrigin::Y4Intent,
        fault_disposition(nonce_dup_origin(ledger, cap, p, new_nonce, parts))
            == Y4Error::LeaseInvariant,
{
}

/// The case clause (i) exists for: p is mask-invalid in the ledger while it
/// is being rekeyed and no two ledger-mask-valid partitions collide, yet a
/// live q ≠ p holds p's new nonce.  It is still Y4's.
pub proof fn rekeying_partition_collision_is_y4_intent(
    ledger: Seq<NonceEntry>,
    cap: DupCapture,
    p: int,
    q: int,
    new_nonce: nat,
    parts: nat,
)
    requires
        capture_well_formed(cap, p, parts),
        0 <= p < ledger.len(),
        !ledger[p].mask_valid,
        ledger[p].nonce == new_nonce,
        !valid_pair_collides(ledger),
        0 <= q < ledger.len(),
        q != p,
        ledger[q].mask_valid,
        ledger[q].nonce == new_nonce,
    ensures
        nonce_dup_origin(ledger, cap, p, new_nonce, parts) == FaultOrigin::Y4Intent,
{
    assert(new_nonce_collides(ledger, p, new_nonce)) by {
        assert(0 <= q < ledger.len() && q != p && ledger[q].mask_valid
            && ledger[q].nonce == new_nonce);
    }
}

/// A device-side verdict means Y4's ledger is clean: no collision of any
/// kind, and every captured partition had its key and nonce written by Y4
/// this boot.
pub proof fn device_side_means_ledger_clean(
    ledger: Seq<NonceEntry>,
    cap: DupCapture,
    p: int,
    new_nonce: nat,
    parts: nat,
)
    requires
        capture_well_formed(cap, p, parts),
        nonce_dup_origin(ledger, cap, p, new_nonce, parts) == FaultOrigin::DeviceSide,
    ensures
        !ledger_collision(ledger, p, new_nonce),
        forall|q: int| #[trigger] cap.bitmap.contains(q) ==> written_in_ledger(ledger, q),
{
    assert forall|q: int| #[trigger] cap.bitmap.contains(q) implies written_in_ledger(
        ledger,
        q,
    ) by {
        if !written_in_ledger(ledger, q) {
            assert(captured_not_written(ledger, cap));
        }
    }
}

/// NonceLedger invariant (`docs/lease_capability.md` I3): ledger-mask-valid
/// partitions carry pairwise distinct nonces.
pub open spec fn nonce_ledger_ok(ledger: Seq<NonceEntry>) -> bool {
    forall|i: int, j: int|
        #![trigger ledger[i], ledger[j]]
        0 <= i < ledger.len() && 0 <= j < ledger.len() && i != j && ledger[i].mask_valid
            && ledger[j].mask_valid ==> ledger[i].nonce != ledger[j].nonce
}

/// Rekey-time invariant: when Y4 fires MASK_REKEY for p, the nonce it wrote
/// for p differs from the nonce of every ledger-mask-valid q ≠ p.
pub open spec fn rekey_fire_ok(ledger: Seq<NonceEntry>, p: int, new_nonce: nat) -> bool {
    forall|q: int|
        0 <= q < ledger.len() && q != p && (#[trigger] ledger[q]).mask_valid
            ==> ledger[q].nonce != new_nonce
}

/// Y4 holding both ledger invariants is exactly the absence of a ledger
/// collision, so a `[12]` under them is never attributed to Y4 intent.
pub proof fn ledger_invariants_exclude_y4_intent(
    ledger: Seq<NonceEntry>,
    cap: DupCapture,
    p: int,
    new_nonce: nat,
    parts: nat,
)
    ensures
        (nonce_ledger_ok(ledger) && rekey_fire_ok(ledger, p, new_nonce)) <==> !ledger_collision(
            ledger,
            p,
            new_nonce,
        ),
        nonce_ledger_ok(ledger) && rekey_fire_ok(ledger, p, new_nonce) ==> nonce_dup_origin(
            ledger,
            cap,
            p,
            new_nonce,
            parts,
        ) != FaultOrigin::Y4Intent,
{
}

} // verus!
