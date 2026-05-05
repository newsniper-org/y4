<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 AMD-V (SVM) 안전장치 사양 — D1d

본 문서는 Y4 의 AMD-V 채택 결정 (`길 1 + sub-option D1d`) 의 정전
사양이다.  D1a (seL4 측 raw-SVM syscall + Y4-VMM 모든 로직) 만으로는
root task 가 raw cap 을 받는 즉시 host 메모리·시간·인터럽트를
오남용할 수 있으므로, **D1d = D1a + 13개 안전장치 카탈로그** 를 의무화.

결정 근거: `.claude-notes/amd-v-verified-survey.md`.
구현 위치: seL4 fork 측 SVM syscall 패치 + `y4-hypercall` repo + Verus
명세 (`proofs/verus/src/amdv/`).

**fork 호환성 contract:** seL4 측 패치는 `docs/sel4_fork_policy.md`
의 Strictly Additive Fork 원칙을 준수해야 한다.  새 syscall / cap
종류 / `CONFIG_Y4_*` build option 만 추가, 기존 코드 path 의 행동
변경 0.  upstream seL4 의 모든 회귀 테스트가 Y4 fork 에서도 pass
유지.

> **상태:** spec only — **VMM 아키텍처 ARCH-II' 채택 (2026-05-04)** —
> D1d 의 monolithic VMM 가정 폐기, capsule 분해 + VeriSMo 검증 기법
> 영감.  자세한 디자인은 `docs/vmm_arch.md`.  Atmosphere/VeriSMo 사실
> 확인 결과: VeriSMo 는 SEV-SNP SVSM (AMD-V hypervisor 아님), Atmosphere
> AMD-V 코드는 publicly 미발견 — 둘 다 직접 채택 X, **검증 기법만 영감**.
>
> **review 진행 중**: S1–S10 ✅ (안전장치 content 그대로 보존), S11–S14
> review 재개 + §2 ABI + §6 PR split 는 ARCH-II' 의 capsule 분해에
> 맞춰 재정렬 (content 변경 0).  구현 미시작.

본 문서는 14 안전장치 catalog 의 정전.  capsule 분해의 매핑은
`docs/vmm_arch.md` §4.

---

## 1. 위협 모델

D1a 의 raw-SVM cap 만 노출하면 root task 는 다음을 수행 가능:

1. VMCB pointer 를 임의 host 메모리에 가리켜 host 영역 read/write
2. intercept 비트 모두 끄기 → side-channel 과 권한 escalation
3. NPT 매핑으로 host frame 을 guest 가 보도록
4. vmrun 을 무한 반복하여 host 진행 차단
5. 다른 CPU 의 VMCB 를 hijack
6. clgi/stgi 로 host 인터럽트 금지
7. TSC offset 으로 wall-clock attribution 회피
8. nested guest 활성화로 검증 표면 폭증
9. MSR permission map 변경으로 host MSR 노출
10. IO permission 으로 호스트 시리얼/디스크 직접 접근

D1d 는 위 10 개 위협 각각에 대한 cap-typed 또는 invariant-enforced
방어를 제공.

---

## 2. seL4 측 ABI (D1a 패치의 contract)

### 새 객체 종류 (cap-typed)

```
ObjectType_SVMVCPU      — SVM VCPU (VMCB 의 cap-typed wrapper)
ObjectType_SVMNPT       — Nested Page Table root (host VSpace 파생)
ObjectType_SVMMsrBitmap — MSR permission bitmap (4 KiB, immutable post-init)
ObjectType_SVMIoBitmap  — IO permission bitmap (12 KiB, immutable post-init)
```

각 객체는 `seL4_Untyped_Retype` 으로 생성.  부모 untyped 의 권한이 곧
객체 권한.

### 새 syscall

```
seL4_X86_SVMVCPU_Configure(vcpu, msr_bitmap, io_bitmap, npt_root,
                           parent_cspace, parent_vspace, cpu_id)
seL4_X86_SVMVCPU_WriteRegister(vcpu, reg_id, value)
seL4_X86_SVMVCPU_ReadRegister(vcpu, reg_id) -> value
seL4_X86_SVMVCPU_Run(vcpu, deadline_ns) -> SVMExitInfo
seL4_X86_SVMVCPU_InjectInterrupt(vcpu, vector, type)
seL4_X86_SVMVCPU_Migrate(vcpu, new_cpu_id)        # S5.2 — cross-CPU 이동
seL4_X86_SVMVCPU_ChangeParent(vcpu, new_cspace, new_vspace)  # S6.3 — ownership transfer
seL4_X86_SVMVCPU_RebaseTsc(vcpu, new_offset)      # S8.2 — Migrate 후 TSC rebase
seL4_X86_SVMVCPU_PollNestedRequest(vcpu) -> Option<NestedRequest>
                                                 # S9.4 — Phase D R-α/R-γ hook
                                                 # v1.0 에서는 항상 None
seL4_X86_SVMMsrBitmap_Create(untyped, profile, custom_whitelist)
                                                 # S10.3 — bitmap 생성
seL4_X86_SVMNPT_Map(npt, host_frame_cap, guest_paddr, perms, page_size,
                    device_caps?)   # device_caps 는 Phase D 의 IOMMU
                                    # 통합 (S3.4); v1.0 은 빈 슬롯 OK
seL4_X86_SVMNPT_Unmap(npt, guest_paddr, page_size)
```

`page_size ∈ { SmallPage 4 KiB, LargePage 2 MiB, HugePage 1 GiB }`
(S3.2).  per-NPT entry 수 ≤ `Y4_AMDV_MAX_NPT_ENTRIES` (build-time
const, default 65536, S3.3).  `device_caps` 는 v1.0 에서 무시,
Phase D 진입 시 IOMMU TLB 동시 program (S3.4).

`Run` 이 vmrun 을 발화. `deadline_ns` 는 caller-supplied nanosecond
budget — seL4 가 boot 시 보정한 TSC frequency 로 cycles 변환. ns 단위
선택은 호스트 클럭 다양성 흡수 + 형상 cross-portable.

`Configure` 는 1회성 (재호출 거부). `parent_cspace` + `parent_vspace`
가 vmrun 호출 가능한 TCB 의 thread-group 경계 (동일 CSpace + VSpace
공유하는 TCB 만 호출 가능).

`Read/WriteRegister` 는 아래 §4 의 화이트리스트 만 허용.

### Bootloader cmdline overrides (모든 ns/timeout/limit 의 3-계층 ceiling 패턴)

Limine multiboot1 cmdline 의 `y4.amdv.*` 키들이 Y4 fork 의 seL4 boot
경로에서 파싱되어 build-time const 의 runtime override 로 동작.  미지정
시 build-time default 적용 (= upstream 동작과 동일).

| 키 | 의미 | build-time default |
|---|---|---|
| `y4.amdv.max_deadline_ns` | S4 의 L2 (vmrun deadline 상한) | 100 ms |
| `y4.amdv.slack_ns` | S4 bounded slack | 50 µs |
| `y4.amdv.max_slack_ns` | S4 slack absolute upper | 1 ms |
| `y4.amdv.pending_timeout_s` | S14 pending approval timeout | 60 s |
| `y4.amdv.max_pending_entries` | S14 pending queue 크기 | 16 |
| `y4.amdv.max_npt_entries` | S3.3 per-NPT entry 상한 | 65536 |
| `y4.amdv.max_tsc_offset` | S8.1 TSC offset 절대값 상한 | 2^62 |
| `y4.amdv.max_custom_msr_entries` | S10.3 per-bitmap custom MSR 수 상한 | 32 |

각 키는 모두 `Y4_AMDV_*_BUILD` (build-time const, 절대 상한) 보다 작아야
함.  위반 시 boot 시점에 panic + 명확한 에러 메시지.

cmdline parser 는 Y4 fork 의 새 함수 — 기존 cmdline 처리 변경 0.
미지정 시 build-time default 적용.

### seL4 측 mandatory checks (vmrun 직전)

1. `vcpu.cpu_id == current_cpu()` — 잘못된 CPU 거부
2. `current_tcb.cspace == vcpu.parent_cspace && current_tcb.vspace ==
   vcpu.parent_vspace` — parent thread group 안의 TCB 만 vmrun 가능.
   Worker pool 패턴 지원 (main thread `Configure`, worker thread `Run`)
3. VMCB 의 control bits 가 §3 의 mandatory mask 를 모두 만족 (mismatch 시 거부)
4. `deadline_ns <= Y4_AMDV_MAX_DEADLINE_NS` (build-time ceiling) — 무한
   vmrun 차단. 기본 100 ms, 형상별 override 가능
5. host state save area 가 seL4 가 관리 (root task 가 못 만짐)

---

## 3. 13 개 안전장치 카탈로그

### S1 — VMCB pointer 격리

**ARCH-II' 매핑:** `vmcb` capsule (단독). orchestrator 는 cap reference
만 보유, raw 메모리 주소 X.

VMCB 가 별도 cap (`SVMVCPU`) — root task 는 VMCB 의 raw 메모리 주소를
모름.  모든 VMCB 필드 접근은 `SVMVCPU_Read/WriteRegister` 또는
`Configure` 만 허용.  필드 화이트리스트는 §4.

### S2 — Intercept floor (mandatory mask)

**ARCH-II' 매핑:**
- `vmcb` capsule — intercept_words 의 16-bit mandatory mask 검증 (Configure
  / WriteRegister 진입점에서 거부).
- `cpuid-emul` capsule — `INTERCEPT_CPUID` trap 시 per-VM CPUID emulation
  table 응답 (아래 § "CPUID emulation 정책" 본문 전체).
- **orchestrator** — `INTERCEPT_VMMCALL` trap 시 RAX (hypercall 번호)
  화이트리스트 검증 후 각 기능 capsule 로 IPC dispatch (D-S2 결정,
  2026-05-04).  별도 hypercall capsule 신설 X — orchestrator 의 thin
  dispatch entry 한 줄.

VMCB 의 intercept_words 가 다음 비트를 **반드시** 1:

| 비트 | 의미 |
|---|---|
| `INTERCEPT_NMI` | NMI 가 host 로 가도록 |
| `INTERCEPT_SMI` | SMI 마찬가지 |
| `INTERCEPT_INIT` | INIT 마찬가지 |
| `INTERCEPT_SHUTDOWN` | guest triple fault 가 host fault 로 |
| `INTERCEPT_VMRUN` | nested vmrun 차단 (S9) |
| `INTERCEPT_VMLOAD/VMSAVE/CLGI/STGI` | nested SVM 차단 |
| `INTERCEPT_HLT` | guest hlt 가 vmexit (CPU 양보) |
| `INTERCEPT_NPF` | nested page fault → host VMM |
| `INTERCEPT_CPUID` | host fingerprinting / microcode rev / undocumented feature 추출 차단. Y4-VMM 측 CPUID emulation table 이 응답 |
| `INTERCEPT_INVD` | guest INVD (writeback 없는 캐시 무효화) 차단 — host cache 손상 방지 |
| `INTERCEPT_WBINVD` | guest WBINVD spam 차단 — host cache DoS 방지 |
| `INTERCEPT_MSR_PROT` | MSR bitmap 강제 활용 (S10 과 짝). 무차별 MSR access 차단 |
| `INTERCEPT_VMMCALL` | guest hypercall 을 Y4-VMM 으로 trap. 화이트리스트 mediation (S7.2) |

총 **16 비트 mandatory**.  이 중 1 비트라도 0 이면 vmrun 거부.

**RDTSC / RDTSCP 미포함:** hardware TSC_OFFSETTING 이 S8 의 TSC offset
을 자동 적용하므로 intercept 불필요. v1.x 의 고보안 프로필에서
opt-in `Y4_AMDV_INTERCEPT_RDTSC=ON` 가능.

**INVLPG / PAUSE / MWAIT / MONITOR 미포함:** NPT 격리 + 성능 영향
큼.  v1.x 의 고보안 프로필 opt-in 후보.

### CPUID emulation 정책 (Y4-VMM 측)

`INTERCEPT_CPUID` 가 trap → Y4-VMM 의 per-VM **CPUID emulation table**
이 응답.  default 정책:

| Leaf | default 노출 |
|---|---|
| 0x0 / 0x1 vendor / family / model | ✅ 그대로 |
| 0x1 stepping / serial bits | ❌ 0 으로 마스크 (fingerprinting 비트 제거) |
| 0x1 ECX/EDX standard feature flags | ✅ 그대로 (AVX/SSE/NX 등) |
| 0x3 CPU serial number (deprecated) | ❌ 항상 0 |
| 0x4 cache topology | ✅ vCPU 수에 맞춰 가공 |
| 0x7 extended feature flags | ✅ 그대로 |
| 0xB / 0x1F extended topology | ✅ vCPU 토폴로지 |
| 0x15 / 0x16 TSC frequency | △ S8 offset 적용 값 |
| **microcode revision** (0x1 EAX low bits + MSR 0x8B) | ❌ default 0 / "unknown" — S14 의 pending approval 경유로만 노출 |
| 0x4000_0000 hypervisor leaf | ✅ "Y4 hypervisor" 식별자 + capability bits |
| 0x8000_0001+ AMD extended | ✅ 그대로 (단 stepping 마스크) |

