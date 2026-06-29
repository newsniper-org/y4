<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Y4 dev mode — host-execution (vkernel 개발 가치의 Y4 흡수)
created: 2026-06-20T15:41:47+09:00   # KST (UTC+9)
revised: 2026-06-29T10:26:30+09:00   # 디버거 = lldb 채택 (§3.7, 사용자 보충)
status: brainstorming (§3.7 디버거 결정 / 나머지 미결)
scope: Y4 capsule + orchestrator + lease 로직을 seL4/QEMU 없이 host
       user-space process 로 실행 (dev/test 전용).  vkernel 의 개발/
       디버깅 가치 흡수.  production 격리 아님
refs:
  - .brainstormings/20260620-153653-dragonfly-vkernel-y4-reinterpretation.md §3.3
  - alloc/src/page_backend.rs (MockPageBackend — 현 dev 토대)
  - .brainstormings/20260618-221518-pluggable-alloc-frontend-contract.md
    (trait-backed mock 패턴)
  - CLAUDE.md §8 (qemu-smoke) / docs/vmm_arch.md (capsule + orchestrator)
---

# Y4 dev mode — host-execution

## 0. 발제

vkernel 의 *최대 강점* = 커널을 host process 로 실행 → gdb 디버그 + 빠른
iteration + crash 격리 (vkernel 재해석 §3.3).  vkernel 자체는 6.0 에서
drop (소프트웨어 MMU 가 production 에 부적합) 됐으나, **그 개발/디버깅
가치는 잃으면 아까움**.  Y4 가 이를 **dev mode** 로 흡수 — Phase C 개발
생산성.

핵심 구분 (vkernel 교훈 준수): **dev mode 의 soft backend 는 test 전용,
production 격리는 항상 하드웨어 virt** (cross-platform §4.B soft-MMU
배제 정합).

## 1. 현 Y4 dev/test 계층

- **MockPageBackend** (`page_backend.rs`) — seL4 page 호출을 host 메모리
  로 mock.  alloc 단위 test.
- **qemu-smoke** — 실제 seL4 + QEMU + Limine.  production-like 통합.
- 사이가 비어있음 — capsule + orchestrator + lease *통합 로직* 을 seL4/
  QEMU 없이 빠르게 디버그할 계층 부재.  qemu-smoke 는 무겁고 (빌드+부팅)
  커널 내부 디버그 불편 (Tier 2 의 lldb `gdb-remote`, §3.7).

## 2. vkernel 의 dev 가치 (흡수 대상)

| vkernel 강점 | 메커니즘 |
|---|---|
| gdb 디버그 | 커널이 host process → 일반 gdb |
| crash 격리 | guest 커널 panic → host 안 죽음 |
| 빠른 iteration | 커널 빌드/부팅 없이 process 재실행 |
| recursive | vkernel 안 vkernel (소프트웨어 무한 중첩) |

## 3. Y4 dev mode 설계

### 3.1 4-tier 개발 계층 (host-Y4 = Tier 1 신설)
| Tier | 내용 | 격리 | 용도 |
|---|---|---|---|
| 0 unit test (현) | MockPageBackend + `cargo test` | 없음 | 로직 단위 |
| **1 host-Y4 (신규)** | capsule + orchestrator + lease 를 host process 로 (mock seL4 + soft backend) | host process/thread (약함, test 전용) | **통합 디버그 + 빠른 iteration + `rust-lldb`** (§3.7) |
| 2 qemu-smoke (현) | 실제 seL4 + QEMU + Limine | seL4 (production-like) | 통합 검증 |
| 3 실HW (Phase D+) | 실제 AMD-V / NPT | 하드웨어 | production |

Tier 1 = vkernel 의 "커널을 process 로" 를 Y4 capsule 에 적용.

### 3.2 mock seL4 (capability / IPC / page trait mock)
- seL4 API (cap invoke / IPC send-recv / page map / untyped retype) 를
  **host 함수로 mock**.  vkernel 의 host syscall intercept 류.
- capability 의미 보존: cap table / derivation 을 host 자료구조 (HashMap)
  로 — capability 로직 (cap 검증, revoke) 디버그 가능.
- IPC: msgport / scheme 를 host channel (mpsc 아님 — §0.5 정합 위해
  단순 queue + 명시적 step) 로.
- **`Kernel` trait** (page_backend.rs 의 PageBackend 패턴 확장): seL4
  호출 전체를 trait 뒤로 → `MockKernel` (dev) / `RealSeL4` (production).
  pluggable allocator 의 trait-mock 패턴과 동형.

