<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4-FIP(`fip/`) vs portable-fip — 상세 비교 (2026-11-01)

> **결정 (2026-11-01, 사용자):** portable-fip 을 외부 의존으로 쓴다(장기적으로 Y4 안으로 흡수, 독립 저장소는
> 추출판). §8 의 **A · B · C · D 전부** 채택, `y4-fip-*` 는 얇은 facade(`y4-fip-core` · `y4-fip-arch`), portable-fip
> 증명은 Y4 proofs 게이트에서 재검증(Verus 백엔드 = 시스템 Z3 4.16.0 고정). G 는 portable-fip 쪽 과제로 남는다.
> §9 의 「Y4 verus-fork 로 portable-fip 증명 재실행」은 측정됐다 — `1a147cd` 에서 fip-strict 7 · fip-core 36 verified.
>
> **핀 갱신 (2026-11-01, `1a147cd` → `06c99ac`, 원격 main):** 더하기만 한 변경이다(기존 API 그대로). 상태 전달
> 조합자 `map_in_place_st` · `compact_map_st`(+ strict 판)가 들어왔다 — 상태를 `FnMut` 캡처가 아니라 값으로 넘겨
> Verus 가 상태 사슬을 증명한다. 그래서 §6-2 · §8-E(상태 있는 클로저)는 `FnMut` 판 없이 이 모양으로 닫힌다(§9 의
> 「Verus 의 `FnMut` 클로저 명세 지원」은 portable-fip `06c99ac` 커밋 메시지가 답했다 — Verus 0.2026.10.18 은
> 지역 변수를 가변 캡처하는 클로저를 거부하고, `FnMut` 호출 뒤 클로저를 불변으로 증명하므로 `FnMut` 증명은 상태에
> 대해 아무것도 말하지 못한다; Y4 는 이 주장을 따로 측정하지 않았다). 조합자마다 `# Panics` 절과 `tests/panic_state.rs` 가 생겼다(꼬리 owned 원소 계약은
> 여전히 결정 대기). Y4 재검증: fip-strict 7 · fip-core **51** verified, 변이 대조 거부 — `just ci-push` 전부 그린.

코드 교환 범위를 정하기 위한 비교다. 결정은 사용자가 내린다. 이 문서는 사실과 측정만 정리하고,
권고는 §8 에 따로 둔다.

- **기준 커밋:** Y4 `6e6e8b1`(MSRV 1.99.1), portable-fip `1a147cd`(원격 main 과 같음).
- **portable-fip 위치:** `/run/media/ybi/6ee5fba6-c9b3-4c4c-954f-89d39f34b4c7/portable-fip`,
  원격 `https://github.com/newsniper-org/portable-fip.git`.
- **읽은 Y4 원본:** `fip/{core,macros,x86_64,aarch64,riscv64}` 소스와 `Cargo.toml`, `justfile` 의 fip 게이트,
  이 디렉터리의 `fip-isa-subcrates-open-items.md`, `docs/design_decisions.md` §2.1,
  perceus scan §5·§5.1 정정 노트, CLAUDE.md, README, NOTICE.
- **Y4 작업 트리:** 측정 중에 Y4 를 바꾸지 않았다. 측정은 Y4 원본 크레이트를 path 의존으로 무는 외부 크레이트
  `/tmp/y4direct` 로 했다.

---

## 1. 한눈에

