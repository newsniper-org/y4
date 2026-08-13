<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Capsule fault isolation / restart — driver crash 시 격리·정지·회수·재시작 (정적 C1/C2/C3 의 동적 확장)
created: 2026-08-12T17:44:02+09:00   # KST (UTC+9)
status: brainstorming (결정: §2.1 2nd-tier unsafe-audit gate / §4.1 FLR 미지원 / §4.2 DMA↔partition / §5.1 reset-group 조율 — 나머지 옵션 탐색).  IPC 발제(atomic-free composition)에서 이어짐
scope: y4-capsules 최종 목표의 fault 차원.  현 capsules/ (16 tests, C1/C2/C3 정적 격리) = baseline.
       tier→isolation 스펙트럼 / fault lifecycle / IOMMU DMA confinement /
       supervision / accelerator 경계 불변식 / verification
refs:
  - capsules/README.md + capsules/src/{isolation,types,pcie,config_space}.rs (baseline, stage 0)
  - proofs/verus/src/capsules/isolation.rs (C1/C2/C3 spec)
  - docs/licensing.md §2 (GPL capsule = 별도 바이너리 / external ABI / no symbol linkage) + §4 (Tock capsule reuse Apache-2)
  - .brainstormings/20260812-173424-ipc-layer-design.md §0.5 (atomic-free composition — fault 통지도 IPC)
  - .brainstormings/20260618-223701-capability-bound-allocation.md (bulk reclaim I4/I6 — faulted capsule 회수와 동형)
  - .brainstormings/20260618-225136-soft-vs-hard-real-time.md (restart budget bounded)
  - CLAUDE.md §6 원칙 1(TCB 최소화) / 4(직접 HW access)
---

# Capsule fault isolation / restart — 정적 격리의 동적 확장

## §0 프레임

**최종 구현 목표 기준.**  현재 `capsules/`(16 tests, stage 0)는
**정적 격리 불변식**만 담고 있다 — C1(token 고유 소유), C2(capsule 은
mint 불가; `mint_into` 는 kernel-path), C3(자원 disjoint-or-explicit-
share).  이것은 **정상 상태(steady-state)의 capsule 이 자기 token 자원만
접근**함을 보장할 뿐, **capsule 이 죽었을 때**를 다루지 않는다.  본
문서는 그 **동적 차원(fault → restart)** 을 탐색.  현 non-goal 인 hot-
plug(Phase C)와 인접하지만, fault 는 그보다 근본적.

핵심 명제:

> **fault handling = C1/C2/C3 를 위반 없이 유지한 채, faulted capsule 을
> 상태에서 안전하게 제거하고 재구성하는 것.**  즉 fault 는 정적 불변식의
> *동적 보존* 문제로 환원된다.

## §0.5 원칙 정합

- **atomic-free composition**(IPC 발제 §0.5): fault 통지 = **seL4 fault
  endpoint → message**, restart 조율 = 트러스티드 supervisor 의 **IPC**.
  cross-CPU 협조에 atomic 안 씀 — 시리즈 관통 원칙 유지.
- **TCB 최소화**(원칙 1) + **직접 HW access**(원칙 4): tenant data 는
  driver stack 을 경유하지 않으므로(§6), driver fault 는 그 device 에
  국한 — tenant/accelerator 무영향.
- **capability isolation**: C1/C2/C3(정적) → fault(동적)로 확장.

## §1 정적 vs 동적 — 기존 C1/C2/C3 이 담지 못한 것

| | 정적 (현 capsules/) | 동적 (본 발제) |
|---|---|---|
| 질문 | capsule 은 무엇을 접근하나 | capsule 이 죽으면 그 자원은 |
| 불변식 | C1/C2/C3 | FA1~FA4 (§7) |
| 대상 | token / cap_set / resource | fault 감지·quiesce·revoke·restart |
| kernel-path | `mint_into`(C2) | revoke + **재-mint**(C2 보존) |

`mint_into` 가 kernel boot path 에 있어 capsule 이 자기 token 을 못
만든다는 C2 는, **restart 시 재-mint 도 kernel-path** 여야 함을 그대로
함의한다 — capsule 은 자신을 restart 할 수 없다(§3, FA4).

## §2 Tock 모델 vs seL4 — Y4 의 fault-isolation 스펙트럼

