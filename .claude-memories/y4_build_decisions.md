---
name: Y4 build & dev 결정 (D1–D4)
description: D1 build system, D2 1차 타겟 arch, D3 외부 의존 통합 방식, D4 Phase B 구현 순서. 사용자가 2026-05-04 에 확정.
type: project
originSessionId: 78ff80c3-5421-425a-9e23-3da166ef2bb9
---
Phase B 진입 직전 (2026-05-04) 사용자가 비교분석 후 직접 확정한 4 개 결정.

**D1 — Build 시스템:** Cargo workspace + justfile, justfile 은
[`/home/ybi/logicutils/`](/home/ybi/logicutils) 의 `freshcheck` /
`stamp` / `lu-par` / `lu-rule` / `lu-deps` 로 incremental + DAG 빌드
orchestration.
**Why:** Rust-only IDE/cargo 통합 + logicutils 의 hash-driven freshness 가
seL4/Limine 같은 외부 build 와도 잘 정합. Bazel 의 학습 곡선/단일 진실원
부담 회피.
**How to apply:** 새 서브트리는 자체 `justfile` 가질 수 있음. top-level
justfile 이 sub-justfile 호출. logicutils 사용법은 `/home/ybi/logicutils/docs/man/*.1`
또는 `docs/agents/recipes.md` 참조.

**D2 — 1차 타겟 아키텍처:** **x86_64 only** (Phase B). aarch64 등 다른
arch 는 해당 형상(핸드헬드/SoC) 작업 시작 시 추가.
**Why:** seL4 + Limine + QEMU 가 x86_64 에서 가장 성숙. WaveTensor 참조
보드(AXAU25) 의 호스트 사이드도 x86. 첫 부팅 검증 속도 최대.
**How to apply:** 모든 빌드 타겟/CI matrix 가 x86_64 only. cross-arch
generic abstraction 은 미리 만들지 말 것 — Phase B 후반 형상 분기 시 그때
도입.

**D3 — 외부 의존 통합:** **hybrid** — non-Rust upstream (seL4, Limine,
필요 시 NetBSD rump) 은 `git submodule` 로 `third_party/`. Tock 의 Rust
crate 들은 cargo `[patch.crates-io]` + git deps 로 워크스페이스에 직접
패치.
**Why:** 두 ecosystem 의 자연스러운 메커니즘을 그대로 사용. Rust 측은
cargo 의 dependency resolver 가 일관성 보장, 비-Rust 측은 submodule 의
upstream rebase 가 쉬움.
**How to apply:** 새 외부 의존 추가 시 — Rust crate 면 `[patch.crates-io]`
또는 git dep, 그 외엔 submodule. 두 메커니즘을 한 의존에 동시 적용 금지
(혼란 방지).

**D4 — Phase B 구현 순서:** `proofs/` 빌드 하네스 → `boot/` (Limine→seL4
QEMU) → `ipc/` & `alloc/` 병렬 → `capsules/` (non-HIU) → 그 다음.
**Why:** formal-first 원칙상 proofs 하네스가 모든 후속 PR 의 전제. boot
는 신규 privileged path 가 0 (binary 합치기) 이라 proofs 직후 자연.
ipc/alloc 은 서로 독립이라 병렬 가능. capsules 는 ipc 위에 얹힘.
**How to apply:** PR 머지 순서를 본 순서로 강제. 한 단계 끝나기 전에
다음 단계의 코드 PR 을 만들지 말 것 (명세 PR 은 가능). HIU/lease 런타임은
`docs/hiu_abi.md` 가 v1.0 frozen 될 때까지 차단.

## Phase B step 2 (boot) 추가 결정 (2026-05-04)

**seL4 버전 핀:** **15.x stable**. 가장 최근 안정 시리즈.
**Limine 버전 핀:** **12.x stable**.
**CMake invocation wrapping:** **logicutils-only**. xtask / cargo-make /
CMakePresets / cmake -P / rust-script 모두 제외.
**Why:** D1 결정("Cargo workspace + justfile + logicutils") 과의 정합성
최우선. 추가 도구 stack 0. 형상별 `-D` flag matrix 는 `boot/<sub>.rules`
(`lu-rule` consumable) + `lu-par` 가 표현. freshness 는 `freshcheck`/
`stamp` (이미 ci 게이트에서 검증된 패턴).
**How to apply:** boot/ 의 cmake 호출은 정확히 `lu-rule --rulefile=boot/sel4.rules <target>`
형태로만. justfile recipe 는 그 한 줄과 logicutils 산출물 후처리만.
형상 matrix 가 늘면 룰 파일 행을 늘리지, justfile recipe 를 늘리지 말 것.
표현력 한계가 실측되면(예: 5+ 형상 × 2 build mode) 그때 재평가.