Y4-VMM 의 emulation table 은 per-VM 정책 — 신뢰 정도에 따라 더 넓은
노출 가능 (S14 의 user approval 과 짝).

### S3 — NPT 격리 (host frame 만 매핑 가능 + huge-page + entry 상한 + IOMMU 통합)

**ARCH-II' 매핑 (2 capsule 책임 분담):**
- `npt` capsule — *static* 책임. `SVMNPT` cap 객체 생성/해체, cap derivation
  검증 (S3.1), huge-page 매핑 검증 (S3.2), entry 상한 강제 (S3.3),
  Phase D 의 IOMMU 통합 (S3.4).  S3 의 거의 전체 본문.
- `npf-handler` capsule — *dynamic* 책임. INTERCEPT_NPF trap 시 vmexit
  디스패치를 받아 page allocation / mapping update / fault inject 결정.
  npt capsule 에 `SVMNPT_Map` 호출 forward.

`SVMNPT_Map` 의 `host_frame_cap` 은 부모 root task 의 VSpace 에서 파생된
frame cap 만 받음.  호스트 kernel frame, IOMMU frame, 다른 root task 의
frame 은 cap 자체로 차단.

**Cap derivation 강제 (S3.1):**
- `host_frame_cap` 의 derivation tree 가 invoking root task 의 untyped
  를 거슬러 올라가야 함 (seL4 cap derivation 자동 검증)
- `SVMNPT` root cap 도 동일 — `seL4_Untyped_Retype(parent_untyped,
  ObjectType_SVMNPT)` 의 부모 untyped 가 root task 의 untyped

**Huge-page 매핑 (S3.2):**

`KernelHugePage = ON` (D1d 결정) 와 정합.  page size 인자 추가:

```
seL4_X86_SVMNPT_Map(npt, host_frame_cap, guest_paddr, perms, page_size)
    page_size ∈ { SmallPage (4 KiB), LargePage (2 MiB), HugePage (1 GiB) }
```

검증:
- guest_paddr 가 page_size 정렬 (4 KiB / 2 MiB / 1 GiB)
- host_frame_cap 의 frame 크기가 ≥ page_size
- 미정렬 / 부족 시 `InvalidArgument`

`perms` 는 `READ` / `WRITE` / `EXECUTE` 비트 — caller 가 필요한 비트만,
host_frame_cap 의 권한이 superset 이어야.

**NPT entry 상한 (S3.3):**

per-VM 무한 NPT 매핑 → host page table 메모리 DoS.  상한 강제:

```
Y4_AMDV_MAX_NPT_ENTRIES   build-time const, default 65536
```

`SVMNPT_Map` 가 NPT 의 현재 mapping 수가 ceiling 이상이면 `NoMemory`
반환.  형상별 override (rack-mount 는 더 높게, handheld 는 더 낮게).

**IOMMU 통합 (S3.4 — Phase D 마일스톤):**

`SVMNPT_Map` 이 NPT 매핑과 동시에 device IOMMU TLB 의 대응 entry 를
program (`SVMNPT_Map` 의 `device_caps` 인자로 device cap 들 전달).
guest 가 자체 driver 로 hardware DMA 시 IOMMU 에서 동일 격리 강제 —
DMA reentrant 공격 차단.

**Phase 분리:** S3.1, S3.2, S3.3 는 Phase C (D1d 첫 패치).  S3.4 는
**Phase D 마일스톤** — Phase D 의 PCIe passthrough 인프라 + IOMMU
programming capsule 와 함께.  v1.0 spec 은 S3.4 의 인터페이스 형태만
미리 명시 (forward-compat).

### S4 — vmrun deadline (preemption 보장, 3-계층 ceiling)

**ARCH-II' 매핑:**
- seL4 측 (D1a) — TSC interrupt 자동 강제 vmexit + cmdline parser (L1
  build-time const + L2 runtime cmdline 적용).
- `vmcb` capsule — **L3 per-VM ceiling 을 vCPU metadata 슬롯에 저장**
  (`SVMVCPU_DeadlineCeiling` cap 의 보조 데이터, VMCB raw 필드 아님).
  D-S4 결정 (2026-05-04) — 별도 deadline capsule 신설 X, vmcb capsule 의
  책임을 `VMCB raw 필드` + `vCPU 보조 metadata` 로 살짝 확장.
- **orchestrator** — vmrun 직전 vmcb capsule 에서 L3 query → `min(L1,
  L2, L3)` 비교 → `deadline_ns` 위반 시 reject.

`Run(vcpu, deadline_ns)` 의 `deadline_ns` 가 다음 3 계층 ceiling 의
**최소값** 이하.  ns 단위 (호스트 클럭 다양성 흡수 + 형상 cross-portable).

#### 3-계층 ceiling

| 계층 | 결정 시점 | 결정 주체 | 의미 |
|---|---|---|---|
| **(L1) `Y4_AMDV_MAX_DEADLINE_NS_BUILD`** | build-time const | 형상 빌더 | 절대 상한.  형상별 cmake override.  default 100 ms |
| **(L2) `y4.amdv.max_deadline_ns` cmdline** | boot-time | 시스템 운영자 | runtime ceiling, ≤ L1.  Limine multiboot1 cmdline 으로 전달.  default = L1 |
| **(L3) `SVMVCPU_DeadlineCeiling` per-VM cap** | `Configure` 시점 | VM 소유자 (사용자) | per-VM 상한, ≤ L2.  default = L2.  더 낮춰서 자기 워크로드에 맞춤 |

**검증:** `deadline_ns ≤ min(L1, L2, L3)` 이어야 vmrun 발화. 위반 시
`InvalidArgument`.

#### Bootloader cmdline override

Limine `module_string: y4.roottask=v0` 와 같은 multiboot1 cmdline 슬롯
사용.  Y4 fork 의 seL4 boot 코드가 cmdline 파싱하여 `Y4_AMDV_*` runtime
ceiling 들을 적용:

```
y4.amdv.max_deadline_ns=100000000     # L2: 100 ms
y4.amdv.slack_ns=50000                # bounded slack: 50 µs
y4.amdv.pending_timeout_s=60          # S14 pending timeout
```

미지정 시 build-time default (L1).  형상별로 단일 binary + 다른
cmdline 으로 운영 — interactive 디바이스 vs server 분리.

**Strictly Additive Fork 정합:** cmdline 파싱은 새 함수, 기존 cmdline
처리 변경 0.  미지정 시 동작은 upstream 과 동일.

#### Bounded slack

`deadline_ns + slack_ns` 안에 항상 vmexit 보장.  `slack_ns` 는:
- bootloader cmdline `y4.amdv.slack_ns` (default 50 µs)
- seL4 boot 시 실측 (vmrun → timer-driven exit 의 평균 + 3σ)
- ceiling 은 build-time const `Y4_AMDV_MAX_SLACK_NS_BUILD` (default 1 ms)

slack 의 의미: host interrupt latency + VMCB save/restore + scheduling
delay 의 합.  실측 후 cmdline 으로 박는 패턴.

#### `deadline_ns = 0` (single-instruction step)

- **`KernelDebugBuild = ON`** 빌드 (현 `boot/x86_64-debug.cmake` 처럼):
  허용 — debugger / instruction-level fuzzer 용.
- **`KernelDebugBuild = OFF`** 빌드 (production): **거부** — `InvalidArgument`.
  side-channel timing attack 의 1-instruction 분해능 차단.

#### 형상별 ceiling 권고

build-time L1 의 형상별 권장값:
- real-time / wave-aligned: 10 ms
- desktop interactive: 100 ms (default)
- batch / server: 1 s
- handheld+dock 저전력: 50 ms

운영자가 cmdline (L2) 으로 미세 조정, 사용자가 per-VM (L3) 으로 더
낮춤.

### S5 — CPU 핀 (+ Migrate + cpu_id semantics + offline re-pin)

**ARCH-II' 매핑:**
- seL4 측 (D1a) — `Configure` / vmrun 직전 cpu_id 검사 + `Migrate` 의
  ASID 재할당 + cap revoke 연동 + CPU offline detection.
- `lifecycle` capsule — Migrate 진입점 + atomic 5-step 의 상위 정합 +
  SMT-aware grouping policy 시행 (S5.1) + offline re-pin 정책 결정 +
  multi-vCPU 의 topology-aware placement (S5.4).
- **orchestrator** — vmrun 직전 빠른 cpu_id 비교 (lifecycle capsule
  cross-call 회피, 정밀 검증은 seL4 측이 처리).

`Configure` 시점에 `cpu_id` 고정.  vmrun 호출 시 `current_cpu() !=
vcpu.cpu_id` 면 거부.  cross-CPU vmrun race 차단 + ASID 혼선 차단 +
wave-aligned scheduling 결정성 유지.

#### S5.1 — `cpu_id` 의 정확한 의미 (logical APIC ID)

`cpu_id` = **logical CPU = unique APIC ID** (SMT 형제 각각 별도 ID).
seL4 측 `current_cpu()` 도 같은 의미.  AMD CPU topology 의 layer:

| Layer | 예시 (Ryzen 7000 series) |
|---|---|
| Socket | 0 |
| CCD (Core Complex Die) | 0..N |
| CCX (Core Complex) | 0..M per CCD |
| Core | 0..7 per CCX |
| **Logical CPU (SMT thread)** | 0..15 per CCX (SMT 활성 시) ← **`cpu_id`** |

**SMT side-channel 권고:**
- L1/L2 cache 가 SMT pair 사이 공유 → guest 의 SMT sibling 이 host
  thread 일 때 cache timing attack 가능
- 권고: 같은 core 의 두 SMT thread 가 모두 guest 측이거나 모두 host
  측이도록 묶음 (sibling-grouping).  build-time option
  `Y4_AMDV_SMT_GROUPING ∈ { isolate-pairs (default), allow-mixed }`
- `isolate-pairs` 일 때 `Configure(cpu_id)` 는 free SMT pair 의 한 쪽
  만 받음, 다른 쪽 자동 차단 — sibling 도 같은 VM 에 묶거나 host
  격리 idle

#### S5.2 — `SVMVCPU_Migrate` syscall

```
seL4_X86_SVMVCPU_Migrate(vcpu, new_cpu_id) -> Result<(), Y4Error>
```

전제 조건:
- vcpu 가 현재 vmrun 중 아님 (vmexit 후 다음 vmrun 전)
- `new_cpu_id` 가 online + invoking thread group (S6) 의 cap 권한 안
- `Y4_AMDV_SMT_GROUPING = isolate-pairs` 면 new_cpu_id 의 SMT sibling
  도 같이 검증

원자적 동작:
1. 현 CPU 의 VMCB cache flush + ASID invalidate
2. VMCB state (host save area + guest state) 를 lock
3. `vcpu.cpu_id ← new_cpu_id`
4. new CPU 의 ASID 공간에서 새 ASID 할당
5. cap 권한 unlock

이후 vmrun 은 `new_cpu_id` 에 pin 된 TCB 만 호출 가능.

검증 부담: 작음.  ASID 재할당의 정확성 + lock 의 atomicity 만 추가
invariant.

#### S5.3 — CPU offline 자동 re-pin

호스트 CPU 가 offline (deep C-state / hotplug / 온도 throttle) 될 때:
1. seL4 측 CPU offline path 가 pinned VM 을 보유한 모든 SVMVCPU 의
   `pending_offline = 1` 설정 (intercept on next vmrun)
2. 다음 vmrun 시도 시 `Y4Error::CpuOffline` 반환 (vmexit 가 아니라
   syscall 거부)
3. Y4-VMM 이 에러 받으면 → `Migrate(vcpu, new_cpu_id)` 호출 →
   재개
4. Y4-VMM 의 default policy: idle CPU 중 같은 CCX 우선 → 같은 socket
   → 임의 CPU 순.  user override 가능

**SMT-aware**: `isolate-pairs` 일 때 SMT sibling 도 같이 이동.

#### S5.4 — multi-vCPU per VM

Y4 의 multi-vCPU guest = N 개 `SVMVCPU` cap, 각각 독립 cpu_id.  AV4
는 per-vCPU 적용.  VM 전체로는 N 개 logical CPU 에 분산 — Y4-VMM 이
topology-aware placement 책임 (NUMA-local, CCX-local 등).

### S6 — Parent thread group 핀 (+ lifecycle, audit, ChangeParent)

**ARCH-II' 매핑:**
- seL4 측 (D1a) — Configure / vmrun 직전 thread group 검사 +
  `ChangeParent` atomic 5-step + cap revoke hook (S6.1).
