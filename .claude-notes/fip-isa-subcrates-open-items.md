<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# `fip/isa-subcrates` 브랜치 점검 — 미결 항목 (2026-10-02)

> **갱신 (2026-10-02):** rustc 1.98 정합(`8ed1f70`)으로 §0 fmt 실패와 §1-2 prefetch 가
> 해소됐고, 브랜치는 main 으로 port 됐다(브랜치는 유지). `fip/` 는 CLAUDE.md §5 ·
> README 레이아웃에 등재했다(§3-3 일부). 나머지 항목은 미결 그대로다.

- **대상:** `fip/isa-subcrates` @ `468a851`. 별도 detached worktree 에서 점검했다(읽기 전용).
- **범위:** `main...fip` 차이 = 워크스페이스 멤버 `fip/{core,macros,x86_64,aarch64,riscv64}` + `Cargo.lock` 의 경로 패키지 5개.
- **방법:**
  1. 직접 실행: fmt / clippy / test / 교차 타깃 check.
  2. 4차원 리뷰(정확성 · 문서 · 통합 · 로드맵). 각 지적마다 반박 검증자 2명을 붙였다.
     - 지적 61건 중 52건이 살아남았고, 중복을 합쳐 아래로 정리했다.
     - 반박돼 버린 9건은 §6 에 있다.
- 줄 번호는 `468a851` 기준이다.

---

## 0. 실측 요약

| 검사 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | ✅ **해소 (2026-10-02, `8ed1f70`)** — 당시: ❌ **실패** — `fip/x86_64/src/lib.rs:47-53`. 이 커밋(a1799cd, 2026-09-04)이 쓰인 뒤 rustfmt 가 1.9.0(rustc 1.98.1)으로 올라가면서 한 줄 포맷을 요구한다. main 은 통과. ⇒ **이 브랜치에서 `just ci` 는 빨간불** |
| `cargo clippy … --all-targets -D warnings` (fip 5개) | ✅ — 단 workspace lints 를 받지 않은 상태라서다(§2-1) |
| `cargo test` | ✅ core 10 unit + 5 doctest, macros 1 doctest. arch 크레이트 3개는 **테스트 0** |
| `cargo check --target` | ✅ x86_64-unknown-none (core·x86_64), aarch64-linux-gnu, riscv64gc-linux-gnu. bare-metal aarch64/riscv64 타깃은 미설치 |
| MSRV 1.94.0 | ✅ 빌드·테스트 통과(검증 에이전트 실측) |
| main 대비 | `HEAD..main` 비어 있음(main 을 이미 두 번 merge). 충돌 없음. 단 main 작업 트리의 미커밋 정정(glossary 등)이 커밋되면 다시 merge 해야 한다 |

---

## 1. 결함 (고쳐야 함)

1. **`per_cache_line::<T>()` 가 `size_of::<T>() > 64` 이면 0 을 돌려준다.**
   - 위치: `fip/{x86_64,aarch64,riscv64}/src/lib.rs` 의 `checked_div` match (x86_64 :35-40, 나머지 :31-36).
   - 「batch 크기」로 쓰면 `chunks(0)` 은 panic 하고, stride 루프는 멈추지 않는다.
   - 지금 호출자가 없어 잠재 결함이다.
   - → `Some(0) => 1` 로 하한을 걸고, ZST/1/64/65/128 B 단위 테스트를 단다.
2. ✅ **해소 (2026-10-02, `8ed1f70` — `asm!("prefetcht0")`, x86_64-unknown-none release asm 에서 인라인 확인).** 당시: **x86_64 `prefetch_read` 가 커널 타깃(`x86_64-unknown-none`)에서 인라인되지 않는다.**
   - `_mm_prefetch` 는 `#[target_feature(enable="sse")]` 인데 해당 타깃은 SSE 를 끈다.
   - 그래서 힌트마다 out-of-line `call`/`jmp` 가 된다(asm 실측). host 에서는 `prefetcht0` 로 인라인된다.
   - SAFETY 주석이 실제 의무(SSE)가 아니라 포인터를 근거로 든다(`:47-48`).
   - 문서(`:42-43`)는 「PREFETCHT0」로 인라인된다고 말한다.
   - → `core::arch::asm!("prefetcht0 [{0}]", in(reg) p, options(nostack, preserves_flags, readonly))` 로 바꾸고, SAFETY 를 고치고, `x86_64-unknown-none` 빌드 게이트를 둔다.
