<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 alloc Verus specifications

`DragonFly lock-free SLAB` front-end + `LLVM scudo` backend (decision A-P1).
seL4 trusted boundary: `seL4_X86_Page_Map / Unmap`, `seL4_X86_PageTable_Map`,
`seL4_Untyped_Retype` (C4).

## v0 invariant catalog

| ID | 항목 | 위치 | 출처 결정 |
|----|---|---|---|
| **alloc_no_overlap** | 살아 있는 두 할당이 서로 겹치지 않음 | `mod.rs` | architecture.md §Memory allocator |
| **S1** | per-CPU magazine 의 disjointness | `slab.rs` | DragonFly SLAB lock-free 핵심 |
| **S2** | zone cache 크기 상한 | `slab.rs` | flush 정책 |
| **S3** | alloc 결과 정렬 | `slab.rs` | 일반 calling convention |
| **B1** | scudo 백엔드 비-overlap | `scudo.rs` | scudo 자체 보장 |
| **B2** | use-after-free 검출 | `scudo.rs` | A-P3 |
| **B3** | 주소 randomization | `scudo.rs` | A-P3 |
| **B4** | guard page 정렬 | `scudo.rs` | A-P3 |
| **B5** | NUMA 노드 locality | `scudo.rs` | C2 SMP-first |
| **B6** | quarantine lifetime 상한 | `scudo.rs` | scudo 메모리 footprint |
| **X1** | SLAB pages ⊆ scudo pages | `boundary.rs` | composition contract |
| **X2** | error propagation 보존 | `boundary.rs` | C3 공통 Y4Error |
| **X3** | composed no-overlap | `boundary.rs` | S1 + B1 ⇒ alloc_no_overlap |

13 개 invariant 모두 v0 에 들어감 — bodies 는 추후 PR 에서 채움.
신규 invariant 추가는 본 표를 함께 갱신 (PR description 에서 명시).

## 작성 / 검증

```sh
just verus            # repo root
# 또는
cd proofs/verus && just verify
```

`just verus` 가 통과해야 PR 머지 가능. 모든 `proof fn ... ensures true {}`
는 placeholder — Phase B step 3 의 후속 PR 에서 실제 명세로 채워진다.

## 비-목표 (v1 이후)

- **NUMA arena 동적 재배정** — 형상별 분기는 Phase D 와 함께
- **scudo 의 thread-local cache 튜닝** — 성능 최적화 영역, v1 spec 이후
- **HIU lease 연동** — `hiu_abi.md` v1.0 frozen 후 별도 spec
