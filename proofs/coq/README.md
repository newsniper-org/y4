<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Rocq theories

Y4 는 Verus 가 표현하지 못하는 high-level invariant 에 한해 **Rocq**
(이전 명칭 Coq) 를 사용한다 (CLAUDE.md §6.6).

> **이름 변경 (2025).** "Coq" → "Rocq". 본 디렉터리 이름은 호환성을 위해
> `coq/` 로 유지하되, 빌드 도구는 새 CLI (`rocq compile`, `rocq makefile`)
> 를 1차로 사용한다. 9.x 이전 버전 (`coqc`, `coq_makefile`) 은 폴백으로만
> 받아들인다.

## 빠른 사용

```sh
just            # = just verify
just verify     # _CoqProject 의 모든 theory 컴파일/검증
just clean      # 생성된 Makefile.rocq 와 .vo / .vos 등 정리
```

또는 repo 루트에서 `just coq`.

## 도구 우선순위

1. `rocq` (Rocq 9.x) — `rocq makefile -f _CoqProject -o Makefile.rocq` →
   `make -f Makefile.rocq`.
2. `coqc` (Rocq 9.x 호환 wrapper 또는 legacy Coq) — `coq_makefile` →
   `make -f Makefile.coq`. 경고를 출력하고 진행.
3. 둘 다 없으면 `https://rocq-prover.org/install` 안내와 함께 fail.

## 명세 작성 규칙

- **모듈 prefix `Y4`.** `_CoqProject` 의 `-Q theories Y4` 가 모든 파일에
  `Y4.` 접두를 부여 — 외부 라이브러리(stdlib, mathcomp 등) 와 충돌 방지.
- **사용 정당성을 첫 줄 docstring 에 명시.** "Verus 로 표현 못 하는
  이유" 가 없으면 PR reject — Verus 가 1순위.
- **Verus spec 과 1:1 정합.** 같은 invariant 가 양쪽에 있으면 양쪽 모두
  최신화. drift 를 방지하기 위해 가능하면 한쪽만 — 위 docstring 규칙.

## 현 상태

- 환경: Rocq 9.1.1 설치 확인 (2026-05-04, `/usr/bin/rocq`).
- theory: `Y4.Placeholder` (`theories/Placeholder.v`) — `1 + 1 = 2` 의
  trivial 정리. 첫 실제 정리 (Phase B step 3 의 IPC refinement 또는
  Phase C 의 lease security theorem) 머지 시 본 placeholder 삭제.
