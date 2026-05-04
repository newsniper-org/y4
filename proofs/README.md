<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Verification Harness

본 디렉터리는 Y4 의 **formal-first** 원칙을 강제하는 빌드/CI 게이트다.
모든 신규 privileged 코드 경로는 같은 PR 안에 Verus 또는 Coq 명세를
동반해야 머지된다 (CONTRIBUTING.md §5).

## 도구 분담

| 도구 | 역할 | 위치 |
|------|------|------|
| **Verus** | Rust-native 명세 + 증명. lease capability invariant, allocator 안전성, IPC 타입 안전성 등 Rust 코드와 직접 정렬되는 모든 증명 | [`./verus/`](./verus/) |
| **Coq** | Verus 가 표현 못 하는 high-level invariant. cross-component 보안 정리, 비-Rust 컴포넌트(seL4 wrapper 등)의 모델링 | [`./coq/`](./coq/) |

원칙은 CLAUDE.md §6.6 ("Formal-first verification") 참조.

## 설치 상태 (2026-05-04 기준)

- **Coq:** 설치 진행 중 (사용자 측). `coqc` 가 PATH 에 들어오면 `just coq`
  가 즉시 동작.
- **Verus:** 아직 미설치. 설치 가이드는 [`./verus/README.md`](./verus/README.md).
  설치 전까지 `just verus` 는 친절한 에러를 출력하고 exit 1.

`just tools-check` (top-level) 가 두 도구의 존재 여부를 확인하고,
Verus/Coq 가 없을 때는 fatal 이 아닌 **warn** 으로 보고한다 — 도구 설치
이전이라도 cargo build / cargo test 는 진행 가능.

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
   gate 가 reject (Phase B 후반에 lint plugin 으로 자동화).
2. CONTRIBUTING.md §5 의 reviewer checklist 가 명시적으로 "proof 산출물
   머지 여부" 를 확인.
