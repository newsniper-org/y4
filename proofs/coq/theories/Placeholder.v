(* SPDX-License-Identifier: Apache-2.0 *)
(* SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors *)

(** * Placeholder Coq theory.

    Y4 reserves Coq for high-level invariants Verus cannot express
    (CLAUDE.md §6.6).  Real Y4 theories will live under sibling
    [.v] files and follow the module hierarchy:

      Y4.Lease.Spec     -- LeaseCap I1–I6 high-level statements
      Y4.IPC.Refinement -- LWKT + scheme fusion refinement proof
      Y4.Sel4.Wrapper   -- model of the seL4 cap invocation we depend on

    Until those land, this file holds a single trivial theorem so the
    harness has a concrete target to compile.
*)

Theorem placeholder_trivial : 1 + 1 = 2.
Proof. reflexivity. Qed.