| 항목 | Y4 `fip/` | portable-fip |
|---|---|---|
| 크레이트 | `y4-fip-core` · `y4-fip-macros` · `y4-fip-{x86_64,aarch64,riscv64}` (5) | `fip-core` · `fip-alloc` · `fip-arch` · `fip-strict` · `fip-macros` · `fip-lint-rules` · `fip-testkit` (7) + 워크스페이스 밖 `tools/fip-prelint` |
| 규모(소스 줄) | core 443 · macros 180 · x86_64 110 · aarch64 82 · riscv64 82 | core 1391 · macros 2439 · strict 357 · arch 173 · alloc 60 · lint-rules 467 · testkit 5589 |
| 외부 의존 | 0 | 0 (워크스페이스 내부 의존만: core→strict·macros, alloc→strict, macros→lint-rules) |
| unsafe | core·aarch64·riscv64·macros `forbid`; x86_64 `deny` + `prefetch_read` 에만 `allow` | core·alloc·lint-rules·macros `forbid`; arch·strict·testkit `deny` + 국소 허용(`prefetch` asm, `unsafe trait FipBound`/`Cert::assume`, 계수 할당기 `impl`) |
| MSRV | 1.99.1 (`6e6e8b1`) | 1.97 (드라이버만 1.99.1 고정) |
| 라이선스 | Apache-2.0 | Apache-2.0 |
| 정의 문서 | Y4-FIP F1–F5, Y4-FBIP, Y4-FIP(n) | `#[fip]`, `#[fip(N)]`, `#[fbip]`, `strict = true` (FP² 기반, 보장 범위 표: README) |
| 강제 | 없음 — `#[fip]` 는 선언, 모양만 검사 | 매크로(토큰) + strict 인증서(타입) + Verus 증명 + `fip-testkit`(실행·토큰) + `fip-prelint`(MIR) |
| Verus | fip 명세 없음 (`proofs/verus/src/` 에 `fip/` 없음, 로드맵 ⒜) | `fip-core`·`fip-alloc`·`fip-strict`, `verus --no-cheating`, 업스트림 Verus 0.2026.10.18 |
| 소비자 | 0 | 0 (라이브러리) |
| 테스트(측정) | `cargo test -p y4-fip-{core,macros,x86_64}`: unit 18 + 3, doctest 5 + 1 + 5 + 3 통과; clippy `-D warnings` 0 건; aarch64·riscv64 는 qemu-user (`just fip-qemu`) | `just ci` 테스트 408 통과, 거부 예제 43/43, Verus 게이트 통과 |

---

## 2. 정의의 대응

| Y4-FIP | 뜻 | portable-fip 대응 | 강제 수준 (portable-fip) |
|---|---|---|---|
| F1 할당 0 | 전이적 호출 그래프(클로저 포함)가 할당 원천을 부르지 않음 | `#[fip]` | 매크로: 본문 토큰 · strict: 인증서 · prelint: MIR 의 `__rust_alloc*` 도달 |
| F2 해제 0 | 소비한 owned 값의 drop 이 해제하지 않음 | `#[fip]` (FBIP 가 아닌 것) | prelint: 암묵 drop 포함 "may free" 오류; testkit: 실행 계수 |
| F3 정적 스택 상한 | 재귀 없음 | `#[fip]`, `#[fip(N)]` (`#[fbip]` 는 재귀 허용) | 매크로: 직접 자기 재귀; testkit callgraph: 크레이트 간 순환; Verus: 검증 크레이트의 비순환 증명; prelint: MIR 순환. 바이트 상한은 어느 쪽도 없음 |
| F4 고유성 | 변형 대상이 `&mut` 또는 owned | 대응 없음 | 검사하지 않음 (`Cell`/`RefCell` 은 빠져나감) |
| F5 고차 인자도 FIP | 넘겨받는 클로저도 FIP | `param(g = K)`, `FipFn`/`Certified`, `fip_fn!` | strict: `FipBound` 컴파일 시 검사; prelint: 모든 호출 지점에서 인자 상한 검사 (`128e508`) |
| Y4-FBIP | F2·F3 해제 | `#[fbip]` | **같은 뜻**: 할당 원천은 거부하고 해제와 재귀를 허용한다 (`fip-macros/src/lib.rs:16` 표, `:1036` 재귀 오류는 `#[fbip]` 에서 꺼짐). 둘 다 FBIP(n) 은 없음 |
| Y4-FIP(n) | 할당 n 회까지 | `#[fip(N)]` | 상한 N 을 summaries/prelint 가 계산해 비교 |
| 할당 원천 | Rust 전역 할당기 **+ `y4-alloc` API + seL4 untyped retype** | Rust 전역 할당기만 (`fip_lint_rules::mir::ALLOC_ENTRY_POINTS`) | **불일치 — §6-1** |

---

## 3. 조합자 API 대응

