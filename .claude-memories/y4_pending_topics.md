---
name: Y4 sign-off 후 후속 논의 대기 항목
description: ARCH-II' + Power management sign-off cycle 완료 후 다음 후속 주제 목록
type: project
originSessionId: 78ff80c3-5421-425a-9e23-3da166ef2bb9
---
## 완료된 sign-off cycle (2026-05-04 ~ 2026-05-07)

1. ✅ **ARCH-II' sign-off 18 단계** (P1.1~P1.6, P2.1~P2.5, P3.1~P3.7)
   + Phase 4 v1.0 frozen 마킹 (2026-05-05)
   - `docs/vmm_arch.md` v1.0 frozen
   - `docs/amdv_safety.md` v1.0 frozen
   - `docs/sel4_fork_policy.md` v1.0 frozen
   - `docs/verus_to_isabelle.md` v1.0 frozen
2. ✅ **소비전력 관리 (power management) sign-off cycle** (2026-05-05~07)
   - `docs/power_safety.md` v1.0 frozen (2026-05-07, Phase 4-power)
     — S15~S23 안전장치 (cpufreq/C-state/RAPL/ACPI/SMT/lease suspend/
       thermal/wake/PSP-mailbox) + AV21~AV40 + form-factor 4 + mobile
       sub-mode 3 (dock/portable/transportation) + universal customizability
       + KDE Plasma 패턴 force-toggle 5 개
   - `docs/power_arch.md` v1.0 frozen (2026-05-07, Phase 4-power)
     — 6 power capsule + power-orchestrator 600 LoC + cross-cluster
       reuse (audit/lifecycle/firmware-approval/npt/msr-bitmap/io-bitmap/
       cpuid-emul) + 23 workspace member
   - **logicutils ALP+CLP+Type Relations 학술적 차별점 추가** —
     "빌드 시스템 = 논리 이론" framing + abductive minimal explanation
     query
3. ✅ **CPU virtualization vendor-neutrality declaration** (2026-05-07)
   - `docs/cpu_virt_compat.md` v0 신설 — AMD-V (SVM) + Intel VT-x 양방향
     vendor-neutral, ARCH-II' capsule cluster 그대로 적용 가능
   - 본격 Intel VT-x backend code 는 v1.x patch 로 deferred
4. ✅ **`.claude-notes/` sub-directory organization** (2026-05-07)
   - `.claude-notes/trackers/` 신설 — tracker / ledger 분리
   - `.claude-notes/README.md` + `trackers/README.md` 정책 명시

## 다음 후속 주제 (대기)

현재 sign-off cycle 모두 완료.  다음 cycle 진입 후보:

1. **Phase C 진입 차단 해제 후 PR-1~PR-5 본격 작업**
   - PR-1: seL4 mainline raw-SVM C 패치 (D1a + D1a' power MSR/ACPI/SMI
     mediation)
   - PR-2: vmrun-orchestrator + 10 capsule (Y4 워크스페이스)
   - PR-3: Verus 명세 + paper artifact
   - PR-4: `y4-verus2isabelle` 도구 산출물 (Isabelle skeleton)
   - PR-5a~d: power-orchestrator + 6 power capsule (sub-PR 분할,
     power_arch §4.4)

2. **WaveTensor Phase 0 진입 결정** — Y4 가 충분히 진행됨, WaveTensor
   측 RTL 작업 (FPGA 타이밍 클로저 등) 진입 가능.  Phase D (PCIe
   passthrough + IOMMU) 의 차단 의존.

3. **Microbench measurement 시작** — Phase C 진입 직후 (`power_safety.md`
   §7.3 항목 1 + `cpu_virt_compat §6` 의 G7 timing-equal input).
   `qemu-smoke` + capsule cluster + KernelDebugBuild=ON 환경 측정.

4. **Phase C 진입 후 신규 unresolved 항목 처리**
   - `power_safety.md` §7.3 의 4 항목 (microbench / lu-rule syntax /
     vehicle bus driver 위치 / Plundervolt mitigation 보강)
   - `power_arch.md` §8.3 의 5 항목 (PmCapsuleMsg encoding / lu-rule
     syntax / vehicle bus / microbench publication / ACLP query example)
   - vmm_arch.md §8 의 7 항목 + amdv_safety §8.3 의 4 항목

5. **`.claude-notes/trackers/` tracker 파일 신설** (Phase C 진입 직후)
   - `power-prior-art-ledger.md` (paper venue / 학술 논문 / 산업 도입)
   - `power-paper-venue-tracker.md` (Phase C 종반 paper draft 시점)
   - `power-threat-ledger.md` (CVE / 학술 논문 새 위협)
