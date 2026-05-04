<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Phase Plan

본 계획은 **WaveTensor RTL 의 실제 운용 전에 Y4 를 먼저 개발**한다는
사용자 결정을 반영한다. 원래 계획(WaveTensor Phase 0 → seL4 capability
스키마 도출)에서, Y4 는 운용 데이터 대신 **HIU ABI 명세** (`hiu_abi.md`)
를 capability 스키마의 입력으로 삼는다. 모든 하드웨어 의존 코드는
`mock` 뒤로 격리되어, FPGA 결합 전에도 모든 layer 가 빌드/검증 가능하다.

가장 큰 리스크는 ABI churn — RTL 이 나중에 모양이 바뀌면 Y4 가 비용을
흡수해야 한다. 이를 막는 단일 게이트가 **Phase A 의 HIU ABI 동결**.

## Phase A — 레포 기반 + 하드웨어 무관 문서 (현재 단계)

| 산출물 | 상태 |
|---|---|
| `LICENSE`, `NOTICE`, `README.md`, `CONTRIBUTING.md` | ✅ 완료 |
| `docs/architecture.md` (CC-BY-4.0 → Apache-2.0 inbound) | ✅ 완료 |
| `docs/licensing.md`, `docs/phase_plan.md` (본 문서) | ✅ 완료 |
| `docs/glossary.md` — RTL 추출 용어 사전 | ✅ 완료 |
| `docs/hiu_abi.md` — v0 draft | ✅ 완료 |
| **`docs/hiu_abi.md` v1.0 동결 (Y4 + WaveTensor 양측 sign-off)** | ⏳ 진행 |
| `docs/lease_capability.md` — capability 스키마 v0 | ⏳ 다음 |

**Phase A → Phase B 진입 트리거:** `docs/hiu_abi.md` 가 `v1.0 frozen`
으로 표시되고 `HIU_ABI_VERSION` 값이 `0x0001_0000` 으로 고정.

## Phase B — seL4 base + 비-HIU 캡슐 (하드웨어 0 % 필요)

본 단계의 모든 코드는 **QEMU + mock HIU** 로 빌드/실행/검증된다.

**구현 순서 (확정):**
1. `proofs/` 빌드 하네스 (Verus + Coq + CI gate) — formal-first 의 전제
2. `boot/` Limine → seL4 QEMU 부팅 ("Hello, Y4")
3. `ipc/` 와 `alloc/` 병렬 (각자 Verus 명세 선행)
4. `capsules/` 비-HIU 캡슐 (PCIe enum / USB stub / CXL stub)

| 산출물 | 비고 |
|---|---|
| `third_party/` 의존 통합 — **hybrid (D3)** | non-Rust upstream(seL4 15.x, Limine 12.x)은 git submodule; Tock 의 Rust crate 들은 `[patch.crates-io]` + git deps |
| CMake invocation wrapping | **logicutils-only** — `boot/<sub>.rules` + `lu-rule` + `lu-par` + `freshcheck`/`stamp`. xtask/cargo-make/CMakePresets 모두 제외. |
| `proofs/` Verus + Coq 빌드 하네스, CI 게이팅 | **첫 PR**. PR-단위 게이트 |
| Limine → seL4 부팅을 QEMU 에서 성립 | **x86_64 only first (D2)**. 다른 arch 는 형상 작업 시 |
| `kernel/` Y4 특화 레이어 (root task, capability 부트스트랩) | seL4 위 specialization |
| `ipc/` LWKT + Redox scheme 융합 + Verus 명세 선행 | formal-first |
| `alloc/` SLUB + lock-free SLAB + mmap-only 융합 + Verus 명세 선행 | formal-first |
| `capsules/` 비-HIU 캡슐 (PCIe enumeration, USB stub, CXL stub) | Tock 캡슐 타이핑 |
| Build orchestration | **Cargo workspace + justfile + logicutils (D1)** — `freshcheck`/`stamp`/`lu-par` |

