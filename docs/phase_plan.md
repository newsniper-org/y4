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

## Phase C — Mock HIU 위 lease 런타임 + 첫 게스트 호스팅 (여전히 하드웨어 0 %)

| 산출물 | 비고 |
|---|---|
| `hiu/mock/` — `hiu_abi.md v1.0` 그대로 구현하는 Rust mock | QEMU MMIO BAR 노출 |
| `hiu/` lease capability 런타임 | mock 위 동작, hard cap=4 |
| XChaCha20 마스킹 / nonce binding on `context_switch` | Verus 불변식 선행 |
| 멀티테넌트 격리 시나리오 통합 테스트 | 전부 mock 기반 |
| TRNG 채널 stub + `trng_unhealthy` 처리 | mock entropy seed |
| **첫 게스트 호스팅 (별도 repo `y4-hypercall`)** | Alpine Linux mini-rootfs. **virtio paravirt only** (게스트 자체 PCIe driver 는 Phase D). 첫 PoC = ramdisk + serial console (G-disk/network/display 결정 G3 통과 후). VMM 위치 = `y4-hypercall` repo. |
| **하드웨어 driver 일부** (별도 repo `y4-drivers`) | virtio-net/blk + e1000e + AHCI + NVMe 1.x + XHCI 1차 PR 범위. Phase C 기간 중 병렬로 진행 |

**Phase C → Phase D 진입 트리거:** mock 기반 멀티테넌트 격리 시나리오
의 atomic-rotate 시퀀스가 Verus 명세 + 통합 테스트 양쪽으로 통과 +
첫 게스트의 virtio paravirt 동작 확인.

> **AMD-V (SVM) 결정 + VMM 아키텍처 (2026-05-04):** seL4 15.0.0 의
> `KernelVTX` 는 Intel VT-x 전용. 현 호스트가 AMD Ryzen APU.  **길 1
> + ARCH-II' 채택** — D1a (seL4 측 raw-SVM syscall 패치) +
> **capsule-decomposed VMM** (10 capsule + thin orchestrator) + Verus
> 부분 증명 + VeriSMo 의 **2-layer concurrency 증명 기법 영감** (코드
> import 0).
> spec: `docs/amdv_safety.md` (14 안전장치 catalog) + `docs/vmm_arch.md`
> (capsule 분해 디자인).
> 4 길 (D1/NOVA/Hyperkernel/SVSM) + Atmosphere/VeriSMo 사실 확인 결과는
> `.claude-notes/amd-v-verified-survey.md`.
>
> **fork 호환성 contract:** Strictly Additive Fork — `docs/sel4_fork_policy.md`.
> upstream seL4 의 모든 회귀 테스트가 Y4 fork 에서도 pass 보장.
>
> **Phase C 진입 전 차단 의존 (이 순서):**
> 1. `docs/amdv_safety.md` v1.0 frozen (S1–S14 안전장치 sign-off)
> 2. `docs/vmm_arch.md` v1.0 frozen (ARCH-II' capsule 분해 sign-off)
> 3. `docs/sel4_fork_policy.md` v1.0 frozen
> 4. `docs/verus_to_isabelle.md` v1.0 frozen + `y4-verus2isabelle`
>    번역기 구현 (statement-only `sorry` + `axiom` opt-in hybrid)
> 5. seL4 측 D1a C 패치 (`CONFIG_Y4_AMDV` gate, default OFF). 회귀
>    게이트 통과
> 6. **vmrun-orchestrator + 10 capsule 첫 PR** (`Y4/vmrun-orchestrator/`
>    + `Y4/capsules/` 의 새 멤버) + Verus 명세 (S1–S14 본문)
> 7. `y4-hypercall` 재정의 — Phase D 의 R-α/R-γ + S14 사용자 CLI 도구
>    repo (core VMM 코드는 Y4 워크스페이스 안)
>
> 4 는 5–6 과 시간상 병렬. 6 의 contribute-back PR 은 도구 산출물
> (Isabelle `.thy` skeleton) 함께 제출.
>
> **위 7 단계 완료 후에 y4-drivers / capsules 깊이 작업 진입.**

## Phase D — 실제 WaveTensor RTL 결합 + PCIe passthrough

WaveTensor 측이 FPGA 타이밍 클로저를 통과한 시점에 진입.

| 산출물 | 비고 |
|---|---|
| mock HIU 를 PCIe BAR 바인딩 캡슐로 교체 | ABI 동결 덕에 **캡슐 교체만으로 끝나야 함** |
| Conformance 테스트: mock vs RTL 동일 ABI trace | regression gate |
| 형상별 부팅 검증 (서버 / 랩톱 / 랙 / 핸드헬드+독 / SoC) | 5 형상 |
| 성능 특성화 + 회귀 게이트 | latency 분포 표 |
| **PCIe device passthrough 인프라** | IOMMU passthrough + per-device BAR cap + IRQ remap + VFIO-eq API. **게스트가 자체 driver 로 hardware 직접 접근하는 보장이 본 단계에서 성립** (Phase B/C 의 게스트는 virtio paravirt only). |
| **하드웨어 driver 확장 PR 들** (별도 repo `y4-drivers`) | Wi-Fi 칩셋 1 개, USB4/Thunderbolt 4, NVMe 2.x, 이동통신 1 모뎀 등 장기 로드맵 항목 진입 |
| **R-α `/dev/kvm` ioctl 프록시** | 게스트가 KVM-기반 VM (QEMU/Firecracker/Kata/최신 VBox/Android emulator) 실행 시 host (Y4-VMM) 에 sibling VM 생성으로 자동 redirect. S9 (nested SVM 차단) 의 우회 — verification 표면 작게 유지 |
| **R-γ paravirt agent** | 비-KVM VM 매니저 (legacy VBox vboxdrv, VMware) 용 wrapper agent + paravirt sibling-VM-create API. R-α 보완 |
| **GPU passthrough** | Waydroid 가속 등 게스트 그래픽 워크로드 활성화 (Waydroid 는 nested virt 무관 — LXC 기반) |

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
