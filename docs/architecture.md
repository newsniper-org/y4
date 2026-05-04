<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 — WaveTensor 가속기 모든 상용 폼팩터의 공통 기반 hypervisor

본 hypervisor 의 정식 명칭은 **Y4** 이다. 이하 본 메모와 다른 메모에서 "Y4" 는 곧 본 custom hypervisor 를 가리킨다.

> **이 문서가 Y4 디자인의 정전(canonical).** 원본 초안은 WaveTensor 프로젝트의
> `.claude-memos/host_a_custom_hypervisor.md` 로 작성되었으며 (CC-BY-4.0), 본 저장소는 그 정전을 Apache-2.0
> 으로 inbound 하여 보관한다 (양 라이선스 호환). WaveTensor 측 사본은 historical context 로만 유지되며,
> Y4 디자인 변경의 진실원은 본 파일이다.
>
> **연관 메모** (WaveTensor 측):
> - `WaveTensor/.claude-memos/remote_accelerator_access.md` — A의 호스트 OS 가 가벼우면 좋다는 원래 결정. 본 메모는 그 항목을 한 단계 더 깊이 파는 변형.
> - `WaveTensor/.claude-memos/sdk_architecture.md` — Rust 중심 stack 일관성. Y4 도 Rust 면 동일 ecosystem.
>
> **용어 사전:** 본 문서에 등장하는 HIU / partitioned TLB / shadow region /
> XChaCha20 / `context_switch` / lease / TRNG / Pod·Cluster·PE / wave 의
> 정확한 정의(RTL 시그널 폭, FSM 상태, threshold 등)는
> [`./glossary.md`](./glossary.md) 에 한 곳으로 모아져 있다. WaveTensor
> 측 정의가 바뀌면 사전을 우선 갱신한다.

## 제안 (확장 — 적용 범위 전면 확대)

WaveTensor 가속기를 탑재하는 **모든 상용 폼팩터의 공통 기반 OS** 로 Y4 (custom hypervisor) 채택:

- 서버팜 시나리오의 호스트 A (`remote_accelerator_access.md`)
- **특수목적 랩탑** (개인 ML/HPO 워크스테이션)
- **랙마운트 머신** (데이터센터 가속기 노드)
- **휴대 + 거치 겸용 머신** (Steam Deck 류 폼팩터)
- 그 외 SoC/SoM 임베디드 채택

각 폼팩터는 hypervisor 의 같은 base 를 공유하고, **사용자는 그 위에 Linux 배포판 또는 DragonFlyBSD 를 게스트 OS 로 직접 얹을 수 있음** — 즉 Type-1 hypervisor 가 guest OS hosting 도 first-class 로 지원.

```
┌────────────────────────────────────────────────────┐
│ User-chosen guest OS (Linux distro, DragonFlyBSD)  │  ← optional
│  - 사용자의 일상 환경                                │
│  - 가속기 SDK / imads-hpo / notebook UI 가 여기서   │
└────────────────────────┬───────────────────────────┘
                         │ paravirt / cap-restricted
┌────────────────────────▼───────────────────────────┐
│ Y4 — WaveTensor custom hypervisor (Type-1, Rust)         │
│  - seL4 microkernel base                            │
│  - Tock capsule 모델 (드라이버 격리)                 │
│  - 융합 IPC (DragonFlyBSD LWKT + Redox scheme)      │
│  - 융합 malloc (DragonFly SLAB + LLVM scudo)         │
│  - HIU / 가속기 lease cap                           │
└────────────────────────┬───────────────────────────┘
                         │
                  ┌──────▼──────┐
                  │ FPGA/ASIC   │  HIU + Pod 들
                  └─────────────┘
```

이전엔 "A 의 daemon" 만 hypervisor 가 호스팅하는 그림이었지만, 이제는 **가속기를 탑재한 모든 머신의 OS layer 그 자체** — 즉 우리 제품의 핵심 SW 자산.

**완전한 from-scratch 가 아님** — 각 참고 프로젝트의 검증된 컴포넌트를 살리면서 우리 특화 layer 만 자체 작성.

## 참고 OS 들 — 단순 영감 vs. 코드 reuse 후보

