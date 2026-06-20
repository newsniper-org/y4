<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: DragonFlyBSD vkernel (6.0 drop) 의 Y4 관점 분석 / 재해석
created: 2026-06-20T15:36:53+09:00   # KST (UTC+9)
status: brainstorming (결정 X — 분석/재해석)
scope: DragonFly 6.0 에서 drop 된 vkernel 의 메커니즘·drop 이유를 Y4 의
       하드웨어-virt + capability + verified 관점에서 재해석
refs:
  - DragonFly 6.0 release notes (vkernel drop — MAP_VPAGETABLE 제거)
  - ~/y4-upstream-refs/dragonfly/sys/platform/vkernel64/ (코드 잔존)
  - ~/y4-upstream-refs/dragonfly/sys/dev/virtual/nvmm/ (HVM 재추가 정황)
  - .brainstormings/20260618-231936-cross-platform-strategy.md (virt = HW ext)
  - alloc/src/page_backend.rs (MockPageBackend — Y4 의 dev/test 측면)
  - docs/amdv_safety.md (AMD-V NPT — 하드웨어 nested page)
---

# DragonFlyBSD vkernel 의 Y4 관점 재해석

## 0. 발제 + 사실 확인

발제 (사용자): DragonFly 6.0 에서 drop 된 vkernel 을 Y4 관점에서
분석/재해석.

**사실 확인 (release notes)**: vkernel drop **확정**.
> "Due to major changes to the VM system we had to remove the
> MAP_VPAGETABLE mmap() feature, and this also means that vkernels will
> not be supported in this release.  Support may be re-added at a later
> time **via HVM** (but not in this release)."

- drop 이유 = VM 시스템 대개편 (extent-based representation) → **`MAP_
  VPAGETABLE` mmap() feature 제거** → vkernel 의 소프트웨어 MMU 불가.
- "re-added later **via HVM**" — 하드웨어 가상화로 재구현 의도.
- 현 HEAD (2026-05) 에 `sys/platform/vkernel64/` 코드 잔존 + `sys/dev/
  virtual/nvmm/` (NVMM port) 존재 → **HVM(NVMM) 기반 재추가 정황** (단
  6.2+ release note 확인은 별도 — 본 분석 핵심은 6.0 drop 의 교훈).

## 1. vkernel 이란

DragonFly 의 독특한 기능 — **커널을 user-space process 로 실행**
(Matt Dillon, Linux UML 류):
- DragonFly 커널 바이너리가 host DragonFly 위 일반 process 로 실행.
- **MMU 가상화 = `MAP_VPAGETABLE`** — host 의 mmap 위에 *소프트웨어
  nested page table* (guest 의 page table 을 host mmap 으로 emulate).
  하드웨어 EPT/NPT 없이.
- CPU = host thread (guest vcpu = host thread), privileged op = signal/
  syscall intercept.
- 가상 디스크/네트워크 = host file/socket (`sys/dev/virtual/vkernel/`).
- **용도**: (a) 커널 개발/디버깅 (gdb 로 커널 디버그, crash 해도 host
  안 죽음), (b) 가벼운 격리 (full VM 보다 가벼움), (c) recursive
  (vkernel 안 vkernel).

## 2. drop 의 정확한 이유 + 시사점 (★ Y4 에 직격)

**이유**: vkernel 의 핵심 = `MAP_VPAGETABLE` (소프트웨어 MMU 가상화).
이게 **host VM 시스템 코어와 강결합** → VM 재설계 (extent-based) 시
유지 불가 → 제거.

**DragonFly 의 결론** ("re-add via HVM"): **소프트웨어 MMU 가상화는
유지보수 부담 (VM 코어 강결합) → 하드웨어 가상화 (HVM) 로 가야 한다.**

**Y4 에의 시사 3가지**:
1. **방향 검증** — Y4 는 처음부터 **하드웨어 virt** (AMD-V SVM + NPT,
   amdv_safety).  vkernel 이 *나중에 HVM 으로* 하려던 것을 Y4 는 처음부터.
   vkernel 의 죽음 = "소프트웨어 virt 패러다임의 한계" 증명 → Y4 의
   하드웨어 virt 방향 확정.
