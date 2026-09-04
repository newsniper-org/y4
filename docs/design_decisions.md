<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 설계 결정 메모 — `.brainstormings/` 시리즈 승격

## 0. 지위 / 출처

본 문서는 `.brainstormings/` 의 브레인스토밍 시리즈(19편)에서 도출된 **설계
결정을 canonical 형태로 통합·승격**한 것이다.  개별 브레인스토밍 문서는
**historical detail(옵션 탐색 + 근거)**로 보존되고, 본 메모는 그로부터
**합의된 결정 + 시리즈를 관통하는 횡단 원칙**을 캐논으로 고정한다.

- 성격: **최종 구현 목표(final implementation target) 기준**의 설계 결정.  현
  Phase B 스캐폴드를 넘어서는 forward 설계가 다수 — 실제 구현 시 본 메모가
  기준이 되고, 상세 근거는 링크된 브레인스토밍을 참조.
- 이미 별도 도메인 문서로 승격된 것: **`docs/hw_mechanism_abstraction.md`**
  (attestation §8.5 → P3).
- 검증(Verus) 착수는 다수가 **verus-fork 안정화 이후**(§3).

## 1. 횡단 원칙 (emergent architecture) ★

시리즈를 관통하며 반복적으로 수렴한 원칙들.  이것이 승격의 핵심 산출물이다.

### P1. atomic-free composition
Y4-authored 어떤 subsystem 도 내부에서 atomic 을 쓰지 않으며, **cross-CPU 협조
는 shared-memory atomic 이 아니라 message-passing(per-CPU·IPC)으로만** 이뤄진다.
seL4 mechanism 내부 동기화(SMP big lock 등)는 **verified given**(원칙 대상 X).
- 근거: alloc §0.5 · ipc §0.5 · scheduler §7
- 귀결: deterministic latency(WCET) · Verus-only 검증(Kani/loom 불요) ·
  cross-platform(arch memory model 회피)

### P2. partition_id = master key
단일 key(`partition_id`, P0..P3 = HIU)가 **모든 격리 차원**을 index 한다 —
CPU memory(MMU+allocator zone) · device DMA(IOMMU domain) · microarchitecture
(LLC cache color · host TLB flush · WaveTensor TLB partition · SMT group ·
seL4 scheduling domain) · crypto key namespace.  **"하나의 partition 모델,
N enforcers."**
- 근거: capability-bound-alloc · capsule §4.2(FA6) · side-channel §2 ·
  scheduler §2 · key-mgmt §3

### P3. 하드웨어/플랫폼/디바이스 메커니즘 추상화
Y4 spec 은 벤더·ISA·디바이스별 **realization 을 고정하지 않고 추상 계약만
고정**; realization·구현시점은 deployment·form-factor·cross-repo ABI 가 결정;
어떤 realization 인지 **provenance 보고**, 수용은 relying party 정책.
- **→ 도메인 문서 `docs/hw_mechanism_abstraction.md` (canonical)**
- registry: CPU virt · RoT · measured-boot 명령 · confidential-VM ·
  device attestation · measurement-log 접근 · modem I/F · RAT · IMS 위치 ·
  UICC 형태 · IOMMU

### P4. Y4-owned vs 추상화 경계
P3 의 여집합: Y4 가 **소유·설계·검증**하는 산출물(RIM · reproducible build ·
provenance · release signing · capability 모델 · spec/proof · IPC/allocator/
capsule semantics)은 추상화 대상이 아니다.  판별: **외부 하드웨어 메커니즘 →
추상화; Y4 저작 산출물 → 소유.**
- 근거: hw_mechanism_abstraction §3 · reproducible-build §11

### P5. standard-first reuse (원칙 5 구체화)
verified/standard base 를 재사용하고 **Y4-특화 층만 저작**: seL4 mechanism
(scheduler) · Limine measured-boot(boot §3.1) · PikeOS/ARINC 653 스케줄링
모델(scheduler §5.1) · 모뎀-네이티브 failover(WWAN §4) · transparency log
(reproducible §7) · Asterinas unsafe-confinement 선례(§3.1).
- **따름정리**: 표준 거동은 종종 **더 안전**하다(WWAN §4 — 모뎀-네이티브 failover
  가 네트워크 예상 범위를 유지 → 이상 거동 오검출 회피).