각 프로젝트가 **어디까지 reuse 가능한지** 명시:

| OS | 라이선스 | reuse 형태 | 가져올 것 |
|----|---------|-----------|----------|
| **LibrettOS** | (확인 필요, 학술 prototype) | 디자인 영감 위주 | privileged ↔ library 모드 전환 모델 |
| **Xen** | GPLv2 (hypervisor), 일부 BSD-style (paravirt headers) | dom0/domU 인터페이스 spec, 일부 도구 (libxc 등) port | Type-1 분할 모델, Iommu 셋업 시퀀스 |
| **seL4** | BSD-2-Clause | **그대로 binary 채택 가능** — verified microkernel 을 base로 깔고 그 위에 우리 가속기 layer | microkernel 자체 + 검증된 capability runtime |
| **DragonFlyBSD** | BSD-3 | vkernel / LWKT 의 **소스 일부 port** | thread scheduler, LWKT IPC 디자인 |
| **Redox OS** | MIT | Rust crate 단위 reuse (relibc, kernel core) | scheme-based IPC, Rust 빌드 인프라 |
| **Tock OS** | MIT/Apache-2 dual | 대거 reuse — embedded core 가 Rust capsule 모델 | capability typing, capsule pattern, 부팅 sequence |

라이선스 측면:
- **seL4 + Tock + Redox + DragonFlyBSD** 는 모두 **BSD/MIT 계열** → 우리 코드와 자유롭게 결합 가능 (proprietary 화 가능)
- **Xen** 은 GPL → hypervisor 부분 직접 link 시 GPL 전염. 인터페이스 spec / ABI 만 따르고 구현은 자체 작성하는 식으로 회피 가능
- **LibrettOS** 는 학술 prototype 이라 라이선스 확인 후 차용 범위 결정

## Licensing 정책 (확정)

**상용화하더라도 hypervisor 자체는 fully free and open source 로 유지.**

이는 다음을 의미:
- hypervisor 핵심 코드 (우리가 작성한 specialization layer + base 통합) 는 **공개 저장소** 에서 항상 접근 가능
- 상용 제품 (랩탑, 랙마운트 등) 에 탑재하더라도 그 안의 hypervisor binary 는 source 와 매칭되며 build 가능
- proprietary fork / 사내 비공개 branch 금지 — 외부 출시 전에 모든 변경이 upstream
- 다른 가속기 / 호스트 환경에서도 제3자가 자유롭게 차용 가능

### 라이선스 선택지 + 권장

| 옵션 | 성격 | 우리 base 호환성 | 권장도 |
|------|------|----------------|-------|
| **Apache-2.0** | permissive + 명시적 특허 grant | seL4(BSD-2), Tock(MIT/Apache-2), DragonFlyBSD(BSD-3) 모두 OK | **★ 1순위** |
| **MIT** | permissive 최소 | OK | 2순위 |
| **BSD-3-Clause** | permissive 전통 | OK | 2순위 |
| **GPLv2** | copyleft | seL4 와 충돌 (BSD-2 + GPLv2 결합 가능하지만 결과물 GPLv2) | 비권장 |
| **GPLv3 / AGPL** | strong copyleft | 위 + tivoization 금지 | 비권장 (임베디드 / 펌웨어 서명 시 문제) |
| **MPL-2.0** | file-level copyleft | OK | 3순위 |

**권장: Apache-2.0**.

근거:
- "fully FOSS 유지" 는 약속이지만 **사용자 / 외부 벤더가 hypervisor 위에 proprietary 게스트 / 응용을 얹는 것** 은 허용해야 함 (Linux 게스트 안의 사내 SW, imads-hpo 의 사용자 모델 등). permissive 가 자연스러움.
- Apache-2.0 의 특허 grant 가 가속기 IP / hypervisor 특허 분쟁 방어에 도움
- seL4 / Tock / BSD 컴포넌트의 라이선스와 모두 호환
- 향후 다른 OSS 프로젝트 (예: Rust crates 생태계) 와 결합 시 마찰 적음

### Linux driver 3순위 tier 와의 상호작용