- `lifecycle` capsule — ChangeParent 진입점 + parent cap dependency 등록/
  해제 + S13 (vcpu lifetime ⊆ parent TCB) 의 자동 destroy chain 트리거.
- `audit` capsule — S6.2 의 "foreign thread group 거부 시도" entry 기록
  + Y4-VMM anomaly detector 의 임계 감시 → lease 회수 trigger.

`Configure` 시점에 `parent_cspace` + `parent_vspace` 고정.  vmrun 호출
시 `current_tcb.cspace == parent_cspace && current_tcb.vspace ==
parent_vspace` 검증 — 같은 thread group (동일 CSpace + VSpace 공유 TCB
집합) 안의 어떤 TCB 든 vmrun 호출 가능. 그 외 TCB 가 cap 을 invoke
해도 거부.

Worker pool 패턴 지원: main thread 가 `Configure`, worker thread 가
`Run` — 같은 process (CSpace+VSpace) 면 OK. 다른 process 의 thread
가 cap 양도받아도 vmrun 발화 차단.

#### S6.1 — Parent group lifecycle (S13 와 연동)

parent_cspace 와 parent_vspace 의 ref count 가 모두 0 (= thread group
의 모든 TCB destroy + 그룹 자체 dissolve) 되면 SVMVCPU 자동 destroy
(cap revoke).  orphan VMCB 차단 + S13 (vcpu lifetime ⊆ parent TCB) 와
정합.

구현: seL4 cap revocation hook 에 `SVMVCPU` 의존성 등록.  parent
CSpace 또는 VSpace cap 의 `Revoke` 가 trigger 하면 종속 SVMVCPU 들
순차 destroy.  destroy 도중 vmrun 호출은 거부 (`Y4Error::BadCap`).

#### S6.2 — 거부 시도 audit (S12 와 연동)

다른 thread group (cspace 또는 vspace 불일치) 의 TCB 가 vmrun 시도 시
S12 ring buffer 에 audit entry 기록:

```
{ts: ..., vm_id: ..., op: "vmrun_rejected_foreign_thread_group",
 invoking_tcb: ..., expected_cspace: ..., actual_cspace: ...,
 expected_vspace: ..., actual_vspace: ...}
```

보안 침해 시도의 흔적 보존.  Y4-VMM 의 anomaly detector 가 이 entry
빈도를 감시 → 임계 초과 시 lease 강제 회수.

#### S6.3 — `SVMVCPU_ChangeParent` syscall (live ownership transfer)

```
seL4_X86_SVMVCPU_ChangeParent(vcpu, new_cspace, new_vspace)
    -> Result<(), Y4Error>
```

전제 조건:
- vcpu 가 현재 vmrun 중 아님 (S5.2 Migrate 와 같은 quiescent 요구)
- invoking TCB 가 *현재* parent thread group 에 속함 (현 소유자만 이동
  지시 가능)
- new_cspace / new_vspace cap 이 invoking thread 의 CSpace 안에 존재

원자적 동작:
1. VMCB state lock
2. 현 parent group 의 cap dependency 해제
3. `vcpu.parent_cspace ← new_cspace`, `vcpu.parent_vspace ← new_vspace`
4. 새 parent group 의 cap dependency 설정 (S6.1)
5. lock 해제

audit (S6.2 와 같은 schema) 에 ownership transfer 기록.

Use case:
- Y4 의 lease scheduler 가 lease 만료 시 vcpu 를 새 tenant 에 양도
- Live migration 시나리오 (Phase D+)
- 권한 위임 (예: management daemon → user-space VMM)

#### S6.4 — fork / exec 시 명시 transfer 강제

seL4 root task 가 자기 fork (새 CSpace+VSpace 생성) 후 새 process 가
vmrun 권한을 원하면 명시적 `ChangeParent` 필요.  fork 자동 상속 X —
권한 양도가 수동/audit 동반.

#### S6.5 — CSpace 또는 VSpace 공유 multi-process

rare 한 패턴 (예: 같은 CSpace 공유하지만 VSpace 다른 process 들).  S6
의 thread group 정의는 **두 cap 모두 일치** — 한 쪽만 공유하면
"thread group" 으로 인정 X.  vmrun 거부.

명확화: thread group ≠ CSpace 공유 + VSpace 공유 따로따로, 항상 두 쌍의
교집합.

### S7 — clgi/stgi/vmsave/vmload/vmmcall 비공개 + GIF host-managed

**ARCH-II' 매핑:**
- seL4 측 (D1a) — **본체**.  vmrun wrapper 의 7-step atomic sequence
  (S7.1) 가 microkernel 안. clgi/stgi/vmsave/vmload 는 syscall 표면에
  노출 X. AV6 invariant 도 microkernel 측 정리.
- **orchestrator** — `INTERCEPT_VMMCALL` mediation 책임 (S7.2). RAX
  화이트리스트 검증 후 각 기능 capsule 로 IPC dispatch (D-S2 결정과
  동일 — orchestrator 의 thin entry, 별도 hypercall capsule X).
- capsule 영역 — 본 안전장치 본체에는 없음 (microkernel 책임).

D1a syscall 표면에 노출 안 함:

- `CLGI` / `STGI` (Global Interrupt Flag 제어)
- `VMSAVE` / `VMLOAD` (host VMCB state save/restore)
- `VMMCALL` (guest → host hypercall — `INTERCEPT_VMMCALL` 추가 mandatory)

seL4 측 vmrun wrapper 가 모두 internal 처리.  root task 가 ring 3 에서
이 명령어를 직접 발화 시도하면 #UD (privilege fault) 자동.

guest 가 host interrupt 를 영구 마스크하거나 hypercall 표면을 우회하는
경로 모두 차단.

#### S7.1 — vmrun wrapper 의 atomic sequence (ring 0)

```
1. cli                           ; host RFLAGS.IF clear
2. vmsave host_vmcb              ; host state save (host VMCB)
3. clgi                          ; Global Interrupt Flag clear (NMI 포함 모두 deferred)
4. vmrun guest_vmcb              ; guest 진입 — vmexit 까지 실행
                                 ; (S4 deadline 의 timer interrupt 가 강제 vmexit)
5. vmload host_vmcb              ; host state restore
6. stgi                          ; GIF set (deferred 인터럽트 재활성)
7. sti                           ; host RFLAGS.IF set
```

본 시퀀스 이외 위치에서 위 6 명령어 호출은 0.  단일 함수 내 atomic.

#### S7.2 — `INTERCEPT_VMMCALL` 추가 mandatory

S2 의 mandatory mask 에 추가 (총 **16 비트** mandatory):

| 비트 추가 | 의미 |
|---|---|
| `INTERCEPT_VMMCALL` | guest 의 vmmcall → vmexit → Y4-VMM 으로 hypercall 전달. guest-host hypercall 표면이 controlled (mediation 가능) |

Y4-VMM 은 vmmcall 의 RAX (hypercall number) 를 검증, 화이트리스트 외
호출은 `Y4Error::InvalidArg`.  hypercall ABI 자체는 별도 spec
(`y4-hypercall` repo).

#### S7.3 — Atomicity Verus invariant 강화 (AV6 확장)

AV6 의 본문은 7-step sequence 의 모든 transition 이 wrapper 안에서만
발생함을 inductive 로 증명:

```
∀ host CPU c, ∀ time t,
    (clgi at t executed on c) ⇒
        ∃ wrapper invocation w on c, t ∈ w.lifetime ∧
        w.steps[3] == clgi at t ∧ w.steps[6] == stgi at t' for t' > t
```

stronger property: GIF 가 wrapper 밖에서 0 으로 유지되는 시간 0 — 모든
clgi 는 reachable stgi 와 짝짓음.

#### S7.4 — Fault during wrapper

step 3–5 사이 (GIF=0) 에 NMI / SMI 발생 시 deferred → step 6 의 stgi
직후 처리.  단:
- step 4 의 vmrun 자체가 timer-driven vmexit (S4) 로 강제 종료
- step 5 의 vmload 가 fault 시 panic (host VMCB 손상 → 회복 불가)
- 모든 deferred interrupt 는 step 6–7 직후 service

deadlock 위험 없음 — S4 의 deadline 이 vmrun 의 무한 진행 차단.

### S8 — TSC offset 상한 (+ Rebase, scaling disable, default offset)

**ARCH-II' 매핑 (3 capsule 협력):**
- `vmcb` capsule — `tsc_offset` 필드 set (Configure 시점 1회) +
  `TSC_SCALING_ENABLE` 비트 0 강제 (S8.3) + S8.1 bound 검증 + S8.4
  default offset 자동 계산 (`-host_TSC_at_Configure_time`).
- `msr-bitmap` capsule — `TSC_RATIO_MSR (0xC0000104)` write 차단 (S8.3)
  + `IA32_TSC_ADJUST (0x3B)` write 차단 (S8.5).  S10 mandatory entry 와 짝.
- `lifecycle` capsule — `RebaseTsc` 진입점 + 직전 `Migrate` 호출과의
  짝 검증 (S8.2 의 quiescent + Migrate-after-only 정책 enforcement).
  실제 offset set 은 vmcb capsule 에 forward.

VMCB 의 `tsc_offset` 필드는 다음 두 syscall 만 허용:
- `Configure(vcpu, ..., tsc_offset_init)` — 1회 초기 set (필수)
- `RebaseTsc(vcpu, new_offset)` — Migrate (S5.2) 와 함께 새 호스트
  진입 시 1회만 (S8.2)

매 vmrun 마다 재설정 차단.  Wall-clock attribution 보존.

#### S8.1 — bound

`|tsc_offset| ≤ Y4_AMDV_MAX_TSC_OFFSET` (build-time const, default
2^62 — host TSC 의 ~2 년 분량 등가).  bootloader cmdline
`y4.amdv.max_tsc_offset` 로 runtime override (§2.5 패턴).

위반 시 `Configure` / `RebaseTsc` 가 `InvalidArgument`.

#### S8.2 — `SVMVCPU_RebaseTsc` syscall

```
seL4_X86_SVMVCPU_RebaseTsc(vcpu, new_offset) -> Result<(), Y4Error>
```

전제 조건:
- vcpu 가 quiescent (vmrun 중 아님)
- 직전 `Migrate` 호출 (S5.2) 의 후속 — Migrate 후 첫 vmrun 전에만 1회
- `|new_offset| ≤ Y4_AMDV_MAX_TSC_OFFSET`

Migrate 와 짝지어 live migration 시 새 호스트의 TSC base 에 맞춤.
Migrate 없는 RebaseTsc 호출은 거부 — TSC 미세 조정으로 side-channel
신호화 차단.

audit (S12) 에 매 RebaseTsc 호출 기록.

#### S8.3 — TSC scaling 명시 disable

VMCB 의 `TSC_RATIO_MSR` (MSR 0xC0000104) 활용 차단.

- S2 의 `INTERCEPT_MSR_PROT` + S10 의 MSR bitmap 에 TSC_RATIO_MSR
  write 차단 비트 강제
- VMCB control 의 `TSC_SCALING_ENABLE` 비트 (있다면) 0 강제
- guest 는 host TSC frequency 와 동일한 rate 로 TSC 진행 — frequency
  변환 X

이유: TSC scaling 이 활성이면 guest TSC 는 host TSC 와 비선형 관계 →
side-channel 모델 분석 어렵고 wall-clock attribution 흐림.  v1.x 의
고보안 프로필 외 기본 disable.

#### S8.4 — Default `tsc_offset` 정책

Configure 시 명시 인자 없으면 자동:

```
tsc_offset_init = -host_TSC_at_Configure_time
```

⇒ guest 가 보는 TSC = 0 부터 시작 (VM boot timestamp = 0).  대부분의
guest OS (Linux, BSD) 가 이 형태를 자연스럽게 받음.  명시 인자가
주어지면 그대로 사용 (단 §S8.1 bound 강제).

guest 시작 TSC = 0 의 이점:
- guest 의 timekeeping 코드가 작은 절대값으로 단순
- TSC overflow 까지 host_TSC_freq × 2^63 cycles 분량 — 수백 년, 실용
  걱정 없음
- 다중 게스트 사이 TSC 비교가 유의미 (모두 자기 시점 0 부터)

#### S8.5 — `IA32_TSC_ADJUST` MSR

S10 MSR bitmap 의 별도 항목으로 명시 — guest 가 자기 TSC 미세 조정
시도 차단 vs 허용은 v1.0 에서 **차단** (drift 보상 없음, simplicity).
drift 보상은 v1.x patch 의 별도 항목.

### S9 — Nested guest 차단 (+ 명시적 에러 path + R-α/R-γ forward-compat hook)