| Y4 | portable-fip | 같은 점 | 다른 점 |
|---|---|---|---|
| `compact(xs, keep: impl FnMut(&T) -> bool)` | `compact(xs, keep: Fn(&T) -> bool)`, `strict::compact(FipFn)` | 같은 알고리즘(읽기 위치 r, 쓰기 위치 w, 유지 원소를 w 로 swap; Y4 는 매번 `xs.swap(r, w)`, portable-fip 은 `w < r` 일 때만 swap — 결과 같음). 꼬리 원소는 drop 되지 않고 순서가 섞인 채 남음. 유지 원소 순서 보존 | **`FnMut` vs `Fn`** — 상태 있는 클로저를 Y4 는 받고 portable-fip 은 받지 않음. portable-fip 은 순열·필터·순서를 Verus 로 증명, Y4 는 테스트 |
| `compact_with(xs, f: impl FnMut(&mut T) -> bool)` | `compact_map(xs, hole, f: Fn(T) -> (T, bool)) -> (usize, T)` | 1-pass 변형 + 압축 | 인자 모양이 다름: `&mut T` 변형 vs 값 소비 + `hole`. Y4 는 버릴 원소도 변형될 수 있다고 문서화 |
| `dedup<T: PartialEq>(xs)` | `dedup_by(xs, same: Fn(&T,&T) -> bool)`, `strict::dedup_by` | 연속 중복만, 비교 대상은 마지막 유지 원소 | 비교 인자 순서: Y4 `xs[r] != xs[w-1]` 즉 `ne(후보, 유지)` vs portable-fip `!same(&xs[w-1], &xs[r])` 즉 `(유지, 후보)` (`slice.rs`). 대칭이 아니거나 `ne` 를 `!eq` 와 다르게 구현한 `PartialEq` 에서만 차이 |
| `fip_update!(place, f)` — `mem::take`, `T: Default` | `update(slot, hole, f) -> T`, `replace`, `strict::update` | 제자리 값 변환 | Y4: 매크로, place 아닌 식 컴파일 오류(`addr_of_mut!`), panic 시 `Default` 가 남음. portable-fip: 함수, `Default` 대신 호출자가 `hole` 을 줌 → **open-items §4-3(capability 토큰에 `Default`) 이 생기지 않음** |
| `fip_map!(slice, f)` — `FnMut`, `Default` | `map_in_place(xs, hole, f: Fn(T) -> T) -> T`, `strict::map_in_place` | 원소별 값 변환, 클로저 1 회 바인딩 | `FnMut` vs `Fn`(Y4 테스트 `fip_map_threads_closure_state` 같은 상태 전달은 portable-fip 에서 불가); `Default` vs `hole` |
| — | `swap(xs, i, j)`, `replace` | | Y4 에 없음 |
| — | `fip_alloc::refill(Box<T>, T) -> (Box<T>, T)` | | Y4 에 없음 (FP² reuse credit 의 명시적 대응) |
| `trait Fip {}` (빈 marker, 결정 대기 §4-2) | `unsafe trait FipBound` (호출 가능 값의 할당 상한) | | 뜻이 다름 — 같은 것을 대체하지 않음 |

---

## 4. 매크로

| | Y4 `#[fip]` | portable-fip `fip-macros` |
|---|---|---|
| 형태 | `#[fip]` 만, 인자 거부 | `#[fip]`, `#[fip(N)]`, `#[fbip]`, `strict = true`, `param(g = K)`, `panic_macros = false`, `#[fip_impl]`, `fip_fn!` |
| 대상 판정 | 속성·가시성·한정자 뒤 `fn` + 몸체 (compile_fail 5개) | 함수·메서드·trait impl 메서드 |
| 검사 | 없음 (문서 절만 덧붙임) | 본문 토큰의 할당 원천·재귀·`core` 표 밖 호출(strict) |
| prelint 연동 | **표식 없음** | 드라이버 아래에서만 `fip::mark` 부착 |
| 의존 | 0 | `fip-lint-rules` (규칙 표 공유) |