3. **`#[fip]` 의 함수 판정이 느슨하다.**
   - 최상위 토큰 어딘가에 `fn` Ident 가 있으면 통과한다(`fip/macros/src/lib.rs:48-51`).
   - 그래서 `type F = fn();` · `const C: fn() = …` · `static` · `impl Tr for fn()` 도 받는다.
   - → 속성·가시성·한정자(const/async/unsafe/extern) 다음 위치에 `fn` 이 있는지 검사하고, compile_fail doctest 를 단다.
4. **`#[fip]` 가 속성 인자를 조용히 버린다.**
   - `_attr` 를 읽지 않는다(`:46`). 로드맵 옵션이 생기면 전방 호환이 위험해진다.
   - → `_attr` 가 비어 있지 않으면 `compile_error!` 를 낸다.
5. **`#[fip]` 의 주입 문서가 확인되지 않은 사실을 단정한다.**
   - 「힙 할당 없음(no-alloc)」을 사실처럼 적는다(`:61-62`). 할당하는 함수, `async fn`, 재귀 함수에도 붙는다.
   - → 「declared FIP — v1 에서 미강제」로 문면을 고친다.
6. **`#[fip]` 의 문서 주입이 앞에 붙어 원래 rustdoc 요약 줄을 가로챈다.**
   - `let mut out = doc; out.extend(item);` (`:66`). 내부 경로(`design_decisions §2.1`)가 하류 문서로 샌다.
   - → 사용자 문서 뒤에 `# FIP` 절로 붙인다.
7. **`fip_update!` 에 place 가 아닌 식을 넣어도 컴파일되고, 아무 일도 하지 않는다.**
   - 원인: `&mut $place` (`fip/core/src/lib.rs:149`). 예: `const K` · `x.clone()` · `f()`.
   - 경고도 없다. 외부 크레이트 매크로 안에서는 `const_item_mutation` lint 가 억제되기 때문이다.
   - → `fn fip_update<T>(&mut T, impl FnOnce(T)->T)` 함수형으로 가거나, compile_fail 테스트를 단다.
8. **`fip_map!` 이 원소마다 클로저 식을 다시 평가한다.**
   - 위치: `:169`. `move` 클로저의 상태가 원소 사이에서 이어지지 않는다(`[11,11,11]` ≠ 기대 `[11,12,13]`).
   - 문서의 `FnMut` 의미(`:156-157`)와 어긋난다.
   - → 루프 전에 한 번 바인딩하거나, 제네릭 함수로 바꾼다.

## 2. 통합 · CI 공백

1. **fip 크레이트 5개 모두 `[lints] workspace = true` 와 `publish = false` 가 없다.**
   - 기존 멤버는 모두 선언한다(`alloc/Cargo.toml:12,30` 등).
   - 그래서 workspace 정책(clippy all=deny · pedantic · missing_docs · unsafe_op_in_unsafe_fn)이 빠졌다.
   - 켜면 pedantic 경고 약 20건이 `-D warnings` 에 걸린다. 종류: `doc_markdown`, `inline_always`, `missing_panics_doc`, `items_after_statements`.
   - → 켜고 고친다. `y4-fip-macros` 에는 `#![forbid(unsafe_code)]` 를 추가한다.
2. **CI 가 host 에서만 돈다.**
   - `just build/test/lint` 은 host `--workspace` 만 돈다(`justfile:80-97`). bare-metal·교차 타깃 빌드가 없어서 위 §1-2 가 숨어 있었다.
   - `rust-toolchain.toml` 에 `targets` 가 없다. aarch64-unknown-none, riscv64gc-unknown-none-elf 는 미설치다.
   - → `just fip-cross` 게이트를 `ci` 에 넣는다. MSRV(1.94.0) job 도 고려한다.
3. **arch 선택 장치가 없다.**
   - 어느 `y4-fip-<arch>` 든 아무 타깃에서나 오류 없이 빌드된다. `target_arch` 가드도, facade 크레이트도 없다.
   - `#[fip]` 는 core 나 arch 크레이트로 재수출되지 않는다.
   - 매니페스트 설명 「FIP macros — … specialization」은 실제 내용(arch 튜닝 상수뿐)과 다르다.
4. **소비자가 0이다.**
   - `alloc`/`ipc`/`capsules`/`kernel` 어디도 fip 에 의존하지 않는다. `Fip` 트레이트 구현체도 0이다.
   - 설계가 지목한 hot path(IPC fast path · 스케줄러 · alloc 메타데이터 · capsule 상태 기계)에는 아직 연결되지 않았다.
   - → 첫 소비자(예: `alloc/src/slab.rs` freelist compaction)를 고르거나, 「소비자 없는 라이브러리」라고 명시한다.
