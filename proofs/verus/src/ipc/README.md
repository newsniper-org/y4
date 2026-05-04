<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 ipc Verus specifications

`Redox scheme` (제어평면) + `DragonFly LWKT msgport` (데이터평면) hybrid
(decision I-P2). seL4 trusted boundary: `seL4_Send/Recv/Call/Reply`,
`seL4_Signal/Wait`, `seL4_CNode_Copy/Move/Mint`, `seL4_Untyped_Retype`,
`seL4_TCB_*` (C4).

## v0 invariant catalog

| ID | 항목 | 위치 | 출처 결정 |
|----|---|---|---|
| **ipc_send_eventually_completes** | 모든 send 가 결국 완료 (delivered/replied/aborted/timeout) | `mod.rs` | architecture.md §IPC liveness |
| **SC1** | scheme path resolution 결정성 | `scheme.rs` | I-P2 제어평면 표준 |
| **SC2** | handle lifetime 이 close 로 한정 | `scheme.rs` | I-P2 |
| **SC3** | scheme verb dispatch 가 caller context | `scheme.rs` | capability isolation |
| **SC4** | SchemeId 유일성 | `scheme.rs` | I-P2 |
| **M1** | send/recv 짝짓기 | `msgport.rs` | LWKT 핵심 |
| **M2** | forward transitivity | `msgport.rs` | LWKT forwardmsg |
| **M3** | abort 는 owner 만 가능 | `msgport.rs` | LWKT abortmsg |
| **M4** | per-CPU queue isolation | `msgport.rs` | C2 SMP-first + LWKT lock-free 핵심 |
| **M5** | priority-inversion 회피 | `msgport.rs` | I-P3 v0 IN |
| **K1** | scheme verb ↔ msgport 등가성 | `consistency.rs` | I-P2 hybrid 일관성 |
| **K2** | cross-layer race 없음 | `consistency.rs` | I-P2 hybrid 일관성 |
| **K3** | handle ↔ endpoint bijection | `consistency.rs` | I-P2 hybrid 일관성 |

13 개 invariant 모두 v0 에 들어감 — bodies 는 후속 PR.
신규 invariant 추가는 본 표를 함께 갱신.

## 작성 / 검증

```sh
just verus            # repo root
# 또는
cd proofs/verus && just verify
```

## 비-목표 (v1 이후)

- **zero-copy 의 ownership transfer** — I-P3 결정상 별도 "shared-frame
  capability" primitive 로 Phase C 즈음 분리. 본 spec 은 send-by-copy 모델.
- **inter-CPU IPI latency 상한** — Phase C 후반의 wave-aligned scheduler
  명세에서.
- **HIU lease 와 IPC 의 정합성** — `lease_capability.md` v1.0 frozen 후
  별도 spec.