측정 (§7): Y4 `#[fip]` 를 단, 할당하는 함수 `y4_declared_but_allocates` 는 어떤 검사에도 나타나지 않는다.
루트 집계(6)에도 빠진다.

---

## 5. arch 튜닝

| | Y4 | portable-fip `fip-arch` |
|---|---|---|
| 구조 | arch 마다 크레이트 3개, `pub use y4_fip_core::*` 재수출, `arch` 모듈 복붙(diff 는 문서 줄뿐) | 한 크레이트, `cfg(target_arch)` 선택 |
| `ARCH` | 크레이트별 상수 | `x86_64` / `aarch64` / `riscv64` / `other` |
| `CACHE_LINE` | 64 고정 (Apple·Cavium 128 은 "override 대상"이라고만 적음 — §4-5 결정 대기) | 기본 64, Apple aarch64 128, `--cfg fip_cache_line="32|64|128"` 로 덮어씀 (다른 값은 컴파일 오류) |
| `per_cache_line` | 하한 1, ZST `usize::MAX` | 같은 계약 |
| `prefetch_read` | x86_64 `prefetcht0` asm; aarch64·riscv64 no-op | x86_64 `prefetcht0`, **aarch64 `PRFM`**; 그 밖 no-op |
| 공통 계약 | 없음 (§4-4 `ArchTuning` 결정 대기) | cfg 단일 크레이트라 trait 이 필요 없음 |
| 게이트 | `bare-metal`(3 타깃), `fip-qemu` | `arch-override` 게이트(덮어쓰기 값 검사) |

---

## 6. 차이가 실질 문제가 되는 곳

(2026-11-01 정정: 각 항목을 두 저장소 소스와 다시 대조했다. 5 는 틀려서 고쳤고, 4 는 측정을 더했다. 나머지는 소스와
일치한다: 1 — `ALLOC_ENTRY_POINTS`, `alloc/src/{integrated,slab}.rs`; 2 — `slice.rs` 의 `F: Fn`; 3 — `slot.rs`
`update(slot, hole, f)`; 6 — 두 `compact` 모두 bound 없음; 7 — 두 `Cargo.toml`; 8 — portable-fip 루트 `Cargo.toml` 의
`unexpected_cfgs`.)

1. **할당 원천.** Y4 F1 은 `y4-alloc` API 와 seL4 untyped retype 을 할당으로 친다. portable-fip 은 Rust 전역
   할당기만 안다.
   - `y4-alloc` 의 `IntegratedAllocator::alloc` · `SlabAllocator::alloc` 은 자기 풀을 쓰는 순수 Rust 함수다
     (`GlobalAlloc` 아님, `alloc/src/integrated.rs`, `slab.rs`).
   - 지금은 `y4-alloc` 이 `fip::export` 를 선언하지 않고 `heapless` 가 그냥 넘겨지므로 ⊤ 오류로 막힌다.
   - 하지만 `y4-alloc` 이 export 를 선언하면 MIR 로 끝까지 분석되는 경로는 "할당 0" 으로 통과한다. 거짓 음성이다
     (규칙 표에서 이끌어 낸 결론이고, 측정하지 않았다).
   - seL4 retype 은 FFI 라 MIR 이 없어 ⊤ 로 안전하게 처리된다.
   - → portable-fip 에 사용자가 선언하는 할당/해제 원천 표식이 필요하다. 하위 라이브러리에 Y4 내용을 넣지 않는
     일반 기능으로 만들 수 있다.
2. **`FnMut` vs `Fn`.** Y4 조합자는 상태 있는 클로저를 받고 그런 테스트도 있다. portable-fip 의 검증 조합자는 `Fn`
   만 받는다. 교체하면 그런 호출자는 바꿔 써야 한다. 지금 소비자가 0 이라 비용은 없다.
3. **`Default` vs `hole`.** open-items §4-3 의 capability 토큰 문제를 portable-fip 의 `hole` 방식이 피한다.
   반대로 `fip_update!` 의 place 검사(`addr_of_mut!` compile_fail)와 매크로 편의는 portable-fip 에 없다.