5. **unsafe-audit gate 는 문서상 근거만 있다.**
   - x86_64 크레이트(`:13`)가 「capsule §2.1 unsafe-audit gate」를 근거로 들지만, 그 게이트를 실행하는 도구나 레시피가 없다.
   - → 게이트를 구현하거나 문면을 고친다. `#![deny(unsafe_code)]` + 해당 fn 에만 `#[allow]` 를 쓴다.

## 3. 낡은 문서

1. **`y4-fip-core` crate 문서의 로드맵이 낡았다**(`fip/core/src/lib.rs:34-40`, `:48`).
   - `#[fip]` proc-macro(ffb83cb)와 per-ISA arch 모듈(a1799cd)이 이미 착지했는데도 「다음 단계」로 남아 있다.
   - 「제공 API」 목록(`:26-32`)에 `#[fip]` 이 없다.
2. **combinator 문서가 부정확하다.**
   - `compact`/`compact_with`/`dedup` 은 반환 길이 뒤의 원소가 「제자리 유지」라고 하지만, 실제로는 swap 으로 순서가 섞인다. 버린 원소는 drop 되지 않는다.
   - `fip_update!` 는 panic 시 `Default` placeholder 가 남는 점을 적지 않았다.
   - macros 문서의 「런타임 TCB 아님」은 「빌드 TCB 에는 듦(P7)」으로 보강해야 한다.
3. **프로젝트 문서가 `fip/` 를 모른다.**
   - `CLAUDE.md` §5 레이아웃 · §2 상태에 없다.
   - `README.md` 에도 없다: `:92`(workspace 멤버 목록), 트리, 상태표, 테스트 목록.
   - `docs/phase_plan.md` 에 위치 없음.
   - `docs/design_decisions.md` §2.1 과 perceus scan §9 는 아직 「설계만」 상태이고, 구현이 §5.1 설계와 다른 점을 기록하지 않았다(강제 0, reuse 미통합, inline-macro → 함수 + macro_rules).