Tock 원본은 이중 격리: **capsule**(type-safe Rust, single AS, semi-
trusted, cooperative) + **process**(HW-isolated MPU, untrusted,
preemptive).  Y4 는 seL4 위에 있어 **full MMU/capability isolation 을
가용** — 따라서 driver tier 에 **격리 강도를 매핑**한다.

| tier | 코드 성격 | isolation 기법 | license |
|---|---|---|---|
| 1st (virtio-drivers / xhci 등) | Rust-native, 신뢰도↑ | type-safe capsule + seL4 thread(panic/hang 격리) | Apache-2 |
| 2nd (포팅 Rust) | Rust, 외부 유래 | **default = full domain; unsafe-audit gate 통과 시 in-AS capsule 로 승격**(§2.1) | mixed |
| 3rd (GPL Linux / rump NetBSD USB, stage 2) | foreign C, 미신뢰 | **seL4 protection domain(HW-isolated) 필수** | GPLv2 등 |

### ★ 핵심 관찰 — license 격리와 fault 격리의 일치

licensing.md §2 는 GPL capsule 을 **"별도 바이너리 / external capsule
ABI / no direct symbol linkage"** 로 규정(GPLv2 transitivity 차단).
그런데 "별도 바이너리 + external ABI + no linkage" = **별도 seL4
protection domain + IPC** 와 물리적으로 동일한 구성.  즉:

> foreign(GPL/rump) driver 에 대해 **license 가 요구하는 격리 경계와
> fault 가 요구하는 격리 경계가 정확히 일치**한다.  하나의 protection
> domain 이 두 목적을 동시에 만족 — 공짜 이중 이득.

Rust-native(1st) 는 type safety 로 메모리 안전은 보장되나 panic/무한
loop 는 못 막으므로 **seL4 thread 단위 격리**(preempt/kill 가능)로
보완.  foreign(3rd) 는 메모리 안전조차 신뢰 못 하므로 **full domain**.

## §2.1 (결정) 2nd-tier 격리 강도 — unsafe-audit gate

2nd-tier("포팅 Rust")를 tier label 로 일괄 배정하지 않는다.  **격리 강도의
진짜 결정 요인은 `unsafe` surface + provenance** 다 — driver 는 본질적으로
MMIO/DMA/volatile 접근에 `unsafe` 가 필요하고, in-AS(single address space)
type-safe capsule 의 격리는 **오직 Rust type safety 에만** 의존하므로
`unsafe` 하나가 공유 AS 전체를 손상시킬 수 있다.

**결정 — default-isolated, 감사로 승격**:
- **default = full protection domain**(HW-isolated).  외부 provenance Rust
  는 감사 전까지 3rd-tier 와 동일 취급(fail-closed).  bad `unsafe` 가
  있어도 별도 AS 라 MMU 가 공유 AS 손상을 차단.
- **in-AS type-safe capsule 로 승격** = **unsafe-audit gate** 통과 시에만:
  모든 `unsafe` 가 **granted MMIO/DMA window 만 건드리는 narrow HAL shim**
  에 갇혀 있고(그 window 는 IOMMU + Mmio token 이 이미 bound), review·
  문서화 완료.  이 조건은 **`unsafe` 를 지정 HAL 모듈 밖에서 금지하는
  lint 로 강제**(alloc §0.5 atomic-free lint 와 동형).
- gate 미통과여도 **safety net 다중**: IOMMU(DMA) + Mmio token + C1/C2/C3
  이 cross-capsule/tenant 영향을 bound → bad `unsafe` 의 blast radius 는
  자기 domain 내부로 제한.  full domain 은 여기에 **공유 AS 보호**를 추가.
- HAL shim(승격된 capsule 의 `unsafe`)은 `sel4_backend` 처럼 **trusted
  boundary** — safe-Rust 로직은 Verus 로 증명, unsafe shim 은 감사 대상.

귀결: tier→isolation 은 **고정 매핑이 아니라 default(격리) + 승격(감사)**.
1st(Y4-authored, 최소 audited unsafe)는 태생적으로 gate 통과 상태.

## §3 fault lifecycle — 감지 → fence → reset → revoke → restart

```
[감지]  seL4 fault endpoint (HW-isolated) │ Rust panic hook (type-safe)
   ▼
[fence]  IOMMU domain → sink/block + IR mask   ★ device 협조 불요, FA3 확보 (§4.1 A)
   ▼
[reset]  device reset ladder — FLR → SBR(group) → class-reset → dead (best-effort, §4.1 B)
   ▼
[revoke]  faulted capsule 의 cap_set(token) revoke → C1/C2/C3 재성립
   ▼        (메모리 회수 = capability-bound-alloc bulk reclaim 과 동형, I4/I6)
[restart]  kernel-path 재-mint (C2 보존) → 새 capsule 로 재열거 (reset 성공 시)
```