**ARCH-II' 매핑 (6 군데 분산 — 흐름 명시):**
- `vmcb` capsule — 5 SVM intercept 비트 (`INTERCEPT_VMRUN` /
  `VMLOAD` / `VMSAVE` / `CLGI` / `STGI`) S2 mandatory mask 와 동일 강제 +
  Configure 시점 EFER.SVME 직접 0 set.
- `msr-bitmap` capsule — EFER write 의 SVME 비트 마스킹 (guest 가 EFER
  write 로 SVME=1 시도 차단).
- `cpuid-emul` capsule — leaf `0x8000_0001.ECX[2]` (SVM feature) 응답 0
  강제 (S9.1.3).
- `nested-request` capsule — nested 시도 detection + per-vcpu
  `nested_request_pending` flag set + `PollNestedRequest` syscall handler
  (S9.4 의 Phase D forward-compat hook).
- `audit` capsule — S9.3 의 `nested_svm_attempt` entry 를 S12 ring
  buffer 에 기록.
- **orchestrator** — vmexit 의 `EXIT_VMRUN` (등) detection + #UD
  inject 결정 + nested-request capsule 으로 dispatch.

**흐름 (vmexit 시):**
```
guest vmrun 시도
  → seL4 측 mandatory mask (vmcb) 가 INTERCEPT_VMRUN 강제 → vmexit
  → orchestrator 가 exit_code = EXIT_VMRUN/VMLOAD/VMSAVE/CLGI/STGI 식별
  → nested-request capsule 에 IPC ("nested 시도 감지")
       → flag set + PollNestedRequest 가 다음 query 시 entry 반환 준비
  → audit capsule 에 IPC ("entry 기록")
       → S12 ring buffer append
  → orchestrator 가 guest 에 #UD inject
  → 다음 vmrun 진행
```

#### S9.1 — 강제 메커니즘 (3 layer)

1. **S2 의 5 SVM intercept 비트 항상 1** — vmrun 시 mandatory mask 검증
2. **EFER.SVME 강제 0** — §4 의 EFER write 가 SVME 비트는 무시
3. **CPUID emulation table 의 SVM feature bit 마스크** — leaf
   `0x8000_0001.ECX[2]` (SVM) 응답 0.  guest OS 가 자기 가상화 능력
   미보유로 인식, 가상화 도구는 fallback 또는 friendly error path

#### S9.2 — Linux 게스트 워크로드 영향

| 워크로드 | 동작 메커니즘 | S9 영향 |
|---|---|:---:|
| Docker (runc), Podman (crun), Buildah/Buildkit, gVisor, Docker-in-Docker, rootless Podman, **Waydroid** | namespace + cgroup + seccomp + LSM (OS-level isolation; nested virt 무관) | **영향 없음** |
| Kata Containers, Firecracker / cloud-hypervisor, VirtualBox / KVM in guest, Android emulator in guest | 게스트 내부에서 다시 vmrun 호출 (true nested virt) | **차단**, R-α/R-γ 로 redirect |

표준 컨테이너 워크로드는 SVM 무관 Linux 기능만 쓰므로 S9 와 호환.
**Waydroid 는 LXC 기반이라 nested virt 불필요** — 표준 게스트에서
정상 동작 (GPU 가속은 Phase D 의 passthrough 의존).

#### S9.3 — 명시적 에러 path (audit + nested_request_pending event)

guest 가 vmrun (또는 다른 SVM 명령어) 시도 → vmexit `EXIT_VMRUN`
(또는 등가) → seL4 측 vmrun wrapper 가 다음을 수행:

1. **#UD inject** (legacy 호환 — 대부분 guest 가 "no SVM" 으로 인식하고
   fallback 또는 graceful fail)
2. **S12 audit ring buffer 에 entry 기록**:
   ```
   {ts: ..., vm_id: ..., op: "nested_svm_attempt",
    instruction: "vmrun" | "vmload" | "vmsave" | "clgi" | "stgi",
    guest_rip: ..., guest_rax: ...}
   ```
3. **`nested_request_pending` event flag set** (per-vcpu) — Y4-VMM 의
   vmexit handler 가 이 flag 를 polling 하면 redirect 결정 가능

guest 사용자 측 효과: SVM-가상화 도구가 "host has no SVM" 이라 인식 →
software emulation 또는 다른 backend 로 자연스럽게 fallback.  silent
실패 X.

#### S9.4 — R-α / R-γ forward-compat hook (Phase D)

본 v1.0 spec 은 차단 + audit + flag 까지.  **R-α / R-γ 의 redirect
구현은 Phase D 의 별도 작업**, 그러나 본 v1.0 에서 인터페이스 hook 만
미리 둠 — Phase D 진입 시 spec patch 없이 추가만 가능:

```
seL4_X86_SVMVCPU_PollNestedRequest(vcpu)
    -> Option<NestedRequest>      # v1.0: 항상 None (S9 가 막음)
                                  # Phase D: nested_request_pending 이
                                  # set 일 때 NestedRequest entry 반환

struct NestedRequest {
    instruction: enum { Vmrun, Vmload, Vmsave, Clgi, Stgi },
    guest_vmcb_paddr: VAddr,      # nested guest 가 본 VMCB 주소
    guest_rip: VAddr,             # nested 명령어 위치
    timestamp: u64,
}
```

Y4-VMM 의 vmexit handler 가 본 syscall 로 nested 시도 정보 수집 →
R-α (ioctl 프록시) 또는 R-γ (paravirt agent) 의 진입점으로 forward.
v1.0 에서는 syscall 자체가 항상 `None` — Phase D 에서 비로소 의미.

##### Auto-redirect 옵션 매트릭스 (Phase D 진입 시)

| 옵션 | 적용 범위 | 권고 |
|---|---|---|
| **R-α `/dev/kvm` ioctl 프록시** | KVM 기반 (QEMU/Firecracker/Kata/최신 VBox/Android emulator) | **Phase D 1순위** |
| **R-γ paravirt agent + API** | 모든 VM 매니저 (legacy VBox/VMware) | **Phase D 2순위** (R-α 보완) |
| R-β nested SVM trap-and-forward | 모든 SVM 사용 | **비채택** (verification 표면 두 배, S9 정체성 손상) |
| R-δ 직접 안내 | — | 폴백 |

R-α 는 게스트 안에 가짜 `/dev/kvm` (kernel 모듈) 을 두고 모든 KVM
ioctl 을 vsock 으로 Y4-VMM 에 forward → Y4-VMM 이 sibling SVMVCPU 생성
(parent=Y4-VMM, not guest).  sibling 도 S1–S14 자동 상속 → 안전장치
일관 유지.  Verification 부담 중 (KVM API → SVMVCPU 매핑의 정확성 +
1–2 새 invariant).

R-γ 는 게스트 측 wrapper agent 가 매니저별 CLI (`vboxmanage`, `emulator`)
를 가로채 image / 옵션을 paravirt API 로 변환. 매 매니저별 어댑터
필요하지만 verification 부담 작음.

### S10 — MSR permission bitmap immutable (+ profile / custom MSR / default-deny / audit)

**ARCH-II' 매핑:**
- `msr-bitmap` capsule — 본체.  `SVMMsrBitmap` cap 객체 + immutable
  post-Configure + profile (dev/production/certified) + custom whitelist
  검증 + S10.1 mandatory entry 강제.
- `audit` capsule — S10.4 default-deny 시 `msr_access_denied` entry 를
  S12 ring buffer 에 기록 + Y4-VMM anomaly detector 의 임계 감시 →
  lease 회수 trigger.

`SVMMsrBitmap` 객체는 **immutable post-Configure** — write cap 미존재
(seL4 cap-typing 강제).  default policy = **deny everything not on
whitelist**.

#### S10.1 — Mandatory bitmap entries (모든 profile 공통)

| MSR | 권한 |
|---|---|
| `EFER (0xC0000080)` | guest write 차단 (S9 의 SVME=0 강제와 짝) |
| `VM_HSAVE_PA (0xC0010117)` | guest read+write 차단 |
| `STAR / LSTAR / CSTAR / SFMASK / FS_BASE / GS_BASE / KERNEL_GS_BASE` | host 가 save/restore, guest 직접 접근 가능 |
| `MSR_GHCB (0xC0010130)` | SEV 미사용 시 guest read+write 차단 |
| `TSC_RATIO_MSR (0xC0000104)` | guest write 차단 (S8.3 scaling disable) |
| `IA32_TSC_ADJUST (0x3B)` | guest write 차단 (S8.5) |
| `MSR 0xC0010020` (AMD PATCH_LOADER) | guest write → **S14 pending approval queue 로 trap** |
| `MSR 0x79` (Intel BIOS_UPDT_TRIG) | 동일 |
| `MSR 0x8B` (microcode rev) | guest read → **S14 pending approval queue 로 trap** |
| **모든 그 외 MSR** | **default deny** + S10.4 audit |

#### S10.2 — Build-time MSR profile

`Y4_AMDV_MSR_PROFILE ∈ { dev, production, certified }` cmake option:

| Profile | default 정책 |
|---|---|
| `dev` | mandatory + 광범위 read 허용 (디버깅 용이). guest 가 host 진단 MSR (PERFCTR, MCA, etc.) read 가능 |
| `production` (default) | mandatory + 표준 OS 운영 MSR 만 (KERNEL_GS_BASE 등). 디버깅 MSR 차단 |
| `certified` | mandatory + 최소 set (`STAR/LSTAR/SFMASK/CSTAR` 만 grant). Phase E 의료/항공/금융 인증용 |

bootloader cmdline 은 build profile 만 인식, profile 자체 변경 불가.

#### S10.3 — Custom MSR whitelist (Y4-VMM paravirtual MSR)

```
ObjectType_SVMMsrBitmap 의 생성 syscall:
seL4_X86_SVMMsrBitmap_Create(untyped, profile,
                              custom_whitelist: &[MsrEntry])
```

Y4-VMM 이 자기 만든 paravirtual MSR (예: hypercall offset 보고, vsock
port 노출) 을 `custom_whitelist` 로 추가.  단:
- mandatory deny 영역과 충돌 시 거부
- 추가 entry 수 ≤ `Y4_AMDV_MAX_CUSTOM_MSR_ENTRIES` (default 32)
- bitmap 생성 1회만 — 이후 변경 불가

#### S10.4 — Default-deny + audit

화이트리스트 외 MSR access (read 또는 write) 가 발생하면:

1. AMD SVM 의 `MSR_PROT` intercept 가 vmexit 트리거
2. seL4 측 vmexit handler 가 audit (S12 ring buffer):
   ```
   {ts: ..., vm_id: ..., op: "msr_access_denied",
    msr: 0x..., direction: "read" | "write", guest_rip: ...,
    guest_rax: ..., guest_rdx: ...}
   ```
3. guest 에 `#GP(0)` (general protection fault) inject — 표준 "MSR
   not supported" 응답
4. Y4-VMM 의 anomaly detector 가 빈도 감시 → 임계 초과 시 lease 회수

#### S10.5 — bitmap migration (S5/S6 와 짝)

`Migrate(vcpu, new_cpu_id)` (S5.2) 와 `ChangeParent(vcpu, ...)` (S6.3)
시 bitmap cap 도 함께 이동.  새 CPU / 새 parent group 도 동일 bitmap
적용.  새 bitmap 생성하고 싶으면 `SVMVCPU` 객체 destroy + 재생성.

#### S10.6 — seL4 fork-policy 정합

seL4 의 Intel VT-x 측에 `MsrBitmap` 객체 (`ObjectType_VMXMsrBitmap`
같은) 가 이미 있음.  본 spec 의 `ObjectType_SVMMsrBitmap` 이 같은 패턴
— Strictly Additive Fork 자연 만족.

### S11 — IO permission bitmap immutable + serial 보호

**ARCH-II' 매핑:** `io-bitmap` capsule 이 `SVMIoBitmap` 객체 생성/해체
+ immutable 정책 enforce + per-port allow request 검증 + S12 audit hook.
orchestrator 는 cap reference 만 보유, bitmap 본체 직접 변경 X.

`SVMIoBitmap` 은 S5 (VMCB) / S6 (MSR bitmap) 과 동일 패턴으로 immutable
— 변경하려면 새 cap 재생성 후 `SVMVCPU_Configure` 로 swap.

**default-block 표:**

| Port | 권한 | 근거 |
|---|---|---|
| `0x3F8-0x3FF` (COM1, host serial) | guest 차단 | host log/console 격리 |
| `0xCF8 / 0xCFC` (PCI config) | guest 차단 | host PCIe 토폴로지 노출 차단 |
| `0x70-0x71` (CMOS) | guest 차단 | RTC / NVRAM 변조 차단 |
| `0x20-0x21, 0xA0-0xA1` (legacy 8259 PIC) | guest 차단 | APIC 사용 강제 |
| `0x40-0x43` (legacy 8254 PIT) | guest 차단 | LAPIC timer / HPET 강제 (host timer 변조 차단) |
| `0x60-0x64` (legacy PS/2 KBC) | guest 차단 | host BIOS 잔재 — host KBC 변조 차단 |
| `0x80` (POST diagnostic) | guest 차단 | host hardware fingerprinting 차단 |
| `0xF0-0xFF` (legacy x87 FPU coprocessor) | guest 차단 | x87 IRQ 13 잔재; modern guest 는 SSE/AVX |
| 그 외 | guest 가 capsule 로 mediate 받은 device 만 | per-device explicit allow |

