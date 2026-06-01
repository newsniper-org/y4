---
name: Y4 sign-off 후 후속 논의 대기 항목
description: ARCH-II' + Power management sign-off cycle 완료 후 다음 후속 주제 + adsmt 통합 watch
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
   - `docs/power_arch.md` v1.0 frozen (2026-05-07, Phase 4-power)
   - **logicutils ALP+CLP+Type Relations 학술적 차별점 추가**
3. ✅ **CPU virtualization vendor-neutrality declaration** (2026-05-07)
   - `docs/cpu_virt_compat.md` v0 신설
4. ✅ **`.claude-notes/` sub-directory organization** (2026-05-07)
   - `.claude-notes/trackers/` 신설
5. ✅ **PR-1 P1.0 인프라 scaffold** (2026-05-07, commit `0f6626e`)
   - `third_party/sel4-patches/` + `sel4-pin.txt` + 3 tools script + justfile
     recipe + `boot/x86_64-debug.cmake` 의 `Y4_AMDV=OFF` default

## 다음 후속 주제 (대기)

1. **PR-1 P1.1 진입** — `001-cap-types-svm.patch` 작성 (4 cap 종류 +
   `*.y4-modified.bf` + `KernelSVM` 활성 + `Y4_AMDV` master flag dispatch)
2. **PR-2~PR-5 본격 작업** — Phase C 진입 차단 해제 단계 5~7 + 8 (PR-5a~d)
3. **WaveTensor Phase 0 진입** — 별도 세션에서 진행 (2026-05-07 결정).
   FPGA 타이밍 클로저 작업 등 RTL 측 진입 가능
4. **Microbench measurement** — Phase C 진입 직후
5. **Phase C 진입 후 신규 unresolved 처리** — `power_safety.md` §7.3
   (4) + `power_arch.md` §8.3 (5) + `vmm_arch.md` §8 (7) + `amdv_safety.md`
   §8.3 (4) + `verus_to_isabelle.md` §8.5 (3) + `cpu_virt_compat.md`
   §8 (3)
6. **`.claude-notes/trackers/` 의 active tracker** — Phase C 진입 시점에
   `power-prior-art-ledger.md` + `power-paper-venue-tracker.md` +
   `power-threat-ledger.md` + `adsmt-integration-tracker.md` (2026-05-29
   신설) 갱신 활성

## adsmt v1.0 unified vision watch (2026-05-29 추가 / 2026-06-01 status 갱신)

**Status (2026-06-01)**:
- adsmt testing 브랜치 = v1.0.0-rc.2 단계, RC2.1~RC2.7 audit cycle 모두
  resolved.  HEAD = `450b986` "preparations for stable v1.0.0 release"
  (2026-05-31).  **v1.0.0 stable release 매우 임박**.
- adsmt-contrib (`~/adsmt-contrib/`, newsniper-org/adsmt-contrib) testing
  브랜치 = `1.0.0` lockstep, Debian channel model (main/testing/stable)
  도입.