4. **NOTICE Acknowledgements 에 아이디어 출처가 빠졌다.**
   - 빠진 것: Perceus(PLDI'21), Frame-Limited Reuse(ICFP'22), FP²(ICFP'23), Koka `fip fun`. crate 문서는 이들을 인용한다.
   - 코드 import 는 0건이다.
5. **P3 「계약은 core 가 공유」 문면에 실체가 없다.**
   - arch 크레이트 3개가 그렇게 적지만(`:10-11`), core 에는 arch 계약(trait/const)이 없다. ARCH/CACHE_LINE/per_cache_line/prefetch_read 는 복붙이다.
   - `docs/hw_mechanism_abstraction.md` registry 행도 없다.

## 4. 결정 필요 (사용자)

1. **aarch64/riscv64 크레이트를 지금 워크스페이스에 둘지.**
   - D2(x86_64 first)와 `hw_mechanism_abstraction.md` §5 3단계(「타 arch backend 는 그 form-factor 착수 시」)와 긴장한다.
   - 갈래: 명시적 예외로 승인해 D2/CLAUDE.md §8 에 기록 / form-factor 착수까지 워크스페이스에서 뺌.
2. **`Fip` 마커 트레이트의 뜻.**
   - 지금은 구현체·bound·`#[fip]` 연동 모두 0이고, 안전 트레이트라 의무도 없다.
   - 갈래: 「소유 heap 포인터 없음 / free 하는 Drop 없음」 같은 뜻을 정해 `unsafe trait` + derive 로 / 정의될 때까지 제거.
3. **capability 타입과 `T: Default` 요구의 충돌.**
   - `fip_update!`/`fip_map!` 은 `mem::take` 를 쓰므로 `Default` 가 필요하다. capability 토큰에 `Default` 를 주면 placeholder 토큰을 mint 하게 되어 C2/P11 을 위반한다.
   - 갈래: placeholder 없는 변형(panic=abort 전제의 replace_with) / `fip_update_with!(place, placeholder, f)`.
4. **arch 계약 형태.** core 에 `ArchTuning` trait(또는 const 계약)을 두고 facade 를 둘지, 지금의 복붙 구조를 유지할지.
5. **`CACHE_LINE` override.** 문서는 「플랫폼별 override 대상」(Apple M / Cavium = 128 B)이라고 하지만 장치가 없다. cfg/feature 로 할지.

## 5. 로드맵 (차단 원인 포함)

| 항목 | 상태 | 실제 차단 원인 |
|---|---|---|
| `#[fip]` no-alloc guard (debug) | 미착수 | y4-alloc 이 `GlobalAlloc` 이 아니다(`alloc/src/integrated.rs:16-19`). 루트 태스크가 y4-alloc 을 링크하지 않는다(`kernel/Cargo.toml` deps 비어 있음). per-CPU 상태도 없다. 사슬: GlobalAlloc 어댑터 → kernel 링크 → per-CPU CPU-id → guard. 대안으로 컴파일 타임 장치(`#[fip]` 크레이트의 `alloc` 금지 lint/그래프 검사)도 있다 |
| Verus 「할당 0 + 스택 bounded」 | 미착수 | verus-fork 하네스는 이미 돈다(54 verified). 진짜 공백은 **명세 인코딩**이다. 둘로 쪼갠다. ⒜ `compact`/`dedup` 기능 명세는 지금 쓸 수 있다(`proofs/verus/src/fip/`, formal-first). ⒝ zero-alloc/스택 상한 인코딩 설계만 verus-fork 안정화에 건다. 소비자가 ipc/alloc 에 들어오기 전에 ⒜ 가 선행되어야 한다 |
| aarch64 `PRFM PLDL1KEEP` / riscv64 `prefetch.r` | no-op stub | 기술적 차단은 없다. PRFM 은 ARMv8-A 기본이고, `prefetch.r` 은 ORI HINT 공간이라 Zicbop 없는 코어에서는 no-op 이다. 막는 것은 §4-1 결정과 `forbid(unsafe_code)` 완화(audited allow) 정책이다 |
| allocator drop-then-reuse fast path (perceus scan §3) | 미착수 | AllocFrontend/Freelist trait 미착지. 지금 SLAB zone 이 이미 LIFO(`alloc/src/slab.rs:66-73`)라 즉시 재사용과 hardened quarantine(B2/B6)·zero-on-free 와의 정합을 먼저 설계해야 한다. bench 측정 게이트도 정의해야 한다 |
| precision(free-at-last-use) → 민감 자료 early-drop + zeroize (design_decisions §2.1) | 담당 없음 | key-mgmt 구현, HIU ABI 동결(임대별 키 `0xE80`). lint 냐 타입(`Zeroizing<T>`)이냐를 먼저 정한다 |
| FP² 「상수 스택」 절반 | 계획 없음 | `#[fip]` 가 재귀 함수를 받는다. 위 Verus ⒝ 와 함께 다룬다 |

## 6. 반박되어 버린 지적 (참고)

검증자 다수가 반박한 것. 재검토가 필요하면 근거는 워크플로 저널 `wf_3ee45741-4f5` 에 있다.
- 「core-only 경계 = 컴파일러 강제 zero-alloc」 주장이 과장이라는 지적(3건 변형)
- `fip_update!`/`fip_map!` panic 시 placeholder 잔류(문서 보강 항목 §3-2 로만 남김)
- crate 문서가 reuse fast path / global allocator hook 을 「있는 것처럼」 쓴다는 지적
- unsafe-audit gate 의 lint 미구현(§2-5 로 흡수)
- FIP Verus 산출물이 deferred 목록에 없다는 지적, FP² 상수 스택 무계획(§5 표로 흡수)
- CACHE_LINE 64 B 가 destructive-interference 크기를 과소평가(§4-5 로 흡수)

---

## 권장 순서 (merge 전 최소)

1. fmt 정정(§0) — `just ci` 녹색 복구.
2. `[lints] workspace = true` + `publish = false` + pedantic 정리(§2-1).
3. §1-1(`per_cache_line` 하한) · §1-2(prefetch `asm!`) · §1-3~6(`#[fip]` 판정·인자·문서) · §1-7~8(매크로) + 테스트.
4. §3 문서 정리(crate 로드맵, CLAUDE.md/README 등재, NOTICE, design_decisions §2.1 realization 상태).
5. 그다음 main 의 미커밋 정정 커밋 → main → fip merge → fast-forward 검토.

§4 결정과 §5 로드맵은 merge 를 막지 않는다. 다만 첫 소비자를 넣기 전에 §4-2 · §4-3 · §5 Verus ⒜ 가 선행되어야 한다.