이전에 언급한 "Linux driver 포팅 = GPLv2 전염" 은 다음과 같이 정리:

- Y4 메인 트리: **Apache-2.0** (위 결정)
- GPL'd Linux driver 를 그대로 차용하는 capsule: **GPLv2 라이선스 그대로 유지**, 별도 binary
- 메인 트리는 GPL capsule 을 **외부 ABI 로만** 호출 — 직접 link 하지 않음
- 이 경계 덕에 Y4 메인 트리 는 Apache-2.0 으로 깨끗 유지, GPL capsule 은 GPL 로 따로 distribute (Linux 처럼)

법적으로 가장 안전한 패턴은 Linux kernel 자체가 채택한 "loadable module + stable in-kernel ABI" 모델과 유사. 우리 capsule 인터페이스를 이 형태로 디자인.

### 상표 / 브랜드는 별도

코드는 FOSS, 그러나 **"WaveTensor" 상표 / 로고 / 인증된 제품 식별자** 는 별도 정책. 외부에서 hypervisor fork 시 자체 이름으로 배포 가능, 우리 상표는 상용 제품에만 사용.

### 외부 기여 / 거버넌스

- 공개 저장소 (GitHub or 자체 git host)
- CLA (Contributor License Agreement) 또는 DCO (Developer Certificate of Origin) 결정 필요
- 보안 취약점 disclosure 정책
- 릴리스 cadence / LTS 정책

이 거버넌스 세부사항은 외부 첫 기여자 발생 시점에 정리. Phase 0 / Phase 1 단계엔 내부 개발만.

## 확정된 기술 stack

Y4 가 기반으로 삼을 컴포넌트 조합 (사용자 결정):

### Microkernel base — **seL4**
- BSD-2-Clause, formally verified
- 우리 코드의 verified TCB 가 됨. 위에 올리는 모든 layer 는 seL4 가 보장하는 capability 격리 안에서 동작.

### Driver / 디바이스 격리 — **Tock capsule 모델**
- MIT/Apache-2 dual
- 각 디바이스 드라이버가 Rust capsule 로 격리, capability typing 으로 cross-driver leak 방지
- HIU / PCIe / USB / CXL controller 모두 capsule 로 작성

### IPC — **DragonFlyBSD LWKT + Redox scheme — 두 layer 동시 노출 (hybrid)**
- DragonFlyBSD LWKT (BSD-3): lightweight kernel thread + per-CPU cache,
  lock 최소화 → 가속기 wave 단위 latency 결정론. 데이터평면 fast-path.
- Redox scheme (MIT): pluggable namespace (file/network/event 통합) →
  사용자 환경 친숙성. 제어평면 표준.
- **두 API 동시 노출 (hybrid):** Redox scheme verbs (`open`/`read`/`write`/
  `close`) 가 제어평면 표준 — lease 발급, capability mint, 자원 발견 등.
  LWKT raw msgport 가 데이터평면 fast-path — HIU MMIO 디스패치, accel-d
  ↔ tenant 전송 등. 두 API 가 서로의 race 를 만들지 않음을 명세에서 증명.
- ipc/ 와 alloc/ 는 서로 독립 — LWKT 는 dispatch-only (메시지는 caller
  가 alloc 에서 받아 넘김).

> **외부 API 경계 변경 기록 (2026-05-04):** 원안의 "scheme 외부 / LWKT
> 내부" 단일 API 대신 두 layer 동시 노출 (hybrid) 로 결정. 이유: scheme
> dispatch overhead (~60–100 cycle) 가 LWKT 의 lock-free 이점 (~10–30
> cycle) 의 2–10× → 데이터평면에서 그대로 흡수 불가. 결정 상세는
> 메모리의 `y4_ipc_alloc_preflight.md` I-P2 참조.

### Memory allocator — **DragonFlyBSD lock-free SLAB + LLVM scudo 융합**
- DragonFlyBSD lock-free SLAB (BSD-3): 멀티코어 확장성 핵심 — per-CPU
  magazine, hot-path 객체 캐시
- LLVM scudo (Apache-2.0): NUMA-aware 백엔드 + 보안 (UAF 검출,
  randomization, guard pages 등 hardened 스택 전체)
