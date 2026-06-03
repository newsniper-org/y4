<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Verus → Isabelle/HOL 번역기 — 설계 사양

> **상태:** **v1.0 frozen** (2026-05-05, Phase 4 일괄 마킹).
> §0 scope clamp (Y4-scope 한정, general-purpose translator 아님) +
> §1 hybrid scope (T-i sorry / T-ii axiom / T-iv SMT-LIB hybrid +
> logicutils augmentation) + §2 매핑 표 (6 sub-section, Y4 도메인 type
> + (T-iv) SMT-LIB + theory file 분리) + §3 도구 형상 (옵션 A 별도
> sibling repo + 8 sub-section) 모두 sign-off.  짝 frozen doc =
> amdv_safety.md / vmm_arch.md / sel4_fork_policy.md.  v1.0 frozen 후
> 도구 자체 구현 PR 진입 + Y4 측 PR-4 의 `.thy` 산출물 짝.

Y4 의 contribute-back 경로 (`docs/amdv_safety.md` §6, `docs/sel4_fork_policy.md`
§4) 의 일부. seL4 mainline 의 verification 트랙은 Isabelle/HOL 이고
Y4 의 정리는 Verus (Z3 SMT). 두 쪽 사이의 다리.

목적: **seL4 개발진이 Y4 의 정리를 자기 도구로 받아 사용**할 수 있도록
번역.  본 도구가 없으면 seL4 팀은 Y4 의 Verus 증명을 처음부터 Isabelle
로 재작성해야 하므로 contribute-back 진입 장벽 큼.

## 0. 범위 한정 (scope clamp)

**General-purpose Verus → Isabelle/HOL translator 는 본 도구의 목표가
아님.**  본 도구는 **Y4 의 Verus 코드를 Isabelle/HOL 의 proper subset
언어로 번역하는 것만을 목표** — Y4 가 사용하는 Verus 기능 부분집합만
지원.

귀결:
- Y4 가 쓰지 않는 Verus 기능 (예: `state_machines_macros` DSL, 일부
  trait 제약, closure / fn-ptr 형 ghost 등) 은 지원 X — 만나면 명확한
  에러 + Y4 측 회피 권고
- Isabelle 측 출력은 HOL 의 *proper subset* — locale / class /
  axiomatization 같은 광범위 feature 는 Y4 invariant 를 표현하기에
  필요한 범위만
- 외부 사용자가 일반 Verus 프로젝트에 본 도구를 적용하려 시도하면
  Y4-specific 가정 (특정 trait 묶음 / 특정 ghost 패턴 / lease cap 같은
  Y4 도메인 type) 으로 인해 실패 가능 — 본 도구는 그 경우 책임 X
- 본 scope 한정 덕에 §4 의 ~1500 LoC 분량이 가능 — general-purpose
  translator 라면 분량 폭증

---

## 1. 범위 — (T-i) + (T-ii) + (T-iv) hybrid + logicutils augmentation

### 1.1 (T-i) Statement-only 번역

Verus 의 `spec fn` / `proof fn` / `exec fn ... ensures` 의 **시그니처
+ requires + ensures + decreases** 만 추출하여 Isabelle/HOL 의
`definition` / `lemma ... sorry` 로 변환.  증명 본문은 `sorry` —
seL4 팀이 Isabelle Isar 로 자체 증명.

추출 정밀화 (A):

| Verus 항목 | (T-i) 추출 |
|---|---|
| `requires` / `ensures` | ◎ 추출 |
| `decreases` | ◎ 추출 (termination) |
| `recommends` | ✗ 추출 0 (hint only, semantic 영향 없음) |
| `assert` | ✗ proof step (T-i 외부) |
| `assume` | ✗ T-i 모드 거부 — Y4 측 도구 panic.  T-ii 모드에서만 별도 axiom 으로 |

장점: 분량 적음, 의미 보존 신뢰.  seL4 팀이 자기 도구로 검증.
단점: seL4 팀의 작업량이 그대로 남음 (도구는 boilerplate 만 줄임).

### 1.2 (T-ii) `axiom` 옵트인

Verus 측 lemma 에 attribute 를 붙이면 Isabelle `axiomatization` 으로
변환.  attribute 는 Y4-scope 고정 (D):

```rust
#[verus_to_isabelle::axiom]
proof fn intercept_floor_holds(vcpu: VCPU)
    ensures s2_holds(vcpu)
{
    // ... Verus / Z3 증명 ...
}
```

→ Isabelle:

```isabelle
axiomatization where
  intercept_floor_holds:
    "S2_holds vcpu"
```

장점: seL4 팀이 Verus 증명을 axiom 으로 신뢰하면 즉시 사용 가능.
단점: trust boundary 가 Verus + Z3 + 본 도구 자체로 이동 (B 의 trust
ledger §1.6).

### 1.3 (T-iv) SMT-LIB hybrid (logicutils-augmented, v1.0 기본 강제, Lean4 mode 없음 — R5.3)

Verus 가 Z3 에 발화하는 SMT-LIB 2 쿼리를 capture → Isabelle 의 `smt`
method 입력으로 emit.  자동 가능한 proof obligation 은 **Isabelle 안
에서 즉시 replay** (sledgehammer 와 동등 효과), structural definition
은 (T-i) sorry, attribute 부착 시 (T-ii) axiom — 3-mode hybrid.

```isabelle
(* 자동 가능한 obligation — (T-iv) emit *)
lemma intercept_floor_implies_no_nested:
  assumes "intercept_floor_holds vcpu"
  shows "no_nested_svm vcpu"
  by (smt (verit) assms s2_implies_s9 ...)   (* SMT-LIB replay *)

(* structural definition — (T-i) sorry *)
lemma vcpu_invariant: "P vcpu" sorry

(* attribute 부착 — (T-ii) axiom *)
axiomatization where
  gif_host_managed: "..."
```

**Mode 결정 알고리즘 (도구 내부):**

```
for each lemma L in Verus input:
    if L has #[verus_to_isabelle::axiom]:
        emit axiomatization (T-ii)
    elif L has SMT-LIB obligation captured AND obligation 자동 가능:
        emit `by (smt (verit) ...)` (T-iv)
    else:
        emit sorry (T-i)
```

