<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Verus → Rocq 도구 (`y4-verus2rocq`) 설계 — P-redesign.4

> **요약:** Y4 의 Verus 측 invariant 를 Rocq 측 `.v` 파일로 emit 하는
> sibling 도구.  verus_to_isabelle 의 §3.1 (A) 패턴과 1:1 정합 — sibling
> repo `~/y4-verus2rocq/`.  adsmt-emit-rocq (Ltac2-only, v0.21 K-full)
> 의 wrapper 형태.
>
> **상태:** P-redesign.4 sign-off 2026-06-01 (R4.1~R4.7 + R5.X 와 짝).
> 도구 scaffold 신설 후 cluster 별 rolling 으로 emission 활성.

## 0. 범위 한정 (scope clamp)

`verus_to_isabelle.md` §0 정합 + Rocq 특화:
- Y4 측 invariant 의 Rocq emit 전용 — general-purpose Verus-to-Rocq
  도구 X
- Ltac2-only (Ltac1 excluded, Rocq ≥ 8.10)
- (T-i) sorry / (T-ii) axiom / (T-iv) SMT-LIB hybrid 의 3-mode 정합 —
  Lean4 backend 제외 (R1=a')

## 1. 범위 — Isabelle 측과의 짝

Rocq 측은 Isabelle 측과 다음 점에서 다름:

| Aspect | Isabelle (y4-verus2isabelle) | Rocq (y4-verus2rocq) |
|---|---|---|
| 도구 | `y4-verus2isabelle` (`docs/verus_to_isabelle.md`) | `y4-verus2rocq` (본 doc) |
| Emit crate | `adsmt-emit-isabelle` (Isar by-tactic) | `adsmt-emit-rocq` (Ltac2 `Theorem ... Proof. ... Qed.`) |
| Classical | `Main` 측 내장 (no import) | `From Stdlib Require Import Logic.` |
| Theory file naming | `Y4_AmdvSafety_Upper_Npt.thy` (flat underscore) | `theories/AmdvSafety/Upper/Npt.v` (nested directory) |
| Module system | flat namespace + `imports` | nested module + `-Q theories Y4` |
| Mode | sorry / axiom / by smt | sorry / Axiom / `by (smt-replay)` 없음 |

R4.6 의 nested directory naming = `_CoqProject` 의 `-Q theories Y4` 와
정합 (`theories/Lease/Spec.v` → `Y4.Lease.Spec`).

## 2. 3 theory 신설 (R4.1, bottom-up)

R4.1 의 작성 순서:

1. **`Y4.Sel4.Wrapper`** — seL4 microkernel 측 wrapper invariant
   - 위치: `proofs/coq/theories/Sel4/Wrapper.v`
   - 내용: D1a vmrun wrapper, GIF host-managed (AV6) high-level spec,
     inductive cap derivation chain
   - Verus 한계: first-order quantifier 만 → inductive proof 가
     자연스럽지 X
2. **`Y4.IPC.Refinement`** — IPC refinement theorem
   - 위치: `proofs/coq/theories/IPC/Refinement.v`
   - 내용: Phase B step 3 의 `proofs/verus/src/ipc/refinement.rs` 의
     cumulative invariant (scheme + msgport LWKT consistency)
   - Verus 한계: per-step refinement 만 (Verus 의 `state_machines_
     macros` 미지원, P3.4 §0 정합) → cumulative invariant 가 Rocq 측
3. **`Y4.Lease.Spec`** — lease security theorem
   - 위치: `proofs/coq/theories/Lease/Spec.v`
   - 내용: lease capability 의 confidentiality + integrity chain,
     XChaCha20 key host-only 보관, WaveTensor HIU ABI 짝
   - Verus 한계: HOL/higher-order quantifier (∃ key, ∀ packet,
     decryptable(packet, key) ⟹ owner(packet) = host)

## 3. 도구 형상

### 3.1 채택 — sibling repo `~/y4-verus2rocq/` (R4.2)

| 옵션 | 의미 | 채택 |
|---|---|:---:|
| **(b) 별도 sibling repo `~/y4-verus2rocq/`** | verus_to_isabelle 의 §3.1 (A) 패턴 정합.  contribute-back 가능 (Rocq 팀 측) | **◎** |
| (a) Y4/proofs/rocq-emit/ 새 wrapper crate | Y4 workspace 내부, 시작 비용 작음, sibling 정책 위배 | ✗ |
| (c) wrapper 없음 (adsmt-emit-rocq 직접 호출) | Y4 domain 매핑 부재 | ✗ |

### 3.2 (b) 채택 시 내부 구조

**Single Cargo crate `y4-verus2rocq`** (binary + lib 분리), `y4-verus2isabelle`
와 1:1 mirror:

```
/home/ybi/y4-verus2rocq/
├── Cargo.toml              # license = "Apache-2.0"
├── LICENSE                 # Apache-2.0 full text
├── NOTICE                  # logicutils BSD-2 + adsmt-emit-rocq tri-license attribution
├── README.md
├── src/
│   ├── lib.rs              # parser / mapper / emitter API
│   ├── main.rs             # CLI binary (v2r)
│   ├── parser/             # syn-based Verus AST parsing (y4-verus2isabelle 와 공유 가능)
│   ├── mapper/             # §2 매핑 (도메인 type 매핑)
│   ├── emitter/            # Rocq Ltac2 text emit + adsmt-emit-rocq wrapper
│   │   ├── mod.rs          # cert → Ltac2 text glue
│   │   ├── adsmt_wrap.rs   # adsmt-emit-rocq cargo git dep call + Y4 domain mapping
│   │   └── pretty.rs       # pretty-print (Ltac2 indent)
│   └── modes/              # (T-i) / (T-ii) hybrid 결정 로직 (Rocq 측은 by smt 없음)
├── tests/
│   ├── unit/               # 도구 자체 unit test
│   └── fixtures/           # Y4 의 proofs/verus/src/{amdv,power}/ snapshot copy
└── tools/
    ├── sync-fixtures.sh    # Y4 의 proofs/ → fixtures/ 자동 sync
    ├── v2r.rules           # logicutils lu-rule 형식 mode override
    └── v2r-build.sh        # logicutils freshcheck/stamp 통합 build entry
```

**Version pin**: Y4 측 `Y4/unified-toolkit-pin.toml` 의 `[adsmt-contrib]`
+ `[rocq]` (NEW) sub-tables 가 source-of-truth.  도구 측은 build 시 read.

**Cargo dep**:

```toml
[dependencies]
adsmt-emit-rocq = { git = "https://github.com/newsniper-org/adsmt-contrib", branch = "testing" }
adsmt-cert      = { git = "https://github.com/newsniper-org/adsmt",         branch = "testing" }
lu-common       = { git = "https://github.com/newsniper-org/adsmt",         branch = "testing" }  # logicutils absorbed
```

**logicutils 11 crate 의 위치** (P5 option 5, 2026-06-01): 모두 adsmt
안 absorbed.  `/home/ybi/logicutils/` standalone repo v0.2.0 = legacy.
Y4 측 sibling 도구는 Rust lib dep = `lu-common` (adsmt git dep), CLI
binary dep = adsmt PKGBUILD 또는 `cargo install --git ...adsmt` 의 system
install.

**분량**: 단일 crate ~1500 LoC + Y4 도메인 매핑 ~200 LoC + adsmt-emit-rocq
wrapper ~100 LoC + Ltac2 indent ~100 LoC + logicutils 통합 ~150 LoC
≈ **~2050 LoC Rust** 추정 (Isabelle 측보다 ~300 LoC 작음 — (T-iv) SMT-LIB
hybrid 부재).

### 3.3 mode 결정 (Rocq 측, Lean4 backend 제외)

```
for each lemma L in Verus input:
    if L has #[verus_to_rocq::axiom]:
        emit Axiom (T-ii)
    elif L has #[verus_to_rocq::admit]:
        emit Admitted. (T-i, Rocq 의 sorry 등가)
    else:
        # adsmt-emit-rocq 호출 — cert step 으로부터 real proof term emit
        # K-full 의 Trans/EqMp/Deduct/Abs/Beta/Inst/InstType 등 채움
        emit Theorem ... Proof. <Ltac2 body>. Qed.
```

Isabelle 측의 (T-iv) SMT-LIB hybrid 는 Rocq 측 X — Rocq 는 SMT-LIB
replay 가 자연스러운 도구 (CompCert / coqhammer 등) 가 있지만 Y4 측은
adsmt-emit-rocq 의 real proof term 으로 충분.

### 3.4 Round-trip test fixtures (R5.2 정합)

`tests/fixtures/` 안에 Y4 의 `proofs/verus/src/{amdv,power}/` 의
snapshot copy.  `tools/sync-fixtures.sh` 가 drift 검출.  CI 측:
`tools/sync-fixtures.sh --dry-run` fail → 도구 측 PR + Y4 측 v2r 재 emit
짝 PR.

### 3.5 CI strategy — 2-tier (verus_to_isabelle §3.5 정합)

#### Tier 1 — `y4-verus2rocq` repo 자체

| Step | 검증 |
|---|---|
| `cargo fmt --check` | code style |
| `cargo clippy -- -D warnings` | lint |
| `cargo test` | unit test |
| `cargo run --bin v2r -- tests/fixtures/ --output /tmp/y4-rocq` | fixture round-trip |
| `rocq makefile -f /tmp/y4-rocq/_CoqProject -o /tmp/Makefile.rocq && make -f /tmp/Makefile.rocq` | generated `.v` 의 syntactic OK |

#### Tier 2 — Y4 측 sub-tier (`Y4/justfile` 의 `verus2rocq` recipe)

| Step | 검증 |
|---|---|
| `cargo install --path ~/y4-verus2rocq` (latest tag) | 도구 install |
| `v2r Y4/proofs/verus/src/{amdv,power}/ --output Y4/proofs/coq/theories/Generated/` | Y4 의 invariant 산출물 generate |
| `just coq` | 생성 산출물의 빌드 |

### 3.6 도구 자체 라이선스

Apache-2.0 (Y4 정합 + adsmt-emit-rocq 의 tri-license (BSD-2 / Apache-2 /
LGPL-2.1+) 와 양립).

## 4. 구현 분량 추정

`verus_to_isabelle.md` §4 와 정합:

| 항목 | LoC |
|---|---|
| parser (syn-based Verus AST) | 400 |
| mapper (Y4 도메인 type 매핑) | 200 |
| emitter (Ltac2 text + adsmt-emit-rocq wrapper + pretty) | 500 |
| modes ((T-i) / (T-ii)) | 100 |
| CLI binary | 100 |
| tests (unit + fixture round-trip) | 300 |
| tools (sync-fixtures / v2r.rules / v2r-build.sh) | 200 |
| logicutils 통합 | 150 |
| adsmt-emit-rocq wrapper | 100 |
| **합** | **~2050 LoC** |

## 5. 차단 의존

- `adsmt-emit-rocq` v0.21 K-full ✅ (이미 land, 2026-05-X)
- `Y4/unified-toolkit-pin.toml` `[adsmt-contrib]` sub-table ✅
  (P-redesign.2 산출물, 2026-06-01)
- `~/verus-fork/` Verus 본체 patch (PR-Verus-Backend) — 본 도구는 Verus
  output 의 SMT-LIB capture 가 필요한 부분 X (adsmt-emit-rocq 가 cert
  로 직접 변환), 따라서 PR-Verus-Backend 의존 0

## 6. seL4 / Rocq 측 채택 시나리오 (verus_to_isabelle §6 정합)

| Step | 작업 |
|---|---|
| 1 | Y4 측 cluster sub-PR 안에 `.v` emission 활성 |
| 2 | seL4 팀 (또는 Rocq community) 에 generated `.v` 공유 |
| 3 | 채택 의향 시 도구 자체 contribute (sibling repo Apache-2.0) |

## 7. 동결 정책

본 doc 은 v0 (frozen 후 v1.x patch 정합).  unresolved 는 §8.

## 8. 미해결 / 추가 결정 필요

1. **Lean4 backend retrofit** — R3.10 의 hook (adsmt v1.1.x 도달 시)
   활성 시점에 `y4-verus2lean/` 신설 검토.  현 시점 deferred.
2. **`Y4/unified-toolkit-pin.toml` 의 `[rocq]` sub-table 추가** — 본 doc
   §3.2 의 build dep.  사용자 확인 후 unified-toolkit-pin.toml 갱신
   (R5.7=b 의 mirror — y4-verus2rocq sibling 측은 sub-table 추가 X).
   Rocq min/max/recommended version (현재 9.1.1 confirmed).
3. **Verus parser 공유** — y4-verus2isabelle 와 y4-verus2rocq 가 같은
   syn-based parser 사용.  공유 crate `y4-verus-parser` 신설 검토 시점.

## 9. fork-policy 와의 정합성

`sel4_fork_policy.md` 와 무관 — 본 도구는 seL4 fork 와 별도 sibling.
도구 자체의 license = Apache-2.0 (Y4 정합).