- Rust toolchain 의무 = **1.96** (adsmt testing v1.0.0 부터).
- 2026-06-01 사용자 trigger: **Y4 verification workflow 재설계 본격
  진입** (P-redesign.1 sign-off — R4=(b')/R6=(b') 으로 hold pattern
  사실상 해제).  본격 작업 (proof body / cross-validation / 도구 구현
  / 재설계 sub-cycle) 모두 즉시 OK.  단 P-redesign.8 (Y4 spec v1.x
  patch 일괄 마킹) 만 adsmt stable release 후 deferred.  R1=(a') 의
  Lean4 제외 (adsmt Lean4/OxiLean blocker).
- 2026-06-01 추가 결정: **Y4 측 dependency = adsmt + adsmt-contrib
  둘 다 testing channel pin** (rolling release 패턴).  feedback memory:
  `feedback_adsmt_testing_channel_pin.md`.  Tracker §10.6.
- **2026-06-01 P-redesign.8 도 즉시 trigger** (사용자 결정): hold
  pattern **완전 해제**.  P-redesign.2~8 모든 sub-cycle 이 stable
  release 무관 즉시 진입 가능.  근거: Y4 testing channel pin (rolling)
  이라 stable release 와 spec 마킹 timing 정합 무의미.  channel 옵션
  (i) testing 계속 채택 (stable 전환 X, hybrid X).  실제 hold 영역 0.

**Hold pattern (2026-05-29 사용자 정책, 2026-06-01 부분 해제)**: adsmt
v1.0 정식 release 가 Y4 verification workflow **전체 재설계** 의 trigger.
Stable release 전까지 spec v1.x patch 본격 적용 X — 단 재설계 *논의*
는 진행.  자세한 정책 = `feedback_adsmt_v1_verification_redesign.md`.

**핵심 사실**: adsmt (`~/AD1/`, newsniper-org/adsmt, BSD-2 OR Apache-2.0
OR LGPL-2.1+, **v1.0.0-rc.2 testing**) 가 v1.0 에서 **3-way unification**
도달 예정:

```
adsmt v1.0 = adsmt-core + logicutils + OxiZ (integrated form)
```

| Component | 역할 |
|---|---|
| lu-kb (logicutils) | user-facing KB surface (ACLP) |
| OxiZ (~408k LoC, 6415 tests, **100% Z3 parity across 8 logics**, Apache-2) | SAT + classical SMT theories + math + proof |
| adsmt | abductive engine + HOL+HKT kernel + type-class + Lean4/Rocq first-class |

**Phased integration** (P1-P5, `~/AD1/.claude-memories/oxiz_relationship.md`):
- P1 v0.11 Bridge / P2 v0.13 Math / P3 v0.15 Proof bridge (landed
  2026-05-14 commit `8bbf97e`) / **P4 v0.17 Coordination (현재)** /
  P5 v0.19 v1.0 decision

**Y4 측 활용 path (v1.0 통합 후 simplified)**:
1. OxiZ → Verus 의 Z3 backend 대체 후보 (pure-Rust, 100% Z3 parity)
2. adsmt 의 abductive engine → logicutils ACLP solver 강화 (이미 unified
   toolkit 일부)
3. unified toolkit → verus_to_isabelle (T-iv) SMT-LIB hybrid backend
4. paper artifact cross-validation (OxiZ ↔ Z3)
5. Lean4 + Rocq first-class → Y4 의 verus_to_isabelle 도구가 Lean4 /
   Rocq backend 자연 지원
6. ACLP-driven build orchestration (lu-kb) → Y4 의 build infrastructure
   학술 차별점 강화

**Timeline 정합**:
- adsmt v1.0 도달 추정 = P5 (v0.19) 완료 후, 시점 미확정
- Y4 의 PR-5 (power capsule) + paper artifact = Phase C 종반
- 두 timeline 정합 가능성 ↑

**추적 위치**: `.claude-notes/trackers/adsmt-integration-tracker.md`
(2026-05-29 신설).  P5 도달 / v1.0 release / Y4 측 spec 갱신 path
ledger 보관.

**Y4 측 spec 갱신 (v1.x patch 분류, v1.0 통합 후)**:
- `power_arch.md` §6.1.8 학술적 차별점 강화 — unified toolkit framing
- `power_arch.md` §6.7 prior art ledger row 추가 (OxiZ)
- `power_arch.md` §5.2 workspace dependency 표 (unified dep 으로 전환)
- `power_arch.md` §5.6 cargo dep 라이선스 (adsmt triple + OxiZ Apache-2)
- `vmm_arch.md` §1.1 참조 자료 통합 표
- `vmm_arch.md` §3.2 Verus version semantic caution (Z3 → OxiZ trust
  boundary)
- `verus_to_isabelle.md` §1.3 (T-iv) backend 명시
- NOTICE — adsmt + OxiZ attribution