- **융합**: DragonFly SLAB front-end + scudo backend. SLUB 의 NUMA-aware
  partial-list 역할은 scudo 가 흡수. OpenBSD malloc 의 보안 가치도 scudo
  가 흡수.
- 가속기 메모리 할당 (zero-copy ring buffer 등) 에 결정론적 latency +
  보안 동시 확보.

> **컴포넌트 변경 기록 (2026-05-04):** 원안의 "SLUB + lock-free SLAB +
> mmap-only" 3-자 융합에서 SLUB 와 OpenBSD malloc 을 제거하고 scudo
> 단일 백엔드로 교체했다. 이유: (i) Linux SLUB 가 GPL-2.0 이라 직접
> 코드 차용 불가, (ii) scudo 의 LLVM hardened 스택이 NUMA-aware +
> OpenBSD malloc 의 보안 가치 (UAF / random / guard) 를 모두 만족,
> (iii) Apache-2.0 라이선스로 Y4 main tree 에 직접 link 가능. 결정
> 상세는 메모리의 `y4_ipc_alloc_preflight.md` A-P1 참조.

### Bootloader — **기존 OSS bootloader 최소 수정 채택** (자체 개발 X)

Y4 자체 부팅 단계는 **자체 작성하지 않고 기존 검증된 OSS bootloader 를 그대로 가져다 사용**한다. 이유:
- bootloader 는 platform 다양성 (UEFI 변종, ARM ATF/EDK2, RISC-V SBI, coreboot payload, Secure Boot 인증 chain) 을 모두 흡수해야 하므로 자체 작성 시 지속적 maintenance 부담이 큼.
- 우리 차별화의 핵심은 가속기 lease capability / HIU 통합 / wave-aligned scheduler 이며 부팅 단계는 그 가치 영역 밖.
- 기존 OSS bootloader 들은 이미 광범위한 hardware coverage + Secure Boot signing infra + serial/network recovery 기능을 갖춤. 이를 **bypass 또는 fork** 해 재구현하는 것은 ROI 가 명백히 음수.

**채택 우선순위** (Y4 가 그 위에 올라가는 chain-load 형태):

| 순위 | bootloader | 라이선스 | 채택 이유 |
|---|---|---|---|
| 1차 | **Limine** | BSD-2-Clause | 모던, minimal, kernel-dev 친화적 인터페이스 (Limine boot protocol). systemd 등 외부 ecosystem 의존성 0 — Y4 의 BSD/Redox/Tock + Rust 정체성과 라이선스·생태계 양쪽에서 1:1 정합. UEFI x86_64 / ARM64 / RISC-V 모두 지원. boot entry 는 평문 `limine.conf` 단일 파일 → Y4 image build 파이프라인이 직접 fwrite. |
| 2차 | **GRUB2** (BLS 패치판) | GPLv3 | 산업 표준. multi-arch 커버리지 압도적. **GPL 전염 회피**: Y4 메인 트리는 GRUB2 binary 를 chain-load 만 하고 직접 link 하지 않음 (Linux kernel 모델과 동일 패턴). BLS entry 직접 write 가능 → grub-mkconfig 없이도 등록. |
| 3차 | **U-Boot** | GPLv2 | ARM SoC / 임베디드 폼팩터에서 사실상 표준. 위 2차 와 동일한 chain-load 격리. SPL → U-Boot proper → Y4 의 일반적 ARM 부팅 chain 그대로 활용. |
| 4차 | **coreboot + LinuxBoot/Heads payload** | GPLv2+ | 펌웨어 수준 신뢰 chain 이 필요한 인증 트랙 (Phase 4) 의 옵션. 본체 펌웨어 수준 영향이라 도입 시점은 한참 후. |
| **제외** | systemd-boot | LGPL-2.1+ | EFI 바이너리 자체는 UEFI 런타임 의존성만 갖지만 **boot entry 관리 체인 (`bootctl`, `kernel-install`, `sdbootutil`) 이 systemd 프로젝트 일부** → Y4 처럼 systemd 를 ship 하지 않는 OS 가 자기 entry 를 유지하려면 외부 systemd 환경에 의존해야 함. BSD-only 개발 머신에서 build 파이프라인이 막히고 Y4 의 ecosystem 정체성과 misalignment. **MicroOS / 일반 systemd-Linux 측의 native bootloader 로는 적합하지만 Y4 측 후보에서는 제외.** |
| **제외** | rEFInd | BSD-3 | 대용량 multi-boot menu 의 정성적 사용성 우수. 그러나 transactional-update 모델과의 통합 hook 부재 → 우리 폼팩터에 적용 시 maintenance 부담 (별도 메모 §`.private` 참조). |