2. **소프트웨어 MMU 회피 교훈** — `MAP_VPAGETABLE` 가 VM 코어 강결합으로
   죽었듯, *소프트웨어 nested page* 는 부담.  → cross-platform §의
   "soft-virt fallback" 아이디어는 **재고/배제** (vkernel 교훈: soft MMU
   는 결국 짐).  하드웨어 NPT/EPT/stage-2/G-stage 만.
3. **HVM 수렴** — DragonFly 가 vkernel 을 NVMM(HVM) 위로 재구현하려는
   방향 = Y4 의 하드웨어-virt hypervisor 와 **수렴**.  게다가 Y4 도 NVMM
   reference (vmm_arch §1.1, contribute-back 가이드) → 같은 HVM 토대.

## 3. Y4 관점 재해석

### 3.1 Y4 = "HVM 시대의 vkernel 후예" (방향 검증)
- vkernel 목적 (가벼운 커널 격리 + 개발) 의 *하드웨어-virt 재구현* 이
  본질적으로 Y4 같은 형태 — verified microkernel (seL4) + HW virt
  hypervisor.
- "re-add via HVM" 의 HVM 재구현 = Y4 의 영역.  단 Y4 는 DragonFly
  vkernel 재구현이 아니라 *독립* (seL4 base + capability + Rust +
  verified).

### 3.2 소프트웨어 MMU 회피 → cross-platform soft-fallback 재고
- cross-platform brainstorming (§4 즈음) 에서 "하드웨어 virt 없는 환경
  의 soft-MMU fallback" 을 잠깐 고려했는데 — **vkernel 교훈상 배제**.
- soft MMU (MAP_VPAGETABLE 류) = VM 코어 강결합 + 유지보수 부담 +
  verification 복잡 (소프트웨어 page table 의 invariant) → Y4 는 하드웨어
  nested page 전제 (NPT/EPT/stage-2/G-stage/radix).
- HW virt 부재 환경 = Y4 의 *dev/test mode* (§3.3) 로만, production 격리
  X.

### 3.3 vkernel 의 개발/디버깅 가치 → Y4 dev mode (★ 흡수할 것)
- vkernel 의 *최대 강점* = 커널을 host process 로 → gdb 디버그, 빠른
  iteration, crash 격리.  이건 HVM 으로 가도 *잃으면 아까운* 가치.
- **Y4 가 이미 부분 보유**: `MockPageBackend` (page_backend.rs) + qemu-
  smoke → Y4 로직을 host 에서 test.  단 vkernel 은 *실제 커널 바이너리*
  를 process 로, Y4 mock 은 *로직 unit test* — 차이.
- **재해석 — "vkernel-style Y4 dev mode"**: Y4 의 capsule + orchestrator
  를 host user-space process 로 실행 (seL4 없이, mock seL4 + 소프트웨어
  page backend) → 커널 코드 없이 Y4 로직 전체를 gdb 디버그 + 빠른
  iteration.  현 mock 의 확장 = "host 에서 도는 Y4" (production 아닌
  dev).  vkernel 이 준 교훈 = 이 dev mode 의 가치 (HW virt 만으론 디버그
  불편).
- 단 §3.2 와 구분: dev mode 의 soft backend 는 *test 전용* (production
  격리 X) — vkernel 이 production 격리까지 soft 로 했다가 죽은 것과 대조.

### 3.4 recursive / nested 재해석
- vkernel = recursive (vkernel 안 vkernel, 소프트웨어라 무한 중첩 쉬움).
- Y4 = 하드웨어 nested virt (R-α/R-γ Phase D, amdv_safety §nested) —
  하드웨어 nested 는 깊이 제한 (HW 2-level) + AV8 `no_nested_svm` 으로
  현재는 *차단* (S9).