- **감지**: HW-isolated capsule 은 seL4 fault endpoint 로 fault 가 트러
  스티드 supervisor 에 전달; type-safe capsule 은 panic hook →
  supervisor 통지(둘 다 IPC, §0.5).
- **fence(★, safety)**: restart 이전에 device 의 DMA/IRQ 를 **먼저**
  차단 — **IOMMU domain 을 sink/block 으로 repoint + IR(interrupt
  remapping) mask**.  IOMMU/IR table 은 device 가 아니라 Y4 소유라
  device 협조가 필요 없어 **항상 성공**(§4.1 A).  이것이 FA3 를 보장하는
  진짜 수단 — 안 하면 zombie device 가 회수된(freed) 메모리로 DMA(Rust
  안전성이 절대 못 막는 경로, §4).
- **reset(availability)**: device 를 재시작 가능 상태로 되돌리는
  best-effort — FLR 있으면 FLR, 없으면 ladder(§4.1 B).  reset 실패 시엔
  device dead(그래도 fence 로 safety 는 이미 확보).
- **revoke**: faulted capsule 의 token 을 kernel 이 revoke → C1/C2/C3 이
  faulted capsule 제외 부분에서 재성립.  메모리는 lease 종료 때의 bulk
  reclaim(capability §I4/I6)과 **동일 메커니즘**으로 회수.
- **restart**: **stateless restart 우선** — driver 재열거(device 는 FLR
  로 이미 초기 상태라 stateless 가 clean).  checkpoint 는 상태 있는
  일부 driver 만(§8 미결).  재-mint 가 kernel-path 이므로 C2 보존.

## §4 IOMMU / DMA confinement — Rust 안전성 + HW 이중

현 `ResourceKind` = `Mmio`/`Irq`.  fault isolation 을 위해 **`Dma`(DMA
window) resource kind 추가**가 필요 — device 가 DMA 하는 대상 영역.

- **Rust type safety 는 device 의 DMA 를 막지 못한다** — buggy/malicious
  driver 가 device 를 시켜 임의 메모리로 DMA 가능.  capsule 코드가
  memory-safe 여도 device 는 그 밖.
- **IOMMU(VT-d / AMD-Vi)** 로 device DMA 를 capsule 의 granted window
  (Dma token)로 confine.  fault isolation = **Rust 안전성 + IOMMU 이중**.
- fault 시 순서 강제: **fence(IOMMU/IR) → Dma token revoke → 메모리 회수**
  (§3).  이 순서가 깨지면 freed 메모리로의 DMA(FA3 위반).  fence 는 IOMMU
  가 담당하므로 device reset(FLR) 성공 여부와 무관(§4.1).
- Dma window 를 capability-bound-alloc 의 partition zone 과 정합시킬지
  (device DMA 대상 메모리도 partition-keyed?)는 §8 미결.

## §4.1 (결정) FLR 미지원 device — safety 와 restartability 분리

FLR(Function Level Reset)은 PCIe **optional** capability(Device
Capabilities 의 "FLR Capable" bit) — 모든 function 이 지원하지 않는다.
그러나 "reset 을 못 하면 어쩌나"로 접근하면 막힌다.  **결정의 핵심
전환**: safety 불변식(FA3)과 restartability 를 **분리**한다.

### (A) safety = IOMMU 로 항상 달성 (device 협조 불요) ★
FA3("freed 메모리로의 zombie DMA 0")를 보장하는 진짜 수단은 device
reset 이 아니라 **IOMMU domain 무력화**다.  faulted device 의 IOMMU
domain 을 트러스티드 supervisor 가 **sink/blocking 구성으로 repoint** —
IOMMU/IR page table 은 device 가 아니라 **Y4 소유**이므로 device 협조가
전혀 필요 없다.  repoint 이후:
- DMA → sink page 또는 IOMMU fault(로깅) → **real 메모리 도달 불가**
- MSI/MSI-X → **interrupt remapping(IR)** mask/drop → interrupt storm 차단
- MMIO → 죽은 capsule 의 Mmio token revoke → 아무도 device 를 몰지 않음

