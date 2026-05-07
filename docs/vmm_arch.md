<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 VMM 아키텍처 — ARCH-II' (capsule-decomposed)

본 문서는 Y4 의 VMM 디자인을 정의한다.  D1a/D1d 의 monolithic VMM
가정을 폐기하고, **capsule-decomposed VMM + VeriSMo 검증 기법 영감**
으로 대체.  결정 근거는 `.claude-notes/amd-v-verified-survey.md` 의
ARCH 비교 (5 candidate architectures) 및 사용자 결정 (2026-05-04).

상위 안전장치 catalog (S1–S14) 는 `docs/amdv_safety.md` 가 그대로
보유 — 본 문서는 그 안전장치들이 *어디 (어느 capsule) 에* 구현되며
*어떻게 (capsule pattern) 격리* 되는지를 정의.

> **상태:** **v1.0 frozen** (2026-05-05, Phase 4 일괄 마킹).
> 핵심 결정 §1 의 10 axis + capsule 분해 §2 의 10 capsule + 1
> orchestrator + VeriSMo 검증 기법 §3 + PR split 매트릭스 §4 + repo
> 구조 §5 + 차별점 §6 모두 sign-off.  `docs/amdv_safety.md` 의 capsule
> 매핑 source.  짝 frozen doc = amdv_safety.md / sel4_fork_policy.md /
> verus_to_isabelle.md.  Phase C 진입 차단 1~4 단계 해제 (§7.5
> phase_plan §C cross-ref).

---

## 1. 핵심 결정