4. **`fip_update!` 의 F2.** `Vec<u8>` 에 쓰면 `Default` 자리값을 덮어쓰며 drop 한다. 실행 시에는 빈 `Vec` 라 해제가
   없지만 정적으로는 "may free" 다 (§7 측정). `hole` 방식도 돌려받은 `hole` 을 호출자가 drop 하면 같은
   판정이 난다. 다만 그 drop 이 호출자 코드에 드러난다. (측정, `/tmp/y4direct`: `fip_core::update` 로 `Vec<u8>` 를
   바꾸고 돌려받은 `hole` 을 버리면 may free 오류, `hole` 을 반환하면 통과. 또 `hole` 을 본문에서
   `Vec::new()` 로 만들면 토큰 단계 `#[fip]` 가 `Vec::new` 를 할당 원천으로 거부한다 — `Vec::new` 는 실제로
   할당하지 않으므로 매크로의 과잉 거부다.)
5. **FBIP 등급의 뜻 — 차이 없음 (정정).** 처음 판에서 "portable-fip `#[fbip]` 는 해제만 푼다"고 적었으나
   틀렸다. `#[fbip]` 는 해제와 재귀를 모두 허용하므로 Y4-FBIP(F2·F3 해제)와 같은 뜻이다. 정렬할 것이 없다.
6. **꼬리 원소 계약(Y4 결정 대기).** 두 구현이 같은 동작이다. portable-fip 은 "꼬리는 순열의 나머지, drop 없음"을
   Verus 로 증명할 뿐, capability 부활을 막는 장치(`!needs_drop` bound 등)는 없다. 교환해도 이 결정은 남는다.
7. **MSRV.** Y4 1.99.1 ≥ portable-fip 1.97 이므로 Y4 가 portable-fip 을 무는 데 문제가 없다. 반대 방향
   (Y4 코드를 portable-fip 으로)은 1.97 에서 빌드되는지 확인해야 한다.
8. **lint 정책.** Y4 는 clippy `all = deny`, `pedantic = warn`, `missing_docs = warn` 이고 portable-fip 도 같다.
   portable-fip 크레이트를 Y4 에서 쓰거나 Y4 크레이트에 `#![cfg_attr(fip_prelint, fip::export)]` 를 넣으면 Y4 의
   `unexpected_cfgs` 에 `cfg(fip_prelint)` 를 선언해야 `-D warnings` 를 통과한다.

---

## 7. 측정 — Y4 원본을 `fip-prelint` 로

- **Y4 워크스페이스 전체를 `cargo fip-prelint` 로:** 오류 없이 끝났다(2.9 s). 다만 표식이 없어 분석한 루트는 0 이다.
  "통과" 가 아니라 "검사 안 함" 이다.
- **외부 크레이트 `/tmp/y4direct`:** Y4 원본 `fip/core` · `fip/macros` 를 path 로 물고, portable-fip `#[fip]` 루트 6개와
  Y4 `#[fip]` 함수 1개를 두었다.

| 함수 | 결과 |
|---|---|
| `compact` · `compact_with` · `dedup` (할당 없는 클로저) | ⊤ 오류 3건 — `y4_fip_core` 가 `fip::export` 를 선언하지 않음 |
| `fip_update!` · `fip_map!` (`u32`) | 통과 (매크로가 호출자 쪽에서 전개되므로 export 불필요) |
| `fip_update!` (`Vec<u8>`) | 오류: may free (`Vec<u8>` 의 drop) |
| Y4 `#[fip]` + `Box::new` | 보고 없음, 루트 집계(6)에서도 빠짐 |

- **비교용 사본 측정:** 이 측정만 사본으로 했다. `/tmp/y4probe` 는 Y4 `fip/core/src` 의 사본에 `fip::export` 한 줄을
  더한 것이다.
  - `compact` · `compact_with` · `dedup` 3건이 통과로 바뀌었다.
  - 할당하는 보조 함수를 부르는 클로저를 `compact` 에 넘기면 할당 ∞ 회와 해제 가능이 호출 경로와 함께 보고됐다.
  - 즉 Y4 조합자를 검사하는 데 필요한 것은 크레이트 루트의 그 한 줄뿐이다.

---