즉 device 를 **"declaw"** — reset 불가여도 device 의 모든 출력(DMA/IRQ/
MMIO 영향)을 fence.  그 후 real 메모리는 **안전하게 회수 가능**.
**⟹ FLR 유무는 FA3(safety)에 영향을 주지 않는다.**  IOMMU 는 정상 운용의
confinement 수단(§4)이자 **fault 시의 universal quiesce 수단**.

### (B) restartability = reset ladder (availability, best-effort)
device 를 **다시 살릴 수 있는가**는 별개(availability)이고, 여기서 FLR
가 의미를 갖는다.  ladder (fine→coarse, fail-closed):
1. **FLR** capable → per-function reset(collateral 0).  최우선.
2. else **Secondary Bus Reset(SBR)** — 부모 bridge 하위 전체 reset.
   collateral 방지 위해 **enumeration 을 reset-group 단위로 정렬**:
   FLR-capable function = 자기 자신이 reset group; non-FLR function =
   부모 bridge SBR scope 를 공유하는 것들이 한 reset group.  **capsule/
   fault-domain 을 reset-group 경계에 맞춰 배정** → SBR 이 그 group 의
   capsule(들)만 reset(FA1 보존).  **한 reset group = 하나의 restart 단위**
   (group 내 capsule 은 함께 재시작).
3. else **device-class reset** — class driver 가 아는 register-level
   reset(NVMe `CC.EN=0` / AHCI HBA reset / xHCI `HCRST`).
4. else **device dead** — 복구 불가 표기.  단 (A)로 **메모리·safety 는
   이미 확보**되어 있으므로 dead 는 순수 availability 손실.  복구는
   coarser reset(maintenance window) 또는 물리 hot-unplug 로만.

### 귀결
- **"FLR 미지원" 은 절대 safety 를 위협하지 않는다** — 최악도 device 가
  un-restartable(availability), 메모리·격리는 IOMMU 로 불변.
- `pcie.rs` enumerator 는 각 function 의 **FLR bit 를 기록**하고 **reset
  group 을 계산**해야(capsule 배정 제약, B-2).  기존 열거 로직의 자연
  확장.
- 조율(IOMMU repoint / SBR)은 트러스티드 supervisor 의 kernel-path
  **단일 writer** 연산 — atomic 아님(§0.5 정합).

## §4.2 (결정) DMA window ↔ partition zone 정합

**관찰**: device 가 DMA 하는 대상 메모리는 **tenant data** 다(NVMe 가
tenant 저장소를 읽어 담는 buffer, NIC 가 받는 packet 등).  따라서 DMA
window 는 **다른 tenant 메모리와 똑같은 partition 규율**을 따라야 하며,
unpartitioned pool 에서 떼오면 cross-tenant 혼입 구멍이 된다.

**결정 — partition_id 가 allocator zone 과 IOMMU domain 을 잇는 단일 key**:
- **DMA window 는 소유 partition 의 zone 에서 할당**(capability-bound-alloc
  P_k, cross-tenant UAF 차단 I1) — 절대 unpartitioned 아님.
- device(를 모는 capsule)가 partition P_k 를 서빙하면 그 **IOMMU domain 은
  정확히 P_k 의 DMA window 만 매핑** → cross-partition DMA 가 **구조적으로
  불가능**(domain 에 타 partition 매핑이 없음).
- ⟹ **하나의 partition 모델, 두 hardware enforcer**: allocator/MMU 가
  **CPU 측**에서 partition 경계를 강제(I1)하듯, **IOMMU 가 device 측**에서
  같은 경계를 강제.  `partition_id`(P0..3, HIU 유래)가 양쪽을 index 하는
  단일 key.
- **partition 간 공유 device**(다중 tenant NIC 등) → **PASID**(PCIe process
  address space) 또는 **SR-IOV VF** 로 one device 에 per-partition IOMMU
  domain 부여.  `partition_id ↔ PASID/BDF` 매핑.  PASID 없는 공유 device 는
  per-partition DMA 격리 불가 → 그 device 는 **single-partition 전용**으로만
  배정(fail-closed).
- **fault 정합(§4.1)**: capsule fault 시 IOMMU domain fence = 그 partition
  의 DMA window fence; revoke 시 DMA 메모리 회수는 **lease-lifecycle bulk
  reclaim**(capability §I4/I6)으로 partition zone 에 반환 — §4.1·§3 과 동일
  경로.

