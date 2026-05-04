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

## Phase A — 레포 기반 + 하드웨어 무관 문서

| 산출물 | 상태 |
|---|---|
| `LICENSE`, `NOTICE`, `README.md`, `CONTRIBUTING.md` | ✅ |
| `docs/architecture.md` (CC-BY-4.0 → Apache-2.0 inbound) | ✅ |
| `docs/licensing.md`, `docs/phase_plan.md` (본 문서) | ✅ |
| `docs/glossary.md` — RTL 추출 용어 사전 | ✅ |
| `docs/hiu_abi.md` — v0 draft | ✅ |
| `docs/lease_capability.md` — capability 스키마 v0 | ✅ |
| **`docs/hiu_abi.md` v1.0 동결 (Y4 + WaveTensor 양측 sign-off)** | ⏳ 진행 |

**Phase A → Phase B 진입 트리거:** `docs/hiu_abi.md` 가 `v1.0 frozen`
으로 표시되고 `HIU_ABI_VERSION` 값이 `0x0001_0000` 으로 고정.
*(HIU 와 무관한 Phase B 산출물은 본 동결 전이라도 진행 가능 —
실제로 Phase B 의 5개 단계는 모두 mock 위에서 완료됨.)*

## Phase B — seL4 base + 비-HIU 캡슐 (하드웨어 0 % 필요) ✅

본 단계의 모든 코드는 **QEMU + mock HIU** 로 빌드/실행/검증된다.

**구현 순서 (모두 ✅):**

| Step | 영역 | 마일스톤 | 결과 |
|---|---|---|---|
| 1 | `proofs/` Verus + Rocq 하네스 + CI gate | placeholder + 모듈 트리 | **50 verified, 0 errors** |
| 2 | `boot/` Limine → seL4 QEMU | seL4 boot path reached | qemu boot OK |
| 3a | `ipc/` Rust 크레이트 — scheme + LWKT msgport hybrid | open/send/wait/forward/abort/priority | **18 tests** |
| 3b | `alloc/` Rust 크레이트 — DragonFly SLAB + hardened backend | bump + slab + scudo contract + integrated | **22 tests** |
| 3c | `scudo-sys/` LLVM scudo C++ FFI | link smoke + alloc/free | **4 tests** + alloc 의 `--features scudo` 시 +2 |
| 3d | Verus refinement proofs (alloc + ipc) | `assume()` → constructive | **10 invariant 추가** |
| 4 | `capsules/` Tock isolation + PCIe enum | C1–C3, P1–P3 | **16 tests** |
| 5 | `kernel/` root task | "Hello, Y4" on serial | `qemu-smoke` PASS |

**확정된 build/dev 결정 (durable, `MEMORY/y4_build_decisions.md`):**

- **D1** Cargo workspace + per-subtree justfile + logicutils
  (`freshcheck`/`stamp`/`lu-par`).
- **D2** x86_64 only first (Phase B 전 단계).
- **D3** hybrid 의존 통합 — non-Rust upstream (seL4 15.0.0, Limine v12.1.0)
  은 git submodule; Rust crate 는 cargo `[patch.crates-io]` + git deps;
  LLVM scudo 는 pin-file + `just scudo-fetch` 로 vendored on demand.
- **D4** Phase B 구현 순서 (위 표).
- **CMake wrapping** logicutils-only (`boot/<sub>.rules` + `lu-rule` +
  `lu-par`). xtask / cargo-make / CMakePresets 모두 제외.

**Phase B → Phase C 진입 트리거:** ✅ 도달 — `qemu-smoke` 가 root task
greeting 까지 검증, 모든 Verus 명세 머지 완료.

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