- 재해석: vkernel 의 recursive 유연성 (소프트웨어) vs Y4 의 nested 안전성
  (하드웨어 + AV8 차단).  Y4 는 recursive 를 *유연성* 보다 *격리/검증*
  우선 → 기본 차단, Phase D 에서 controlled nested (R-α `/dev/kvm` proxy).
- dev mode (§3.3) 에선 soft recursive 가능 (test 용, vkernel 처럼).

### 3.5 lightweight isolation spectrum
- vkernel = full-VM 과 container 사이 (커널 격리, 소프트웨어).
- Y4 의 spectrum: capsule (Tock-style, capability typing) < lease
  (partition, capability) < full guest VM (AMD-V).
- 재해석: vkernel 이 채운 "중간 격리" 를 Y4 는 capsule + lease 로 (더
  강한 격리 + verified + capability).  vkernel 의 소프트웨어 중간 격리 →
  Y4 의 capability 중간 격리 (하드웨어 + 검증).

## 4. 종합 — vkernel 의 교훈을 Y4 가 어떻게 위치시키나

| vkernel 측면 | Y4 의 대응 |
|---|---|
| 소프트웨어 MMU (MAP_VPAGETABLE) | ❌ 배제 (drop 교훈) — HW NPT/EPT/stage-2 |
| HVM 재구현 의도 | ✅ Y4 가 그 방향 (HW virt hypervisor) — 수렴 |
| 커널을 process 로 (개발/디버깅) | ✅ 흡수 — "vkernel-style Y4 dev mode" (mock 확장) |
| recursive (소프트웨어) | △ dev mode 만 soft recursive, production 은 HW nested (AV8 차단) |
| 가벼운 격리 (중간) | ✅ capsule + lease (capability, verified) 로 재구현 |

**한 줄**: vkernel 의 죽음은 "소프트웨어 virt → 하드웨어 virt" 패러다임
전환의 증거이고, Y4 는 그 전환의 *다음 세대* (HW virt + capability +
verified).  단 vkernel 이 남긴 *개발/디버깅 가치* 는 Y4 가 dev mode 로
흡수해야 (잃으면 아까움).

## 5. 결정 / 미결

### 방향 (분석 결론)
- vkernel 의 소프트웨어 MMU 는 Y4 가 배제 (drop 교훈 — cross-platform
  soft-fallback 도 재고/배제)
- vkernel 의 개발/디버깅 가치 = "vkernel-style Y4 dev mode" 로 흡수
  (MockPageBackend + 소프트웨어 page backend 확장, test 전용)
- vkernel 의 HVM 재구현 방향 = Y4 의 하드웨어 virt 와 수렴 (방향 검증)

### 미결
- **Y4 dev mode 의 범위** — 현 MockPageBackend + qemu-smoke 로 충분한가,
  아니면 "host 에서 도는 Y4 전체" (capsule+orchestrator user-space) 까지
  만들 가치?  개발 생산성 vs 구현 비용
- **dev mode 의 mock seL4** — seL4 API 를 host 에서 mock (vkernel 의
  host syscall intercept 류) 하는 비용.  capability 의미 보존 여부
- **vkernel64 코드 잔존 / NVMM 재추가** — 현 dragonfly HEAD 의 vkernel64
  가 HVM(NVMM) 기반 재구현인지 확인 (6.2+ release note) → 재구현이면
  Y4 와 직접 비교 사례 (DragonFly 의 HVM-vkernel vs Y4)
- **soft recursive dev mode** — dev mode 에서 vkernel 식 recursive
  (Y4 안 Y4) 가 디버깅에 유용한지

### 후속 / 연결
- cross-platform §의 soft-virt fallback 을 **명시적 배제** 로 갱신
  (vkernel drop 교훈) — 추후 cross-platform 문서에 cross-ref
- contribute-back 가이드 (§NVMM) 와 연결 — DragonFly 가 vkernel 을 NVMM
  위로 재구현하면, Y4 의 NVMM reference 와 같은 토대 → 상호 참조 가능
- "Y4 dev mode" 가 별도 발제/design 가치 (MockPageBackend 확장 +
  host-Y4) — Phase C 개발 생산성
