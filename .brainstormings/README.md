<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# `.brainstormings/` — design 탐색 기록

본 디렉터리는 sign-off cycle 전의 **brainstorming / 옵션 탐색** 기록.
결정 (`.claude-notes/`) 이나 지속 추적 (`.claude-notes/trackers/`) 과
구분되는, "아직 결정 X — 방향 탐색" 단계의 산출물.

## 성격 구분

| 종류 | 위치 | 단계 |
|---|---|---|
| **Brainstorming** (옵션 탐색) | `.brainstormings/` (본 디렉터리) | 결정 전 — 방향/tradeoff 나열 |
| **Decision archive** (결정 record) | `.claude-notes/` | 채택 후 record, 갱신 종료 |
| **Tracker / ledger** (지속 갱신) | `.claude-notes/trackers/` | 새 정보 도착 시 row 추가 |

brainstorming 이 sign-off cycle 로 승격되면 결정 부분은 `.claude-notes/`
또는 docs/ 의 design memo 로 이전, 본 기록은 historical reference 로 보존.

## 파일 명칭 convention

```
<YYYYMMDD>-<HHMMSS>-<topic-kebab>.md
```

- timestamp = **UTC+9:00 (KST)** 기준 (생성 시점)
- frontmatter 에 `created: <ISO8601 +09:00>` + `status` + `scope` + `refs`

예: `20260618-212218-alloc-frontend-improvements.md`

## git tracking

**Git-tracked** — design 흔적 보존 (`.claude-notes/` 정책 정합).
contribute-back paper / 코드 리뷰 / 산업 도입 시 audit reference.

## 기록 목록

