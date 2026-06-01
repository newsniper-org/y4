---
name: adsmt v1.0 release 가 Y4 verification workflow 전체 재설계 trigger
description: adsmt v1.0 release 전까지 Y4 측 verification workflow 의 본격 변경 X (hold pattern), v1.0 release 시점에 전체 재설계 진입
type: feedback
originSessionId: 78ff80c3-5421-425a-9e23-3da166ef2bb9
---
adsmt v1.0 정식 release 가 나오면 Y4 의 verification workflow **전체**
를 대대적으로 재설계할 예정.  Stable release 전까지는 hold pattern —
spec 의 v1.x patch 본격 적용 X.

**2026-06-01 부분 해제**: adsmt testing 브랜치가 v1.0.0-rc.2 + "preparations
for stable v1.0.0 release" 단계 도달.  사용자가 **재설계 논의 시작
trigger** 발화.  → 재설계 논의는 진행, 단 spec 의 v1.x patch 본격 적용
은 stable release 도달 후 일괄 진입.

**Why:** adsmt v1.0 = adsmt + logicutils + OxiZ 의 3-way unified
toolkit (`.claude-notes/trackers/adsmt-integration-tracker.md`).
v1.0 release 시점에 Y4 측 verification stack 의 backend (Verus 의 Z3
의존 / Rocq tactic / ITP frontend / paper artifact 측 SMT cross-
validation 등) 가 모두 통합 toolkit 으로 전환되므로, 그 전 시점에
부분 변경 시 redesign 시점에 모순 발생 + 작업 중복.  사용자가 2026-05-29
명시: "기다리고 있도록".

**How to apply:**

- **hold (adsmt v1.0 release 전):**
  - Y4 측 AV1~AV40 proof body 본격 작성 deferred
  - unsafe + proof 짝 lint 자동화 (Phase B 후반 예정이었음) deferred
  - Rocq theory 본격 (Y4.Lease.Spec / Y4.IPC.Refinement / Y4.Sel4.Wrapper) deferred
  - paper artifact 의 SMT cross-validation 실험 deferred
  - `y4-verus2isabelle` 도구 본격 구현 (PR-4) deferred — adsmt 의
    Lean4/Rocq/Isabelle multi-ITP emit 활용 가능성
  - `verus_to_isabelle.md` §1.3 (T-iv) SMT-LIB hybrid backend 본격
    결정 deferred
  - Verus 의 Z3 backend 측 결정 deferred — OxiZ 로 전환할지 v1.0
    시점 평가

- **OK (verification workflow 직접 변경 X 인 작업):**
  - PR-1 의 patch series (cap types / syscall / vmrun wrapper / etc.)
    — seL4 microkernel 측 C 코드, verification workflow 와 분리
  - capsule cluster 의 Rust 코드 작성 — `proofs/verus/` 의 statement
    변경 0 한정
  - phase_plan 갱신 / cross-doc cross-ref 정합 / repo organization
  - `.claude-notes/trackers/` 갱신
  - microbench measurement 인프라 준비 (실제 cross-validation 실행은
    deferred)
  - Vendor-neutrality 측 spec (cpu_virt_compat) 갱신
  - WaveTensor 측 작업 (별도 세션)

- **release 후 (adsmt v1.0 도달 시):**
  - Y4 측 verification workflow 전체 재설계 진입 — 본격 sign-off cycle
  - power_arch §6.1.8 / §6.7 / §5.2 / §5.6 + vmm_arch §1.1 / §3.2 +
    verus_to_isabelle §1.3 / §3.3 + NOTICE 의 v1.x patch 본격 진입
    (adsmt-integration-tracker §5 ledger)
  - paper artifact §6.5 의 (ii) Verus 증명 산출물 본격 작성 (proof
    body 채움) — unified toolkit 위에서

- **추적:** `.claude-notes/trackers/adsmt-integration-tracker.md` 의
  §7 Watch milestones — adsmt P5 (v0.19) / v1.0 release 도달 시
  본 memory 와 tracker 둘 다 갱신, hold pattern 해제.