**최소 수정 원칙**:
- 우리는 bootloader 의 **소스 트리를 fork 하지 않는다**. upstream 패키지 (또는 distro 빌드) 를 그대로 사용.
- Y4 에 필요한 추가 항목 (예: HIU 부팅 단계의 IOMMU 사전 셋업, partitioned TLB 초기화) 은 **bootloader 가 아니라 Y4 의 첫 stage** 에서 처리. bootloader 는 단순히 Y4 를 load + execute 만.
- Secure Boot key enroll, BLS entry 형식 등 spec 표준에 정렬된 인터페이스만 사용.
- **그래도 패치가 필요한 경우** (예: 가속기 PCIe IDE 가 부팅 시 어떻게 해야 하는 등): upstream 에 patch 제출이 1순위, 우리 트리의 fork 는 최후수단. fork 시에도 git submodule 로 격리 + 상시 upstream rebase.

**결과**:
- Y4 image 는 standard EFI executable (UEFI) 또는 ARM Image (U-Boot) 형태로 빌드. distro 의 sdbootutil / grub2-mkconfig 가 자동으로 entry 등록.
- bootloader maintenance 가 우리 책임 외 — distro / upstream 이 보안 패치 / 신규 hardware 지원 자동 제공.
- Phase 4 인증 시 bootloader 부분은 "사용된 공식 OSS + 인증 받은 Secure Boot signing" 으로 패스.

### Verification 방식 — **선 정식증명 → 후 구현** (formal-first)

**핵심 원칙**: 컴포넌트 작성 전에 **명세 + 증명을 먼저 작성**, 증명이 통과한 후에야 구현.

- seL4 가 이미 microkernel 부분 정식증명 → Y4 는 그 위 layer 의 증명 만 추가하면 됨
- 도구 후보:
  - **Coq / Lean 4** — high-level 명세 + 증명
  - **Frama-C / Why3** — C 코드 (driver 일부) 정식 검증
  - **Creusot / Verus** — Rust 코드 정식 증명 (우리 stack 과 직접 일치)
  - **Kani** — Rust bounded model checking (보완용)
- **Verus** 가 Rust 친화적이라 1차 후보. 단, 표현력 한계가 있으면 핵심 보안 invariant 만 Coq, 나머지는 Verus

formal-first 가 가져다 주는 결과:
- 가속기 lease capability 의 **OS-level 격리 보장이 수학적으로 증명됨** → 의료/항공/금융 인증 트랙 진입 시 강력
- side-channel 회피 invariant (TLB flush, scheduler isolation) 가 증명 단계에서 명세화 → HIU 의 hardware 보호와 SW 보호가 동일 모델로 검증

### 우리 특화 layer (자체 작성)

위 base 위에 다음만 자체 작성:
- HIU 통합 — context_switch / partitioned TLB / shadow regions / XChaCha20 capability binding
- 가속기 lease scheduler (wave-aligned preemption)
- 가속기 daemon RPC (서버팜) / 로컬 IPC (워크스테이션) 를 동일 namespace API 로 노출
- guest OS hosting (Linux/DragonFlyBSD) — paravirt 인터페이스

공통 테마:
1. **TCB 최소화** (small kernel, microkernel/library OS)
2. **Capability-based isolation** (lease 의 OS-level 표현)
3. **Rust 우선** (메모리 안전 + 우리 SDK stack 과 일치)
4. **Direct hardware access** (PCIe/USB/CXL → 가속기 daemon 까지의 hop 최소화)
5. **검증된 base 위에 specialization 만 작성** — from-scratch 부담 최소화

## 전체 아키텍처 (개략)