| 축 | 결정 |
|---|---|
| Base | seL4 (`third_party/sel4`, 15.0.0 핀) — 변경 없음 |
| **VMM 위치** | **Y4 의 capsules 패턴 안 — 10 capsule + 1 orchestrator (§2)** |
| Threat model | **VMM core (orchestrator) trusted, intercept handler 들 capsule 격리** (Tock-style defense in depth).  **Trusted scope = seL4 microkernel + D1a 패치 + orchestrator.**  capsule 들은 *cluster-scope-trusted* — 자기 capsule 스코프 안에서 신뢰, 다른 capsule / 다른 tenant 에 대해서는 **격리 보장 invariant 가 강제**.  Verus invariant (P2.2 의 capsule_dependency_acyclic 등) 가 cluster 안 capsule 사이 격리 보존을 inductive 로 닫음 |
| 검증 도구 | Verus (Y4 의 verus-bin) — VeriSMo 의 Microsoft fork 와 분리 유지 |
| **검증 기법** | **VeriSMo 의 2-layer concurrency 증명 패턴을 inversion 적용** (OSDI '24 paper §3-5).  **Upper layer = cross-tenant / cross-CPU concurrency** (다른 lease cluster 와의 isolation), **Lower layer = within-cluster capsule concurrency** (같은 lease 안 capsule 사이 cooperation).  자세한 매핑은 §3.1 |
| 검증된 primitives | Y4 자체 작성 — VeriSMo 코드 직접 import 0, 영감만 |
| AMD-V 코드 reference | bhyve / NVMM (BSD-2) 알고리즘 port |
| seL4 인터페이스 | D1a 의 `CONFIG_Y4_AMDV` raw-SVM cap (변경 없음) — fork policy 의 Strictly Additive.  **Intel VT-x 측은 동일 flag 가 `KernelVTX` (mainline 기존) 도 enable — vendor-neutral single-flag dispatch (`docs/cpu_virt_compat.md` §5)** |
| upstream contribute-back | seL4 mainline (D1a 의 raw-SVM C 패치) + paper (capsule + VeriSMo 기법 통합 사례).  **paper venue TBD** (§8 unresolved 항목 7) |
| 라이선스 | Apache-2.0 (Y4 single-license).  bhyve / NVMM 알고리즘 port 시 **BSD-2 attribution 보존** (NOTICE 갱신).  VeriSMo 코드 import 0 이라 Microsoft fork 라이선스 영향 0 |

---

## 2. Capsule 분해 — 10 개 capsule + 1 orchestrator

```
                  ┌────────────────────────────────────────┐
                  │  Y4 Lease Scheduler (기존)             │
                  │  → 본 cluster 를 binding 단위로 부여   │
                  └────────────────────────────────────────┘
                                    │
                                    ▼
                  ┌────────────────────────────────────────┐
                  │  vmrun Orchestrator (TCB core, trusted)│
                  │  - 7-step atomic sequence (S7)         │
                  │  - deadline (S4) / cpu pin (S5) /      │
                  │    thread group (S6) 강제              │
                  │  - 매 vmexit 후 적절한 intercept       │
                  │    capsule 로 dispatch                 │
                  └────┬──────────┬──────────┬─────────────┘
                       │          │          │
        ┌──────────────┼──────────┼──────────┼─────────────┐
        ▼              ▼          ▼          ▼             ▼
  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
  │ VMCB cap │ │ NPT cap  │ │ MSR      │ │ IO       │ │ Firmware │
  │ capsule  │ │ capsule  │ │ bitmap   │ │ bitmap   │ │ approval │
  │  (S1)    │ │  (S3)    │ │ capsule  │ │ capsule  │ │ capsule  │
  │          │ │          │ │  (S10)   │ │  (S11)   │ │  (S14)   │
  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘
                       ▲          ▲          ▲             ▲
                       │          │          │             │
        ┌──────────────┴──────────┴──────────┴─────────────┴─────────┐
        ▼              ▼          ▼          ▼             ▼         ▼
  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
  │ CPUID    │ │ NPF      │ │ Audit    │ │ Nested   │ │ Lifecycle│
  │ emul.    │ │ handler  │ │ capsule  │ │ request  │ │ capsule  │
  │ capsule  │ │ capsule  │ │  (S12)   │ │ pending  │ │  (S13)   │
  │  (S2)    │ │  (S3.4   │ │          │ │ flag     │ │          │
  │          │ │   IOMMU) │ │          │ │ capsule  │ │          │
  │          │ │          │ │          │ │  (S9)    │ │          │
  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘
```

각 capsule = `y4-capsules` workspace 의 멤버 (Tock-style isolation 적용,
C1–C3 invariant 자동 상속).  capsule 사이 통신 = `y4-ipc` 의 scheme +
LWKT msgport hybrid.

### 2.1 capsule 목록

| # | Capsule | 책임 | 안전장치 |
|---|---------|------|---------|
| 1 | **vmcb** | VMCB cap 격리 + Read/WriteRegister 화이트리스트 (§4) + TSC offset 관리 | S1, S8 |
| 2 | **npt** | NPT cap 격리 + huge-page 매핑 + IOMMU 통합 (S3.4 Phase D) | S3 |
| 3 | **msr-bitmap** | MSR bitmap immutable + profile (dev/production/certified) + custom whitelist + default-deny | S10 |
| 4 | **io-bitmap** | IO bitmap immutable + serial/PCI/CMOS/PIC 차단 + capsule mediation grant | S11 |
| 5 | **firmware-approval** | microcode/firmware mutation pending queue + Y4-VMM CLI 인터페이스 + audit | S14 |
| 6 | **cpuid-emul** | INTERCEPT_CPUID handler — per-VM CPUID emulation table | S2 (CPUID 부분) |
| 7 | **npf-handler** | INTERCEPT_NPF handler — guest 의 page fault 처리, NPT 와 짝지음 | S3 (NPF 부분) |
| 8 | **audit** | S12 ring buffer + S6.2/S9.3/S14 의 audit entry 통합. anomaly detector | S12 |
| 9 | **nested-request** | nested SVM 시도의 audit + Phase D 의 R-α/R-γ forward-compat hook (PollNestedRequest) | S9 |
| 10 | **lifecycle** | VCPU lifetime ↔ parent thread group lifetime hook + cap revocation 연동 | S6.1, S13 |

### 2.2 vmrun Orchestrator (trusted core)

`y4-vmrun-orchestrator` — workspace member, 가장 작은 trusted 구성 요소.

책임:
- seL4 의 raw-SVM syscall (D1a) 호출
- vmrun 직전 mandatory check (S2 mandatory mask, S4 deadline, S5 cpu_id,
  S6 thread group)
- 7-step atomic vmrun sequence (S7) — seL4 측에 위임
- vmexit 후 exit_code 검사 → 적절한 intercept capsule 로 IPC dispatch
- intercept capsule 의 응답 받아 다음 vmrun 으로 진행
- INTERCEPT_VMMCALL 의 RAX 화이트리스트 검증 + 각 기능 capsule 로 IPC
  forward (D-S2 결정)

#### 2.2.1 LoC budget (D)

`Y4_AMDV_ORCHESTRATOR_LOC_BUDGET` build-time const, **default 800 LoC**.
CI 자동 검사 — orchestrator workspace member 의 line count (주석 / 빈줄
/ test 제외) 가 budget 초과 시 build fail.  formal-first 원칙의 검증
가능한 TCB budget — 정성적 "수백 LoC" 표현 거부.

#### 2.2.2 IPC 인터페이스 추상 형태 (H)

orchestrator ↔ capsule 의 모든 통신은 **message-typed IPC** —  직접
메모리 공유 0, capsule cap 의 invoke 로만 가능:

```rust
// 추상 형태 (concrete msgport msg type 은 §8 unresolved 1)
trait CapsuleVerb<Cap, OpTag, Payload, Result> {
    fn invoke(cap: Cap, op: OpTag, payload: Payload) -> Reply<OpTag, Result>;
}
```

verb 묶음:
- `vmcb.read_register(reg_id) → value`
- `vmcb.write_register(reg_id, value) → ()`
- `npt.map(host_frame_cap, guest_paddr, perms, page_size) → ()`
- `msr_bitmap.access_intercept(msr, dir, gpr_state) → DispatchDecision`
- `cpuid_emul.handle(leaf, subleaf, gpr_state) → CpuidResponse`
- `firmware_approval.queue(op, payload, scope) → EntryId`
- ... (각 capsule 별 verb 4–8 개)

concrete msgport msg type / scheme verb encoding 은 P1.6.i 의 §8
unresolved 1 에서 결정.

### 2.3 Capsule trust model (B)

§1 Row 3 의 *cluster-scope-trusted* 를 sub-policy 로 풀어 적음:

| 주체 | trust 등급 | 격리 보장 |
|---|---|---|
| seL4 microkernel + D1a 패치 | **fully trusted** | TCB 본체.  Verus + Isabelle/HOL 양쪽 검증 |
| orchestrator | **fully trusted** | TCB 보조.  LoC budget 강제 + Verus 검증 |
| capsule (10 개 모두) | **cluster-scope-trusted** | 자기 cluster 안에서 신뢰, **다른 cluster / 다른 tenant 의 capsule 에 대해서는 적대적 가정**.  격리 invariant (`capsule_dependency_acyclic` + `cross_capsule_no_shared_memory` 등) 강제 |
| guest | **untrusted** | 모든 vmexit 가 다음 capsule 에 input validation |

**Capsule 사이 통신 규칙 (Tock C1 isolation 정합):**
1. message-typed IPC 만 — 직접 메모리 공유 0
2. Cap-mediated — capsule A 가 capsule B 호출 시 B 의 cap 보유 필요
3. orchestrator 가 cap 분배의 single source — capsule 사이 cap derive
   X (cap dep 트리 acyclic 보존)

### 2.4 Capsule 의존 그래프 — DAG 보장 (C)

```
                orchestrator
                     │  (single source — 모든 capsule cap 분배)
       ┌────┬────┬────┼────┬────┬────┐
       ▼    ▼    ▼    ▼    ▼    ▼    ▼
     vmcb npt msr-bm io-bm cpuid-emul npf-handler firmware-approval nested-request
       │    │    │    │    │           │             │                  │
       └────┴────┴────┴────┴───────────┴─────────────┴──────────────────┘
                              │  (sink-only, 다른 capsule 에 의존 X)
                              ▼
                     ┌──────────────┐
                     │  audit       │  (모든 capsule 의 entry sink)
                     │  lifecycle   │  (모든 capsule 의 destroy sink)
                     └──────────────┘
```

**의존 방향 명시:**
- orchestrator → 모든 capsule (single direction, capsule cap 분배)
- 8 개 intercept capsule (vmcb / npt / msr-bm / io-bm / cpuid-emul /
  npf-handler / firmware-approval / nested-request) → audit + lifecycle
  (entry/destroy sink 만)
- audit + lifecycle → **다른 capsule 에 의존 X** (sink-only)
- capsule 사이 horizontal 의존 0 → DAG 보장

Verus invariant `capsule_dependency_acyclic(cluster)` (P2.2 의 신규
invariant) 가 본 그래프의 acyclicity 를 inductive 로 닫음.

### 2.5 Capsule fault 시 cluster 거동 (E)

**v1.0 default policy: 모든 capsule fault → cluster 전체 lease revoke
(S13 sequence trigger).**

```
capsule.fault detected
    └─ lifecycle capsule 가 LeaseRevoke 신호 발화 (S13.3 master sequence)
    └─ S13.1 sibling destruction order 진입
    └─ cluster 전체 atomic teardown
```

근거:
- v1.0 단계에서 per-capsule restart policy 는 검증 표면 폭증 (recovery
  state machine + restart-after-fault invariant)
- fault 흔적이 다른 capsule state 에 contaminate 됐을 가능성 — fail-safe

**Phase D forward-compat:** per-capsule restart policy 도입.  capsule
별 (recoverable / non-recoverable) 분류 → recoverable capsule (예:
cpuid-emul) 만 restart, non-recoverable (vmcb/npt/audit/lifecycle) 는
v1.0 그대로 cluster revoke.  §8 unresolved 6 번 항목.

### 2.6 Capsule cluster scope (G)

**per-VM (lease) capsule cluster, single instance.**

| capsule | instance 수 |
|---|---|
| `vmcb` | **per-vCPU** (multi-vCPU VM 시 N 개) |
| `npt` | per-VM 1 개 (모든 vCPU 가 공유 NPT) |
| `msr-bitmap` | per-VM 1 개 |
| `io-bitmap` | per-VM 1 개 |
| `firmware-approval` | per-VM 1 개 |
| `cpuid-emul` | per-VM 1 개 |
| `npf-handler` | per-VM 1 개 |
| `audit` | per-VM 1 개 (단일 ring buffer, multi-vCPU 모두 적재) |
| `nested-request` | per-VM 1 개 |
| `lifecycle` | per-VM 1 개 |

multi-vCPU VM 시 vmcb capsule 만 N 개, 나머지 9 capsule 은 lease-scoped
shared.  audit ring buffer 단일화 + memory overhead 절감 + S12.8 의
per-CPU per-VM ring 패턴 정합 ("per-VM" = lease-scoped).

### 2.3 Lease integration

`hiu/lease` (Phase D blocked) 의 `LeaseCap` 이 본 capsule cluster 를
binding 단위로 부여:

```rust
struct LeaseCap {
    // ... 기존 필드 (partition_id, key, nonce, ...)

    // Per-vCPU capsule (G=a — multi-vCPU 시 N 개)
    vmcb_caps: Vec<Cap<VmcbCapsule>>,

    // Per-VM (lease-scoped) capsule, single instance
    npt_cap:                Cap<NptCapsule>,
    msr_bitmap_cap:         Cap<MsrBitmapCapsule>,
    io_bitmap_cap:          Cap<IoBitmapCapsule>,
    cpuid_emul_cap:         Cap<CpuidEmulCapsule>,
    npf_handler_cap:        Cap<NpfHandlerCapsule>,
    firmware_approval_cap:  Cap<FirmwareApprovalCapsule>,
    nested_request_cap:     Cap<NestedRequestCapsule>,
    audit_cap:              Cap<AuditCapsule>,       // sink-only (§2.4)
    lifecycle_cap:          Cap<LifecycleCapsule>,   // sink-only (§2.4)
}
```

lease 발급 = capsule cluster 생성 + lease cap 에 binding.
lease 회수 = atomic revoke chain (lifecycle capsule 가 trigger).

---

## 3. VeriSMo 검증 기법의 통합

### 3.1 2-layer concurrency 증명 패턴

VeriSMo paper (USENIX OSDI '24, **Best Paper Award**) §3-5 의 핵심:

> "We address [hypervisor concurrency] challenge by dividing
> verification into two layers. The upper layer handles the
> concurrent hypervisor execution, while the lower layer handles
> [VeriSMo]'s own concurrent execution."

Y4-VMM 에 적용 시 **threat model 이 정반대**:

| Layer | VeriSMo 의 의미 | Y4-VMM 의 의미 |
|---|---|---|
| Upper | hypervisor (untrusted) 의 concurrency 흡수 | **다른 CPU / 다른 tenant 의 cluster** 와의 concurrency 흡수 |
| Lower | VeriSMo (trusted) 자체 thread 의 concurrency | **본 cluster 안 capsule 들** 의 concurrency |

#### Verus 명세 작성 가이드 (S1~S14 전체 분류)

모든 invariant 는 *어느 layer 에서 성립* 하는지 명시.  S1~S14 의 분류
ground truth (Verus 작성 시 본 표를 reference):

| 안전장치 | Layer | 근거 |
|---|---|---|
| S1 VMCB pointer 격리 | Lower | within-cluster (vmcb capsule 의 cap-only access) |
| S2 Intercept floor | Lower | within-cluster (vmcb capsule 검증 + cpuid-emul 응답) |
| S3 NPT 격리 | **Upper** | cross-tenant (host frame 만 매핑, 다른 tenant frame 차단) |
| S4 vmrun deadline | Lower | within-cluster (vmcb metadata + orchestrator 비교) |
| S5 CPU 핀 + Migrate + offline re-pin | **Upper** | cross-CPU isolation, ASID race 차단 |
| S6 parent thread group 핀 | **Upper** | cross-tenant (다른 thread group 의 vmrun 차단) |
| S7 GIF host-managed | Lower | within-cluster (microkernel wrapper 본체) |
| S8 TSC offset 상한 | Lower | within-cluster (vmcb + msr-bitmap + lifecycle 협력) |
| S9 nested 차단 | Lower | within-cluster (5 capsule 협력) |
| S10 MSR bitmap immutable | **Upper** | cross-tenant (immutable 보장이 다른 tenant 의 변경 차단) |
| S11 IO bitmap immutable | **Upper** | cross-tenant (동일) |
| S12 audit trail | Lower | within-cluster (audit capsule 의 ordering, S12.8 의 AV12) |
| S13 lifetime ↔ parent TCB | **Upper** | cross-tenant (orphan frame 부재가 cross-tenant leak 차단) |
| S14 firmware approval | **Upper** | cross-VM (host-wide 영향, lease holder 격리) |

Upper 8 + Lower 6 분포.  Verus 모듈 분리: `proofs/verus/src/amdv/upper.rs`
+ `proofs/verus/src/amdv/lower.rs` 로 layer 별 정리.

### 3.2 Verus version 정합성

VeriSMo 는 Microsoft fork (`microsoft/verus`, 2022-2024 prototype) 사용.
Y4 는 Arch Linux `verus-bin` (현 시점 latest stable).  코드 import 0
이라 Verus version 충돌 없음 — 우리는 latest verus-bin 기준 작성, 영감만.

**Semantic 정합 caution:** VeriSMo 의 invariant statement 를 직접 인용
시 Y4 측 Verus 의미와 정합 검증 필요 — 특히 `Tracked<T>` / `Ghost<T>` /
`PointsTo<T>` 의 ghost state semantic 이 Microsoft prototype 과 latest
stable 사이 변동 가능 (이전 ghost API 가 stable 에서 재명명 또는
deprecation 된 경우 존재).  복사한 invariant 는 latest verus-bin 으로
re-prove 필수.  본 caution 은 `verus_to_isabelle.md` §8.1 (Tracked/Ghost
매핑) 과 짝.

### 3.3 차용 범위 (G)

명시적 분류:

**차용 ⊃** (Y4 가 영감으로 활용)
- 2-layer concurrency 분리 패턴 (§3.1)
- Ghost state 사용 패턴 (Tracked/Ghost 의 ownership 추적)
- Lock-free invariant proof 패턴 (per-CPU ring 의 ordering 등, S12.8 와 짝)

**차용 ⊄** (Y4 가 채택 X)
- VeriSMo 의 SEV-SNP-specific 정리 (현 호스트 SEV-SNP 미보유)
- Attestation report 검증 (Y4 는 lease cap 으로 대체)
- Secure boot 관련 정리 (Y4 는 Limine + seL4 boot 흐름)
- VeriSMo 의 SVSM-specific guest interface (Y4 는 hypervisor side)

### 3.4 Attribution

NOTICE 갱신 (`Acknowledgements / Methodology Inspiration` 신규 섹션 —
reuse manifest 와 분리, 코드 import 0 인 영감 자료 묶음) + 본 문서 §3
인용 + paper draft 시 BibTeX entry.

#### 인용 본문

```
Y4-VMM 의 2-layer concurrency 증명 기법은 다음 paper 의 영감:
  "VeriSMo: A Verified Security Module for Confidential VMs",
  Ziqiao Zhou et al., USENIX OSDI '24 (Best Paper Award).
  https://www.usenix.org/conference/osdi24/presentation/zhou
코드 import 0, 검증 방법론만 차용 (자세한 차용 범위는 §3.3).
```

#### BibTeX entry

```bibtex
@inproceedings{zhou2024verismo,
    title     = {{VeriSMo}: A Verified Security Module for Confidential {VMs}},
    author    = {Zhou, Ziqiao and Chen, Anjali and Delignat-Lavaud, Antoine and
                 Fournet, C\'edric and Kohlbrenner, David and Kohlweiss, Markulf and
                 Parno, Bryan and Protzenko, Jonathan and Ramananandro, Tahina and
                 Rastogi, Aseem and Swamy, Nikhil and Wittrock, Peter and Yang, Christoph M.
                 and others},
    booktitle = {Proceedings of the 18th USENIX Symposium on Operating Systems
                 Design and Implementation (OSDI '24)},
    year      = {2024},
    note      = {Best Paper Award},
    url       = {https://www.usenix.org/conference/osdi24/presentation/zhou}
}
```

(저자 목록 정확성은 paper draft 단계에서 USENIX 공식 BibTeX 와 재대조.)

#### 참조 자료 통합 표 (D)

본 §3 의 VeriSMo + bhyve / NVMM (§1 row 7) + Atmosphere (사실 확인 결과)
까지 포함한 모든 참조 자료의 단일 ledger:

| 자료 | 출처 / venue | 라이선스 | 차용 형태 | 위치 |
|---|---|---|---|---|
| VeriSMo | USENIX OSDI '24 (Best Paper) | (paper) | 검증 기법 영감 (§3.1, §3.3) — 코드 import 0 | NOTICE Acknowledgements + 본 §3 |
| bhyve (FreeBSD AMD-V VMM) | FreeBSD `sys/amd64/vmm/` | BSD-2 | AMD-V 알고리즘 reference | NOTICE reuse manifest, `~/y4-upstream-refs/bhyve/` (TBD) |
| NVMM (NetBSD AMD-V) | NetBSD `sys/dev/nvmm/` | BSD-2 | AMD-V 알고리즘 reference | NOTICE reuse manifest, `~/y4-upstream-refs/nvmm/` (TBD) |
| Atmosphere | mars-research/atmosphere (SOSP '25 artifact) | MIT | "Verus + AMD-V 가능성 시연" 으로 영감 (직접 채택 X — AMD-V 코드 publicly 0) | NOTICE Acknowledgements |
| seL4 | `third_party/sel4` | BSD-2 | binary as-is + D1a 패치 | NOTICE reuse manifest |
| Tock | crate 일부 + capsule pattern | MIT/Apache-2 | algorithmic reuse | NOTICE reuse manifest |
| DragonFly LWKT, Redox scheme | y4-ipc | BSD-3 / MIT | algorithmic port | NOTICE reuse manifest |

---

## 4. 안전장치 catalog 의 capsule 매핑 (PR split 갱신)

`docs/amdv_safety.md` §6 의 PR split 표 가 monolithic 에서 capsule 분해
형태로 재작성되어야 함.  매핑:

| 안전장치 | seL4 측 (D1a 패치) | orchestrator | capsule |
|---|---|---|---|
| S1 VMCB pointer 격리 | ◎ ObjectType_SVMVCPU | ○ cap reference 보유 | **vmcb** |
| S2 intercept floor | ◎ vmrun 직전 검사 | ◎ VMMCALL mediation (D-S2) | **vmcb** (mandatory mask), **cpuid-emul** (CPUID 응답) |
| S3 NPT 격리 | ◎ ObjectType_SVMNPT + cap-derived | △ | **npt** (static), **npf-handler** (dynamic) |
| S4 deadline | ◎ TSC interrupt 자동 + cmdline parser | ◎ L3 query + min(L1,L2,L3) 비교 | **vmcb** (L3 ceiling metadata, D-S4) |
| S5 CPU pin | ◎ Configure + vmrun 검사 + Migrate + ASID + offline detect | ◎ vmrun 직전 빠른 cpu_id 비교 | **lifecycle** (Migrate 진입점 + SMT grouping + offline re-pin policy) |
| S6 thread group | ◎ Configure + vmrun 검사 + ChangeParent + cap revoke | ◎ 검증 | **lifecycle** (parent dependency), **audit** (S6.2 거부 entry) |
| S7 GIF host-managed | ◎ vmrun wrapper 7-step (본체) | ◎ VMMCALL mediation (D-S2 와 동일) | — (microkernel 영역) |
| S8 TSC offset | ◎ Configure + RebaseTsc syscall | ○ | **vmcb** (offset / scaling 비트), **msr-bitmap** (TSC_RATIO/TSC_ADJUST 차단), **lifecycle** (Rebase ↔ Migrate 짝) |
| S9 nested 차단 | ◎ INTERCEPT_VMRUN/etc 강제 | ◎ exit_code 식별 + #UD inject | **vmcb** (intercept), **msr-bitmap** (EFER.SVME), **cpuid-emul** (SVM bit), **nested-request** (flag/Poll), **audit** (entry) |
| S10 MSR bitmap | ◎ ObjectType_SVMMsrBitmap + Create syscall | △ | **msr-bitmap** (본체), **audit** (S10.4 default-deny entry) |
| S11 IO bitmap | ◎ ObjectType_SVMIoBitmap + Create syscall | △ | **io-bitmap** |
| S12 audit | △ SVMExitInfo struct | ○ | **audit** (ring buffer + anomaly detector) |
| S13 lifetime | ◎ cap revoke 연동 | △ | **lifecycle** |
| S14 firmware approval | ◎ MSR/SMI/CPUID trap path | △ pending queue dispatcher | **firmware-approval** (queue + CLI) |

3-column 매트릭스로 PR 책임 분배 명확화.

**결정 ledger (D-S2, D-S4, 2026-05-04):**
- **D-S2** — `INTERCEPT_VMMCALL` hypercall mediation 은 별도 hypercall
  capsule 신설 X. orchestrator 의 thin dispatch entry (~50 LoC) 가 RAX
  화이트리스트 검증 후 각 기능 capsule 로 IPC forward.
- **D-S4** — L3 per-VM deadline ceiling 은 `vmcb` capsule 의 vCPU
  metadata 슬롯에 저장. 별도 deadline-policy capsule 신설 X. vmcb
  capsule 책임이 `VMCB raw 필드` + `vCPU 보조 metadata` 로 살짝 확장.

---

## 5. Repo 구조

### 5.1 Y4 워크스페이스 안 (core VMM)

| 경로 | 형태 | 내용 |
|---|---|---|
| `Y4/proofs/verus/src/amdv/` | 기존 디렉터리, 모듈 재정렬 | **2-축 layout (D=a):** `upper/` + `lower/` 안에 per-capsule 파일 (`upper/{npt,cpu_pin,thread_group,bitmap_immut,lifetime,firmware}.rs` + `lower/{vmcb,intercept_floor,deadline,gif,tsc,nested,audit}.rs`).  §3.1 Upper/Lower 분류 + capsule 매핑 양쪽 직접 표현 |
| `Y4/capsules/vmm-vmcb/` ~ `vmm-lifecycle/` | 신규 sub-crate **10 개** (C=a) | 각 capsule = 독립 workspace member.  의존 격리 + fault isolation 단위 + capsule 별 SPDX 헤더 명확.  기존 `y4-capsules` 는 test-fixtures + capsule pattern 공통 trait 으로 유지 |
| `Y4/vmrun-orchestrator/` | 신규 workspace member (B=a) | Trusted core, ~800 LoC budget (§2.2.1).  `Cargo.toml` workspace `members = [..., "vmrun-orchestrator"]` 추가 |

#### 5.1.1 Workspace dependency 표 (G)

| Crate | 의존 |
|---|---|
| `y4-capsules-vmm-vmcb` ~ `vmm-lifecycle` (10 개) | `y4-ipc` + `y4-alloc` + `y4-capsules` (test fixtures + 공통 trait) |
| `y4-vmrun-orchestrator` | 위 10 capsule sub-crate 모두 + `y4-ipc` + `y4-alloc` + seL4 raw-SVM cap binding |
| `y4-kernel` (root task) | orchestrator + lifecycle capsule (lease binding) |

기존 5 멤버 (`y4-alloc`, `y4-capsules`, `y4-ipc`, `y4-roottask`,
`y4-scudo-sys`) + 신규 11 (10 capsule + orchestrator) = **16 workspace
member**.

### 5.2 ISO 동봉 / 부팅 (F)

**동적 로드 모드.**  capsule binary 들은 ISO 의 `boot/modules/` 디렉
터리에 묶여 부팅되며, `kernel/` 의 module loader 가 부팅 후 lease
발급 시 필요한 capsule 만 동적 로드.

근거:
- y4-drivers 의 동적 로드 정책 (`y4_drivers_and_guest_plan.md`) 과 정합
- ISO 크기 절감 — 비활성 capsule 메모리 0 점유
- capsule 교체 단위 부팅 ISO 변경 0 (Phase D 의 mock HIU → RTL HIU
  swap 과 동일 패턴)

### 5.3 Y4 외부 sibling repo

| 경로 | 라이선스 | 내용 |
|---|---|---|
| `/home/ybi/y4-hypercall/` (E=a) | Apache-2.0 | **사용자측 CLI / API tooling repo (A=a 재정의 확정).** core VMM 코드는 Y4 워크스페이스 안 — 본 repo 는 외부 사용자 인터페이스 전용.  포함: S14 pending approval CLI (`pending list/show/dry-run/approve/reject/whitelist`), S12 audit query CLI, Phase D 의 R-α (`/dev/kvm` ioctl 프록시) + R-γ (paravirt agent), `y4-hypercall watch` 의 push-notification daemon |
| `/home/ybi/y4-drivers/` (이전 결정) | Apache-2.0 + GPL-capsule mixed | y4-driver-virtio-* / e1000e / ahci / nvme / xhci 등.  세부는 `y4_drivers_and_guest_plan.md` |
| `/home/ybi/y4-verus2isabelle/` (H=a) | Apache-2.0 | **Verus → Isabelle/HOL 번역기 — Y4 의 Verus 코드를 Isabelle/HOL 의 proper subset 언어로 번역하는 것만을 목표로 함**.  `verus_to_isabelle.md` 의 (T-i) statement-only `sorry` + (T-ii) `axiom` opt-in hybrid.  **General-purpose translator 는 본 repo 의 목표가 아님** — Y4 가 사용하는 Verus 기능 부분집합만 지원.  ~1500 LoC Rust 추정 |
| `/home/ybi/y4-upstream-refs/{bhyve,nvmm,dragonfly,redox-kernel,...}/` (기존) | (각 upstream 라이선스 보존) | read-only reference, Y4 가 알고리즘 port 시 검토용 |

### 5.4 .claude-notes 위치 정책 (I)

`Y4/.claude-notes/` — **git-tracked**.  gitignore X — 본 repo 의
의사결정 흔적 보존이 contribute-back paper + 미래 코드 리뷰의 reference.
설계 검토 도중 Claude Code 가 생성한 notes 가 commit 의 일부.

#### Sub-directory 구조

| 경로 | 성격 | 자세히 |
|---|---|---|
| `.claude-notes/` (root) | **Design memo / decision archive** — 갱신 종료된 historical record | `.claude-notes/README.md` |
| `.claude-notes/trackers/` | **Tracker / ledger** — 지속 갱신 파일 묶음 (CVE / 학술 논문 / venue deadline / 위협 발견 등) | `.claude-notes/trackers/README.md` |
| `.claude-notes/_completed/` | **Completed work archive** | (기존 디렉터리) |

현재 root: `amd-v-verified-survey.md` (ARCH 비교 ledger, ARCH-II' 채택
결정 record 포함, 갱신 종료된 archive).
현재 trackers/: placeholder (Phase C 진입 후 `power-prior-art-ledger.md` /
`power-paper-venue-tracker.md` / `power-threat-ledger.md` 등 신설).

본 분리는 **`.claude-memories/` 와 별개** — `.claude-memories/` 는
Claude Code 의 project memory 의 read-only mirror (`tools/git-hooks/`
의 pre-commit hook 이 자동 sync, CLAUDE.md §5 + §8 정합).

---

## 6. 차별점 (contribute-back 의 형태)

### 6.1 학술적 차별점

다음 5 항목에서 **공개된 prior art 의 부재** (2026-05-04 시점,
`.claude-notes/amd-v-verified-survey.md` 의 5-way 비교 + Atmosphere /
VeriSMo / Hyperkernel / NOVA / SVSM 사실 확인 결과 기준):

1. **seL4 + Y4 capsule pattern + VeriSMo 검증 기법** 의 통합
2. **trusted-hypervisor 모델** 위에서 VeriSMo 의 untrusted-hypervisor
   기법 *역적용* — **layer inversion (§3.1 표): Upper = cross-tenant
   (다른 cluster 와의 격리), Lower = within-cluster (capsule 협력).**
   두 threat model 의 verification 기법 통일성 시험
3. AMD-V 의 hypervisor-층 verification (VeriSMo 의 SVSM-층 verification
   과 보완)
4. **microkernel + capsule + lease** 가 결합된 verified VMM
5. **Verus → Isabelle/HOL proper-subset translator** —
   Z3-based push-button verification 결과를 Isabelle/HOL 의 interactive
   verification 환경으로 자동 import 가능한 도구 (Y4-scope 한정,
   `verus_to_isabelle.md` §0 scope clamp).  seL4 mainline contribute-back
   진입 장벽 절감의 별도 학술 산출물

### 6.2 산업 차별점

paper 의 "Why does this matter outside the academy" 답변:

- **5 형상 cross-portable host OS** — 단일 verified base 가 server-farm
  / 랩톱 / rack-mount / 핸드헬드+독 / 임베디드 SoC 모두 커버
- **WaveTensor 가속기 통합** — HIU lease capability 가 hypervisor 의
  primitive
- **Lease-based multi-tenancy** — 시간/자원 격리 단위가 OS 의 first-
  class concept
- **Apache-2.0 patent grant** — 상업 도입 시 patent retaliation 우려 0
- **Strictly Additive Fork policy (sel4_fork_policy.md)** — Y4 가
  도입한 변경이 upstream seL4 의 회귀 0 보장 → 기존 seL4 사용자가
  Y4 fork 도 안전하게 채택 가능

### 6.3 contribute-back 경로 (timeline + 의존)

| 산출물 | 게시 plan | timeline | 의존 |
|---|---|---|---|
| **C 패치 (D1a raw-SVM)** | seL4 mainline PR | Phase C 진입 직후 | sel4_fork_policy.md frozen + 회귀 게이트 통과 |
| **Verus 명세 (S1~S14 + capsule cluster)** | GitHub Y4 의 v1.0 frozen tag + paper artifact | Phase C 종반 | C 패치 머지 또는 review 진입 후 |
| **Verus → Isabelle/HOL skeleton** | Verus contribute-back PR 의 첨부 + `y4-verus2isabelle` repo 공개 | Phase C 종반 + Verus 명세 산출물 짝 | `y4-verus2isabelle` 도구 v1.0 + 50+ invariant round-trip 검증 통과 |

### 6.4 paper venue 후보 fit 분석

| Venue | fit 평가 | 비고 |
|---|---|---|
| **SOSP workshop (e.g. PLOS)** | ◎ 가장 자연 fit | "verified hypervisor on a verified microkernel" 같은 작은-paper 트랙 정확히 매칭.  **1 순위** |
| **PLOS** | ◎ workshop 진입 ↑ | SOSP 동반 workshop, programming languages and operating systems 교집합 |
| **SOSP main track** | ○ | 심사 통과 시 영향 ↑.  Phase C 종반 의 결과 강도 + 5 차별점 (§6.1) 의 evidence 충분성에 따라 시도 |
| **OSDI main track** | △ main track 어려움 | systems-evaluation 비중 ↑, paper 의 measurement 분량 요구.  evidence 충분 시 시도 |
| **ASPLOS** | △ HW 측면 약함 | 본 paper 는 verification 중심, ASPLOS 의 HW/SW co-design 자연 fit X.  WaveTensor 통합 paper 별도 시 후보 |
| **S&P (Oakland)** | ○ security framing 가능 | "verified hypervisor as TCB minimization" framing 시 자연.  단 paper 가 systems-heavy 라 review match 차순위 |

**Phase C 종반 시점에 1 순위 = SOSP workshop (PLOS), 2 순위 = SOSP /
OSDI main track 시도.**  본 결정은 §8 unresolved 항목 7 (paper venue
TBD) 와 짝 — Phase C 종반 의 결과 강도에 따라 재평가.

### 6.5 paper artifact 형식

**USENIX / ACM artifact badge 자격 충족 목표** (Available + Functional
+ Reproducible).

artifact 묶음:

| # | 산출물 | 형태 |
|---|---|---|
| (i) | Y4 GitHub repo 의 v1.0 frozen tag | git tag, immutable |
| (ii) | Verus 증명 산출물 | `proofs/verus/` 트리, `just verus` 1-command 재실행 |
| (iii) | qemu reproducibility script | `qemu-smoke` + Verus rerun + capsule cluster boot 의 single shell script |
| (iv) | Isabelle skeleton | `y4-verus2isabelle` 도구로 자동 생성, `.thy` 파일 묶음 |

### 6.6 재현성 패키지 위치

**`/home/ybi/y4-paper-artifact/`** (sibling repo, paper draft 시점에
Y4 의 frozen tag 에서 cherry-pick 으로 생성).  Y4 안에 `paper/`
포함시키지 않음 — Y4 본 트리 분량 ↓ + paper artifact 의 versioning
독립 + USENIX/ACM 의 artifact submission 시 별도 zip 패키징 자연.

paper 게시 후 GitHub release 에 동일 artifact mirror.

---

## 7. 동결 정책

본 문서는 v0 design draft.  `v1.0 frozen` 마킹 조건:

- §1 핵심 결정 9 항목 사용자 sign-off
- §2 capsule 분해 (10 capsule + orchestrator) 사용자 sign-off
- §3 VeriSMo 검증 기법 attribution 사용자 sign-off
- §4 PR split 매트릭스 사용자 sign-off
- §5 repo 구조 사용자 sign-off (`y4-hypercall` 재정의 포함)

frozen 후 `docs/amdv_safety.md` 의 §2 ABI + §6 PR split 도 본 문서와
정합 맞춰 갱신.  S1–S14 안전장치 content 는 그대로 보존.

---

## 8. 미해결 / 추가 결정 필요

### 8.1 Orchestrator ↔ capsule IPC surface (P1.6 결정 = c)

**Hybrid: msgport msg_type enum primary + scheme verb fallback (debug
build only).**

```rust
// 정상 path — 컴파일타임 결정, Verus exhaustive match 친화
enum CapsuleMsg {
    VmcbReadReg(VcpuId, RegId),
    VmcbWriteReg(VcpuId, RegId, u64),
    NptMap(GuestPaddr, HostFrameCap, Perms, PageSize),
    NptUnmap(GuestPaddr),
    MsrAccessIntercept(VcpuId, Msr, Direction, GprState),
    CpuidHandle(VcpuId, u32, u32, GprState),
    FirmwareApprovalQueue(VcpuId, FirmwareOp, Payload, Scope),
    NptHandleNpf(VcpuId, GuestPaddr, FaultBits),
    NestedRequestPoll(VcpuId),
    AuditAppend(AuditEntry),
    LifecycleNotify(LifecycleEvent),
    // ... (capsule 별 verb 4-8 개)
}

// debug build — scheme verb (string-keyed Redox-style)
#[cfg(feature = "debug-scheme-verbs")]
fn scheme_dispatch(verb: &str, payload: &[u8]) -> Result<...>;
```

정상 path 는 enum 으로 zero-overhead, debug build 만 scheme verb 추가
wire (live debugging / capture & replay).  Verus invariant
`exhaustive_match(handle_vmexit, CapsuleMsg)` 가 enum 의 모든 variant
에 대한 handler 를 강제.

### 8.2 Capsule 의존 그래프 acyclicity Verus invariant (P1.6 결정 = c)

**(a) topological order existence + (b) reachability cycle 부재 둘 다
lemma + 동치 증명.**

```verus
spec fn capsule_dep_edges(cluster: Cluster) -> Set<(CapsuleId, CapsuleId)>;

// (a) ground truth — paper 안의 표현
proof fn capsule_dependency_acyclic_topo(cluster: Cluster)
    ensures exists |order: Seq<CapsuleId>|
        order.no_duplicates() &&
        order.to_set() == cluster.capsules &&
        forall |i: CapsuleId, j: CapsuleId|
            capsule_dep_edges(cluster).contains((i, j))
                ==> order.index_of(i) < order.index_of(j)
{ ... }

// (b) inductive proof 본문 — reachability 정의 + cycle 부재
spec fn reachable(g: Set<(CapsuleId, CapsuleId)>, a: CapsuleId, b: CapsuleId) -> bool;

proof fn capsule_dependency_acyclic_reach(cluster: Cluster)
    ensures forall |a: CapsuleId, b: CapsuleId|
        a != b ==> !(reachable(capsule_dep_edges(cluster), a, b) &&
                     reachable(capsule_dep_edges(cluster), b, a))
{ ... }

// 동치 lemma
proof fn topo_iff_no_cycle(cluster: Cluster)
    ensures (∃ order ... topological) ⟺ (∀ a, b ... no cycle)
{ ... }
```

§2.4 의 ASCII 의존 그래프 (orchestrator → 8 intercept capsule → audit
+ lifecycle, 가로 의존 0) 이 본 invariant 의 ground truth.  proofs/verus/
src/amdv/upper.rs 또는 lower.rs 모듈 (P2.2 결정 시 확정) 에 entry.

### 8.3 vmexit 당 boundary crossing 상한 (P1.6 결정 = c)

**per-exit-code 별 상한 표 + 절대 상한 8 cross (safety net).**

```rust
// build-time const, Verus invariant 와 짝
const Y4_AMDV_MAX_BOUNDARY_PER_EXIT: [u8; N_EXIT_CODES] = {
    let mut t = [8u8; N_EXIT_CODES];  // 절대 상한 8
    t[EXIT_CPUID]   = 2;   // orchestrator → cpuid-emul → orchestrator
    t[EXIT_NPF]     = 4;   // → npf-handler → npt → audit → return
    t[EXIT_MSR]     = 3;   // → msr-bitmap → audit → return
    t[EXIT_VMMCALL] = 4;   // → orchestrator dispatch → 1 capsule → audit → return
    t[EXIT_VMRUN]   = 4;   // → nested-request → audit → orchestrator (#UD inject) → return
    t[EXIT_SMI]     = 4;   // → vmcb → firmware-approval → audit → return
    t[EXIT_INTR]    = 1;   // → orchestrator (host interrupt 직접 처리)
    t
};

// Verus invariant
proof fn boundary_count_bounded(e: VmExit)
    ensures boundary_count(handle_vmexit(e)) <= Y4_AMDV_MAX_BOUNDARY_PER_EXIT[e.code]
        && Y4_AMDV_MAX_BOUNDARY_PER_EXIT[e.code] <= 8;
```

근거:
- S4 deadline (≤ 100 ms) 과 정량 짝 — 100 ms / 8 cross / per-CPU IPC
  cost 가 budget 의 worst case
- per-exit 표는 정밀, 절대 8 은 safety net (새 exit code 추가 시 자동
  안전)
- CI 자동 검사 — capsule cluster build 가 boundary count overflow 발견
  시 fail

### 8.4 Capsule fault recovery (P1.6 결정 = a, Phase D defer)

**v1.0 spec 유지: 모든 capsule fault → cluster 전체 lease revoke (§2.5).**

Phase D 의 per-capsule restart 정책은 Phase D 진입 시점에 검토 — PCIe
passthrough / IOMMU programming capsule 도입 시 fault recovery 의 의미가
변하므로 함께 결정.

근거:
- v1.0 verification 표면 최소화 — recovery state machine 이 capsule
  fault 마다 새 invariant 도입
- fault 흔적이 다른 capsule state 에 contaminate 됐을 가능성 — fail-safe
- §2.5 의 forward-compat hook 이 Phase D 진입 시 spec patch 0 으로
  per-capsule restart 추가 가능

### 8.5 (이전 4) — `y4-hypercall` 재정의 후 기존 docs/phase_plan 정합 갱신

**닫힘 (2026-05-04 phase_plan.md 갱신 완료).**

### 8.6 (이전 5) — VeriSMo 이외 영감 자료 read-only refs

**~/y4-upstream-refs/{bhyve,nvmm}/ 추가 필요** — Phase C 진입 직전
실행할 작업, sign-off 종속 0.  메모: Phase C 진입 전에 실행.

### 8.7 (이전 7) — Contribute-back paper 게시 venue

**Phase C 종반 시점 재검토.**  현 추천 (§6.4): 1순위 SOSP workshop /
PLOS, 2순위 SOSP / OSDI main track 시도.

### 8.8 (Phase D) — 추가 검토 영역

Phase D 진입 시 필요할 수 있는 결정 (현 시점 unresolved 0, 진입 시
spec patch 로 추가):

- IOMMU programming capsule 의 fault model 과 §2.5 정합
- nested-request capsule 의 R-α / R-γ implementation 분리 (현 spec 은
  PollNestedRequest stub 만)
- audit capsule 의 disk-backed persistence (S12.4 forward-compat hook
  활성화)
- per-capsule restart policy (§8.4)