**explicit allow 경로:** Phase D 자체-driver guest 가 own hardware 접근
원하면 per-device IOMMU 통과 + io-bitmap capsule 이 발급한 별도 cap.
io-bitmap capsule 은 allow 요청을 **port-range 단위 (per-device 그룹화)**
로 검증, 매 발급 entry 를 S12 ring buffer 에 기록.

### S12 — Audit trail

**ARCH-II' 매핑:**
- `audit` capsule — 본체.  ring buffer + anomaly detector + entry sink
  + Y4-VMM 측 read API.
- seL4 측 (D1a) — `SVMExitInfo` struct 반환만 (capsule 이 받아서 해석).
- orchestrator — vmexit dispatch 시 entry 를 audit capsule 로 forward
  (multiplex hub).

본 안전장치는 S6.2 (foreign thread group 거부) / S9.3 (nested SVM
attempt) / S10.4 (MSR default-deny) / S11 (io-bitmap allow grant) / S14
(firmware approval queue + decision) 의 모든 entry sink 의 단일 source
of truth.

#### S12.1 — Ring buffer 배치 (A=a-iii)

**Per-VM ring buffer + global aggregated read-only mirror.**

```
audit_capsule {
    per_vm_ring: Map<VmId, RingBuffer>,         // 각 VM 격리, write 만 가능
    global_mirror: ReadOnlyMerged<RingBuffer>,  // anomaly detector + host operator 용
    ...
}
```

per-VM ring 은 lease holder 의 own entries 만 read.  global mirror 는
host operator 만 read (G=g-ii).  cross-VM 분석은 mirror 경유.

#### S12.2 — Entry schema (B=b-ii)

**고정 header + per-tag variant payload.**

```rust
struct AuditEntry {
    ts: u64,                  // host TSC at vmexit
    vm_id: VmId,              // 8-byte tenant identifier
    severity: enum { Trace, Info, Warning, Critical },
    op_tag: enum {            // discrimination
        VmrunRejectedForeignThreadGroup,  // S6.2
        NestedSvmAttempt,                 // S9.3
        MsrAccessDenied,                  // S10.4
        IoBitmapGrantIssued,              // S11
        FirmwareApprovalQueued,           // S14
        FirmwareApprovalDecided,          // S14
        VcpuMigrate,                      // S5.2
        VcpuChangeParent,                 // S6.3
        VcpuRebaseTsc,                    // S8.2
        // ... extensible (v1.x)
    },
    payload: AuditPayload,    // op_tag 별 variant
}

enum AuditPayload {
    NestedSvmAttempt {
        instruction: SvmInstruction,
        guest_rip: VAddr,
        guest_rax: u64,
    },
    MsrAccessDenied {
        msr: u32,
        direction: enum { Read, Write },
        guest_rip: VAddr,
        guest_rax: u64,
        guest_rdx: u64,
    },
    // ...
}
```

Verus invariant 와 anomaly detector 모두 정형 schema 필요 — 자유형
string 거부.

#### S12.3 — Overflow 정책 (C=c-iii)

**Two-tier 분리:**

| Tier | 대상 op_tag | 정책 | Buffer 크기 (default) |
|---|---|---|---|
| **Priority** | severity ≥ Warning (S6.2 / S9.3 / S14 / 등) | **block-on-full** — vmexit handler back-pressure, vmrun stall 까지 감수 | 1024 entries / VM |
| **Trace** | severity ≤ Info | circular overwrite | 4096 entries / VM |

침해 흔적 (priority tier) 손실 0 보장.  일반 trace 는 유연성 유지.
buffer 크기는 build-time const `Y4_AMDV_AUDIT_PRIORITY_RING_SIZE` /
`Y4_AMDV_AUDIT_TRACE_RING_SIZE`, cmdline 으로 형상별 override 가능.

#### S12.4 — Persistent storage (D=d-i + Phase D forward-compat hook)

**Phase B/C: memory only.**  Y4 의 disk driver 가 Phase D 까지 부재 →
부트 잃음 수용.

**Phase D forward-compat hook:** `audit` capsule 의 dump API 인터페이스
v1.0 spec 에 미리 정의 — Phase D 진입 시 spec patch 없이 (d-ii) 추가
가능:

```rust
trait AuditPersistence {
    fn on_overflow_threshold(...) -> ...;   // v1.0: no-op
    fn on_anomaly_trigger(...) -> ...;      // v1.0: no-op
    // Phase D: disk driver 가 두 콜백을 구현
}
```

#### S12.5 — Sanitize (E=e-ii)

**암호학적 erase — XChaCha20 key destroy.**

매 lease 발급 시 audit capsule 이 per-lease 256-bit XChaCha20 key 생성
+ 192-bit nonce.  ring buffer 의 모든 entry 가 본 key 로 in-place 암호화
저장 (write 시 encrypt, read 시 decrypt).

lease 회수 trigger:
1. lifecycle capsule 이 `LeaseRevoke` 신호 send
2. audit capsule 이 own key 를 secure zeroize (memory + cache flush)
3. ring buffer 의 ciphertext 는 그대로 남지만 **decrypt 불가** — 암호학적
   erase 완료
4. 후속 GC 가 ciphertext frame 을 자유 영역으로 반환

WaveTensor masking 정책과 정합 (HIU 의 192-bit nonce / 256-bit key 패턴
재사용).  Verus 측 invariant: `key_destroyed ⇒ ∀ e ∈ buffer,
decryptable(e) = false`.

key 재사용 0 — lease 마다 신선한 key (zero-fill 의 forward secrecy 등가).

#### S12.6 — Anomaly detector (F=f-i)

**audit capsule 내부 component.**

```
audit_capsule {
    detector: AnomalyDetector {
        thresholds: BTreeMap<OpTag, RateThreshold>,
        per_vm_counters: ...,
        ...
    },
    ...
}
```

detect rules (v1.0 default):
- `VmrunRejectedForeignThreadGroup` ≥ 3 / 분 → **즉시 lease revoke**
- `NestedSvmAttempt` ≥ 100 / 분 → warning, lease holder 통지
- `MsrAccessDenied` ≥ 1000 / 분 → warning
- `FirmwareApprovalQueued` ≥ `MAX_PENDING_ENTRIES` 90% → warning

threshold 는 v1.x 에서 user-configurable per-VM cap.

detection 결과 → lifecycle capsule 의 `LeaseRevoke` 또는 user notification
channel.

#### S12.7 — Read 권한 (G=g-ii)

| 주체 | 접근 |
|---|---|
| **lease holder** (parent thread group 의 TCB) | own VM 의 per-VM ring 만 read.  decrypt key 보유 (lease 발급 시 수령) |
| **host operator** (root task) | global mirror 전체 read.  per-VM key 들 lease 발급 시 escrow 수령 |
| 다른 tenant | 0 |

cross-tenant audit 격리 강제: 각 lease 의 XChaCha20 key 가 독립 → 다른
tenant 의 ring 을 byte-level 로 받아도 decrypt 0.

#### S12.8 — 멀티 CPU 동시 write (H=h-ii)

**Per-CPU ring + global merge on read.**

```
audit_capsule {
    per_cpu_per_vm_rings: [[RingBuffer; N_VMS]; N_CPUS],  // lock-free hot path
    ...
}
```

매 vmexit handler 가 own CPU 의 own VM ring 에 lock-free append (allocator
SLAB 의 per-CPU magazine 패턴과 정합).  read 시 N_CPUS 개 ring 을 ts
순서로 merge — read latency ↑ 수용 (read 빈도 << write 빈도).

merge 알고리즘: K-way merge sort, K = N_CPUS.  복잡도
O(total_entries · log N_CPUS).

Verus invariant (AV12): `∀ entry e written on CPU c, ∃ read view r,
e ∈ r ∧ ∀ e' written before e on c, ts(e') ≤ ts(e)` — per-CPU ordering
보존.

### S13 — VCPU lifetime ↔ parent TCB lifetime

**ARCH-II' 매핑:**
- `lifecycle` capsule — 본체.  parent group cap revoke detection +
  sibling capsule destruction orchestration + race resolution.
- seL4 측 (D1a) — cap derivation tree refcount hook + cap revoke path
  안의 destroy callback (Strictly Additive Fork).
- 모든 sibling capsule (vmcb, npt, msr-bitmap, io-bitmap, audit,
  firmware-approval) — 자기 destroy callback 등록.

`SVMVCPU` 객체의 lifetime 이 parent TCB 의 lifetime 안에 항상 들어감.
parent TCB destroy 시 SVMVCPU 도 자동 destroy (cap revoke).  orphan
VMCB / NPT 가 host 메모리에 떠도는 path 차단.

ARCH-II' 에서는 SVMVCPU destroy 가 단순 cap revoke 가 아니라 **capsule
cluster 전체의 atomic teardown** — 아래 sub-decision 들이 cluster 의
destruction sequence 정의.

#### S13.1 — Sibling destruction order (A=a)

**Strict reverse-creation order:**

```
firmware-approval → audit → io-bitmap → msr-bitmap → npt → vmcb → SVMVCPU
```

firmware-approval 가 가장 먼저 destroy — pending entry 들을 모두 reject
처리 (timeout 도달 전이라도) + 결정 결과를 audit 에 마지막 송신 후
종료.  audit 이 그 다음으로 entry 받는 순간을 보장 (다른 capsule
destroy 가 audit entry 를 발생시킬 수 있으므로 audit 이 가장 늦게
destroy 시작 → 그러나 key destroy 는 S13.3 의 master sequence 에서
마지막).

의존성 그래프 정합 — 각 capsule 의 destroy callback 이 의존하는 sibling
capsule 이 살아있는 상태에서만 호출.

#### S13.2 — Mid-vmrun race (B=a)

parent TCB destroy / lease revoke 가 vmrun 중에 도착 시:

1. lifecycle capsule 이 `pending_destroy = 1` 설정 (vCPU 단위)
2. 현재 vmrun 은 그대로 진행, S4 의 deadline (≤ 100 ms) 또는 자연
   vmexit 에 도달
3. vmexit handler 가 `pending_destroy` 확인 → 다음 vmrun 0, capsule
   destroy sequence (S13.1) 진입
4. fallback: deadline + bounded slack (`Y4_AMDV_MAX_SLACK_NS_BUILD`,
   default 1 ms) 초과 시 IPI 강제 종료 (E=a 의 forced fallback 과 통합)

S4 deadline 으로 bounded teardown 보장 — atomic teardown deterministic.

#### S13.3 — lease key destroy 순서 (C=a)

**Master sequence: lease 회수 = capsule cluster 전체 teardown trigger.**

```
LeaseRevoke
    └─ lifecycle capsule 의 group destroy queue 에 모든 vCPU enqueue (D)
    └─ for each vCPU in cluster:
         └─ S13.2 mid-vmrun race resolve
         └─ S13.1 sibling destruction order
    └─ all vCPUs destroyed
    └─ audit capsule 의 XChaCha20 key 최종 zeroize (S12.5)
    └─ ciphertext frame 반환
```

audit key destroy 는 모든 vCPU 가 destroyed 된 **이후** — multi-vCPU
환경에서도 cross-vCPU audit entry 가 일관 보호.  S12.5 와 정합.

#### S13.4 — Multi-vCPU 동시 destroy (D=a)

`parent group cap revoke` 가 single trigger.  lifecycle capsule 이
group destroy queue 에 cluster 의 모든 vCPU 를 enqueue → 순차 또는
병렬 처리 (S13.1 의 sibling order 는 per-vCPU, vCPU 들 사이 순서는
arbitrary).

vCPU 들 사이 lifetime 격차 0 (queue 의 단일 master commit) — race 의
window 0.

#### S13.5 — Forced vs graceful (E=a)

**단일 destroy path (graceful → forced fallback):**

```
1. graceful: S13.2 의 deadline 대기 (≤ 100 ms)
2. forced fallback: deadline + slack (default 1 ms) 초과 시 IPI 강제 vmexit
   - 강제 vmexit 후 VMCB state 는 INVALID 마킹
   - vmcb capsule destroy callback 이 INVALID state frame 도 정상 sanitize
3. forced fallback 후에는 S13.1 의 sibling order 그대로
```

API 단일 — 사용자는 lease revoke 만 호출, graceful/forced 결정은
lifecycle capsule 내부.  DoS (graceful 무한 대기) 위험 0.

#### S13.6 — Verus invariant (AV15)

```
forall frame f, alive(f) ==> exists cap c,
    owns(c, f) ∧ ¬revoked(c)
```

