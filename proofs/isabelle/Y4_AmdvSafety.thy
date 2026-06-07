(* SPDX-License-Identifier: Apache-2.0 *)
(* SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors *)

(*
  Y4_AmdvSafety — Layer 1 cluster sub-grouping (R7.7).

  amdv cluster (21 AV — lower 12 + upper 9 + AV2-D Phase D) 의 모든
  Layer 2 per-Verus-module theory 의 single import entry.

  imports list 는 emit script 가 av-proof-body-tracker §6 의 Cluster 1
  (amdv lower, PR-2a) + Cluster 2 (amdv upper, PR-2b) 진행 상태에 따라
  rolling 갱신.

  Source-of-truth: docs/amdv_safety.md §5 (AV1~AV20 + AV2-D catalog,
  frozen v1.0) + .claude-notes/trackers/av-proof-body-tracker.md §2.
*)

theory Y4_AmdvSafety
  imports
    Main

    (* === Cluster 1 — amdv lower (12 AV) ====================
       PR-2a, av-proof-body-tracker §2 Cluster 1
       AV1 / AV3 / AV6 / AV7 / AV8 / AV11+12+13 (shared audit.rs) /
       AV16 / AV18 / AV19 / AV20 *)

    (* Y4_AmdvSafety_Lower_InterceptFloor      (* AV1, R7.11 milestone *) *)
    (* Y4_AmdvSafety_Lower_Deadline            (* AV3 *) *)
    (* Y4_AmdvSafety_Lower_Gif                 (* AV6 *) *)
    (* Y4_AmdvSafety_Lower_Tsc                 (* AV7 *) *)
    (* Y4_AmdvSafety_Lower_Nested              (* AV8 *) *)
    (* Y4_AmdvSafety_Lower_Audit               (* AV11+AV12+AV13 shared *) *)
    (* Y4_AmdvSafety_Lower_VmcbWhitelist       (* AV16 *) *)
    (* Y4_AmdvSafety_Lower_ClusterDep          (* AV18 *) *)
    (* Y4_AmdvSafety_Lower_Boundary            (* AV19 *) *)
    (* Y4_AmdvSafety_Lower_Dispatch            (* AV20 *) *)

    (* === Cluster 2 — amdv upper (9 AV) ======================
       PR-2b, av-proof-body-tracker §2 Cluster 2
       AV2+AV2-D (shared npt.rs) / AV4 / AV5 / AV9+10 (shared) /
       AV14+15 (shared lifetime.rs) / AV17 *)

    (* Y4_AmdvSafety_Upper_Npt                 (* AV2+AV2-D *) *)
    (* Y4_AmdvSafety_Upper_CpuPin              (* AV4 *) *)
    (* Y4_AmdvSafety_Upper_ThreadGroup         (* AV5 *) *)
    (* Y4_AmdvSafety_Upper_BitmapImmut         (* AV9+AV10 shared *) *)
    (* Y4_AmdvSafety_Upper_Lifetime            (* AV14+AV15 shared *) *)
    (* Y4_AmdvSafety_Upper_Firmware            (* AV17 *) *)
begin

end