```
┌──────────────────────────────────────────────┐
│ A (가속기 호스트)                             │
│                                              │
│ ┌────────────────────────────────────────┐   │
│ │  Y4 (Type-1 Rust hypervisor)       │   │
│ │  ─────────────────────────────────────  │   │
│ │  ↑ 책임:                                │   │
│ │   - boot, IOMMU, HIU shadow region 셋업 │   │
│ │   - lease scheduler (capability)        │   │
│ │   - IPC (RPC / shared memory)           │   │
│ │   - PCIe/USB/CXL host stack             │   │
│ │   - 사이드채널 hygiene (TLB flush 등)    │   │
│ └────────────────────────────────────────┘   │
│   │ caps                  │ caps             │
│   ▼                       ▼                  │
│ ┌─────────┐         ┌────────────┐           │
│ │ accel-d │         │ tenant-VM  │   ...     │
│ │ daemon  │         │ (lease 점유)│           │
│ │ (priv.) │         └────────────┘           │
│ └─────────┘                                  │
│   │                                          │
│   ▼                                          │
│ FPGA 가속기 (HIU + Pod 들)                    │
└──────────────────────────────────────────────┘
```

핵심 elements:
- **bare-metal**: A 에 OS 가 따로 없음. hypervisor 가 곧 OS 의 핵심
- **accel-d**: 가속기 daemon 은 hypervisor 가 부여한 capability 로만 가속기 접근
- **tenant compartment**: M 에서 잡 발사 시 A 에 lightweight compartment 생성 — 가속기 lease 부여, 끝나면 해제
- **shared memory ring**: tenant ↔ daemon 사이 zero-copy (HIU 의 zero-copy 와 자연스럽게 결합)

## 가속기 입장에서의 의의

### HIU 의 기존 보안 메커니즘과 자연스럽게 맞물림

`HIU.v` 가 이미 갖춘 기능들이 hypervisor 책임과 정확히 매핑됨:

| HIU 기능 | hypervisor 측에서 활용 |
|---------|------------------------|
| `context_switch` 신호 | tenant lease 전환 시 hypervisor 가 raise — 동일 cycle 에 TLB flush 발동 |
| Partitioned TLB (4 partition) | tenant 4개 까지는 각 partition 격리 → 동시 multi-tenant 가능 |
| Shadow regions (16 entries) | hypervisor 가 tenant 별 메모리 영역을 shadow 로 등록 → IOMMU 보호 |
| XChaCha20 masking | tenant 별 key/nonce — 동일 가속기에서 다른 tenant 가 옛 데이터 reconstruct 불가 |
| `flush-on-context-switch` | 자동 — 따로 OS 가 청소할 필요 없음 |

→ 우리 가속기는 **이미 hypervisor-friendly 한 디자인**. Y4 가 그 잠재력을 실제로 활용.

### Lease 의 OS-level 모델링

`remote_accelerator_access.md` 의 lease 가 ad-hoc daemon-level 객체였던 것을 **hypervisor capability** 로 격상:

- M이 RPC 로 lease 요청 → hypervisor 가 capability token 발급
- token 이 tenant compartment 의 cap-table 에 등록
- 그 compartment 가 가속기 MMIO/메모리에 접근 시 hypervisor 가 cap 검증
- token 만료 시 hypervisor 가 (a) compartment teardown, (b) HIU `context_switch` raise, (c) 가속기 상태 sanitize 의 3-step atomic 청소 자동 수행

Linux 위에서 사용자 공간 daemon 으로 같은 일을 하려면 syscall 다수 필요. hypervisor 라면 단일 op.

## 트레이드오프

### 장점

- **TCB 작음 → 가속기 데이터 보안 ↑**: tenant 데이터가 일반 Linux 스택을 거치지 않음
- **결정론적 latency**: cgroup/preempt 같은 일반 OS 변수가 없으므로 RPC 응답 시간이 더 예측 가능
- **side-channel 강화**: Spectre/Meltdown 류 공격면 축소 (fewer privilege levels, fewer shared structures)
- **lease 격리 OS-level**: capability 로 강제, 사용자 공간 버그가 cross-tenant leak 으로 이어지지 않음
- **Rust 일관성**: SDK + imads stack 과 동일 ecosystem
- **WaveTensor 기능 활용도 ↑**: HIU 의 partitioned TLB / shadow regions / XChaCha20 이 첫째날부터 의미 있게 동작