orphan VMCB / NPT / bitmap frame 이 host memory 에 잔류 0 의 직접 표현.
S13.1 의 reverse order + S13.2 의 deadline-bounded resolve + S13.3 의
master sequence 가 본 invariant 의 inductive 증명을 닫음.

`proofs/verus/src/amdv/upper/lifetime.rs` 에 entry (§5.2 catalog 정합).

#### S13.7 — Cap revocation hook 위치 (G=a)

seL4 측 D1a 패치 안의 **기존 cap revoke path 에 직접 hook 등록** (Strictly
Additive Fork).  새 코드 path 추가만, 기존 cap revoke 흐름 변경 0.

```c
// seL4 fork: src/object/cnode.c 와 등가 위치 (concrete path TBD)
#ifdef CONFIG_Y4_AMDV
    if (cap_get_capType(cap) == cap_svm_vcpu_cap) {
        y4_lifecycle_capsule_notify(cap);  // IPC to lifecycle capsule
    }
#endif
```

atomic 보장 — race window 0.  lifecycle capsule polling 패턴 거부 (B=a
의 deadline-bounded race resolve 와 일관).

#### S13.8 — Ref count detection (H=a)

seL4 의 **기존 cap derivation tree refcount auto-hook** 재사용.  parent
CSpace / VSpace cap 의 ref count 가 0 도달 시 cap revoke 자동 trigger
→ S13.7 hook 으로 lifecycle capsule 에 전달.

별도 reference counter / GC 0 — 기존 메커니즘 재사용으로 검증 표면
최소화 + Strictly Additive Fork 자연 만족.

#### S13.9 — State sanitize 방식 (I=b)

**XChaCha20 erase + frame return** — S12.5 의 audit ring buffer sanitize
와 동일 패턴.

각 sibling capsule (vmcb, npt, msr-bitmap, io-bitmap, firmware-approval)
이 lease 발급 시 own 256-bit segment key 수령 (lease key derivation 의
sub-key, HKDF-Expand).  destroy 시:

1. capsule 이 자기 segment 안의 모든 frame 을 in-place 암호화 상태
   유지 (write 시 이미 encrypted)
2. segment key 를 secure zeroize
3. ciphertext frame 을 untyped 로 반환
4. 후속 retype 이 frame 을 새 용도로 재할당해도 ciphertext 는
   복호화 0 — forward secrecy

WaveTensor masking 정책 (HIU 192-bit nonce + 256-bit key) 패턴 재사용.
lease key 1 개 → HKDF 로 N 개 segment key 파생 → 각 capsule own segment
encrypt → lease 회수 시 master key destroy → 모든 segment 암호학적 erase.

### S14 — Firmware / microcode mutation pending approval

**ARCH-II' 매핑:**
- `firmware-approval` capsule — 본체.  pending queue + 결정 dispatch +
  CLI 인터페이스 server-side.
- `msr-bitmap` capsule — `WRMSR(0xC0010020 / 0x79)` / `RDMSR(0x8B)` trap
  forward (S10.1 mandatory entry 가 firmware-approval 로 화살표).
- `vmcb` capsule — `INTERCEPT_SMI` (S2 mandatory) trap 의 `OUT 0xB2` payload
  추출 후 forward.
- `cpuid-emul` capsule — `CPUID(0x1).EAX[3:0]` (microcode bits) 응답을
  firmware-approval capsule 의 readout 정책에 따라 마스크.
- `audit` capsule — S12.2 schema 의 `FirmwareApprovalQueued` /
  `FirmwareApprovalDecided` op_tag 으로 entry 기록.
- `lifecycle` capsule — S13.1 sibling destruction order 안에 포함
  (audit 직후 destroy).
- **orchestrator** — vmexit dispatch 만, 직접 책임 없음.

guest 가 host firmware 또는 microcode 상태를 변경하는 모든 시도가
**pending approval queue** 에 들어가며, 사용자가 Y4-VMM CLI 또는
host UI 로 명시적 승인할 때까지 결과는 guest 에 반환되지 않음.
승인 없이는 silent allow / silent fail 둘 다 X.

**대상 operation:**

| 종류 | 트리거 |
|---|---|
| **Microcode 업데이트** | `WRMSR(0xC0010020, ...)` (AMD PATCH_LOADER), `WRMSR(0x79, ...)` (Intel BIOS_UPDT_TRIG) |
| **Microcode rev readout** | `RDMSR(0x8B)` (host microcode revision MSR), `CPUID(0x1).EAX[3:0]` 의 microcode bits |
| **UEFI runtime variable mutation** | guest 가 UEFI runtime 에 접근 가능한 형상에서 `SetVariable` 호출 |
| **UEFI capsule / firmware update** | `UpdateCapsule` runtime service |
| **SMI invocation** | guest 가 `OUT 0xB2, port` 로 SMI 트리거 (S2 의 INTERCEPT_SMI 와 짝) |
| **ACPI _OSI / firmware control method** | DSDT 의 firmware-mutating method 호출 |
| **PSP / fTPM 명령** | AMD Platform Security Processor mailbox 에 mutation 명령 |
| **MTRR / PAT 변경** | `WRMSR(0x200..0x20F)`, `WRMSR(0x277)` 같은 cache attribute 변경 (host VM 격리에 영향) |

**Pending queue 정책:**

| 항목 | 값 | 결정 |
|---|---|---|
| 최대 entry 수 | build-time `Y4_AMDV_MAX_PENDING_ENTRIES` (default 16) + cmdline `y4.amdv.max_pending_entries` override | A=a-iv (S4 3-계층 ceiling 패턴 정합, 형상별 override) |
| Timeout | build-time `Y4_AMDV_PENDING_TIMEOUT_S` (default 60s) + cmdline `y4.amdv.pending_timeout_s` override | B=b-i |
| Auto-rejection | timeout 도달 시 guest 에 `Y4Error::Timeout` | — |
| Concurrent decision | per-entry CAS on `decision_state: Pending → Approved/Rejected` (lock-free, 첫 결정 winner, 두 번째 시도 `Y4Error::AlreadyDecided`) | H=h-ii |

**사용자 인터페이스 (`y4-hypercall` user-side CLI repo):**

`y4-hypercall` 은 ARCH-II' 채택 (2026-05-04) 후 사용자 CLI tooling repo
로 재정의 — core VMM 코드는 Y4 워크스페이스 안의 `firmware-approval`
capsule + orchestrator 가 보유.  CLI 는 vsock socket 으로 capsule 의
server-side 와 통신.

```
y4-hypercall pending list
    [#0] vm=guest-1 op=microcode_update payload_hash=sha256:af3c... requested=12s ago timeout=48s scope=host-wide
    [#1] vm=guest-2 op=cpuid_microcode_rev requested=2s ago timeout=58s scope=vm-local

y4-hypercall pending show <id>           # 상세 (payload disassembly 등)
y4-hypercall dry-run <id>                # E=e-i: payload 검증 + simulated effect report, 적용 X
y4-hypercall approve <id>                # 승인 + host 적용 (또는 readout 노출)
y4-hypercall reject <id>                 # 거부, guest 는 SecurityViolation 받음
y4-hypercall reject --all-pending        # batch 거부

# F=f-ii: per-VM pre-approved hash 화이트리스트
y4-hypercall whitelist add <vm-id> --payload-hash sha256:...
y4-hypercall whitelist list <vm-id>
y4-hypercall whitelist remove <vm-id> --payload-hash sha256:...
```

**권한 (C=c-ii):**

| 주체 | 권한 |
|---|---|
| **host operator** (root task) | 모든 entry approve/reject (host-wide + vm-local 둘 다) + 화이트리스트 관리 |
| **lease holder** (own VM 의 parent thread group TCB) | own VM 의 `scope=vm-local` entry 만 approve/reject (cpuid_readout 등). `scope=host-wide` (microcode/SMI/PSP/MTRR) 는 0 |

**Host-wide 영향 처리 (D=d-ii):**

`scope=host-wide` entry 의 approve 시 CLI 가 cross-VM warning 표시 +
명시적 두 번째 confirm 강제:

```
y4-hypercall approve #0
    ⚠ HOST-WIDE OPERATION
    op=microcode_update payload_hash=sha256:af3c...
    이 patch 는 host 의 모든 CPU core 에 적용됩니다.
    영향 받는 active guest: 3 개 (guest-1, guest-2, guest-3)
    적용 후 모든 guest 의 microcode rev 가 변경됩니다.

    계속하려면 confirmation token 입력: [4-자리 토큰]: ____
```

token 은 capsule 측이 entry 별 1회용 random 생성, CLI 가 복사 입력 →
race / 오타 차단.

**Pre-approved hash 화이트리스트 (F=f-ii):**

per-VM cap 의 `approved_payload_hashes: Set<Sha256>` — 화이트리스트
매치 시 firmware-approval capsule 이 auto-approve, **단 audit entry
는 그대로 기록** (forensic 보존).  화이트리스트 update 자체도 audit
대상 (op_tag 추가: `FirmwareApprovalWhitelistChanged`).

operator 가 신뢰 patch hash 만 사전 등록 → 일상 microcode 갱신 자동화,
unknown patch 는 여전히 manual approve.

**Notification mechanism (G=g-i):**

push + polling hybrid:

| Channel | 용도 |
|---|---|
| **push (vsock socket)** | host operator 의 등록 daemon (또는 `y4-hypercall watch`) 에 즉시 송신.  payload: entry ID + op_tag + scope + timeout |
| **push (syslog)** | severity ≥ Warning 자동 송신 (S12 anomaly detector 와 정합) |
| **push (desktop notification)** | desktop 형상의 form factor 한정, libnotify 로 popup |
| **polling (`pending list`)** | fallback, push channel 미등록 시 |

`y4-hypercall watch` 는 vsock 에 long-poll 등록, 새 entry 도달 시 stdout
즉시 출력 (CLI 가 fzf / dmenu 등으로 wrap 가능).

**Audit 강제 (S12 와 짝):**

매 pending entry + 결정 (approve / reject / timeout) 이 S12 ring buffer
에 구조화 기록:

```
{ts: ..., vm_id: ..., op: "microcode_update", payload_sha256: ...,
 decision: "approved" | "rejected" | "timeout", approver: ...,
 host_microcode_rev_before: ..., host_microcode_rev_after: ...}
```

**Threat 차단:**
- 악성 / 손상된 guest 가 host microcode 를 silent 로 다운그레이드 / 변조 → 즉시 pending 상태, 사용자 승인 없이는 0
- guest 의 microcode rev fingerprinting (host-specific bug detection 후 spectre 변형 등) → readout 도 pending
- 자동화된 supply-chain attack (악성 firmware capsule push) → 사용자 명시적 승인 필요

**legitimate use case 동선:**
1. 사용자가 호스트 microcode 패치 적용을 *원하는* 경우: 호스트에서 직접
   `y4-hypercall apply-host-microcode <patch>` (guest 와 무관)
2. guest 안의 OS 가 update 시도: pending 으로 잡히고, 사용자가 신뢰하면
   approve. 신뢰하지 않으면 reject.
3. guest 가 host CPU info 가 필요하다고 주장: 사용자가 emulation table
   에 미리 노출 정책 설정 (per-VM cap) — 설정 없으면 매 readout 마다
   pending.

---

## 4. VMCB 필드 — `vmcb` capsule 의 R/W API 가 허용하는 reg_id 표면

본 §4 의 화이트리스트 검증은 **vmcb capsule** 의 책임 (ARCH-II' 매핑,
S1 / S2 / S8 등의 capsule 본체).  orchestrator / 다른 capsule 은 cap
reference 로만 vmcb 에 접근, raw 메모리 주소 X.

### 4.1 화이트리스트 (R/W 허용)

| reg_id | 필드 | R/W | sibling capsule / 안전장치 | 강제 메커니즘 |
|---|---|---|---|---|
| `RAX/RBX/.../R15` | guest GPR | R/W | vmcb capsule | (none) |
| `RIP` | instruction pointer | R/W | vmcb capsule | (none) |
| `RFLAGS` | guest flags | R/W (IF 비트 ignore) | vmcb capsule (S7 짝) | RFLAGS write 시 IF 비트는 **ignore** (host 가 GIF 로 관리, S7) — 즉 RFLAGS write 가 IF 변경 0 |
| `CR0/CR2/CR3/CR4/CR8` | guest CR | R/W (WP/SMEP 강제) | vmcb capsule | CR write handler 가 **CR0.WP=1 / CR4.SMEP=1 비트를 OR-mask** (write value 와 무관 항상 set). guest 가 0 으로 시도해도 1 강제 |
| `DR6/DR7` | debug | R/W (debug build only) | vmcb capsule | `KernelDebugBuild = OFF` 시 W reject |
| `EFER` | guest EFER | R/W (SVME=0 강제) | vmcb capsule (S9) + msr-bitmap capsule (EFER write trap) | SVME 비트를 **AND-mask** (write value 와 무관 항상 0) |
| `XCR0` | XSAVE control | R/W (host 지원 mask) | vmcb capsule | XCR0 write handler 가 **boot 시 cached `host_xsave_mask` 와 AND-mask**. guest 가 host 미지원 비트 set 시도해도 0 (read-back 도 마스킹된 값) |
| `CS/DS/ES/FS/GS/SS/LDTR/TR/IDTR/GDTR` | segment | R/W | vmcb capsule | (none) |
| `intercept_words` | intercept bits | R only | vmcb capsule (S2) | mandatory mask 강제 (Configure / capsule-internal 만 W) |
| `tsc_offset` | TSC offset | W only via Configure / RebaseTsc syscall | vmcb capsule (S8) + lifecycle capsule (S8.2 RebaseTsc ↔ Migrate 짝) | 직접 WriteRegister 차단. S8.1 bound 강제 |
| `vmcb_clean` | clean bits | R only | vmcb capsule | clean bit 자동 관리, capsule internal W |