### P6. terminate-then-re-originate (정체성/신뢰 경계)
정체성·신뢰 경계에서는 bridge/passthrough 가 아니라 **민감 컨텍스트를 종단하고
로컬 인터페이스를 재기원**한다.  인스턴스: WWAN switch(session) · 2계층
credential(USIM) · 데이터경로(IMS=terminate, internet=route).  복제-방지가
이로써 **구조적으로 강제**됨.
- 근거: WWAN §1/§2/§3/§5

### P7. TCB 최소화의 source→runtime 확장
신뢰 최소화가 source → build → boot → runtime → attestation 을 관통한다:
verified seL4(runtime) + measured boot(boot) + reproducible build·RIM·provenance
(build/source) + attestation(증명).  **독립 rebuilder 가 build 신뢰까지 최소화.**
- 헤드라인: "감사 가능한 code 가 실제 도는 code 임이 증명"(reproducible §1)
- 근거: attestation · boot · key-mgmt · reproducible-build

### P8. verified-base-bind
attestation 이 임의 바이트가 아니라 **형식 증명된 seL4**(측정 해시 == 증명된
버전)에 bind → "형식 증명된 microkernel 이 부팅됨"을 증명.  **measured boot ×
formal verification.**
- 근거: attestation §3 · boot §7

### P9. 검증-모드 경계
층마다 검증 모드가 다르다 — **Verus**(management/logic/invariant) · **rebuild-
and-compare**(reproducibility) · **서명 체인**(provenance) · **trusted
assumption**(crypto primitive 강도 · seL4-inherited).  Verus 는 "올바르게
다룬다"를 증명하지 "암호가 안전하다"를 증명하지 않는다.
- 근거: key-mgmt §9 · reproducible §9 · side-channel §8 · WWAN §9

### P10. fail-closed + 안전·결정성 > throughput
반복된 선택: async cross-CPU free(alloc) · IOMMU-fence-always(capsule §4.1) ·
Y4-mediated demux(WWAN §3) · anti-rollback floor(boot §6.4) · stateless
restart(capsule §3).  미신뢰/불확정 상황에서 **안전 쪽으로 닫는다.**

### P11. capability-based everything
lease · MMIO · DMA window · CPU 시간(SC) · key 접근 · 서비스(voice/SMS/MCX/
bearer) · USIM-AKA 권한 = 전부 capability.  **단일-보유 capability 가 단일성을
강제**(USIM-AKA, 네트워크 컨텍스트).
- 근거: capability-bound-alloc · capsule C1/C2/C3 · scheduler §5 · key-mgmt §2 ·
  WWAN §5

## 2. 주제별 결정 digest

각 항목 = 핵심 결정 한 줄 + 상세 브레인스토밍 링크(historical).

### 2.1 Memory allocator (front-end)
- [alloc-frontend-improvements](../.brainstormings/20260618-212218-alloc-frontend-improvements.md)
  — DragonFly SLAB(최종 목표)의 atomic-free SLUB-style 현대화; §0.5 atomic 배제
  (불가침); SLUB 데이터 구조만(unqueued+metadata-in-page); cross-CPU free =
  IPC 위임; Verus-only.
- [pluggable-alloc-frontend-contract](../.brainstormings/20260618-221518-pluggable-alloc-frontend-contract.md)
  — 2-tier trait(`AllocFrontend`+`Freelist` 분리) + Verus contract(C1~C4/FL1~FL3);
  generic-first; atomic-free = lint + Verus.
- [y4-variant-slob](../.brainstormings/20260618-222938-y4-variant-slob.md)
  — Linux SLOB 의 Y4 변형(global lock→atomic-free, coalescing→deferred);
  C3 form-factor 차등 첫 사례.
- [capability-bound-allocation](../.brainstormings/20260618-223701-capability-bound-allocation.md)
  — partition-keyed zone(P0..3, I1 cross-tenant UAF 차단) + lease-lifecycle
  bulk reclaim; **모든 aggregation 은 lease lifecycle 에서만**(atomic 0);
  `CapabilityAlloc<F>`.  (P2·P11 의 씨앗.)