**Phase B → Phase C 진입 트리거:** seL4 위에서 비-HIU 캡슐 IPC 왕복이
QEMU 통합 테스트로 통과 + 모든 신규 privileged path 의 Verus 명세
머지 완료.

## Phase C — Mock HIU 위 lease 런타임 (여전히 하드웨어 0 %)

| 산출물 | 비고 |
|---|---|
| `hiu/mock/` — `hiu_abi.md v1.0` 그대로 구현하는 Rust mock | QEMU MMIO BAR 노출 |
| `hiu/` lease capability 런타임 | mock 위 동작, hard cap=4 |
| XChaCha20 마스킹 / nonce binding on `context_switch` | Verus 불변식 선행 |
| 멀티테넌트 격리 시나리오 통합 테스트 | 전부 mock 기반 |
| TRNG 채널 stub + `trng_unhealthy` 처리 | mock entropy seed |

**Phase C → Phase D 진입 트리거:** mock 기반 멀티테넌트 격리 시나리오
의 atomic-rotate 시퀀스가 Verus 명세 + 통합 테스트 양쪽으로 통과.

## Phase D — 실제 WaveTensor RTL 결합

WaveTensor 측이 FPGA 타이밍 클로저를 통과한 시점에 진입.

| 산출물 | 비고 |
|---|---|
| mock HIU 를 PCIe BAR 바인딩 캡슐로 교체 | ABI 동결 덕에 **캡슐 교체만으로 끝나야 함** |
| Conformance 테스트: mock vs RTL 동일 ABI trace | regression gate |
| 형상별 부팅 검증 (서버 / 랩톱 / 랙 / 핸드헬드+독 / SoC) | 5 형상 |
| 성능 특성화 + 회귀 게이트 | latency 분포 표 |

**Phase D → Phase E 진입 트리거:** mock vs RTL conformance 가 모든 ABI
경로에서 일치 + 5 형상 booting + 외부 보안 민감 워크로드 1 개 이상.

## Phase E — Formal-verified 인증 트랙 + 외부 출시

| Track | Standard |
|-------|----------|
| Medical | FDA 510(k) |
| Aviation | DO-178C |
| Financial | FIPS 140-3 |

formal-first 가 세 트랙 모두 병렬 진입을 가능하게 한다. 우선순위는 외부
수요로 결정. 인증 dossier + 서명 릴리스 아티팩트 공개.

## Cross-phase notes

- **Cumulative validation.** 각 phase 경계는 운용 데이터(또는 mock 검증)
  + 증명 산출물 양쪽을 요구. 시간만으로는 진입하지 않음.
- **Mock-first invariant.** Phase B–C 의 모든 코드는 RTL 없이 빌드/테스트
  가능해야 한다. RTL 의존이 새는 PR 은 머지 금지.
- **Driver tier order:** DragonFlyBSD (1st) → NetBSD via rump (2nd) →
  Linux GPL capsule (3rd, 격리). `licensing.md` §"Linux driver tier".
- **Bootloader stays Limine** 전 phase. Phase E 인증 트랙이 trust-chain
  요구로 4th-tier (coreboot+payload) 를 강제할 때만 변경.

## 원래 계획과의 차이 요약

| 항목 | 이전 계획 | 현 계획 |
|---|---|---|
| 진입 트리거 | WaveTensor Phase 0 운용 데이터 + RTL FPGA 검증 | HIU ABI 동결 (양측 sign-off) |
| capability 스키마 도출 | 운용 데이터로부터 도출 | ABI 명세로부터 도출 |
| 하드웨어 결합 시점 | Phase 1 부터 | Phase D 부터 (B/C 는 mock) |
| ABI churn 리스크 | 낮음 (실측 후 작성) | **중간** — `hiu_abi.md` 동결로 완화 |

본 변경은 CLAUDE.md §2 의 "Phase 1 entry trigger" 기재를 무효화한다.
CLAUDE.md 다음 갱신 시 본 phase_plan 으로 포인터 일원화 예정.