### 4.2 syscall 표면 노출 X — capsule-internal 또는 microkernel-internal

| 필드 | 책임 주체 | 비고 |
|---|---|---|
| `H_SAVE_PA` | seL4 측 D1a (microkernel only) | host VMSAVE / VMLOAD area, S7 wrapper 안 |
| `HSAVE_PA` (MSR `0xC0010117`) | msr-bitmap capsule (S10.1 deny) + seL4 D1a (write source) | guest read+write 차단 |
| `MSRPM_BASE` | **msr-bitmap capsule** only | MSR permission bitmap 의 host 물리 주소.  capsule internal |
| `IOPM_BASE` | **io-bitmap capsule** only | I/O permission bitmap 의 host 물리 주소.  capsule internal |
| `NCR3` | **npt capsule** only | nested page table root.  capsule internal — `SVMNPT_Map` 만이 NPT 변경 경로 |
| `ASID` | **lifecycle capsule** + seL4 D1a | S5.2 Migrate atomic 5-step 안의 ASID 재할당.  capsule + microkernel |
| `TLB_CONTROL` | vmcb capsule internal | 매 vmrun flush 정책 — capsule 이 결정.  syscall 노출 X |
| `VMCB_CLEAN` (write) | vmcb capsule internal | clean bit 자동 관리, vmcb capsule 만 update |
| `EVENTINJ` | vmcb capsule + orchestrator (S9.3 #UD inject) | guest 에 fault inject 시점.  orchestrator 의 inject API 만 |
| `LBR_VIRTUALIZATION_ENABLE` | vmcb capsule internal | LBR (Last Branch Record) 가상화 disable 강제 (side-channel 차단) |
| `AVIC_*` (Advanced Virtual Interrupt Controller) | v1.0 disable, Phase D 검토 | guest 의 IPI 가속 패스 — verification 표면 ↑ |
| `VMCB_PA` 자체 | seL4 측 D1a only | VMCB 본체의 host 물리 주소.  cap-only access (S1) |

### 4.3 vmcb capsule 의 보조 metadata 슬롯 (D-S4 결정)

VMCB raw 필드와 **분리된** capsule-internal 보조 데이터.  syscall 표면
노출 0, capsule 내부에서만 read/write:

```rust
// vmcb capsule 안의 per-vCPU state
struct VmcbCapsuleState {
    vmcb_frame:           HostFrame,            // VMCB raw 필드 (§4.1 의 화이트리스트가 적용되는 영역)

    // 보조 metadata (D-S4) — VMCB raw 필드 아님, capsule-internal
    l3_deadline_ceiling_ns:    u64,             // S4 의 L3 per-VM cap (orchestrator 가 vmrun 직전 query)
    parent_thread_group_pin:   ParentGroup,     // S6 의 cspace + vspace 쌍
    cpu_pin:                   LogicalApicId,   // S5 의 cpu_id
    smt_grouping_member:       Option<SmtPair>, // S5.1 SMT pair 정보 ('isolate-pairs' 모드)
    pending_destroy:           bool,            // S13.2 mid-vmrun race resolve
    segment_key:               Aes256Key,       // S13.9 sibling segment key (HKDF-derived)
}
```

orchestrator → vmcb capsule 의 verb (`vmcb.query_l3_deadline_ceiling` 등)
로만 metadata 접근.

### 4.4 sibling capsule key segment 표 (S13.9 cross-ref)

S13.9 의 lease key → HKDF-Expand → N 개 segment key.  각 capsule 이
own segment 보유:

| capsule | own segment frame 종류 | encrypt 대상 |
|---|---|---|
| `vmcb` | VMCB raw frame (per-vCPU) + §4.3 metadata | host 가 directly access X (encrypted-at-rest) |
| `npt` | NPT page table frames | guest 의 GPA → HPA mapping 메타데이터 |
| `msr-bitmap` | MSRPM frame (8 KiB) | bitmap 의 1 비트 = 1 MSR R/W permission |
| `io-bitmap` | IOPM frame (12 KiB + bit set) | I/O port permission |
| `firmware-approval` | pending queue + per-VM whitelist + token table | F=f-ii whitelist + audit chain |
| `audit` | per-VM ring buffer (priority + trace tier) + per-CPU rings | S12.5 의 본 정책의 본체 (XChaCha20 erase 의 master) |

lease 회수 시 lease master key destroy → HKDF-Expand 가 deterministic
이지만 input key 가 사라지므로 모든 segment key 도 재구성 0 →
forward secrecy.

---

## 5. Verus invariant 카탈로그 (proofs/verus/src/amdv/)

### 5.1 카탈로그 구조

본 §5 의 invariant 는 **statement only v1.0 frozen**.  본문 (proof body)
은 Phase C 진입 시 채움 — formal-first 원칙의 statement-first sign-off.

P1.4 §5.1 의 2-축 layout (`upper/` + `lower/`) 와 정합 — 각 invariant 가
Layer (§3.1 의 Upper/Lower 분류) + 책임 capsule + proof file 명시.

### 5.2 Invariant 표 (20 항목, P1.6 / P2.1 결정 통합)

| AV | Layer | 책임 capsule | Invariant | Safety / 결정 | Proof file | Status |
|---|:---:|---|---|---|---|---|
| **AV1** | Lower | vmcb + cpuid-emul | `intercept_floor_holds(vcpu)` — S2 의 16-bit mandatory mask 가 vmrun 직전 항상 만족 | S2 | `lower/intercept_floor.rs` | v1.0 |
| **AV2** | Upper | npt + npf-handler | `npt_subset_of_parent_vspace(vcpu)` — S3.1 NPT mapping 이 부모 VSpace 의 frame 만 + S3.2 alignment + S3.3 entry 상한 | S3.1/3.2/3.3 | `upper/npt.rs` | v1.0 |
| **AV2-D** | Upper | npt | `npt_iommu_consistent(vcpu)` — S3.4 NPT entry 와 IOMMU TLB entry 의 정합 | S3.4 | `upper/npt.rs` | **Phase D** |
| **AV3** | Lower | vmcb + orchestrator | `vmrun_terminates_within(vcpu, deadline_ns + slack_ns)` — S4 의 3-계층 ceiling 의 최소값 + bounded slack 안에서 항상 종료. `KernelDebugBuild = OFF` 시 `deadline_ns = 0` 거부 | S4 | `lower/deadline.rs` | v1.0 |
| **AV4** | Upper | lifecycle + orchestrator | `cpu_pinned(vcpu)` — S5.1 logical APIC ID 일치, S5.2 Migrate atomicity, S5.3 offline path 의 vmrun 차단, S5.4 multi-vCPU per-vcpu 적용, SMT grouping (`isolate-pairs`) 만족 | S5 | `upper/cpu_pin.rs` | v1.0 |
| **AV5** | Upper | lifecycle + audit | `parent_thread_group_pinned(vcpu)` — S6.1 lifecycle, S6.2 audit, S6.3 ChangeParent atomicity, S6.4 fork 시 명시 transfer, S6.5 (cspace ∧ vspace) 일치 | S6 | `upper/thread_group.rs` | v1.0 |
| **AV6** | Lower | seL4 D1a + orchestrator | `gif_host_managed()` — S7.1 의 7-step atomic sequence 강제. ∀ clgi 는 reachable stgi 와 짝지음.  vmsave/vmload/vmmcall 도 wrapper 외부 발화 0 | S7 | `lower/gif.rs` (microkernel 측 본체) | v1.0 |
| **AV7** | Lower | vmcb + msr-bitmap + lifecycle | `tsc_offset_bounded(vcpu)` — S8.1 bound, S8.2 RebaseTsc Migrate-after-only, S8.3 TSC scaling disabled, S8.4 default = -host_TSC_at_Configure, S8.5 TSC_ADJUST write 차단 | S8 | `lower/tsc.rs` | v1.0 |
| **AV8** | Lower | vmcb + msr-bitmap + cpuid-emul + nested-request + audit | `no_nested_svm(vcpu)` — S9.1 의 3-layer (mandatory intercept ∧ EFER.SVME=0 ∧ CPUID SVM bit=0). nested_request_pending flag.  v1.0 PollNestedRequest = None | S9 | `lower/nested.rs` | v1.0 |
| **AV9** | Upper | msr-bitmap + audit | `msr_bitmap_immutable_post_configure(vcpu)` — S10 immutable + profile + custom whitelist + default-deny + Migrate 시 bitmap 이동 | S10 | `upper/bitmap_immut.rs` | v1.0 |
| **AV10** | Upper | io-bitmap + audit | `io_bitmap_immutable_post_configure(vcpu)` — S11 의 9 default-block port group + immutable | S11 | `upper/bitmap_immut.rs` (shared) | v1.0 |
| **AV11** | Lower | audit | `audit_trail_complete(vcpu)` — S12 매 vmexit 가 ring buffer 에 기록 | S12 | `lower/audit.rs` | v1.0 |
| **AV12** 🆕 | Lower | audit | `audit_per_cpu_order(buffer)` — per-CPU ring 의 ordering 보존 (∀ entry e written on CPU c, ∃ read view r, e ∈ r ∧ ∀ e' written before e on c, ts(e') ≤ ts(e)) | S12.8 | `lower/audit.rs` | v1.0 |
| **AV13** 🆕 | Lower | audit | `audit_key_destroyed_unreadable(buffer)` — `key_destroyed ⇒ ∀ e ∈ buffer, decryptable(e) = false` (S12.5 의 XChaCha20 erase) | S12.5 | `lower/audit.rs` | v1.0 |
| **AV14** 🔄 | Upper | lifecycle | `vcpu_lifetime_subset(parent_tcb)` — vcpu lifetime ⊆ parent TCB lifetime *(이전 AV12)* | S13 | `upper/lifetime.rs` | v1.0 |
| **AV15** 🆕 | Upper | lifecycle | `orphan_frame_absent(host_memory)` — `∀ frame f, alive(f) ⇒ ∃ cap c, owns(c, f) ∧ ¬revoked(c)` (S13.6) | S13.6 | `upper/lifetime.rs` | v1.0 |
| **AV16** 🔄 | Lower | vmcb | `vmcb_field_whitelist_holds(vcpu)` — §4.1 화이트리스트 외 write 거부, §4.2 capsule-internal/microkernel-internal 노출 0 *(이전 AV13)* | §4 | `lower/vmcb_whitelist.rs` | v1.0 |
| **AV17** 🔄 | Upper | firmware-approval + audit | `firmware_mutation_pending_then_approved(vcpu)` — S14 의 모든 성공한 firmware mutation 이 pending → user-approved → applied 순서, audit trail 에 결정 기록 *(이전 AV14)* | S14 | `upper/firmware.rs` | v1.0 |
| **AV18** 🆕 | Lower | orchestrator (cluster-wide) | `capsule_dependency_acyclic(cluster)` — §2.4 의 의존 그래프 acyclicity, P1.6 §8.2 의 (a) topological order existence + (b) reachability cycle 부재 동치 | §2.4 / §8.2 | `lower/cluster_dep.rs` | v1.0 |
| **AV19** 🆕 | Lower | orchestrator | `boundary_count_bounded(vmexit)` — `boundary_count(handle_vmexit(e)) ≤ Y4_AMDV_MAX_BOUNDARY_PER_EXIT[e.code] ≤ 8` (P1.6 §8.3) | §8.3 | `lower/boundary.rs` | v1.0 |
| **AV20** 🆕 | Lower | orchestrator | `exhaustive_match(handle_vmexit, CapsuleMsg)` — §8.1 의 enum 모든 variant handler 강제 | §8.1 | `lower/dispatch.rs` | v1.0 |

🆕 = 신규 (6 추가, P1.6 + P2.2 결정 통합) / 🔄 = 번호 변경 (3 shift) / **Phase D** = forward-compat 항목