- [perceus-reference-scan](../.brainstormings/20260904-200712-perceus-reference-scan.md)
  — Koka **Perceus/FP² reuse analysis** 차용(clean-room 기법, license 무관):
  ★ reuse → **allocator drop-then-reuse fast-path**(per-CPU hot-slot, freelist
  왕복 회피; atomic-free P1·deterministic WCET·Verus·measurement gate;
  Frame-Limited Reuse[ICFP'22]로 bound).  ★ **FIP 규율**(FP²[ICFP'23]: provable
  zero-alloc + constant stack) → hot/RT 무할당 경로(IPC fast path·scheduler·
  capsule 상태기계)를 **`core`-only(no `alloc`) 로 컴파일러가 zero-alloc 강제** +
  `&mut` in-place(WCET·bounded stack·P10).  **precision**(free-at-last-use) →
  민감 자료 **early-drop + zeroize** 로 잔존 window 최소화(§2.7 key-mgmt).
  **배제**: atomic RC(§0.5/P1 위반, cross-CPU 는 IPC) · RC-everywhere(Rust 정적
  ownership 우월; Lean 4 "Counting Immutable Beans" borrowing = Rust 정적 borrow
  의 동적판) · effect-handler · cycle.  §5.1 비교 **`alloc::Rc` vs FIP-macro**:
  목표가 Perceus 가치 포착이면 **FIP-macro 우선**(Rc = Perceus 가 줄이려는 RC
  baseline), std `Rc` 는 드문 동적-공유(비-hot, 단일-CPU) fallback, **`Arc` 는
  hot 경로 금지**.

### 2.2 IPC
- [ipc-layer-design](../.brainstormings/20260812-173424-ipc-layer-design.md)
  — scheme(제어평면)+LWKT msgport(데이터평면) hybrid; ★ **atomic-free
  composition**(P1); cross-CPU free = msgport `FreeReq` 위임(async, C1 보존);
  zero-copy = shared region cap; cross-layer invariant F1~F4.

### 2.3 Real-time & scheduler
- [soft-vs-hard-real-time](../.brainstormings/20260618-225136-soft-vs-hard-real-time.md)
  — hard=WCET+admission(capability budget)+formal WCET; soft=p99+reservation;
  §6.5 thermal throttle 선택적 유예(soft `_PSV`만, hardlimit 불변).
- [scheduler-design](../.brainstormings/20260813-134809-scheduler-design.md)
  — seL4 MCS·domain 메커니즘 위 policy 층(원칙 5); ★ **partition_id → seL4
  domain**(4 partition+system, temporal isolation·flush 경계·master-key 수렴);
  SMT gang=domain 이 whole core 소유; admission=capability feasibility;
  §5.1 feasibility = **PikeOS/ARINC 653 계층적**(fixed-priority, EDF 배제);
  per-CPU+IPC(atomic-free 경계).

### 2.4 Capsule fault isolation
- [capsule-fault-isolation-restart](../.brainstormings/20260812-174402-capsule-fault-isolation-restart.md)
  — 정적 C1/C2/C3 의 동적 확장; ★ **license 격리 = fault 격리 일치**(GPL/rump
  foreign driver); FLR 미지원=IOMMU domain repoint 으로 safety 항상 확보,
  restartability 만 ladder degrade; **DMA↔partition = `partition_id` 단일 key**
  (FA6); reset-group=2-phase + Y4 mini-selector; FA1~FA6.

### 2.5 Side-channel isolation
- [side-channel-isolation](../.brainstormings/20260813-124610-side-channel-isolation.md)
  — ★ **masking ⊥ side-channel**(직교, 둘 다 필요); ★ **partition_id = 모든
  격리 차원 master key**(P2, μarch 확장); seL4 Time Protection 상속 + lease
  boundary flush; 잔여 thermal/power = 정량 bound; SMT sibling=동일 partition.

### 2.6 Cross-platform & 개발 방법론
- [cross-platform-strategy](../.brainstormings/20260618-231936-cross-platform-strategy.md)
  — multi-ISA(AArch64/RISC-V/POWER/ARCv3); atomic-free 가 arch memory model
  회피; virt=cpu_virt_compat arch 확장; bhyve multi-arch/NVMM x86; non-GPL 대안.
- [dragonfly-vkernel-y4-reinterpretation](../.brainstormings/20260620-153653-dragonfly-vkernel-y4-reinterpretation.md)
  — vkernel drop(soft-MMU) = soft→HW virt 전환 = Y4 방향 검증; soft-MMU 배제.
- [y4-dev-mode-host-execution](../.brainstormings/20260620-154147-y4-dev-mode-host-execution.md)
  — 4-tier 개발 계층(host-Y4 신규); `Y4Core<K: Kernel>` generic-mock; soft 는
  dev 에 가둠; §3.7 디버거=lldb.
