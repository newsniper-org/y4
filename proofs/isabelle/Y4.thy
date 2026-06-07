(* SPDX-License-Identifier: Apache-2.0 *)
(* SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors *)

(*
  Y4 — Top-level Isabelle/HOL theory entry.

  R7.7 sign-off (2026-06-03): 2-layer imports chain
  - Layer 1 (cluster sub-grouping): Y4_AmdvSafety, Y4_PowerSafety
  - Layer 2 (per-Verus-module 1 .thy, cherry-pick): 모든 generated theory

  Y4 가 PR-Verus-Backend (R3.11+R3.12+R7.3) land 후 `just emit-isabelle`
  으로 Layer 2 의 per-AV .thy 생성 — 첫 emit milestone = AV1
  Y4_AmdvSafety_Lower_InterceptFloor (R7.11).

  본 file 의 imports list 는 emit script 가 (av-proof-body-tracker §6
  의 cluster 진행 상태에 따라) rolling 갱신.

  cross-ref: proofs/isabelle/README.md
*)

theory Y4
  imports
    Main
    (* Layer 1 — cluster sub-grouping (PR-Verus-Backend land 시 활성) *)
    (* Y4_AmdvSafety *)
    (* Y4_PowerSafety *)

    (* Layer 2 — flat list (per-AV theory, emit milestone 별 추가) *)
    (* 첫 emit (R7.11):
       Y4_AmdvSafety_Lower_InterceptFloor *)
begin

end