#### 1.3.1 logicutils augmentation

(T-iv) 의 mode 분기 + per-statement re-emission 비용 관리는 Y4 의
build orchestration 도구 **logicutils** (`/home/ybi/logicutils/`) 와
통합:

| logicutils 기능 | 본 도구에서의 활용 |
|---|---|
| `freshcheck` (hash-driven freshness) | 매 Verus statement 의 hash 기록.  변경된 statement 만 재추출 + 재 emit.  `.thy` 산출물에 input statement hash 박음 |
| `stamp` (signature recording) | 매 emit 결과에 (T-i / T-ii / T-iv) mode tag + Verus version + Z3 version stamp.  reproducibility 검증 |
| `lu-rule` (per-form-factor flag) | `tools/v2i.rules` 에 mode override (예: 특정 invariant 강제 sorry, 특정 invariant 강제 axiom) — 형상별 build 처럼 paper-track / contribute-back-track 분리 |
| `lu-par` (DAG-aware parallel) | invariant 사이 의존 그래프 (예: AV6 의 AV1 의존, AV2-D 의 AV2 의존) DAG 정렬 + 병렬 emission |

mode 결정의 deterministic 보장:
- 같은 input + 같은 logicutils stamp = bit-identical `.thy` 산출물
- Verus / Z3 / 도구 자체의 version pin → reproducibility (PR-4 의 paper artifact 정합)

### 1.4 도구 모드 옵션

| flag | 의미 | 사용 |
|---|---|---|
| (default, no flag) | (T-iv) SMT-LIB hybrid + (T-ii) attribute axiom + (T-i) sorry fallback.  v1.0 기본 강제 | 일반 use case |
| `--no-smt` | (T-iv) 비활성.  (T-i) sorry + (attribute 시) (T-ii) axiom 만.  Z3 / Isabelle smt method 환경 부재 시 fallback | air-gapped 환경 |
| `--all-sorry` | 모든 lemma sorry, attribute / SMT-LIB 무시.  seL4 팀 100% 자체 증명 모드 | seL4 팀의 audit-only |
| `--all-axiom` | 모든 lemma axiom (논의용 / 제안 단계, production X) | discussion |
| `--respect-attrs` | (T-ii) axiom mode 활성 명시 (default 에 이미 활성) | legacy compat |

### 1.5 AV1~AV20 catalog 와의 짝 (F, **Lean4 backend 제외 명시 — R5.3**)

`amdv_safety.md` §5.2 의 20 invariant 의 default mode 는 (T-i) sorry —
seL4 팀이 자체 증명 권장.  단 다음 2 invariant 는 (T-ii) axiom 후보로
attribute 부착:

