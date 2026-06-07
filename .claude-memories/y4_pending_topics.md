---
name: Y4 sign-off 후 후속 논의 대기 항목
description: 모든 sign-off cycle (ARCH-II', power, vendor-neutrality, P-redesign.1/.2/.6) 진행 상태 + 다음 후속 주제 + adsmt v1.0 watch
type: project
originSessionId: 78ff80c3-5421-425a-9e23-3da166ef2bb9
---
## 완료된 sign-off cycle

### 2026-05-04 ~ 2026-05-07

1. ✅ **ARCH-II' sign-off 18 단계** (P1.1~P1.6, P2.1~P2.5, P3.1~P3.7)
   + Phase 4 v1.0 frozen 마킹 (2026-05-05)
   - `docs/vmm_arch.md` v1.0 frozen
   - `docs/amdv_safety.md` v1.0 frozen
   - `docs/sel4_fork_policy.md` v1.0 frozen
   - `docs/verus_to_isabelle.md` v1.0 frozen
2. ✅ **소비전력 관리 (power management) sign-off cycle** (2026-05-05~07)
   - `docs/power_safety.md` v1.0 frozen (2026-05-07, Phase 4-power)
   - `docs/power_arch.md` v1.0 frozen (2026-05-07, Phase 4-power)
   - logicutils ALP+CLP+Type Relations 학술적 차별점 추가
3. ✅ **CPU virtualization vendor-neutrality declaration** (2026-05-07)
   - `docs/cpu_virt_compat.md` v0 신설
4. ✅ **`.claude-notes/` sub-directory organization** (2026-05-07)
   - `.claude-notes/trackers/` 신설
5. ✅ **PR-1 P1.0 인프라 scaffold** (2026-05-07, commit `0f6626e`)
   - `third_party/sel4-patches/` + `sel4-pin.txt` + 3 tools script +
     justfile recipe + `boot/x86_64-debug.cmake` 의 `Y4_AMDV=OFF` default

### 2026-06-01 (verification workflow 재설계 cycle)

6. ✅ **P-redesign.1** (commit `54eb6f9`) — 재설계 scope + 11 결정 ledger
   - R1=(a') Lean4 제외 / R2=b dual backend / R3=b unified pin / R4=b'
     즉시 / R5=d 3 Rocq theories / R6=b' 즉시 / R7=a adsmt-emit-isabelle
     wrapper / R8=a lint sub-PR / R9=a lu-smt 통합 / R10=a P-redesign.#
     형식 / R11=a v1.x patch only
   - Sub-cycle 분할 P-redesign.1~8 + Lean4 watch + testing channel pin
     정책 (§10.6)
7. ✅ **Hold pattern 완전 해제** (commit `72f76de`) — P-redesign.8 도
   stable release 기다리지 X (사용자 결정).  실제 hold 영역 0.  channel
   옵션 (i) testing 계속 채택
8. ✅ **P-redesign.2 + P-redesign.6** (commit `91cd38a`)
   - **P-redesign.2** Verus dual backend + `unified-toolkit-pin.toml`:
     `verus_to_isabelle.md` §3.6 rewrite (verus-pin + isabelle-pin →
     unified, sub-table 3 = adsmt + adsmt-contrib + oxiz, logicutils
     absorbed), `Y4/unified-toolkit-pin.toml` + `.lock` 신설, Verus
     `--backend=z3|oxiz|dual` flag, cargo `[patch.crates-io]` git dep,
     `cpu_virt_compat.md` §8 Lean4 watch 추가
   - **P-redesign.6** SMT cross-validation tracker 신설
     (`.claude-notes/trackers/smt-cross-validation-tracker.md`, R6.1~R6.10
     sign-off)
9. ✅ **adsmt P5 option 5 confirmed** — 이전 P5 옵션 (a)/(b) closed.
   **option 5 = bidirectional embed** (adsmt + adsmt-contrib 독립 repo +
   logicutils absorbed into adsmt).  근거: adsmt-contrib `a838525` 본문
   + adsmt main `a937058` 의 `state/logicutils-frozen/` 보존
10. ✅ **P-redesign.3** (2026-06-01) — AV1~AV40 proof body 4 cluster
    작성 plan + 12 결정 (R3.1~R3.12) sign-off
    - 신설: `.claude-notes/trackers/av-proof-body-tracker.md`
    - 4 cluster boundary: amdv lower (12) → amdv upper (9) → power upper
      (11) → power lower (5), 합 ~9400 LoC
    - File layout: amdv_safety.md §5 표 자연 grouping (AV9+AV10 →
      `upper/bitmap_immut.rs`, AV11+AV12+AV13 → `lower/audit.rs`, AV14+AV15
      → `upper/lifetime.rs`)
    - 의존 graph DAG: AV6→AV1, AV2-D→AV2, AV23→AV22, AV30→AV4 — `lu-par`
      topological order
    - PR-N scope: PR-2 = amdv 2 cluster (PR-2a + PR-2b), PR-5d = power 2
      cluster (PR-5d.1 + PR-5d.2)
    - **R3.11 = Verus 본체 patch (`--backend=z3|oxiz|adsmt`) 별도 sub-PR**
      (upstream contribute-back path 분리, ~800 LoC)
    - **R3.12 = Verus fork 측 adsmt third backend (opt-in 3-way)** —
      abductive verdict reporter (`--report-abductive-on-unknown`), AV5/
      AV12/AV15/AV23/AV24/AV30 6 invariant 가 활용 후보 (higher-order
      quantifier).  Self-application evidence (paper artifact §6.5 (vii))
    - Cross-validate timing: cluster 완료 시 batch (smt-cross-validation-
      tracker §2 에 row 1 회 / cluster)
    - Lean4 retrofit hook: adsmt v1.1.x 도달 시 R3.10 activation (cpu_virt_
      compat.md §8 (4) Lean4 watch row trigger 갱신)
11. ✅ **PR-Verus-Backend 준비** (2026-06-01) — R3.11 + R3.12 의 별도 세션
    entry point.  `.claude-notes/trackers/pr-verus-backend-tracker.md`
    신설 (9 phase P-vb.1~P-vb.9, 분량 ~1100 LoC).  Verus 본체 fork =
    `~/verus-fork/` (사용자가 수동 clone — 대기).  Cross-ref: av-proof-
    body-tracker §1 R3.11/R3.12 + smt-cross-validation-tracker §9 +
    unified-toolkit-pin.toml [verus]
12. ⚠️ **P-redesign.4** (2026-06-01) — **superseded by R7 sign-off
    (2026-06-03, item 16)**.  sibling repo (`~/y4-verus2rocq/`) wrapper
    가정이 wrong — Verus fork (R7.3) 가 `-V emit-rocq` 자동 emit + adsmt-
    emit-rocq CLI 가 cert→.v 직접 변환.  R4.1 의 manual 3 theory plan +
    R4.5 Ltac2-only + R4.6 nested naming 은 유지 (Verus 표현 못하는 high-
    level), `~/y4-verus2rocq/` sibling 도구 + `docs/verus_to_rocq.md` 는
    폐기
13. ⚠️ **P-redesign.5** (2026-06-01) — **superseded by R7 sign-off
    (2026-06-03, item 16)**.  sibling repo (`~/y4-verus2isabelle/`)
    wrapper 가정이 wrong — Verus fork (R7.3) 가 `-V emit-isabelle` 자동
    emit + adsmt-emit-isabelle CLI 가 cert→.thy 직접 변환.  R5.3 Lean4
    제외 + R5.4 theory file naming `Y4_<Domain>_<Module>.thy` 는 유지
    (R7.7 정합), `~/y4-verus2isabelle/` sibling 도구 + verus_to_isabelle.md
    §3 의 sibling 측 본문은 superseded (§3.6 unified-toolkit-pin 만 유지)
15. ✅ **Verus submodule integration** (2026-06-03) — system verus 호출
    완전 폐기, `<Y4>/verus-fork/` git submodule (newsniper-org/verus
    fork, branch `backend-pluggable`) 의 binary 호출로 전환
    - `git submodule add -b backend-pluggable https://github.com/newsniper-org/verus verus-fork`
    - `.gitmodules` entry 추가
    - `proofs/verus/justfile`: `verus_bin := ".../verus-fork/source/target-
      verus/release/verus"`, install 안내 → submodule init + vargo build
    - `Y4/justfile` tools-check: `command -v verus` → submodule path
      `[ -x verus-fork/.../verus ]` check
    - `CLAUDE.md` §5 repository layout: `verus-fork/` (submodule) 추가
    - `NOTICE`: Verus entry 갱신 — newsniper-org fork + branch backend-
      pluggable + R3.11/R3.12 의 작업 위치 명시
    - `pr-verus-backend-tracker.md` §3: clone path `~/verus-fork/` → `<Y4>/
      verus-fork/`, VSCode setup + 별 세션 진입 path 갱신, Y4 cross-ref
      doc 의 `../` parent path 명시
14. ✅ **PR-Verus-Backend flag mechanism 갱신** (2026-06-03) — 새
    `--backend=` flag 정의 X.  Verus 의 기존 `-V <key>` extended-multi
    flag mechanism (`-V cvc5` 패턴 mirror) 안에 `-V oxiz` + `-V adsmt`
    + `-V report-abductive-on-unknown` 추가.  `SmtSolver` enum 확장
    (`Z3` / `Cvc5` + 신규 `OxiZ` + `Adsmt`).  `SmtVerdict::Abductive`
    variant 신설 (adsmt only).  cross-validation 의 `dual` / `triple`
    은 Verus 본체 flag X — Y4 측 `just verus-cross-validate` script 의
    multi-invocation 로직.  Patch 분량 ~800 → **~500 LoC** (mechanism
    신설 부담 ↓, ~300 LoC 절약).  upstream contribute-back path 자연성
    ↑.  영향 file: docs/verus_to_isabelle.md §3.6, av-proof-body-tracker
    §1 R3.11+R3.12, smt-cross-validation-tracker §6+§9, pr-verus-backend-
    tracker §1+§4+§5
16. ✅ **R7 sign-off — Y4 Verus ↔ seL4 Isabelle/HOL 통합 plan** (2026-
    06-03, R7.1~R7.12)
    - `verus-fork/examples/consumer/justfile` 의 의도 발견: Verus fork
      (PR-Verus-Backend, R3.11+R3.12+R7.3 scope) 가 `-V adsmt` + `-V
      emit-isabelle` / `-V emit-rocq` + AOT prelude bank + JIT trace
      load 모두 직접 wire → Y4 측 wrapper 도구 의미 0
    - **P-redesign.4 + P-redesign.5 superseded** (item 12 + 13).
      sibling repo `~/y4-verus2rocq/` + `~/y4-verus2isabelle/` 폐기.
      `docs/verus_to_rocq.md` 삭제, `docs/verus_to_isabelle.md` §3
      supersede note (§3.6 unified-toolkit-pin 만 유지)
    - **PR-Verus-Backend scope 확장 (R7.3)**: ~500 → ~850 LoC, 12 phase
      (P-vb.10 emit + P-vb.11 AOT + P-vb.12 JIT 신규).  **모든 phase
      land 완료 ✅ (2026-06-03)** — verus-fork backend-pluggable HEAD 가
      patch 모두 ship (rc.28 sound + rc.29 Tseitin complete + consumer/
      justfile template).  Y4 측 PR-Verus-Backend 별 세션 진입 작업 0,
      R7.11 milestone 즉시 진입 가능
    - **`proofs/isabelle/` 신설 (R7.7)**: 2-layer imports chain (Layer 1
      cluster sub-grouping `Y4_AmdvSafety.thy` + `Y4_PowerSafety.thy` +
      Layer 2 per-AV flat list + top-level `Y4.thy` 가 양 layer 모두
      imports).  README + Y4.thy + Layer 1 cluster .thy placeholder 신설
    - **`proofs/coq/theories/Generated/` 신설 (R7.8)** — `_CoqProject`
      에 glob + R4.1 manual 3 theory 와 별도 위치 (nested directory)
    - **L4.verified inbound = 사용자 manual (R7.10)** — Y4 측 통합 X,
      seL4 팀이 자체 l4v 환경에서 `proofs/isabelle/` import.  trust 가능
      조건 = PR-Verus-Backend land + adsmt rc.28+/rc.29+ 도달 (모두 충족)
    - **adsmt baseline 갱신 (R7.6)**: rc.6-1 → **rc.29** (`03f33a9`, sound
      + complete 확보) + adsmt-contrib **rc.28** (`33349dc`).  branch =
      testing (rolling), stable v1.0.0 release 시 stable-v1 channel 전환
      예정
    - **`proofs/verus/justfile` consumer pattern adapt (R7.2)** — 8
      recipes (verify / verify-adsmt / verify-cvc5 / verify-oxiz /
      verify-adsmt-fast / aot-bake / cross-check / emit-isabelle /
      emit-rocq)
    - **신설 tracker**: `.claude-notes/trackers/y4-sel4-integration-
      tracker.md` (R7 ledger + per-cluster emission record + l4v
      milestone watch)
    - Plan ledger: `~/.claude/plans/jazzy-gliding-puppy.md`

## 진행 가능한 다음 후속 주제

### Verification workflow 재설계 cycle (즉시 진입 가능)

| Sub-cycle | 내용 | 의존 | 비고 |
|---|---|---|---|
| ~~PR-Verus-Backend~~ | ~~Verus 본체 patch~~ | — | ✅ **모두 land 완료 (2026-06-03)** — verus-fork backend-pluggable HEAD ship.  Y4 측 즉시 verify-adsmt + emit-isabelle + cross-check 가능 |
| P-redesign.7 | unsafe + proof 짝 lint 자동화 spec (adsmt type-class layer 활용) | P-redesign.3 ✅ | 즉시 (P.3 sign-off 완료) |
| P-redesign.8 | Y4 spec v1.x patch 일괄 마킹 (power_arch + vmm_arch + verus_to_isabelle + cpu_virt_compat + amdv_safety + NOTICE) | P.2~7 완료 | (P.7 후, hold X) |

### 기타 후속 주제

1. **PR-1 P1.1 진입** — `001-cap-types-svm.patch` 작성 (4 cap 종류 +
   `*.y4-modified.bf` + `KernelSVM` 활성 + `Y4_AMDV` master flag dispatch)
2. **PR-2~PR-5 본격 작업** — Phase C 진입 차단 해제 단계 5~7 + 8 (PR-5a~d)
3. **WaveTensor Phase 0 진입** — 별도 세션에서 진행 (2026-05-07 결정)
4. **Microbench measurement (P-redesign.6 의 baseline)** — Verus
   `--backend=oxiz` flag 가 작동 가능해진 시점 즉시
5. **Phase C 진입 후 신규 unresolved 처리** — power_safety.md §7.3 (4)
   + power_arch.md §8.3 (5) + vmm_arch.md §8 (7) + amdv_safety.md §8.3
   (4) + verus_to_isabelle.md §8.5 (3) + cpu_virt_compat.md §8 (4 — Lean4
   watch 포함)
6. **`.claude-notes/trackers/` 의 active tracker 갱신** —
   `power-prior-art-ledger.md` + `power-paper-venue-tracker.md` +
   `power-threat-ledger.md` (Phase C 진입 후 활성) +
   `adsmt-integration-tracker.md` (active) +
   `smt-cross-validation-tracker.md` (active, 2026-06-01)
7. **`capsules/src/config_space.rs`** — 사용자 별도 clippy lint, 본 cycle
   외 (unstaged 유지)

## adsmt v1.0 watch (현 status 2026-06-01)

**adsmt status (2026-06-03 갱신)**:
- testing + main branch HEAD = `2c46803` "fix .gitmodules" (2026-06-03)
  + tag **`v1.0.0-rc.6-1`** = 동일 commit (첫 tag 는 `v1.0.0-rc.1`,
  RC.2~RC.5 + RC.6 는 tag 없이 commit chain 으로 진행)
- 이전 P-redesign.2 sign-off baseline (`450b986`) 이후 변경:
  RC.2/3/4 typed-enum hot patch + leo4 v1.0.0-rc.1 reflection + `4cba9be`
  oxiz fork update (branch `0.2.2` → `0.2.3-feat/enable-writer`, commit
  `8d2ec3f` → `1297944`) + `2c46803` fix branch naming
- **v1.0.0 stable release 임박** (RC.6-1 = 마지막 RC?)
- `.lock` baseline adsmt commit 갱신: `450b986` → `2c46803`

**adsmt-contrib status**:
- testing HEAD = `a838525` "archlinux: 3 source-only PKGBUILDs" (2026-06-01)
- main HEAD = `1b73e6f` (same commit message, 2026-06-01)
- pkgver discipline = adsmt main 와 lockstep

**P5 option 5 (bidirectional embed) confirmed**:
- adsmt + adsmt-contrib = 두 독립 repo, 독자 release cadence
- logicutils = adsmt 안 absorbed (`state/logicutils-frozen/docs/man/`)
- OxiZ = adsmt 측 fork (Honey-Be/oxiz, feat/enable-writer) 의 transitive
  dep

**OxiLean / Lean4 측 progress (R1=a' 제외 사유 watch)**:
- leo4 fork 가 2026-05-31 evening 3 batches 완료 — OxiLean path
  "effectively complete"
- mainline Lean 4 (mslean4) path 는 post-RC, adsmt v1.2.x
- Y4 측 deferred 유지 — adsmt v1.1.x 도달 시 reconsider

**Y4 측 dependency 정책**:
- adsmt + adsmt-contrib 둘 다 **testing channel pin** (rolling),
  `feedback_adsmt_testing_channel_pin.md`
- Rust toolchain 의무 1.96
- `Y4/unified-toolkit-pin.toml` + `.lock` baseline 신설 (2026-06-01)

**Hold pattern 변화**:
- 2026-05-29 hold (verification workflow 변경 X)
- 2026-06-01 오전 부분 해제 (논의 시작)
- 2026-06-01 P-redesign.1 (R4=b'/R6=b' 본격 작업)
- **2026-06-01 P-redesign.8 도 즉시 — Hold pattern 완전 해제, 실제
  hold 영역 0**

**Tracker**: `.claude-notes/trackers/adsmt-integration-tracker.md`
(2026-05-29 신설, active, §10 P-redesign ledger + §10.6 channel pin
+ Lean4 watch).

## 현 git status

- branch `main` = `origin/main` 보다 6 commit 앞 (push 대기)
- `capsules/src/config_space.rs` clippy lint 만 unstaged (사용자 별도)