### 3.3 soft page backend (host mmap, test 전용)
- PageBackend trait 의 host-mmap impl — MockPageBackend 확장 (현재는
  메모리 흉내, 실제 mmap 으로 page fault / mapping 흉내 가능).
- **vkernel 의 MAP_VPAGETABLE 와 구분**: vkernel 은 production 도 soft
  (그래서 drop).  Y4 soft backend 는 **명시적 test 전용** — production
  은 항상 seL4 + HW NPT.  vkernel 교훈 준수.

### 3.4 capsule + orchestrator 를 host process 로
- vmm-vmcb / vmm-npt / ... capsule + vmrun-orchestrator 를 host process
  로 실행 (mock Kernel + soft backend).
- vendor backend (`vcb_amd.rs` 등) 의 HW 의존 부분은 mock (또는 bhyve/
  nvmm reference 와 대조 — algorithm 검증).
- → CapsuleMsg dispatch / 7-step atomic sequence / AV invariant 로직을
  `rust-lldb` 로 step 디버그 (§3.7).

### 3.5 경계 — dev 전용 (★ vkernel 교훈)
- dev mode 는 **production 격리 X** — host process 격리는 약함 (capability
  의미는 mock, 실제 강제 X).  test/디버그 전용.
- **production 은 항상 Tier 2/3** (seL4 + HW virt).
- determinism / real-time / verification 비대상 (dev mode 는 test
  harness, production path 만 verified — Verus 는 real seL4 path).
- vkernel 이 production 까지 soft 로 갔다가 죽은 것과 **명시적 대조** —
  Y4 dev mode 는 soft 를 dev 에 *가둠*.

### 3.6 trait 정합 (pluggable 패턴 재사용)
- `PageBackend` (이미) + `Kernel` (신규) trait → mock/real 양 impl.
- pluggable allocator (AllocFrontend / Freelist trait) + capability
  (CapabilityAlloc<F>) 의 **trait-backed 설계가 dev mode 를 자연 지원**
  — backend 를 mock 으로 바꾸면 host 실행.
- generic-first (pluggable §2.4) 정합 — `Kernel` 을 generic param 으로
  `Y4Core<K: Kernel>` → `Y4Core<MockKernel>` (dev) / `Y4Core<RealSeL4>`
  (production).  compile-time 분기, 런타임 비용 0.

### 3.7 디버거 = lldb (gdb 대신) — 사용자 보충 (2026-06-29)

**lldb 를 Y4 기본 디버거로 채택** (gdb 대신).  근거:

1. **★ license 정합** — lldb = **Apache-2.0 (LLVM exception)** ↔ Y4
   single-license **Apache-2.0** (CLAUDE.md §3).  gdb = **GPL-3** → Y4
   의 GPL 격리 철학 (`licensing.md`, GPL-capsule isolation) 과 충돌.
   디버거는 dev tool 이라 직접 link X (license 강제 아님) 이나, **권장
   도구의 license 일관성** 이 Y4 의 Apache-2 생태계 정합.
2. **LLVM 생태계 정합** — Rust (LLVM) + scudo (LLVM compiler-rt) +
   ARCv3 (LLVM backend 필요, cross-platform §3) 모두 LLVM → lldb 가 같은
   토대.  `rust-lldb` 공식 wrapper (rustup 제공).
3. **BSD 정합** — bhyve/nvmm reference (FreeBSD/NetBSD) 작업 시 lldb
   (FreeBSD base system 기본 디버거).  Y4 의 BSD/Redox/Tock 생태계 지향
   (CLAUDE.md §4 reuse manifest) 정합.
4. **cross-platform** — lldb multi-arch (LLVM arch backend 정합) —
   AArch64/RISC-V/POWER 디버깅 일관.

**Tier 별 적용**:
| Tier | 디버거 |
|---|---|
| 0 unit test | `rust-lldb` (host process) |
| **1 host-Y4** | `rust-lldb` (host process — lldb 명확 우위: license + LLVM + Rust) |
| 2 qemu-smoke | lldb `gdb-remote` (QEMU gdbstub = gdb remote protocol → lldb 연결).  단 QEMU+lldb 의 일부 호환성 이슈 (hw breakpoint 등) 시 **gdb fallback** 허용 |
| 3 실HW | lldb + JTAG/probe (arch 별) |

**gdb 병용 정책**: **lldb 기본** + gdb 는 Tier 2 (QEMU gdbstub) 의 특정
seL4 디버그 기능에서 lldb 호환성 부족 시 *옵션 fallback*.  개인 선호
허용 (강제 X), 단 Y4 **공식 권장 + 문서/CI 예시 = lldb**.