> **Backend scope**: Isabelle/HOL + Rocq 만.  Lean4 backend 는 R1=(a')
> sign-off (2026-06-01) 로 제외 — adsmt 측 Lean4/OxiLean blocker (leo4
> v1.0.0-rc.4 + adsmt-lean-binding v1.0.0-rc.4 후 unblocked, mainline
> Lean 4 path 는 v1.2.x post-RC).  Retrofit hook = adsmt v1.1.x 도달
> 시점, `cpu_virt_compat.md` §8 (4) + `av-proof-body-tracker.md` §8.

| AV | 안전장치 | (T-ii) axiom 후보 사유 |
|---|---|---|
| **AV2** | S3 NPT 격리 | cap derivation 의 seL4 측 invariant 와 직결 — seL4 팀이 axiom 수용 가능성 ↑ |
| **AV6** | S7 GIF host-managed | microkernel 측 본체 (D1a 의 vmrun wrapper 안), seL4 팀이 본문 직접 검증 가능 |

기타 18 invariant 는 attribute 부착 0 → default (T-i) sorry + (T-iv)
SMT-LIB hybrid 시도 (가능 시 자동 채움, 불가능 시 sorry).

seL4 팀이 AV2 / AV6 의 axiom 수용 거부 시 `--no-smt --no-axioms` 옵션으로
순수 (T-i) sorry export — 항상 fallback 가능.

### 1.6 Trust ledger (B)

| 모드 | trust 책임 |
|---|---|
| **(T-i) sorry** | Isabelle/HOL 의 sorry 채움이 seL4 팀 책임.  Y4 측 도구 책임 0 — 도구는 syntactic translation 만 |
| **(T-ii) axiom** | trust = Verus + Z3 + `y4-verus2isabelle` 도구 자체.  도구 버그 → wrong axiom 발생 가능 → Y4 측이 (T-ii) 사용 시 **별도 audit log + diff verification 필수** (§3.x 도구 형상의 logicutils stamp 가 audit chain) |
| **(T-iv) SMT-LIB hybrid** | trust = Verus + Z3 + Isabelle 의 `smt` method 의 SMT-LIB replay 정확성.  Isabelle smt method 는 mainline Isabelle 의 standard tactic — 추가 trust boundary 없음 |

### 1.7 seL4 팀 inbound contract (G)

도구 결과를 받은 seL4 팀의 사용 흐름:

1. Y4 PR-4 (`amdv_safety.md` §6.2) 가 `.thy` 산출물 첨부 + logicutils
   stamp.  artifact 형식: `theory Y4_AmdvSafety imports Main begin ...
   end` + version pin file (Verus / Z3 / 본 도구 / Isabelle 권장 version)
2. seL4 팀이 자체 Isabelle 환경에서 import — `theory Foo imports
   Y4_AmdvSafety begin ...` 패턴
3. sorry 채움 (Isar 자체 증명) 또는 axiom 수용 결정
4. (T-iv) SMT-LIB replay 결과 fail 시 도구 재실행 또는 manual Isar
   완성

Y4 측 forward-compat 보장:
- import path / theory 이름 변경 = v2 (incompatible)
- `imports` 의 추가 = v1.x patch (기존 use 무영향)
- AV# 추가 = v1.x patch
- AV# 제거 / 의미 변경 = v2

---

## 2. 입력 → 출력 매핑

### 2.1 핵심 매핑 표

| Verus | Isabelle/HOL |
|---|---|
| `spec fn f(x: T) -> R { body }` | `definition f :: "T ⇒ R" where "f x = body"` |
| `proof fn lemma() requires P, ensures Q` | `lemma name: "P ⟹ Q" sorry` (또는 axiom 또는 `by (smt (verit) ...)`) |
| `pub struct S { a: T, b: U }` (named fields) | `record S = a :: T b :: U` (D=a 결정) |
| `pub struct S(T, U)` (tuple struct / newtype) | `datatype S = S T U` (D=a 결정) |
| `pub enum E { A, B(T) }` | `datatype E = A | B T` |
| `Set<T>` | `'a set` (HOL.Set) |
| `Seq<T>` | `'a list` |
| `Map<K, V>` | `K ⇀ V` (option-valued partial) |
| `nat`, `int` | `nat`, `int` |
| Fixed-width int (`u8` / `u16` / `u32` / `u64` / `usize` / `i*`) | abstract `nat` (unsigned) 또는 `int` (signed) — width 정보 폐기 (G=a) |
| `forall\|x: T, y: U\| P(x, y)` (multi-binder) | `∀x::T y::U. P x y` (E=a) |
| `forall\|x\| forall\|y\| P` (nested) | `∀x. ∀y. P` (직역, E=a) |
| `exists\|x: T\| P(x)` | `∃x::T. P x` |
| `==>` | `⟶` |
| `&&`, `\|\|` | `∧`, `∨` |
| `decreases m` | termination via `function ... by (relation "measure m")` |
| `recommends` | (무시 — Verus 의 hint, 의미 변경 X, A 정밀화) |
| `assume(P)` | (T-i) 모드에서는 도구 panic, (T-ii) 모드에서만 `axiomatization where ... : P` (위험 — 명시 표시 권장) |
| `assert(P)` | (무시 — proof step, T-i 외부, A 정밀화) |
| `#![trigger ...]` annotation | v1.0 무시 (§8 unresolved 항목 2 와 짝) |

### 2.2 Operator precedence / parenthesization (F)

도구는 모든 복합식에 **explicit parenthesization** 강제 — Isabelle/HOL
의 default precedence 의존 X.  출력의 reproducibility 보장 (§1.3
logicutils stamp 와 짝):

```isabelle
(* 좋음 — explicit *)
"((P x) ∧ (Q x)) ⟶ (R x)"

(* 도구가 emit X — implicit precedence 의존 *)
"P x ∧ Q x ⟶ R x"
```

### 2.3 (T-iv) SMT-LIB 매핑 (A 추가)

P3.4 §1.3 의 (T-iv) SMT-LIB hybrid 매핑.  Verus 의 Z3 SMT-LIB 2 쿼리 →
Isabelle smt method 입력 + replay:

| SMT-LIB 2 | Isabelle smt method |
|---|---|
| `(declare-sort S 0)` | (Y4 측 type 으로 이미 매핑, 재emit X) |
| `(declare-fun f (T1 ... Tn) R)` | `consts f :: "T1 ⇒ ... ⇒ Tn ⇒ R"` (Y4 측 spec fn 과 짝) |
| `(assert φ)` | `lemma "φ" by (smt (verit) <기존 fact 들>)` |
| `(forall ((x T)) ψ)` | `∀x::T. ψ` (T 매핑 적용) |
| `(=> P Q)` | `P ⟶ Q` |
| `(and ...)` / `(or ...)` / `(not P)` | `∧` / `∨` / `¬P` |
| `(check-sat)` | (Verus 측 의무, 도구는 무관) |
| `(get-proof)` Z3 proof certificate | Isabelle smt method 가 자체 replay (proof certificate import 필요 시 sledgehammer fallback) |

(T-iv) 의 대상 = automatable proof obligation 만.  structural
definition / spec body 는 (T-i) 또는 (T-ii) hybrid (P3.4 §1.3 결정
알고리즘).

### 2.4 Y4 도메인 type 매핑 (B 추가, P1.4 §0 scope clamp 정합)

Y4-scope 한정 도메인 type — 일반 Verus translator 가 다루지 않는
Y4-specific type 의 axiomatic 매핑:

| Y4 Verus type | Isabelle/HOL emit |
|---|---|
| `Cap<T>` | `'a cap` (axiomatic type) + `cap_owns :: 'a cap ⇒ 'a ⇒ bool` + `cap_revoked :: 'a cap ⇒ bool` predicate |
| `LeaseCap` | `record lease_cap = partition_id :: nat, vmcb_caps :: vmcb_capsule cap list, npt_cap :: npt_capsule cap, ...` (vmm_arch.md §2.3 의 struct 직역) |
| `VcpuId` | `type_synonym vcpuid = nat` |
| `RegId` | `datatype reg_id = RAX | RBX | ... | RIP | RFLAGS | CR0 | ...` (§4.1 화이트리스트 enum) |
| `CapsuleMsg` | `datatype capsule_msg = VmcbReadReg vcpuid reg_id | VmcbWriteReg vcpuid reg_id u64 | ...` (vmm_arch.md §8.1 enum 직역) |
| `AuditEntry` | `record audit_entry = ts :: nat, vm_id :: nat, severity :: severity_tag, op_tag :: op_tag, payload :: audit_payload` (S12.2 schema 직역) |
| `XChaCha20Key`, `Aes256Key` | `'a key` (axiomatic) + `key_destroyed :: 'a key ⇒ bool` + `decryptable :: 'a key ⇒ bytes ⇒ bool` predicate (S12.5 / S13.9 정합) |
| `HostFrame`, `GuestPaddr`, `VAddr` | `type_synonym host_frame = nat`, `guest_paddr = nat`, `vaddr = nat` (abstract) |
| `Y4Error` | `datatype y4_error = InvalidArgument | NoMemory | BadCap | Timeout | AlreadyDecided | ...` |

### 2.5 미지원 처리 정책 (C)

미지원 Verus 기능 만남 시 **도구 panic + 명확한 에러** (P1.4 §0 scope
clamp + P3.4 §1.1 정합):

| 미지원 항목 | 도구 동작 | Y4 측 회피 권고 |
|---|---|---|
| `Tracked<T>` / `Ghost<T>` (linear ghost state) | panic with `error: linear ghost state not supported (Y4-scope)` | Y4 측 invariant 작성 시 ghost state 대신 `spec fn` / `proof fn` 으로 표현 |
| `verifier::trusted` | (T-ii) axiom 으로 직접 매핑 (attribute 없이도) | Y4 측 명시 attribute 부착 권고 (audit chain 명확) |
| Closure / function pointer | panic with `error: closure/fn-ptr not supported (Y4-scope)` | spec fn 의 named definition 으로 분리 |
| `state_machines_macros` DSL | panic with `error: state_machines_macros DSL not supported (Y4-scope)` | manual spec fn / proof fn 으로 풀어 작성 |
| 그 외 미지원 syntax | panic + 도구 issue tracker URL 안내 | 본 doc §8 unresolved 에 신규 항목 추가 후 v1.x patch |

silent skip 절대 X — wrong axiom / wrong sorry 발생 risk.

### 2.6 Theory file 분리 정책 (H)

**Per-Verus-module 1 `.thy` 파일** — `proofs/verus/src/amdv/` 의 module
구조와 1:1:

| Verus 모듈 | Isabelle theory file |
|---|---|
| `proofs/verus/src/amdv/upper/npt.rs` | `Y4_AmdvSafety_Upper_Npt.thy` |
| `proofs/verus/src/amdv/upper/cpu_pin.rs` | `Y4_AmdvSafety_Upper_CpuPin.thy` |
| `proofs/verus/src/amdv/upper/thread_group.rs` | `Y4_AmdvSafety_Upper_ThreadGroup.thy` |
| ... (각 upper/* + lower/* 1 파일) | ... |
| `proofs/verus/src/amdv/lib.rs` (top-level) | `Y4_AmdvSafety.thy` (모든 sub-module imports) |

`imports` 관계는 Y4 측 mod 의존 그래프와 1:1 — `lib.rs` 가 `mod
upper::npt;` 등을 선언하면 `Y4_AmdvSafety.thy` 가 `imports
Y4_AmdvSafety_Upper_Npt ...` 로 emit.

장점:
- AV# 추가 시 해당 Verus module 의 `.thy` 만 재 emit (logicutils
  freshcheck 정합)
- seL4 팀이 일부 invariant 만 import 가능 (`theory Foo imports
  Y4_AmdvSafety_Upper_Npt begin ...`)
- Y4 의 §3.1 Layer (Upper/Lower) 분류가 file naming 으로 직접 노출

---

## 3. 도구 형상

### 3.1 채택 — 옵션 (A) 별도 sibling repo

| 옵션 | 의미 | 채택 |
|---|---|:---:|
| **(A) 별도 repo `/home/ybi/y4-verus2isabelle/`** | sibling 패턴 (P1.4 §5.3 정합).  y4-drivers / y4-hypercall 와 동일.  seL4 팀에 도구 자체 contribute 가능 (Apache-2.0) | **◎** |
| (B) Y4 워크스페이스 멤버 `Y4/tools/v2i/` | 시작 비용 작음, 재사용성 약간 낮음, P1.4 §5.3 sibling 정책 위배 | ✗ |
| (C) Verus 측 backend plugin | 가장 깨끗하지만 Verus 팀과 협의 + Y4-scope 한정 보존 어려움 (general-purpose 압력) | ✗ |

### 3.2 (A) 채택 시 내부 구조 (P-redesign.5 갱신, 2026-06-01)

**Single Cargo crate `y4-verus2isabelle`** (binary + lib 분리).
**Lean4 backend 제외** (R1=a' / R5.3 정합 — adsmt 측 Lean4/OxiLean
blocker, adsmt v1.1.x 도달 시 retrofit hook).

```
/home/ybi/y4-verus2isabelle/
├── Cargo.toml              # license = "Apache-2.0"
├── LICENSE                 # Apache-2.0 full text
├── NOTICE                  # logicutils BSD-2 + adsmt-emit-isabelle tri-license attribution
├── README.md
├── src/
│   ├── lib.rs              # parser / mapper / emitter API (round-trip test 노출용)
│   ├── main.rs             # CLI binary
│   ├── parser/             # syn-based Verus AST parsing
│   ├── mapper/             # §2 매핑 (도메인 type 매핑 §2.4 포함)
│   ├── emitter/            # Isabelle Isar text emit + adsmt-emit-isabelle wrapper (P-redesign.5)
│   │   ├── mod.rs          # cert → Isar text glue
│   │   ├── adsmt_wrap.rs   # adsmt-emit-isabelle cargo git dep call + Y4 domain mapping
│   │   └── pretty.rs       # pretty-print
│   └── modes/              # (T-i) / (T-ii) / (T-iv) hybrid 결정 로직 (§1.3, Lean4 mode 없음)
├── tests/
│   ├── unit/               # 도구 자체 unit test
│   └── fixtures/           # Y4 의 proofs/verus/src/{amdv,power}/ snapshot copy (D)
└── tools/
    ├── sync-fixtures.sh    # Y4 의 proofs/ → fixtures/ 자동 sync (drift 검출)
    ├── v2i.rules           # logicutils lu-rule 형식 mode override (P3.4 §1.3.1)
    └── v2i-build.sh        # logicutils freshcheck/stamp 통합 build entry
```

**Version pin**: 별도 `verus-pin.toml` / `isabelle-pin.toml` **삭제** —
Y4 측 `Y4/unified-toolkit-pin.toml` 의 `[verus]` + `[isabelle]` +
`[adsmt-contrib]` sub-tables 가 single source-of-truth (§3.6 정합).
도구 측은 build 시 `Y4/unified-toolkit-pin.toml` 의 sub-table 을 read.

**adsmt-emit-isabelle wrapper (R5.2)**:

```toml
# Cargo.toml
[dependencies]
adsmt-emit-isabelle = { git = "https://github.com/newsniper-org/adsmt-contrib", branch = "testing" }
adsmt-cert          = { git = "https://github.com/newsniper-org/adsmt",         branch = "testing" }
```

`src/emitter/adsmt_wrap.rs` 가 Verus AST → adsmt cert (`canonical::Certificate`)
변환 후 `adsmt_emit_isabelle::emit_isabelle(&cert)` 호출.  classical
machinery 는 Isabelle `Main` 측 무효 (adsmt-emit-isabelle 의 정합).

**분량**: 단일 crate ~1500 LoC + attribute opt-in ~100 LoC + Y4 도메인
매핑 ~200 LoC + (T-iv) SMT-LIB hybrid ~300 LoC + logicutils 통합 ~150
LoC + adsmt-emit-isabelle wrapper ~100 LoC ≈ **~2350 LoC Rust** 추정.

**Theory file naming (R5.4)**: §2.6 의 frozen 형식 유지 —
`Y4_AmdvSafety_Upper_Npt.thy` (underscore-only, Isabelle `imports` 의
flat namespace 정합).  Rocq 측 sibling 도구 (`y4-verus2rocq`,
`docs/verus_to_rocq.md`) 가 nested directory (`theories/AmdvSafety/Upper/
Npt.v`) 패턴 채택 — 두 도구의 naming 분리는 의도적 (각 backend 의 module
system 정합).

**Cluster-rolling integration (R5.5)**: `av-proof-body-tracker.md` §5
의 각 cluster sub-PR 안에 wrapper emission 활성 — cluster 별 rolling.

### 3.3 logicutils 통합 — cargo dep link

`y4-verus2isabelle` 가 logicutils 를 **cargo dependency 로 직접 link**.
라이선스 호환 (BSD-2-Clause + Apache-2.0 = single-license 단방향 호환,
attribution 보존만 필요):

```toml
[dependencies]
logicutils-core = { path = "/home/ybi/logicutils", version = "0.1" }
# 또는 git dep + pin
```

장점:
- type-safe API (sub-process invoke 보다 깔끔)
- 단일 binary (sub-process spawn overhead 0)
- `freshcheck` / `stamp` 직접 호출 — Verus statement hash 추적 + emit
  결과의 reproducibility stamp 박음
- `lu-rule` / `lu-par` 도 library API 로 호출 — `tools/v2i.rules` 파일
  로 mode override + invariant 의존 그래프 DAG 정렬

NOTICE 갱신 — `y4-verus2isabelle/NOTICE` 에 logicutils BSD-2 attribution
추가 (Apache-2.0 § 4 (d) 정합).

### 3.4 Round-trip test fixtures (D)

`tests/fixtures/` 안에 Y4 의 `proofs/verus/src/amdv/` 의 snapshot copy.
신규 invariant 추가 시 drift 검출:

```sh
#!/bin/bash
# tools/sync-fixtures.sh
set -euo pipefail
src=/home/ybi/Y4/proofs/verus/src/amdv
dst=tests/fixtures/y4-amdv
rsync -av --delete --itemize-changes "$src/" "$dst/"
echo "[sync-fixtures] last sync: $(date -u +%FT%TZ) (Y4 commit $(cd /home/ybi/Y4 && git rev-parse HEAD))" \
    > "$dst/.sync-stamp"
```

CI 측 검증: `tools/sync-fixtures.sh` 가 `--dry-run` 모드에서 변경 감지
시 fail (drift 검출).  drift 발견 → 도구 측 PR + Y4 측 v2i 재 emit
짝 PR.

### 3.5 CI strategy — 2-tier (E)

#### Tier 1 — `y4-verus2isabelle` repo 자체

GitHub Actions (또는 local just) 의 step:

| Step | 검증 |
|---|---|
| `cargo fmt --check` | code style |
| `cargo clippy -- -D warnings` | lint |
| `cargo test` | unit test |
| `cargo run --bin v2i -- tests/fixtures/y4-amdv/ --output /tmp/y4-thy` | 50+ invariant fixture round-trip |
| Isabelle build of generated `.thy` | sorry / axiom / smt method 모두 syntactic OK |

#### Tier 2 — Y4 측 sub-tier (`Y4/justfile` 의 `verus2isabelle` recipe)

| Step | 검증 |
|---|---|
| `cargo install --path /home/ybi/y4-verus2isabelle` (latest tag) | 도구 install |
| `v2i Y4/proofs/verus/src/amdv/ --output Y4/proofs/isabelle/` | Y4 의 50+ invariant 산출물 generate |
| Isabelle build of `Y4/proofs/isabelle/` | seL4 팀 환경 simulating |

Tier 2 는 PR-3 / PR-4 의 paper artifact 생성 시점에 실행 — Y4 의 매
PR commit 마다 실행 X (cost ↑).

### 3.6 Unified toolkit pin — `Y4/unified-toolkit-pin.toml` (P-redesign.2, 2026-06-01)

이전 `verus-pin.toml` + `isabelle-pin.toml` 분리 형식 → **`Y4/unified-
toolkit-pin.toml` single file + sub-table 형식** 으로 합병.  adsmt v1.0
unified vision (3-way: adsmt + logicutils + OxiZ, **logicutils absorbed
into adsmt**, P5 option 5 bidirectional embed) 정합.

**Lean4 / OxiLean 제외** (R1=a', adsmt 측 Lean4/OxiLean blocker — adsmt
v1.1.x 도달 후 reconsider, 현 시점 deferred — `cpu_virt_compat.md` §8
unresolved 정합).

#### `Y4/unified-toolkit-pin.toml` 형식 (R2.1 / R2.2 / R2.3)

```toml
# SPDX-License-Identifier: Apache-2.0
# Y4 unified verification toolkit pin (P-redesign.2 sign-off 2026-06-01)
# Companion `unified-toolkit-pin.lock` captures build-time commit
# hashes for reproducible-build evidence (R2.10).

[verus]
# 지원 Verus version range — latest verus-bin 의 stable + 1 minor 이전
min_version = "0.x.y"
max_version = "0.x+1.*"
recommended = "0.x.z"

[isabelle]
# 지원 Isabelle version range — 양측 환경 격리, best-effort emit
min_version = "Isabelle2024"
max_version = "Isabelle2025+1"
recommended = "Isabelle2025"

[adsmt]
# adsmt = abductive engine + HOL+HKT kernel + type-class layer +
# Rocq/Isabelle first-class.  Lean4 backend 는 deferred (R1=a').
# logicutils 는 adsmt 안 absorbed — sub-table 별도 X (P5 option 5).
repo    = "https://github.com/newsniper-org/adsmt"
channel = "testing"      # rolling (feedback_adsmt_testing_channel_pin)
# 본 channel pin 은 floating; build-time commit hash 는 .lock 에 capture

[adsmt-contrib]
# adsmt-emit-rocq + adsmt-emit-isabelle (lib crates only, source-only
# packaging per a838525).  Lean4 emit (adsmt-emit-lean) 미활용 (R1=a').
repo    = "https://github.com/newsniper-org/adsmt-contrib"
channel = "testing"      # rolling, adsmt 와 lockstep

[oxiz]
# OxiZ — pure-Rust Z3 reimplementation, 100% Z3 parity across 8 logics.
# adsmt 측 fork (Honey-Be/oxiz) feat/enable-writer branch 의 transitive
# — adsmt 측 [patch.crates-io] 가 redirect, Y4 측 별도 pin 불필요
fork_repo   = "https://github.com/Honey-Be/oxiz"
fork_branch = "feat/enable-writer"
transitive  = true
```

#### `Y4/unified-toolkit-pin.lock` 형식 (R2.10, reproducibility)

```toml
# Auto-generated by build / publish workflow.  Captures the exact
# commit hashes resolved from the testing-channel HEADs at build time.
# logicutils-driven verification: freshcheck --method=hash on the .lock
# file gates the proofs/ rebuild (proofs/README.md §"워크플로우" 정합).

[adsmt]
commit_sha    = "<resolved at build time>"
captured_at   = "<ISO 8601 timestamp>"
verus_version = "<from verus --version>"

[adsmt-contrib]
commit_sha    = "<resolved at build time>"
captured_at   = "<ISO 8601 timestamp>"

[oxiz]
# adsmt 측 [patch.crates-io] 의 transitive 결과
submodule_commit_sha = "<from adsmt submodule pin>"
```

#### Cargo `[patch.crates-io]` mechanism (R2.4)

Y4 측 워크스페이스의 `Cargo.toml` 에 git dep redirect:

```toml
[patch.crates-io]
# adsmt + adsmt-contrib 의 testing channel 의 latest HEAD 추종
# (rolling).  build 시점 commit hash 는 unified-toolkit-pin.lock 에
# capture.
adsmt-core        = { git = "https://github.com/newsniper-org/adsmt", branch = "testing" }
adsmt-cli         = { git = "https://github.com/newsniper-org/adsmt", branch = "testing" }
adsmt-engine      = { git = "https://github.com/newsniper-org/adsmt", branch = "testing" }
adsmt-theory      = { git = "https://github.com/newsniper-org/adsmt", branch = "testing" }
adsmt-cert        = { git = "https://github.com/newsniper-org/adsmt", branch = "testing" }
adsmt-emit-rocq     = { git = "https://github.com/newsniper-org/adsmt-contrib", branch = "testing" }
adsmt-emit-isabelle = { git = "https://github.com/newsniper-org/adsmt-contrib", branch = "testing" }
# adsmt-emit-lean — Lean4 측 deferred (R1=a'), 미포함
```

**대안 — Arch Linux PKGBUILD 측 system-wide install (사용자 환경 측 옵션)**:
`adsmt-src-testing` + `adsmt-contrib-src-testing` PKGBUILD 설치 시
`/usr/src/adsmt/` + `/usr/src/adsmt-contrib/` 로 source tree 배치.  Y4
측 `Cargo.toml` 의 `[patch.crates-io]` 가 path dep 으로 redirect 가능.
양 방식 (git dep / PKGBUILD path) 의 build 결과는 동일 (channel pin
+ commit hash capture 정합).

#### Verus multi-backend 활성화 메커니즘 (R2.6 / R2.7, 갱신 2026-06-03)

**Verus 의 기존 `-V <key>` extended-multi flag mechanism** (rust_verify/
src/config.rs `OPT_EXTENDED_MULTI: &str = "V"`) 안에 새 backend 옵션 추가
— 기존 `-V cvc5` (EXTENDED_CVC5) 패턴 정합.  새 `--backend=` flag 정의
**X** (Verus 본체 patch 분량 ↓, upstream contribute-back path 의 자연성 ↑).

```sh
# 단일 backend (strict, R2.7 fallback X)
just verus                       # 기존 default (Z3, no flag)
just verus -- -V oxiz            # 신규 OxiZ backend (EXTENDED_OXIZ)
just verus -- -V adsmt           # 신규 adsmt backend (EXTENDED_ADSMT,
                                 #   R3.12 opt-in third — abductive verdict
                                 #   reporter 와 짝)
just verus -- -V adsmt -V report-abductive-on-unknown
                                 # 위 + adsmt unknown 시 hypothesis JSON emit

# Cross-validation (P-redesign.6 의 실험) — Verus 본체 flag X
# Y4 측 just verus-cross-validate script 의 multi-invocation 로직
just verus-cross-validate        # script 가 internally 3 invocation:
                                 # (1) verus (Z3 default)
                                 # (2) verus -V oxiz
                                 # (3) verus -V adsmt   (R3.12 opt-in 6
                                 #     invariant 한정, av-proof-body-tracker §7)
                                 # → 결과 diff + smt-cross-validation-tracker §2
```

**Verus 본체 patch 의 영향 범위**:
1. `source/air/src/context.rs` 의 `SmtSolver` enum 확장: `Z3` + `Cvc5`
   + **`OxiZ`** + **`Adsmt`**
2. `source/rust_verify/src/config.rs` 의 `EXTENDED_KEYS` 에 `EXTENDED_OXIZ`
   + `EXTENDED_ADSMT` + `EXTENDED_REPORT_ABDUCTIVE_ON_UNKNOWN` 추가 +
   solver 선택 로직 갱신 (현 `if extended.contains_key(EXTENDED_CVC5)
   { Cvc5 } else { Z3 }` → match 형태로)
3. `source/air/src/smt_process.rs` (또는 동등 위치) 에 OxiZ + adsmt
   backend impl 추가 (cvc5 impl mirror)
4. Verdict enum 의 4번째 variant `Abductive { candidates, explain }`
   추가 — adsmt 만 사용 가능, z3/OxiZ/cvc5 시점에 unreachable

**`-V dual` / `-V triple` 추가하지 않음** — cross-validation 은 별
backend 의 separate run + 결과 diff (R2.7 정합), Verus 본체에 의미 없는
flag.  Y4 측 script 가 multi-invocation 로직 처리.

`proofs/verus/justfile` 의 `verify` recipe 갱신 — `just verus` 가 default
(Z3) 호출, cross-validation 은 `just verus-cross-validate` 별도 recipe.

#### Proof artifact trust marker (R2.8)

매 verify success 후 `.lu-store/` 의 stamp 에 다음 metadata 기록:

```json
{
  "backend":          "oxiz",
  "proof_sha256":     "...",
  "verus_version":    "0.x.z",
  "oxiz_version":     "0.2.2+enable-writer",
  "adsmt_commit":     "<from unified-toolkit-pin.lock>",
  "timestamp":        "..."
}
```

logicutils `stamp record` 가 본 metadata 를 BLAKE3 hash 와 함께 stamp
store 에 기록 — paper artifact §6.5 (vii) 의 logicutils-driven artifact
verification 의 input.

#### seL4 측 채택 시나리오 (기존 §1.7 mirror, Lean4 backend 측 변경 X)

기존 §1.7 의 inbound contract 그대로.  Y4 PR-4 가 `.thy` 산출물 첨부
+ unified-toolkit-pin.lock 동봉 (build 재현성 evidence).

#### 이전 분리 file (`verus-pin.toml` + `isabelle-pin.toml`) closed

P3.6 §3.2 의 별도 file 형식은 P-redesign.2 (2026-06-01) 부터 closed —
unified-toolkit-pin.toml 의 sub-table 으로 통합.  `y4-verus2isabelle`
도구 측은 본 unified file 만 read.

### 3.7 도구 자체 라이선스 (H)

**Apache-2.0** — Y4 single-license 정책 정합 (`CLAUDE.md` §3, `docs/
licensing.md`).  SPDX 헤더:

```rust
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors
```

NOTICE 의 reuse manifest:
- syn / quote crate (Apache-2 OR MIT)
- logicutils-core (BSD-2-Clause)
- (다른 cargo dep 들)

### 3.8 seL4 팀에 도구 자체 contribute (I)

v1.0 frozen 후 seL4 팀에 제안:

1. Y4 PR-4 (`amdv_safety.md` §6.2) 의 `.thy` 산출물 + 도구 자체를 함께
   제출
2. seL4 팀이 평가 — 도구의 50+ invariant round-trip 정확성 + Y4-scope
   한정의 합리성
3. 채택 시 seL4 organization 의 `verus2isabelle` repo 로 fork upstream
4. seL4 팀이 자체 maintenance — Y4 는 client 로 latest tag pull
5. Y4-scope 한정은 그대로 보존 — 외부 사용자가 일반 Verus 프로젝트 적용
   시 책임 X (P1.4 §0 scope clamp 정합)

거부 시 Y4 단독 유지 — sibling repo 그대로 publish (Apache-2.0).

---

## 4. 구현 분량 추정

| 단계 | 분량 |
|---|---|
| Verus AST parser (`syn` crate + Verus attribute 파싱) | ~300 LoC |
| 타입 매핑 + 기본 식 번역 | ~400 LoC |
| `quantifier` / `decreases` / pattern match | ~300 LoC |
| `Set` / `Seq` / `Map` API 매핑 | ~200 LoC |
| Isabelle text emit + pretty-print | ~200 LoC |
| Test fixtures (proofs/verus/src/ 의 50+ invariant 를 round-trip) | ~300 LoC |
| **합계 (Phase 1 — `sorry` only)** | **~1500 LoC Rust** |

attribute opt-in (T-ii) 추가: +~100 LoC.

---

## 5. 차단 의존

도구 자체는 외부 의존 적음:
- `syn` crate — Rust 파싱
- `quote` crate — Rust → Rust 출력 (Isabelle 출력은 자체 emitter)
- Isabelle 2024+ 설치 (테스트만)

차단 없이 즉시 진입 가능 — D1d / fork policy / y4-hypercall 작업과
**병렬 가능**.

---

## 6. seL4 측 채택 시나리오

1. Y4 가 `y4-verus2isabelle` 발표 + 사용 가이드 + 자체 50+ invariant
   round-trip 검증
2. seL4 팀이 도구 평가 — sorry 기반 statement skeleton 의 정확성
3. Y4 의 D1d 안전장치 13 개 (S1–S13) 을 Isabelle skeleton 으로 export
4. seL4 팀이 자체 Isar 증명 작성 (또는 일부 axiom 으로 수용)
5. seL4 mainline 의 AMD-V 패치가 verification 트랙도 포함된 형태로 머지

도구가 없을 때 vs 있을 때:
- 도구 없음: seL4 팀은 Y4 의 Rust 코드 + 영문 description 만 받음.
  Isabelle 작성을 처음부터.
- 도구 있음: Isabelle skeleton 을 자동 받음. lemma statement / 종속성
  / 보조 정의 모두 정렬됨. 본 작업이 "재작성" 이 아니라 "증명 채움".

---

## 7. 동결 정책

본 문서는 v0 spec.  `v1.0 frozen` 마킹 조건:
- §1 hybrid scope 사용자 sign-off
- §2 매핑 표 사용자 sign-off
- §3 repo 형상 결정

frozen 후 도구 구현 PR 진입.  도구 구현은 D1d 의 seL4 측 패치와
**병렬** 가능 — 차단 없음.

---

## 8. 미해결 / 추가 결정 필요

### 8.1 닫힘 ledger (sign-off 또는 sub-decision 으로 해결됨)

| # | 항목 | 닫힘 사유 |
|---|---|---|
| 1 | Verus 의 `Tracked<T>` / `Ghost<T>` 매핑 | **v1.0 미지원 확정** (A).  Y4 측 invariant 작성 시 ghost state 회피 (P3.5 §2.5 정합).  v1.x patch 로 매핑 추가는 실수요 (Y4 의 AV1~AV20 catalog 가 ghost state 사용 시) 발생 시점에 검토 |
| 2 | Verus 의 trigger annotation (`#![trigger ...]`) | **v1.0 무시 + emit 0** (P3.5 §2.1 정합).  Verus 의 trigger 는 Z3 quantifier 호출 hint, Isabelle 측 quantifier 는 외부 hint 무관 — semantic 영향 0 검증됨 |
| 3 | Isabelle 의 locale / class 활용 | **v1.0 미사용 확정**.  Y4 의 AV1~AV20 이 trait bound polymorphic spec 0 (모두 concrete domain type, P3.5 §2.4).  Y4 측이 향후 trait-based spec 도입 시 v1.x patch 검토 |
| 4 | Round-trip test 의 ground truth | **확정** — `tests/fixtures/y4-amdv/` 에 Y4 의 `proofs/verus/src/amdv/` snapshot copy + `tools/sync-fixtures.sh` 자동 sync (P3.6 §3.4 결정).  신규 invariant 추가 → sync script 가 drift 검출 → 도구 측 PR + Y4 v2i 재 emit 짝 PR |

### 8.2 v1.0 통합 항목 (§8 에서 제외)

- **(T-iv.a) SMT-LIB hybrid** → P3.4 §1.3 에 v1.0 기본 강제 (logicutils-
  augmented).  §8 unresolved 영역 X.

### 8.3 v2 (incompatible) 후보 — multi-backend / 외부 chain

본 도구의 v2 (구조적 변경) 단계에서 검토할 중간 언어 경유 옵션 — v1.0
단독 불채택, 검토 시점에 재평가:

| 후보 | 의미 | 채택 시 영향 | v1.0 불채택 사유 |
|---|---|---|---|
| **(T-iii.a) Why3 backend** | Verus → WhyML → Why3 의 Isabelle/HOL backend (외주). 무료로 Coq / Z3 cross-validation 도 얻음 | 도구 chain 길이 ↑.  Tracked<T> ↔ WhyML ML-style type 매핑 분량 ↑ | WhyML 학습 곡선 + Why3 의 Isabelle backend 의존 + v1.0 ~2250 LoC 초과 분량 |
| **(T-v.a) Y4-IR + Isabelle backend (multi-backend 진입점)** | Y4 자체 IR 설계 → Isabelle / HOL Light / Rocq / Lean4 multi-backend.  long-term flexibility | greenfield IR 설계 부담 + 의미 검증 비용 | Y4 측 Isabelle 외 backend 실수요 0 — single backend 면 IR 의 의미 X |

채택 trigger:
- (T-iii.a) — Y4 측 invariant 가 Coq cross-check 또는 Z3 외 SMT solver
  검증 필요 시
- (T-v.a) — seL4 외 다른 verification 도구 (HOL Light / Rocq / Lean4)
  로의 contribute-back 실수요 발생 시

### 8.4 비채택 후보 closed ledger (P3.4 E 재검토 정합)

다음 5 후보는 **closed** — 미래 검토 시 재시도 차단을 위해 사유 명시:

| 후보 | closed 사유 |
|---|---|
| **Boogie / IVL 경유** | Verus 자체가 Boogie-style verifier 라 가까운 IR 존재하나, Boogie → Isabelle 산업 도구 부재 (semantics 형식화 paper 만, production-grade 0) |
| **Rocq 경유** | Y4 가 이미 Rocq 사용하나, Rocq → Isabelle 번역기 production-grade X.  dependent type ↔ simple type mismatch |
| **F\* 경유** | VeriSMo 가 F* 변형 사용 — 호환성 잠재성 ↑ 그러나 Verus → F* 번역기 부재.  학습 곡선 ↑↑ |
| **Dedukti / OpenTheory** | universal proof checker 후보, 그러나 SMT-side 입력 약함 (proof obligation 지원 X).  Y4 의 (T-iv) SMT-LIB hybrid 와 mismatch |
| **TLA+ / TLAPS 경유** | 동시성 invariant 표현 강함, 그러나 함수형 spec (Verus 의 `spec fn`) 표현 약함 — semantics mismatch |

위 5 후보는 v2 단계에서도 재시도 X — Y4 의 verification chain 의 Verus
+ Z3 + Isabelle 결합과 fundamental mismatch.

### 8.5 신규 unresolved (Phase C 진입 후 결정)

1. **Attribute namespace 충돌** — 현재 `#[verus_to_isabelle::axiom]` 만
   사용.  미래 추가 attribute (`#[verus_to_isabelle::sorry]` 강제 sorry,
   `#[verus_to_isabelle::skip]` emit 제외 등) 도입 시 namespace 정책.
   현재 미정 — Phase C 진입 후 도구 사용 패턴 관찰 후 결정.
2. **(T-iv) SMT-LIB Z3 proof certificate ↔ Isabelle smt method 호환성**
   — 현재 best-effort (일부 obligation 자동 replay fail 가능).  fail
   시 sledgehammer fallback 정책 + 어떤 obligation 이 fail 하는지의
   특성화 (Y4 의 AV1~AV20 catalog 안에서 measurable).  Phase C 진입
   직후 microbench (50+ invariant 의 (T-iv) success rate 측정).
3. **Verus polymorphic generics (`<T>`)** — Y4 의 도메인 type 외에
   등장 시.  현 도구는 P3.5 §2.4 의 9 도메인 type 만 매핑, 그 외
   generic spec fn 만나면 panic.  Y4 의 신규 invariant 에서 등장 시
   v1.x patch 로 type variable polymorphic emit 추가 검토.

---

## 9. fork-policy 와의 정합성

본 도구의 산출물 (`*.thy` 파일) 이 seL4 fork 의 어디에 들어가는지:

| 위치 | 의미 |
|---|---|
| (i) seL4 fork 의 `proof/Y4/` 디렉터리 | additive — fork-policy §1 contract 만족. 기존 proof/ 변경 0. |
| (ii) seL4 fork 의 기존 `proof/sel4-{arch}/` 와 통합 | 기존 정리 변경 위험 — fork-policy 의 forbidden 항목과 충돌 가능. 비추천 |
| (iii) Y4 측 `proofs/isabelle/` 별도 트리 | seL4 fork 와 분리. PR 로 mainline 보낼 때 (i) 로 이동 |

**권고: (iii) 시작, mainline contribute 시 (i) 로 이동.**  fork-policy
의 Strictly Additive 원칙 자연 만족.
