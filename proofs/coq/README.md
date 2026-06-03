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
- **Ltac2-only** (P-redesign.4 R4.5, 2026-06-01).  모든 manual proof
  body 의 첫 줄에 `Set Default Proof Mode "Ltac2".` — adsmt-emit-rocq
  의 emit 정합.  Ltac1 은 PR reject (legacy tactic language).
- **Nested directory naming** (R4.6).  `Y4.<Domain>.<Module>` →
  `theories/<Domain>/<Module>.v` (예: `Y4.Lease.Spec` →
  `theories/Lease/Spec.v`).  `_CoqProject` 의 `-Q theories Y4` 정합.

## 3 theory 신설 계획 (P-redesign.4, 2026-06-01)

R4.1 의 작성 순서 (bottom-up) — 의존 graph 정합:

1. **`Y4.Sel4.Wrapper`** (`theories/Sel4/Wrapper.v`) — seL4 microkernel
   측 wrapper invariant.  D1a 의 vmrun wrapper + GIF host-managed (AV6)
   의 high-level spec 모음.  Verus 의 first-order 한계 넘는 부분 (예:
   inductive cap derivation chain).
2. **`Y4.IPC.Refinement`** (`theories/IPC/Refinement.v`) — Phase B step
   3 의 `proofs/verus/src/ipc/refinement.rs` 의 high-level refinement
   theorem (scheme + msgport 의 LWKT-style consistency).  Verus 측은
   per-step refinement, Rocq 측은 cumulative invariant.
3. **`Y4.Lease.Spec`** (`theories/Lease/Spec.v`) — Phase C 의 lease
   security theorem (lease capability 의 confidentiality + integrity
   chain, XChaCha20 key 의 host-only 보관).  WaveTensor 측 HIU ABI 와
   짝지어지는 high-level proof.

`Y4.Placeholder` (`theories/Placeholder.v`) 는 위 첫 theory (Sel4.Wrapper)
land 시 삭제 — R4.4 정합.

## adsmt-emit-rocq 통합 (R4.2, 2026-06-01)

3 theory 의 cert 산출물 emission 은 별도 sibling repo
**`~/y4-verus2rocq/`** 가 처리 (verus_to_isabelle 의 §3.1 (A) 패턴 정합).
설계 spec: `docs/verus_to_rocq.md`.

도구 측 의존:

```toml
# ~/y4-verus2rocq/Cargo.toml
[dependencies]
adsmt-emit-rocq = { git = "https://github.com/newsniper-org/adsmt-contrib", branch = "testing" }
adsmt-cert      = { git = "https://github.com/newsniper-org/adsmt",         branch = "testing" }
```

`av-proof-body-tracker.md` §5 의 각 cluster sub-PR 안에 `.v` emission
활성 (cluster 별 rolling, R4.7).

## 현 상태

- 환경: Rocq 9.1.1 설치 확인 (2026-05-04, `/usr/bin/rocq`).
- theory: `Y4.Placeholder` (`theories/Placeholder.v`) — `1 + 1 = 2` 의
  trivial 정리.  R4.1 의 첫 theory (`Y4.Sel4.Wrapper`) land 시 삭제.
- 도구: `~/y4-verus2rocq/` (P-redesign.4 sign-off 후 신설, R4.2=b sibling
  repo).