### 단점 / 큰 비용

- **개발 부담 (완화됨, 그러나 여전히 큼)**: from-scratch 가 아니더라도 base OS 채택 + porting + 통합 + 인증은 수 사람-월 ~ 수 사람-년. 기존 Linux + daemon 대비 10~30배 일 (from-scratch 100배 대비 완화)
- **드라이버 생태계 부족**: PCIe/USB/CXL host controller 드라이버는 base OS (seL4 / Tock 등) 가 가진 만큼만 활용. **3-tier driver 전략** 으로 완화 — DragonFlyBSD (BSD) 1순위 → NetBSD via rump kernel 2순위 → Linux port (해당 driver 한정 GPLv2) 최후수단
- **메인테넌스**: 보안 패치, 새 하드웨어 지원 등 모든 책임이 우리 팀
- **디버깅 도구 빈약**: gdb-stub, kprobes, eBPF 같은 도구 모두 직접 만들거나 포기
- **사용자 onboarding 어려움**: 외부 기여자가 Linux 익숙 → Y4 학습 곡선
- **합법성/인증**: 특정 산업 (의료/항공) 인증을 받으려면 OS 자체도 인증 트랙. seL4 처럼 formally verified 가야 인증이 짧아짐

### 중간 옵션

| 옵션 | 부담 |
|------|------|
| (i) Linux + 가속기 driver/daemon (현 plan) | 작음 |
| (ii) Linux + KVM 위에 LibrettOS/Redox/Tock 의 light tenant compartment | 중간 |
| (iii) **bare-metal Rust hypervisor 직접 (Y4 — 이 메모의 제안)** | 큼 |
| (iv) seL4 도입 + 그 위에 가속기 daemon | 중대형 (seL4 자체 학습 + 통합) |
| (v) Xen Type-1 + dom0 가 Rust unikernel | 중간 |

(iii) 이 가장 깨끗하지만 가장 비쌈. 실무적으로는 **(iv) seL4 위에 daemon** 또는 **(v) Xen + Rust unikernel dom0** 가 절충점.

### 권장 진입 경로 (확장 spec 반영)

```
Phase 0  Linux + Rust daemon              ← 현재 작업 진행 중
            |
            v   (PoC + 가속기 RTL 안정화 후)
Phase 1  seL4 + 가속기 daemon 만 (서버팜 A 한정)
            |  - seL4 위 daemon 이 lease cap / HIU 통합
            |  - guest OS 호스팅 미포함 (Linux on bare-metal A 와 병존)
            v
Phase 2  융합 IPC + Tock capsule + 융합 malloc 작성
            |  - 각 컴포넌트는 formal 명세 + 증명 → 구현 순
            |  - Verus / Coq 도구체인 정착
            v
Phase 3  Type-1 hypervisor 화 + guest OS hosting
            |  - Linux distro / DragonFlyBSD 게스트 hosting paravirt
            |  - 랩탑 / 랙마운트 / 휴대-거치 폼팩터 공통 base 로 진입
            v
Phase 4  formal-verified 인증 트랙 + 외부 출시
```

각 Phase 사이에 **충분한 운영 데이터 + 정식증명 마일스톤** 누적 후 다음 단계로. 가속기 RTL 자체가 안정화되기 전에 hypervisor 작업까지 병행하면 둘 다 안정화 안 됨 — Phase 1 진입 자체를 **가속기 보드 검증 완료 + GPU 비교 (III) 완료** 이후로 잡음.

## 우리 일정과의 관계

- 현재 진행 중: 가속기 RTL + Vivado 합성 + (a) mitigation
- 직후: HIU loopback (IV), BCAST (II), GPU 비교 (III), MCM 분할
- **그 다음** 영역: 가속기 사용 SW stack (SDK, daemon, notebook UI, imads-hpo 통합)
- **그 다음** 의 그 다음: 본 메모의 hypervisor 영역