- [asterinas-reference-scan](../.brainstormings/20260813-125742-asterinas-reference-scan.md)
  — MPL-2.0 framekernel; ★ unsafe-confinement(OSTD) = capsule §2.1 unsafe-audit
  gate 의 whole-OS 선례(crate-level `#![forbid(unsafe_code)]`); single-AS/
  Linux-ABI/sound-not-proven 배제; clean-room Apache-2.0 차용.
- [contribute-back-guide-bhyve-nvmm](../.brainstormings/20260619-160308-contribute-back-guide-bhyve-nvmm.md)
  — clean-room 양방향성(가져오기 GPL 회피 + 돌려주기 license 자유); 최고 가치=
  Verus 발견 spec 모호성 report.

### 2.7 보안 / supply-chain 클러스터
- [attestation-measured-boot](../.brainstormings/20260813-132525-attestation-measured-boot.md)
  — secure+measured 합성; DRTM 선호+vendor-neutral RoT; ★ **verified base bind**
  (P8); device attestation(SPDM) 별도; 주 stance=Y4 trusted(SEV-SNP/TDX/CoVE=
  옵션); ★ §8.5 → `hw_mechanism_abstraction.md` 승격(P3).
- [key-management](../.brainstormings/20260813-205326-key-management.md)
  — ★ key=capability, use-without-extraction; partition_id → key namespace(P2);
  ★ measured-boot sealing(verified TCB 아래서만 unseal); (key,nonce) 유일;
  crypto agility=hw_mechanism_abstraction; Verus=management, crypto 강도=trusted.
- [boot-chain-deep-dive](../.brainstormings/20260817-153920-boot-chain-deep-dive.md)
  — 실제 Limine→seL4→Y4 chain; ★ **DRTM 이 bootloader 다양성·attestation 단일성
  양립**; §3.1 measured 소비=부트로더·펌웨어 무관 `MeasurementLogSource`(a');
  per-ISA boot RoT; §6 transactional=**per-chunk A/B + Y4 mini-selector**
  (chunk-K/chunk-P, anti-rollback floor).
- [reproducible-build-supply-chain](../.brainstormings/20260826-231914-reproducible-build-supply-chain.md)
  — ★ **reproducible build 가 source→runtime gap 을 닫는다**(P7); RIM=Y4-owned
  per-chunk·per-arch 서명 manifest; SLSA L3+ provenance; transparency log=기존
  채택; 검증 3모드(P9); trust loop 완결.

### 2.8 Comms (WWAN)
- [wwan-modem-switch-hub](../.brainstormings/20260827-000813-wwan-modem-switch-hub.md)
  — Y4 switch-hub(≠bridge) M×N; ★ **복제-방지 = 단일 네트워크-컨텍스트 3층
  불변식**(session/registration/credential) → Verus; 데이터경로 internet=route/
  NAT·IMS=**terminate-then-re-originate**(P6); 3단 standard-first failover(P5);
  2계층 credential+UICC 유일중재; baseband IOMMU confinement(미신뢰 3rd-tier);
  MC=consumer 동등; hw_mechanism_abstraction 정합(P3).

## 3. 검증 상태 / deferred

- **Verus 착수 = verus-fork 안정화 이후**(Verus 업스트림 breaking-change 대응
  지연): boot §8 log-parser·PCR-replay, boot §8 root-task invariant(보류),
  key-mgmt/WWAN 복제-방지 불변식 등.
- **⏳ hiu_abi.md v1.0 frozen 의존**: capability-bound partition quota,
  side-channel accelerator-internal, attestation/key-mgmt device attestation
  (SPDM), scheduler wave-aligned.
- **deferrable(지금/나중 무관)**: reproducible-build SLSA 레벨·RIM 포맷·
  bootstrappable/DDC 시점; WWAN MCX 깊이(⏳ 표준 정합 별도)·IMS 위치 기본값·
  vSIM 프로토콜·인증(GCF/PTCRB) 전략.

## 4. cross-reference

- `docs/hw_mechanism_abstraction.md` — P3 의 canonical 문서(§8.5 승격)
- `docs/architecture.md` — Y4 canonical 설계(현 구현 기준)
- `docs/cpu_virt_compat.md` · `docs/amdv_safety.md` · `docs/power_safety.md`
  — P3 registry 의 구체 사례
- `.brainstormings/README.md` — 브레인스토밍 시리즈 인덱스(historical detail)
- `CLAUDE.md` §6 — engineering 원칙(P1~P11 이 이를 구체화)
