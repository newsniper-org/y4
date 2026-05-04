<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 capsules Verus specifications

Tock-style isolated drivers — each capsule holds a typed capability
set granted at boot and may not mint new capabilities.  HIU is
intentionally absent from Phase B step 4 (blocked on
`docs/hiu_abi.md` v1.0 frozen per `MEMORY/y4_basics.md`).

## v0 invariant catalog

| ID | 항목 | 위치 | 출처 결정 |
|----|---|---|---|
| **capsules_invariant** | C1 + C2 + C3 결합 | `mod.rs` | Tock isolation 핵심 |
| **C1** | cap-token 유일 소유 | `isolation.rs` | Map 구조적 |
| **C2** | 캡슐은 mint 불가 | `isolation.rs` | Tock capability typing |
| **C3** | 자원 disjoint 또는 explicit share | `isolation.rs` | Tock isolation |
| **P1** | PCIe enum 결정성 | `pcie.rs` | lease cap binding 안정성 |
| **P2** | enum 주소 unique | `pcie.rs` | 버스 walk 알고리즘 |
| **P3** | enum 호출은 BusEnumerator cap 필요 | `pcie.rs` | capability gate |
| **U1** | USB port count 비-감소 | `usb.rs` | 스텁 |
| **U2** | URB completion totality | `usb.rs` | 스텁 |
| **X1c** | CXL region_id 유일 | `cxl.rs` | 스텁 |
| **X2c** | CXL coherent read totality | `cxl.rs` | 스텁 |

11 개 invariant 모두 v0 에 들어감.  PCIe (P1–P3) 가 이번 단계의 가장
구체적 영역 — Y4 가 WaveTensor 가속기를 호스트 PCIe 버스에서 발견하는
경로의 spec.  USB / CXL 는 stub — 첫 실제 드라이버 PR 시 본문 강화.

## 작성 / 검증

```sh
just verus            # repo root
```

## 비-목표 (Phase B step 4 범위 외)

- **HIU 캡슐** — `docs/hiu_abi.md` v1.0 frozen 후 별도 spec.
- **드라이버 본체 구현** — Phase B step 4 는 spec 만; 실제 PCIe enum
  Rust 코드는 dependent PR.
- **NetBSD rump 통합** — USB 의 첫 실제 드라이버는 rump 경유 (D3/`licensing.md`
  driver tier 2).  spec 갱신은 그 통합 PR 에서.