| 파일 | topic | 생성 (KST) | status |
|---|---|---|---|
| [20260618-212218-alloc-frontend-improvements.md](20260618-212218-alloc-frontend-improvements.md) | DragonFly SLAB(최종 목표) 의 atomic-free SLUB-style 현대화 — §0.5 atomic 배제(불가침) / SLUB 데이터 구조만(unqueued+metadata-in-page) / cross-CPU free=IPC 위임 / Verus-only verified / freelist 미결(§2.9) / capability layer | 2026-06-18 21:22 (rev 21:57) | brainstorming |
| [20260618-221518-pluggable-alloc-frontend-contract.md](20260618-221518-pluggable-alloc-frontend-contract.md) | Pluggable allocator front-end + 규약 — 2-tier trait(`AllocFrontend`+`Freelist` 분리) + Verus contract(C1~C4 / FL1~FL3) / generic-first 선택(dyn 보류) / C2 atomic-free=lint+Verus / reference+custom(form-factor) / Linux 대비 contract-by-construction 우위 | 2026-06-18 22:15 (rev 22:24) | brainstorming |
| [20260618-222938-y4-variant-slob.md](20260618-222938-y4-variant-slob.md) | Y4-variant SLOB (embedded front-end 구현체) — Linux SLOB 의 Y4 변형: global lock→atomic-free / in-band→metadata 긴장 / first-fit→bounded-fit / coalescing→deferred / size-class-less 가 Freelist trait 일반성 시험 / 메모리 극소 ↔ Y4 원칙의 "메모리 비용 순 계층화" (C3 form-factor 차등 첫 사례).  §4.5 SLUB_TINY 참고 — SLOB-variant vs SLUB-tiny-config 재고(Y4 pluggable 이 Linux 택1 딜레마 무효화) | 2026-06-18 22:29 (rev SLUB_TINY) | brainstorming |
| [20260618-223701-capability-bound-allocation.md](20260618-223701-capability-bound-allocation.md) | Capability-bound allocation (front-end 직교 cross-cutting layer, Y4 고유) — partition-keyed zone(P0..3, I1 메모리 따름정리, cross-tenant UAF 차단) / partition memory quota(lease §4 빈 칸) / lease-lifecycle bulk reclaim(I4/I6) / sealed zero-on-free / C5 capability contract / `CapabilityAlloc<F>` wrapper / §2.8 partition CPU 모델 결정(aggregation 은 lease lifecycle 에서만→atomic 0).  기존 allocator 누구도 안 한 capability-aware (paper §6.1 최강 차별점) | 2026-06-18 22:37 | brainstorming (§2.8 결정 / 일부 ⏳ hiu_abi) |
| [20260618-225136-soft-vs-hard-real-time.md](20260618-225136-soft-vs-hard-real-time.md) | soft/hard real-time 달성 (allocator 시리즈 → Y4 전체 real-time 확장) — hard=WCET 분석(seL4 verified+bounded components)+admission(capability budget)+formal WCET(AV3 확장)+wave-aligned deadline / soft=p99+background 분리+graceful degradation+MCS reservation / mixed-criticality(seL4 MCS+lease criticality attr) / §6.5 thermal throttle 선택적 유예(soft `_PSV`만 Hard lease bounded 유예, hardlimit AV27 불변).  Y4 = formally-verified hard RT hypervisor 위치 | 2026-06-18 22:51 (rev §6.5 thermal) | brainstorming (§6.5 부분 결정 / ⏳ hiu_abi/Phase C) |
| [20260618-231936-cross-platform-strategy.md](20260618-231936-cross-platform-strategy.md) | Cross-platform multi-ISA (AArch64 / RISC-V RV64GC / POWER64 LE+BE / ARCv3) — ISA-independent core 최대화(§0.5 atomic-free 가 arch memory model 회피) / virtualization=cpu_virt_compat 의 arch 축 확장(abstract AV + `<topic>_<arch>.rs`) / endian=LE 우선+POWER BE type-safe newtype / verified base 차등(seL4) / 우선순위 AArch64→RISC-V→POWER→ARCv3.  §3.1 질문2: bhyve multi-arch VMM(amd64+arm64+riscv) → AArch64/RISC-V virt = bhyve port, NVMM=x86 only(NetBSD portability≠NVMM virt).  §3.2 질문3: non-GPL 대안 = A(Hafnium/libsel4vm/OpenBSD vmm) / B(spec clean-room, 모든 arch 항상 열림 — POWER PAPR/ARC PRM) / C(rust-vmm device).  reference 부재≠막힘 | 2026-06-18 23:19 (rev 06-19 bhyve/nvmm 확보+non-GPL 경로) | brainstorming |
| [20260619-160308-contribute-back-guide-bhyve-nvmm.md](20260619-160308-contribute-back-guide-bhyve-nvmm.md) | Contribute-back 가이드 (Y4 virt → bhyve/NVMM 역방향) — clean-room(경로 B)이 contribute-back 을 쉽게(Y4 독립 저작물→dual-license `Apache-2 OR BSD-2`) / 최고 가치=Verus 발견 spec 모호성·soundness(코드 무관 bug report) / POWER·ARC virt(bhyve 부재) 신규 기여 / Rust→C=algorithm·spec transfer / clean-room 양방향성(가져오기 GPL 회피 + 돌려주기 license 자유) | 2026-06-19 16:03 | guide (대비) |
| [20260620-153653-dragonfly-vkernel-y4-reinterpretation.md](20260620-153653-dragonfly-vkernel-y4-reinterpretation.md) | DragonFly vkernel (6.0 drop) 의 Y4 재해석 — drop 이유=MAP_VPAGETABLE(소프트웨어 MMU) VM코어 강결합→제거, "re-add via HVM".  교훈: 소프트웨어 virt→하드웨어 virt 패러다임 전환 = Y4 방향 검증 / soft-MMU 배제(cross-platform §4.B 명시 배제로 갱신) / vkernel 개발·디버깅 가치=Y4 dev mode 흡수 / Y4="HVM 시대 vkernel 후예"(seL4+capability+verified 독립) | 2026-06-20 15:36 | brainstorming |
| [20260620-154147-y4-dev-mode-host-execution.md](20260620-154147-y4-dev-mode-host-execution.md) | Y4 dev mode — host-execution (vkernel 개발 가치 흡수) — 4-tier 개발 계층(unit / **host-Y4 신규** / qemu-smoke / 실HW) / `Kernel` trait mock + soft page backend(test 전용) / capsule+orchestrator 를 host process 로 gdb 디버그 / `Y4Core<K: Kernel>` generic-mock(pluggable 패턴) / ★ soft 는 dev 에 가둠(vkernel 교훈 — production 은 seL4+HW virt).  Phase C 개발 생산성 | 2026-06-20 15:41 | brainstorming |