## 8. 교환 후보 (결정용 목록, 권고 포함)

각 항목은 독립적이다. 방향은 `Y4 → pf` (portable-fip 이 가져감), `pf → Y4` (Y4 가 가져감) 둘이다.

| # | 방향 | 대상 | 근거 | 비용·위험 |
|---|---|---|---|---|
| A | pf → Y4 | `fip-core` 조합자로 `y4-fip-core` 조합자를 대체 (`compact`, `dedup_by`, `compact_map`, `map_in_place`, `update`, `replace`, `swap` + `strict::*`) | Verus 증명이 이미 있음 (Y4 로드맵 ⒜ 를 대신함); `hole` 이 §4-3 을 해소 | `FnMut` → `Fn`, `compact_with` → `compact_map` 모양 변경; `fip_update!`/`fip_map!` 의 place 검사 상실; 내부 의존(`fip-strict`, `fip-macros`) 이 따라옴 |
| B | pf → Y4 | `fip-arch` 로 `y4-fip-{x86_64,aarch64,riscv64}` 를 대체 | §4-4 · §4-5 를 함께 해소, aarch64 `PRFM` 구현 있음 | Y4 의 arch 별 크레이트 구조와 `bare-metal`/`fip-qemu` 게이트를 고쳐야 함; seL4 arch 선택과 cfg 선택의 관계 정리 |
| C | pf → Y4 | `fip-macros` 로 Y4 `#[fip]` 를 대체 | 실제 검사 + prelint 표식 | 등급 정의는 같음(§6-5); proc-macro 빌드 TCB 가 커짐 (2439 줄 + `fip-lint-rules`) |
| D | pf → Y4 | Y4 크레이트에 `#![cfg_attr(fip_prelint, fip::export)]` + `cfg(fip_prelint)` 선언만 | 코드 교환 없이 prelint 검사 가능 (§7) | 일반 빌드 영향 없음; Y4 `#[fip]` 가 표식을 내지 않으면 루트는 여전히 0 |
| E | Y4 → pf | `FnMut` 조합자 (검증 없는 변형으로) | 상태 있는 클로저 지원 | Verus 의 `FnMut` 명세 지원을 먼저 확인해야 함 (미측정) |
| F | Y4 → pf | place 검사 매크로 (`addr_of_mut!` 기법) | 매크로 형태 편의 | `Default` 를 쓰면 §4-3 을 다시 들여옴 → `hole` 판으로만 의미 있음 |
| G | Y4 → pf (일반화) | 사용자 선언 할당/해제 원천 (`fip::alloc_source` 등) | Y4 F1 정의와의 차이 해소 (§6-1) | portable-fip 신규 기능; Y4 이름은 넣지 않음 |

권고 순서 (참고용): G → D → A·B → C. (H "FBIP 정의 정렬" 은 정정 후 필요 없어 뺐다.) 근거는 이렇다.
- G 가 없으면 Y4 의 F1 을 prelint 로 검사할 수 없다.
- D 는 비용이 거의 없다.
- A·B 는 Y4 미결 §4-3 · §4-4 · §4-5 와 로드맵 ⒜ 를 함께 없앤다.
- C 는 등급 정의가 이미 같으므로(§6-5) 정의 조정 없이 할 수 있다. 다만 빌드 TCB 가 커지는 것을 받아들일지가 남는다.

---

## 9. 이 문서가 확인하지 않은 것

(확인한 것: portable-fip `fip-core` · `fip-arch` · `fip-alloc` · `fip-strict` 는 stable(1.99.1) `--release` 로
`x86_64-unknown-none` · `aarch64-unknown-none` · `riscv64gc-unknown-none-elf` 세 타깃 모두 빌드된다, 2026-11-01 측정.)


- Y4 `verus-fork` 로 portable-fip 증명을 다시 돌리는 것. portable-fip 은 업스트림 Verus 0.2026.10.18 로 검증했고,
  `6e6e8b1` 메시지는 verus-fork 가 0.2026.10.18 에 rebase 됐다고 한다. 호환은 측정하지 않았다.
- Verus 의 `FnMut` 클로저 명세 지원 (§8-E).