**정합 불변식 (FA6, §7)**: `Dma token.partition == IOMMU domain.partition
== alloc zone.partition` (3자 partition coherence).  mint 시 강제, 불일치
= cross-partition DMA 경로 = tenant 격리 붕괴.

## §5 supervision / restart policy

- **supervisor**(트러스티드, kernel-path) = Erlang/OTP-유사 supervision
  tree.  자식(capsule) fault → 정책에 따라 restart.
- **restart limit**: crash loop 방어 — N회 초과 재시작 시 device 를
  **dead 로 표기**(무한 재시작 금지, RT budget 소모 방지).
- 조율은 fault endpoint **message**(§0.5, atomic 아님).
- **RT 정합**(real-time §): crashing driver 가 다른 tenant 의 deadline 을
  못 깨도록 **restart budget bounded** + fault handling 자체 WCET 상한.

## §5.1 (결정) reset-group 다중-capsule 재시작 조율

§4.1 B-2 의 SBR 은 reset-group 전체(부모 bridge 하위 non-FLR 멤버 전부)를
reset 하므로, group 에 여러 capsule 이 있으면 한 capsule fault 의 복구가
건강한 sibling 까지 reset 한다.  이 조율을 확정.

**전제**: group 멤버는 정의상 전부 non-FLR(FLR-capable 는 자기 자신이
group, §4.1 B-2).  즉 group 내 finer reset 은 device-class reset(B-3)뿐.

**2-phase 복구 (collateral 최소화, fail-closed)**:
1. **Phase 1 — 무-collateral**: faulted 멤버만 fence(IOMMU, 항상) →
   **device-class reset** 시도.  성공 시 그 capsule 만 restart, sibling
   무영향.
2. **Phase 2 — group escalation (Phase 1 불가/실패 시에만)**: supervisor 가
   **group 전 멤버를 먼저 fence**(각 IOMMU/IR declaw — sibling 이 reset
   도중 DMA 못 하도록) → **SBR** → **topology 순서(bridge→하위)로 재열거·
   재-mint**, 단 **dead 표기된 멤버는 skip**.  건강한 sibling 은 bounded
   outage 를 겪지만 fence 선행으로 **데이터 손상 0**.

**소유·권한**:
- **SBR capability(bridge control)는 supervisor 전용 trusted 자원** —
  capsule 은 보유 불가(C2 를 reset 권한으로 확장).  group reset 을 촉발할
  수 있는 유일 주체 = supervisor.
- restart-limit 은 **per-member**.  한 멤버가 임계 초과 → 그 멤버를 **dead**
  로 표기 = **영구 IOMMU fence + 재-mint 거부**(이후 group SBR 이 와도
  supervisor 가 그 멤버만 재구성 안 함, device 는 reset+fenced 로 방치).
  ⟹ **shared-SBR group 에서도 crash-loop 이 bound**(loop 유발 멤버가 먼저
  dead 되어 이후 group restart 에서 제외).

**불변식 (FA5, §7)**: 임의 SBR 이전에 reset-group 전 멤버가 IOMMU/IR fence
됨 — group reset 도중 어떤 멤버도 DMA 불가(sibling 데이터 손상 0).

## §6 accelerator 경계 불변식 — 왜 Y4 가 Linux 보다 driver fault 에 강한가 (★)

- **Y4**: tenant data 는 driver stack 을 경유하지 않는다(원칙 4 — 직접
  HW access, accelerator 는 별도 경로).  driver 는 capability-confined
  capsule → fault 는 그 device 에 국한, 최악도 "그 device dead".
  tenant/accelerator 는 무영향.
- **Linux**: driver 가 kernel 과 **동일 신뢰 경계** → driver bug =
  kernel panic / 임의 메모리 손상 위험.  driver 하나가 전체를 위협.
- 이 대비가 TCB 최소화(원칙 1)의 구체적 payoff — driver 를 신뢰 경계
  **밖**에 두므로 driver 품질과 tenant 안전이 분리된다.

## §7 verification — fault isolation 불변식

정적 C1/C2/C3 위에 동적 불변식 4개:

- **FA1 (fault containment)** — capsule `c` 의 fault 는 `c' ≠ c` 의
  token/자원에 무영향.  즉 C1/C2/C3 이 `c` 를 제외한 상태에서 보존.
- **FA2 (revoke → well_formed)** — faulted capsule 을 revoke 한 뒤
  `CapsulesState::well_formed()` 재성립(C1∧C2∧C3).
