// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! LBID mapping-table write precondition (`docs/hiu_abi.md` §2.8).
//!
//! The LBID table (`0x800`–`0xBFF`, 256 × 4 B) is one device-global table
//! with no partition tag.  WaveTensor canonical 3.7.1 lists the overlapping
//! slot range as the fifth write-time rejection, but its v1.0 disposition
//! (`e88f6f4` `03-hiu.typ:1094`) is that a v1.0 device does not judge it and
//! accepts; non-overlap is a host duty (E.1.1 duty 4, `e-defined.typ:208`),
//! and no v1.0 fault bit carries it.  Today's RTL accepts as well.
//! Two valid entries naming overlapping ranges land two fragment streams in
//! the same slots (FAB-4).  In v1.0 the only barrier is therefore Y4's own
//! check before every proxied write — this module states it.
//!
//! The meaning of the entry's length field is still open on the WaveTensor
//! side (B.5).  Y4 judges conservatively on the half-open range
//! `[start, start + len)` and never writes a valid entry with `len == 0`.

use vstd::prelude::*;

verus! {

/// One LBID entry as Y4 writes it: `[31]` V, `[15:8]` range start,
/// `[7:0]` length.  Reserved bits are always written 0 (not modelled).
pub struct LbidEntry {
    pub valid: bool,
    pub start: nat,
    pub len: nat,
}

/// Two entries overlap when both are valid and their ranges intersect.
pub open spec fn lbid_overlap(a: LbidEntry, b: LbidEntry) -> bool {
    a.valid && b.valid && a.start < b.start + b.len && b.start < a.start + a.len
}

/// A single entry is well-formed for a device with `slots` physical slots:
/// a valid entry has a non-empty range inside the slot index space.
pub open spec fn lbid_entry_ok(e: LbidEntry, slots: nat) -> bool {
    e.valid ==> (e.len > 0 && e.start < 256 && e.len < 256 && e.start + e.len <= slots)
}

/// Table invariant: every entry well-formed, no two distinct entries overlap.
pub open spec fn lbid_table_ok(t: Seq<LbidEntry>, slots: nat) -> bool {
    &&& forall|i: int| 0 <= i < t.len() ==> #[trigger] lbid_entry_ok(t[i], slots)
    &&& forall|i: int, j: int|
        0 <= i < t.len() && 0 <= j < t.len() && i != j ==> !#[trigger] lbid_overlap(t[i], t[j])
}

/// Precondition of a proxied write of `e` at index `idx`: the entry is
/// well-formed and overlaps no *other* resident entry.
pub open spec fn lbid_write_ok(t: Seq<LbidEntry>, idx: int, e: LbidEntry, slots: nat) -> bool {
    &&& 0 <= idx < t.len()
    &&& lbid_entry_ok(e, slots)
    &&& forall|j: int| 0 <= j < t.len() && j != idx ==> !#[trigger] lbid_overlap(e, t[j])
}

/// A write that passes `lbid_write_ok` preserves the table invariant.
pub proof fn lbid_write_preserves(t: Seq<LbidEntry>, idx: int, e: LbidEntry, slots: nat)
    requires
        lbid_table_ok(t, slots),
        lbid_write_ok(t, idx, e, slots),
    ensures
        lbid_table_ok(t.update(idx, e), slots),
{
    let u = t.update(idx, e);
    assert forall|i: int| 0 <= i < u.len() implies #[trigger] lbid_entry_ok(u[i], slots) by {
        if i != idx {
            assert(u[i] == t[i]);
        }
    }
    assert forall|i: int, j: int|
        0 <= i < u.len() && 0 <= j < u.len() && i != j implies !#[trigger] lbid_overlap(u[i], u[j]) by {
        if i == idx {
            assert(u[j] == t[j]);
        } else if j == idx {
            assert(u[i] == t[i]);
            assert(!lbid_overlap(e, t[i]));
        } else {
            assert(u[i] == t[i] && u[j] == t[j]);
        }
    }
}

/// Invalidating an entry (V=0) never breaks the table invariant — Y4's
/// switch-time reload clears the outgoing lease's entries this way.
pub proof fn lbid_invalidate_preserves(t: Seq<LbidEntry>, idx: int, slots: nat)
    requires
        lbid_table_ok(t, slots),
        0 <= idx < t.len(),
    ensures
        lbid_table_ok(t.update(idx, LbidEntry { valid: false, start: 0, len: 0 }), slots),
{
    let e = LbidEntry { valid: false, start: 0, len: 0 };
    assert(lbid_write_ok(t, idx, e, slots));
    lbid_write_preserves(t, idx, e, slots);
}

/// Boot state: Y4 writes V=0 to every entry before any proxied write
/// (`docs/hiu_abi.md` §2.5 step 5), so its ledger starts all-invalid.
pub open spec fn lbid_all_invalid(t: Seq<LbidEntry>) -> bool {
    forall|i: int| 0 <= i < t.len() ==> !(#[trigger] t[i]).valid
}

/// The all-invalid boot table satisfies the table invariant.
pub proof fn lbid_all_invalid_ok(t: Seq<LbidEntry>, slots: nat)
    requires
        lbid_all_invalid(t),
    ensures
        lbid_table_ok(t, slots),
{
    assert forall|i: int| 0 <= i < t.len() implies #[trigger] lbid_entry_ok(t[i], slots) by {
        assert(!t[i].valid);
    }
    assert forall|i: int, j: int|
        0 <= i < t.len() && 0 <= j < t.len() && i != j implies !#[trigger] lbid_overlap(t[i], t[j]) by {
        assert(!t[i].valid);
    }
}

} // verus!
