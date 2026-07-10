<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Verification Harness

본 디렉터리는 Y4 의 **formal-first** 원칙을 강제하는 빌드/CI 게이트다.
모든 신규 privileged 코드 경로는 같은 PR 안에 Verus 또는 Rocq 명세를
동반해야 머지된다 (CONTRIBUTING.md §5. "Verification expectations").

## 도구 분담

| 도구 | 역할 | 위치 |
|------|------|------|
| **Verus** | Rust-native 명세 + 증명. lease capability invariant, allocator 안전성, IPC 타입 안전성 등 Rust 코드와 직접 정렬되는 모든 증명 | [`./verus/`](./verus/) |
| **Rocq** (formerly Coq) | Verus 가 표현 못 하는 **Y4 자체** high-level invariant — cross-component 보안 정리, inductive cap derivation chain 등. 예: `Y4.Sel4.Wrapper` = Y4 가 seL4 syscall 경계에 씌우는 **wrapper** 의 불변식이며, **seL4 자체 proof (Isabelle/HOL) 와는 독립** (seL4 를 Rocq 로 모델링하는 것이 아님) | [`./coq/`](./coq/) |
| **Isabelle/HOL** | seL4 대면 전용 — Y4 의 Verus 증명을 seL4 팀이 소비하도록 `.thy` 로 emit (seL4 mainline verification 트랙이 Isabelle/HOL 이므로). L4.verified inbound contract. R7 emit pipeline (`verus_to_isabelle.md` §1.7) 산출물 | [`./isabelle/`](./isabelle/) |

원칙은 CLAUDE.md §6.6 ("Formal-first verification") 참조.

## 현황 (2026-06-29 기준 — Phase B 완료 + R7 emit pipeline 반영)

- **Verus:** **`verus-fork/` git submodule** (branch `backend-pluggable`)
  의 빌드 산출물 사용 — `verus-fork/source/target-verus/release/verus`
  (`vargo build --release`). system `verus-bin` (AUR) / `/usr/bin/verus`
  **미사용**: submodule 이 Y4 backend patch (`-V oxiz` / `-V adsmt` /
  emit-isabelle/rocq 등, PR-Verus-Backend) 를 포함해야 하기 때문. vstd 는
  submodule 빌드에서 link. 설치·빌드 절차는 `proofs/verus/justfile` 및
  `.claude-notes/trackers/pr-verus-backend-tracker.md` §3 참조.
- **Rocq:** opam 설치 (`~/.opam/default/bin/rocq` 신규 CLI + `coqc`
  legacy 호환), version **9.1.1**.
- **명세:** **`just verus` → 54 verified, 0 errors** (alloc + ipc +
  capsules + **amdv (AV1 `intercept_floor` + top-level `vmrun_safe`)** +
  error + 모듈 top-level + refinement). 개수는 proof 추가마다 변하므로
  **`just verus` 출력이 authoritative** — 본 숫자는 스냅샷.
  **`just coq` → Rocq 9.1.1 trivial theorem placeholder 통과**
  (`theories/Placeholder.v`; R4.1 의 첫 실제 theory land 시 삭제 예정).
- **Refinement:** alloc/ipc 각 `refinement.rs` — **9개** proof fn
  (alloc 4 + ipc 5) 가 `assume()` 대신 executable spec function + 귀납
  증명으로 discharge. [`./verus/src/alloc/refinement.rs`](./verus/src/alloc/refinement.rs)
  + [`./verus/src/ipc/refinement.rs`](./verus/src/ipc/refinement.rs).

## 워크플로우

```sh
# 모든 게이트 (fmt + lint + test + verus + coq) 를 hash-stamp 적용해 실행
just ci

# 증명만 따로
just proofs                  # = just verus && just coq

# Verus 만
just verus                   # = just proofs/verus

# Coq 만
just coq                     # = just proofs/coq
```

## CI gating 정책

`just ci` 가 green 인 PR 만 머지 대상. 신규 privileged path 가 추가되었는데
대응 명세가 없으면, 명세 파일이 빠진 게 git diff 에서 자명하게 보이도록
다음 두 정책이 함께 강제된다:

1. `kernel/`, `hiu/`, `ipc/`, `alloc/`, `capsules/` 의 새 함수가 `unsafe`
   를 도입하면 `proofs/verus/` 의 대응 spec 파일이 같은 PR 에 없으면 lint
   gate 가 reject.  (자동화 lint plugin 은 **P-redesign.7 로 예정 — 현재
   미구현**, 그때까지는 CONTRIBUTING.md §5 reviewer 수동 확인으로 강제.)
2. CONTRIBUTING.md §5 의 reviewer checklist 가 명시적으로 "proof 산출물
   머지 여부" 를 확인.