### 5.3 Layer 분포 통계

- **Upper (cross-tenant / cross-CPU)**: AV2, AV2-D, AV4, AV5, AV9, AV10, AV14, AV15, AV17 = **9 invariant**
- **Lower (within-cluster / capsule cooperation)**: AV1, AV3, AV6, AV7, AV8, AV11, AV12, AV13, AV16, AV18, AV19, AV20 = **12 invariant**

§3.1 의 S1~S14 Layer 분류 (Upper 8 / Lower 6) 와 비례 — Lower invariant
가 더 많은 이유는 §8.1/8.2/8.3 의 cluster 인프라 invariant (AV18/19/20)
가 모두 Lower (capsule 협력 측면) 라 그렇다.

### 5.4 Cross-ref 일관성 갱신 ledger

번호 충돌 해결 (C1):
- S12.8 본문의 "AV12" → 변경 0 (audit_per_cpu_order, 그대로)
- S13.6 본문의 "AV13" → **AV15 로 갱신** (orphan_frame_absent)

이전 catalog 의 AV12-AV14 cross-ref → 새 AV14/AV16/AV17 로 shift.

---

## 6. Contribute-back PR 분리 계획

### 6.1 안전장치 매핑 표 (cross-ref)

> ARCH-II' 의 3-column 매핑 표 (seL4 측 D1a / orchestrator / capsule)
> 는 **canonical source = `docs/vmm_arch.md` §4**.  본 §6 는 ARCH-II'
> 변경 시 단일 갱신점 보존을 위해 표 본문 보유 X — 매핑 조회는
> vmm_arch.md §4 참조.
>
> D-S2 (VMMCALL hypercall mediation) 와 D-S4 (L3 deadline ceiling 저장
> 위치) ledger 도 `docs/vmm_arch.md` §4 끝부분.

### 6.2 PR 분리 단위 (4 PR, P1.5 §6.3 timeline 정합)

| PR | 게시 plan | 내용 | 의존 (선결) | timeline |
|---|---|---|---|---|
| **PR-1** | seL4 mainline PR | D1a 의 raw-SVM C 패치 (`CONFIG_Y4_AMDV` gate, default OFF).  4 객체 종류 (SVMVCPU / SVMNPT / SVMMsrBitmap / SVMIoBitmap) + 6 syscall (Configure / Run / Migrate / ChangeParent / RebaseTsc / PollNestedRequest) + microkernel 측 검사 (S2 mandatory mask / S3.1/3.2/3.3 / S5 / S6 / S7 wrapper / S9 mandatory).  ~수백 LoC C | `sel4_fork_policy.md` v1.0 frozen + `amdv_safety.md` v1.0 frozen + 회귀 게이트 통과 | Phase C 진입 직후 |
| **PR-2** | Y4 워크스페이스 (Y4 GitHub) | **vmrun-orchestrator + 10 capsule sub-crate** (`Y4/vmrun-orchestrator/` + `Y4/capsules/vmm-{vmcb,npt,msr-bitmap,io-bitmap,firmware-approval,cpuid-emul,npf-handler,audit,nested-request,lifecycle}/`).  S1~S14 의 capsule 본체 코드 + Verus statement + 단위 test | PR-1 머지 또는 review 진입 | Phase C 중반 |
| **PR-3** | Y4 frozen tag + paper artifact | Verus 명세 + 증명 본문 (`proofs/verus/src/amdv/{upper,lower}/`).  AV1~AV20 + AV2-D 의 statement-only → body 채움.  USENIX/ACM artifact badge 충족 (P1.5 §6.5) | PR-2 머지 + capsule cluster 빌드 통과 | Phase C 종반 |
| **PR-4** | seL4 contribute-back PR-1 첨부 + `y4-verus2isabelle` 공개 | `y4-verus2isabelle` 도구 v1.0 산출물 — Y4 의 Verus 증명 → Isabelle/HOL `.thy` skeleton 자동 생성.  seL4 팀에 contribute-back 진입 장벽 절감 | PR-3 + `verus_to_isabelle.md` v1.0 frozen + 50+ invariant round-trip 검증 통과 | Phase C 종반 + PR-3 와 짝 |

### 6.3 PR-2 의 capsule cluster 분리 옵션 (sub-PR)

PR-2 는 11 workspace member (orchestrator + 10 capsule) 동시 도입 — 단일
PR 로 묶을 수도, capsule 별 sub-PR 로 쪼갤 수도 있음.  sign-off 후 PR
실작업 시점에 결정 — `phase_plan.md` Phase C 진입 단계에서 다시 검토.

권고:
- 단일 PR-2 (모든 11 member 동시) — 의존성 짧은 cycle, review 부담 ↑
- sub-PR (capsule 별) — review 부담 분산, 단 의존성 cycle 길음.
  추천 분할: `[orchestrator + audit + lifecycle]` → `[vmcb + npt]` →
  `[msr-bitmap + io-bitmap + cpuid-emul]` → `[npf-handler + nested-
  request + firmware-approval]` 의 4 sub-PR (DAG 의 sink-first 순서)

---

## 7. 동결 정책 (frozen / sign-off)

본 문서는 v0 spec.  `v1.0 frozen` 마킹 조건:

### 7.1 sign-off 조건

- **§3 14 개 안전장치 catalog (S1~S14) 모두 사용자 sign-off** — sub-decision
  포함 (§7.3 ledger)
- **§4 VMCB 필드 화이트리스트 + capsule scope + metadata 슬롯** 사용자 sign-off
- **§5 Verus invariant catalog 의 20 항목 statement** 사용자 sign-off
  (proof body 는 Phase C 진입 시 채움 — formal-first 의 statement-first
  sign-off)
- **§6 PR 분리 계획 (4 PR 단위)** 사용자 sign-off
- **ARCH-II' adoption (2026-05-04, `docs/vmm_arch.md` 채택)** 이 본 spec 의
  capsule 매핑 source — vmm_arch.md v1.0 frozen 와 짝 frozen

### 7.2 짝 doc 일괄 frozen 의존

본 doc 의 v1.0 frozen 은 **다음 4 doc 의 v1.0 frozen 과 짝으로만 발화**
(Phase 4 일괄 마킹 — sign-off 재구성 §1 정합):

| Doc | 짝 frozen 조건 |
|---|---|
| `docs/vmm_arch.md` | ARCH-II' canonical, capsule 매핑 source |
| `docs/sel4_fork_policy.md` | Strictly Additive Fork contract — D1a 패치 형식 의존 |
| `docs/verus_to_isabelle.md` | Verus → Isabelle/HOL skeleton 도구 — PR-4 의존 |
| `docs/amdv_safety.md` (본 doc) | 14 안전장치 + AV1~AV20 catalog |

4 doc 중 하나라도 sign-off 미완료면 본 doc frozen 발화 X — 단일 doc 수정
시 다른 doc 의 cross-ref 가 stale 되는 risk 차단.

### 7.3 안전장치 sub-decision sign-off ledger

S1~S14 의 각 sub-decision 채택 record (cross-ref 단일점):

| Safety | sub-decision 묶음 | 채택 |
|---|---|---|
| S1 | 본문 그대로 | 2026-05-04 |
| S2 | (a-i) intercept floor + 16-bit mandatory mask + CPUID emulation policy + INTERCEPT_VMMCALL 추가 + **D-S2 = orchestrator dispatch entry** | 2026-05-04 |
| S3 | (e) 본문 (3.1/3.2/3.3 + 3.4 Phase D forward-compat) | 2026-05-04 |
| S4 | (a) + (c) + (d') user-customizable + bootloader cmdline override + **D-S4 = vmcb capsule metadata 슬롯** | 2026-05-04 |
| S5 | (a) + (b) + (c) + (d) (logical APIC ID + Migrate atomic + offline re-pin + multi-vCPU per-vcpu) | 2026-05-04 |
| S6 | (f) parent thread group + lifecycle + ChangeParent + audit + fork transfer | 2026-05-04 |
| S7 | (d) clgi/stgi/vmsave/vmload/vmmcall 모두 syscall 표면 외 + INTERCEPT_VMMCALL mediation | 2026-05-04 |
| S8 | (d) Rebase + scaling disable + default offset + TSC_ADJUST 차단 | 2026-05-04 |
| S9 | (c) 3-layer 강제 + 명시 에러 path + R-α/R-γ forward-compat hook | 2026-05-04 |
| S10 | (e) immutable + profile + custom whitelist + default-deny + audit + Migrate 시 이동 | 2026-05-04 |
| S11 | (d) 9 default-block port group + immutable | 2026-05-04 |
| S12 | A=a-iii / B=b-ii / C=c-iii / D=d-i + Phase D hook / E=e-ii XChaCha20 / F=f-i / G=g-ii / H=h-ii | 2026-05-04 |
| S13 | A=a / B=a / C=a / D=a / E=a / F=a / G=a / H=a / I=b XChaCha20 segment-key | 2026-05-04 |
| S14 | A=a-iv / B=b-i / C=c-ii / D=d-ii / E=e-i dry-run / F=f-ii whitelist / G=g-i push+polling / H=h-ii CAS | 2026-05-04 |
| ARCH-II' adoption | vmm_arch.md 채택 (D1d monolithic 폐기, capsule 분해 + VeriSMo 영감) | 2026-05-04 |

### 7.4 v1.x patch / v2 의 정의

frozen 후 변경 분류:

| 분류 | 정의 | 예시 |
|---|---|---|
| **v1.x patch (backwards-compatible)** | (i) AV1~AV20 statement **약화 X** (강화는 OK).<br>(ii) syscall ABI 의 forward-compat 영역 활성화 (예: `PollNestedRequest` v1.0 = None 이 Phase D 에서 entry 반환).<br>(iii) 새 안전장치 (S15+) 추가 — 기존 S1~S14 의 invariant 약화 X 한정.<br>(iv) audit op_tag enum 의 새 variant 추가.<br>(v) capsule 의 새 verb 추가 (`CapsuleMsg` enum 의 새 variant).<br>(vi) build-time const 의 default 값 조정 (사용자 cmdline override 의 동작 보존). | Phase D 의 R-α/R-γ 활성화, IOMMU programming capsule 추가, audit disk-backed persistence 활성화 |
| **v2 (incompatible)** | AV statement 약화 또는 syscall ABI breaking 또는 capsule cluster 의 trust model 변경. | 예: orchestrator 에서 capsule 으로 cap 분배 model 변경, lease cap 의 fundamental schema 변경 |

frozen 후 v1.x patch 는 PR review + paper artifact 업데이트, v2 는 별도
재검토 cycle (S1~S14 + ARCH-II' 재검토 + paper revision).

### 7.5 frozen 후 진입 가능 작업

본 spec frozen → **Phase C 진입 차단 해제**:

`phase_plan.md` §C 의 7 단계 중:
1. ✅ `docs/amdv_safety.md` v1.0 frozen
2. ✅ `docs/vmm_arch.md` v1.0 frozen (짝 frozen)
3. ✅ `docs/sel4_fork_policy.md` v1.0 frozen (짝 frozen)
4. ✅ `docs/verus_to_isabelle.md` v1.0 frozen (짝 frozen)
5. (열림) seL4 D1a C 패치 (PR-1) 진입
6. (열림) vmrun-orchestrator + 10 capsule (PR-2) 진입
7. (열림) `y4-hypercall` 사용자 CLI repo (R-α/R-γ + S14 CLI) 진입

§6.2 의 4 PR 도 진입 가능.

---

## 8. 미해결 / 추가 결정 필요

### 닫힘 (sign-off 전 결정 — 2026-05-04)

1. ✅ **MAX_VMRUN_DEADLINE** → **`Y4_AMDV_MAX_DEADLINE_NS` build-time
   const, 기본 100 ms, 형상별 override**.  단위 ns (cross-arch portable).
   API: `Run(vcpu, deadline_ns)`.  자세히는 §2 ABI + §S4.
2. ✅ **multi-thread VMM** → **(ii) parent thread group**.  AV5 가
   "parent_tcb_pinned" → "parent_thread_group_pinned" (CSpace + VSpace
   공유 TCB 집합).  Worker pool 지원.

### v1.0 frozen 후 v1.x patch 로 닫힐 항목

3. **Audit trail 의 retention 정책** — ring buffer 크기, lease 회수
   시 sanitize 의 zeroing 보장.  runtime default → patch 가능.
4. **SVM nested page table walk 시 page table cache (TLB)** —
   `tlb_control` 필드의 default 값.  매 vmrun flush 가 안전하지만
   성능 손실 — runtime default → patch 가능.
5. **#VMEXIT 처리 중 host 가 vmload 호출 가능한가** — host state save
   영역 관리 정책.  seL4 내부 detail → patch 가능.
6. **AMD ASID 관리** — guest 별 ASID 할당, ASID 0 (host) 와 충돌 회피.
   seL4 내부 detail → patch 가능.