즉 본 메모는 **현재 작업의 직접 trigger 가 아니며**, 가속기 자체가 안정 시제품으로 검증된 후의 "차세대 인프라" 영역. 지금은 *방향성 기록* 용.

## 미해결 / 추후 결정

- ~~**Type-1 vs Type-2**~~ → **Type-1 확정** (모든 폼팩터 공통 base)
- ~~**Verification 야망 수준**~~ → **선 정식증명 → 후 구현 (formal-first) 확정**. Verus/Rust + Coq 보완.
- **micro-VM 단위**: 게스트 OS hosting 시 KVM 식 full VM ? Firecracker microVM ? Xen-style domU ? — Phase 3 진입 시 결정
- **드라이버 출처**: Tock capsule 모델로 작성하되 reference impl 은 어디서? 우선순위 stack:
  1. **DragonFlyBSD 의 BSD 라이선스 driver** (1순위) — cherry-pick + Rust capsule 로 wrap
  2. **NetBSD driver via rump kernel** (DragonFlyBSD 미지원 장치) — NetBSD 의 anykernel 기술. rump kernel 이 driver 를 userspace 컴포넌트로 격리해 다른 시스템에 이식 가능. BSD-2 라이선스 → 우리 stack 과 호환
  3. **Linux driver port** (위 둘 다 미지원 장치, 최후수단) — **해당 driver 한정 GPLv2 전염 감수**. 격리 전략:
     - GPL 모듈을 별도 capsule binary 로 분리 → 나머지 hypervisor 와 license boundary 명확
     - 외부 ABI 만 노출, GPL 코드를 직접 link 하지 않는 user-space helper 가 wrap
     - 해당 driver 가 다루는 디바이스의 docs/spec 이 공개되어 있다면 장기적으론 자체 작성으로 교체 (라이선스 cleanup)

  카테고리별 1차 후보:
  - PCIe / IOMMU: seL4 + DragonFlyBSD
  - USB host (XHCI/EHCI): NetBSD via rump (DragonFlyBSD 의 USB stack 대비 NetBSD 가 더 portable)
  - CXL: 신생 표준 → 자체 작성
  - 일반 Ethernet NIC: DragonFlyBSD 1순위, 부족 시 NetBSD rump
  - GPU (display only, 가속기와 별개): 사용자가 게스트 OS 안에서 기존 driver 사용 — hypervisor 가 GPU 를 게스트에 passthrough
- **scheduler**: 가속기 lease 전환은 wave 단위와 정렬되어야 함 — DragonFlyBSD LWKT 의 per-CPU 모델을 가속기 wave 단위로 확장
- **guest OS 호환**: Linux 와 DragonFlyBSD 가 paravirt interface 를 어떻게 쓰는가? Xen-PV 호환? virtio? 또는 자체 인터페이스?
- ~~**부팅 / 펌웨어**~~ → **Bootloader 자체 개발 X, 기존 OSS bootloader 최소 수정 채택** 으로 확정 (위 §Bootloader). 1차 Limine, 2차 GRUB2-BLS, 3차 U-Boot, 4차 coreboot. systemd-boot / rEFInd 는 Y4 에서 제외 (이유는 §Bootloader 표 참조). 폼팩터별로 위 우선순위 안에서 선택.
- **업데이트 / OTA**: hypervisor + guest OS 의 양면 업데이트 정책 (signed image, A/B partition 등)
- **인증 트랙**: 의료 (FDA 510k), 항공 (DO-178C), 금융 (FIPS 140-3) 중 어느 것을 먼저? formal-first 라 모두 가능하지만 우선순위 결정 필요

## 진입 트리거

- 가속기 RTL 이 FPGA 합성 완료 + 보드 동작 검증된 후
- daemon (Phase 0) 이 Linux 위에서 실제 운영 데이터 수집 — 어떤 lease 패턴, 어떤 보안 요구가 있는지 확인된 후
- 외부 사용자 (보안 민감 도메인) 가 발생 시 — 인증 요구가 hypervisor 채택의 ROI 를 정당화
- 또는 사용자께서 "Phase 1/2/3 본격 시작" 명시