- **FA3 (fence-before-free)** — Dma token revoke·메모리 회수 **이전**에
  device 의 DMA/IRQ 가 IOMMU/IR 로 fence → freed 메모리로의 DMA 0.
  **device reset(FLR) 성공 여부와 무관하게 성립**(§4.1 A) — IOMMU/IR
  table 은 Y4 소유라 device 협조 불요.  IOMMU config invariant("granted
  window 외 DMA 불가")의 fault-time 특수화.
- **FA4 (restart 는 kernel-path)** — 재-mint 가 C2 보존(capsule 자가
  restart 불가).
- **FA5 (group-fence-before-SBR)** — 임의 SBR 이전에 reset-group 전
  멤버가 IOMMU/IR fence(§5.1) → group reset 중 어떤 멤버도 DMA 불가,
  건강한 sibling 데이터 손상 0.  FA3 의 group 확장.
- **FA6 (partition coherence)** — `Dma token.partition == IOMMU
  domain.partition == alloc zone.partition`(§4.2).  mint 시 강제; device
  측 partition 경계를 IOMMU 가 CPU 측 I1 과 동일하게 강제.

Y4-side supervisor·revoke·restart 로직 = **Verus**.  underlying HW
isolation(fault endpoint, domain 경계) = **seL4 integrity proof**
(Isabelle) — Y4.Sel4.Wrapper invariant 가 이 둘을 잇는다.

## §8 결정 / 미결 요약

**결정 방향(강)**:
- tier → isolation: 1st = type-safe capsule + seL4 thread; **2nd = default
  full domain, unsafe-audit gate 통과 시 in-AS capsule 승격(§2.1)**; 3rd =
  full protection domain(§2)
- **license 격리 = fault 격리 일치**(GPL/rump foreign driver, §2 ★)
- fault lifecycle = 감지 → **fence(IOMMU/IR, safety)** → **reset ladder**
  → revoke → **stateless restart**(§3)
- **FLR 미지원 device (결정 §4.1)**: safety(FA3)는 **IOMMU domain
  repoint** 으로 device reset 무관하게 **항상 확보**; restartability 만
  ladder(FLR → SBR reset-group → device-class reset → dead)로 degrade.
  "FLR 없음"은 availability 손실일 뿐 **safety 무영향**.  enumeration 이
  reset-group 계산(capsule 배정 제약, non-FLR 은 SBR scope 단위 group).
- **DMA↔partition 정합 (결정 §4.2)**: DMA window 는 소유 partition zone
  에서 할당; `partition_id` 가 allocator zone 과 IOMMU domain 의 단일
  key → cross-partition DMA 구조적 불가.  공유 device = PASID/VF, 불가 시
  single-partition 전용.  3자 coherence FA6.
- **reset-group 재시작 조율 (결정 §5.1)**: 2-phase(무-collateral class
  reset → group fence+SBR+topology 재열거, dead skip); SBR cap =
  supervisor 전용; restart-limit per-member → dead=영구 fence+재-mint 거부
  로 shared-SBR crash-loop bound.  FA5.
- IOMMU 필수 — `ResourceKind::Dma` 추가; 정상 confinement(§4)·fault 시
  universal quiesce(§4.1 A)·partition enforcer(§4.2); fence-before-free
- 조율 = IPC/fault endpoint(§0.5), restart budget bounded(§5)
- driver fault 는 accelerator/tenant 무영향(§6, 원칙 1·4)

**미결(설계 필요)**:
- checkpoint 필요 driver 존재 여부(대개 stateless 로 족하나 예외 조사)
- restart limit 수치(dead 표기 임계) — dead 후 복구 경로는 §4.1(B-4)로
  결정됨(coarser reset / hot-unplug)
- PASID 미지원 공유 device 를 single-partition 전용화할 때의 배정 정책
  구체(어느 partition 우선? time-sharing 배제?)

**⏳ 선행 의존**:
- 실 seL4 fault endpoint(capsules roadmap stage 1, kernel/ 제공)
- IOMMU 드라이버(VT-d/AMD-Vi) — Phase B/C 경계
- hot-plug(Phase C, 현 non-goal)와 restart 경로 통합

## §9 다음 발제 후보

- **side-channel isolation** (예정) — accelerator 공유 시 cross-tenant
  side-channel(cache/TLB/thermal/contention) 방어 + XChaCha20 masking 결합
- attestation / measured boot — tenant 가 TCB 를 암호학적으로 검증