> 솔직한 nuance: Tier 0/1 (host process) 는 lldb 명확 우위 (license +
> rust-lldb).  Tier 2 (QEMU+seL4) 는 전통적으로 gdb 가 더 검증됨 (QEMU
> gdbstub 친화) → lldb `gdb-remote` 우선 시도하되 막히면 gdb.  즉 "lldb
> 기본, Tier 2 에서 실용적 gdb fallback".

## 4. vkernel 과의 대조 (Y4 dev mode 가 교훈 지킴)

| | DragonFly vkernel | Y4 dev mode |
|---|---|---|
| 무엇을 host 로 | 전체 커널 | capsule + orchestrator + lease 로직 |
| MMU | MAP_VPAGETABLE (소프트웨어) | mock / host mmap (test 전용) |
| production 사용 | ✅ (그래서 VM 재설계 때 drop) | ❌ dev 전용 (교훈 준수) |
| 격리 | 소프트웨어 (production 격리 시도) | mock (검증 X, 디버그용) |
| production 격리 | (자기 자신) | seL4 + HW virt (별도 Tier 2/3) |

→ Y4 dev mode = vkernel 의 *개발 가치* 만 취하고 *production-soft 실수*
는 회피.  "soft 는 dev 에 가둔다".

## 5. 가치 (Phase C 개발 생산성)

- **iteration 속도** — capsule/orchestrator 로직 변경 → host process 재실행
  (seL4 빌드 + QEMU 부팅 불필요).  초 단위 iteration.
- **lldb 디버그** (§3.7) — capsule 내부 (CapsuleMsg dispatch, AV 로직,
  lease lifecycle) 를 host `rust-lldb` 로 step / breakpoint / core dump.
  qemu-smoke
  의 커널 내부 디버그 난이도 회피.
- **CI** — host-Y4 통합 test (qemu 보다 빠름) → CI 단계 추가 (cargo
  test = Tier 0, host-Y4 = Tier 1, qemu-smoke = Tier 2).
- **algorithm 검증** — vendor backend 로직을 bhyve/nvmm reference 와
  host 에서 대조 (contribute-back §2.B 의 Verus 발견물 토대).

## 6. 결정 / 미결

### 확정 (사용자)
- **디버거 = lldb 기본 (§3.7)** — license 정합 (lldb Apache-2 ↔ Y4
  Apache-2, gdb GPL-3 회피) + LLVM 생태계 (Rust/scudo/ARCv3) + BSD 정합.
  Tier 0/1 = `rust-lldb`, Tier 2 = lldb `gdb-remote` (gdb fallback 허용)

### 방향 (잠정)
- Tier 1 host-Y4 dev mode 신설 — `Kernel` trait mock + soft page backend
  (test 전용) + capsule/orchestrator host 실행
- trait-backed (pluggable 패턴) — `Y4Core<K: Kernel>` generic, mock/real
- soft 는 dev 에 가둠 (vkernel 교훈, production 은 seL4+HW virt)

### 미결
- **범위** — 현 MockPageBackend + qemu-smoke 로 충분한가 vs Tier 1 풀
  구현 (mock seL4 전체).  개발 생산성 이득 vs mock seL4 구현/유지 비용
- **mock seL4 충실도** — capability/IPC 의미를 어디까지 mock (cap
  derivation / revoke / IPC ordering).  너무 충실하면 seL4 재구현,
  너무 얕으면 디버그 가치 ↓
- **Verus 와의 관계** — dev mode 는 verification 비대상이나, mock 과
  real 의 *동작 동치* 를 어떻게 보장 (mock 이 real 과 다르면 디버그가
  거짓).  trait contract (pluggable C-류) 로 mock/real 양쪽 spec?
- **recursive dev mode** — vkernel 식 Y4-안-Y4 (host 에서 nested) 가
  nested virt (R-α) 디버그에 유용한지
- **Tier 1 ↔ Tier 2 격차** — host-Y4 에서 통과한 게 qemu-smoke 에서
  깨지는 경우 (mock 부정확) 최소화

### 후속 / 연결
- vkernel 재해석 §3.3 의 구체화 — "vkernel-style Y4 dev mode" 의 설계.
- pluggable allocator (trait-backed) + capability (CapabilityAlloc<F>)
  의 trait 설계가 dev mode 를 자연 지원 — 같은 generic-mock 패턴.
- cross-platform §4.B soft-MMU 배제와 정합 (soft = dev 전용).
- Phase C 진입 시 개발 생산성 인프라로 우선 검토 (capsule/orchestrator
  본격 구현 전에 dev mode 갖추면 iteration ↑).
