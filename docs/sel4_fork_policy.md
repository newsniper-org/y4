<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 seL4 fork 정책

> **상태:** **v1.0 frozen** (2026-05-05, Phase 4 일괄 마킹).
> Strictly Additive Fork contract §1 (6 row 보장 + 4 원칙 + §1.1
> 라이선스 정합) + 회귀 게이트 §2 (G1~G5) + 패치 형식 §3 (6 sub-
> section, dual SPDX + byte-equal 결정성 + master flag + `*.y4-modified.bf`
> + Verus 짝 PR + G6 timing-equal optional) + Y4 fork repo 형상 §6
> ((F3) overlay patch directory + 8 sub-section) 모두 sign-off.  짝
> frozen doc = amdv_safety.md / vmm_arch.md / verus_to_isabelle.md.

Y4 가 seL4 mainline (`third_party/sel4`, 현재 핀 15.0.0) 에 패치를
얹는 fork 를 운용할 때 **반드시** 만족해야 하는 호환성 contract.

본 정책은 **D1a (AMD-V raw-SVM C 패치)** 를 비롯해 **모든 seL4 fork
변경**에 적용.  AMD-V 안전장치 catalog 는 `docs/amdv_safety.md`,
ARCH-II' (capsule-decomposed VMM) 디자인은 `docs/vmm_arch.md`.

> **적용 범위:** 본 정책은 **seL4 microkernel 측 패치에만 적용**.
> Y4 측 capsule cluster (`Y4/capsules/vmm-*` + `Y4/vmrun-orchestrator/`)
> 는 별도 codebase 로 본 정책 외부.

---

## 1. 핵심 contract — Strictly Additive Fork

**원본 seL4 에서 작동하는 모든 것은 Y4 fork 에서도 변경 없이 동일하게
작동해야 한다.**

이는 다음을 의미한다:

| 영역 | 보장 |
|---|---|
| **기존 syscall** | signature, ABI, 의미론 변경 금지.  뺄셈/리네임/대체 모두 금지 |
| **기존 cap 객체 종류** | layout, 권한, lifecycle 변경 금지 |
| **기존 build option** | 같은 flag 가 같은 동작.  default 변경 금지 |
| **기존 user program (root task / capsule)** | 재컴파일 / 헤더 변경 / 링크 변경 없이 동일 binary 가 동일하게 동작 |
| **기존 verification artifact** | seL4 의 Isabelle/HOL proof 가 Y4 fork 에서도 그대로 유효 (기존 정리 invalidate 금지). **Y4 측 신규 Verus 명세 (AV1~AV20) 는 seL4 의 Isabelle/HOL proof 와 독립** — 새 cap 종류 / 새 syscall 의 invariant 만 다루며, 기존 seL4 정리에 의존 X (cross-tool 의존이 contribute-back 진입 장벽 ↑) |
| **기존 보안 보장** | seL4 의 capability 격리, IPC 결정성, scheduler 공정성 등 모두 유지.  **raw-SVM cap (SVMVCPU / SVMNPT / SVMMsrBitmap / SVMIoBitmap) 도 기존 cap-typing 강제와 동일 격리 — 새 cap 종류는 기존 cap 격리 invariant 의 sub-case 로 흡수** |

Y4 fork 의 변경은 **순수 additive** (4 원칙):
1. 새 syscall 추가 — OK
2. 새 cap 객체 종류 추가 — OK
3. 새 build option 추가 (default OFF) — OK
4. 기존 코드 path 의 행동 변경 — **금지**

### 1.1 라이선스 정합

D1a 패치 자체는 **BSD-2-Clause** 로 contribute-back (seL4 mainline 의
라이선스 보존).  SPDX 헤더 dual line:

```c
// SPDX-License-Identifier: BSD-2-Clause
// SPDX-FileCopyrightText: 2014, General Dynamics C4 Systems  // upstream
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors  // Y4 추가
```

Y4 측 capsule cluster (`Y4/capsules/vmm-*`, `Y4/vmrun-orchestrator/`) 는
**Apache-2.0** 그대로 — `CONTRIBUTING.md` §3 의 ported-code SPDX 규칙
정합.

---

## 2. 회귀 검증 (regression gate)

모든 Y4 fork 패치 PR 은 다음 게이트를 통과해야 머지:

| 게이트 | 의미 |
|---|---|
| **G1 — sel4test 통과** | upstream seL4 의 sel4test 회귀 스위트 (`https://github.com/seL4/sel4test`) 가 Y4 fork 에서 0 fail.  CI 에서 매 PR 자동 실행 |
| **G2 — seL4 unit test** | upstream 의 cocotb/cmake 측 unit test 가 0 fail |
| **G3 — Y4 의 기존 root task** | y4-roottask (Phase B step 5 의 "Hello, Y4") 가 변경 없이 동일하게 부팅 |
| **G4 — Verus 명세 무영향** | Y4 측 `proofs/verus/` 의 50+ invariant 가 모두 변경 없이 verified 유지.  새 invariant 추가만 허용 |
| **G5 — diff audit** | seL4 fork 의 diff 가 새 파일 추가 또는 명확히 표시된 `#ifdef CONFIG_Y4_*` 영역만 변경.  기존 함수 시그니처 / 데이터 구조 / control flow 의 변경 0 |

CI 스크립트: `tools/sel4-fork-check.sh` (Phase C 진입 시 추가).

---

## 3. 패치 형식 표준

Y4 fork 의 모든 변경은 다음 표시:

### 3.1 새 파일

기존 SPDX 헤더 + dual SPDX-FileCopyrightText (B = §1.1 정합):

```c
/* SPDX-License-Identifier: BSD-2-Clause */
/* SPDX-FileCopyrightText: 2014, General Dynamics C4 Systems */          /* upstream seL4 보존 */
/* SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors */
/* Y4-fork: This file is added by the Y4 fork.
 * Upstream seL4 is BSD-2-Clause; this file follows the same license.
 * See docs/sel4_fork_policy.md.
 */
```

`SPDX-License-Identifier` 는 BSD-2-Clause **single** (license 동일).
SPDX-FileCopyrightText 만 dual line — upstream attribution 보존.

### 3.2 기존 파일 변경

각 변경 블록을 `#ifdef CONFIG_Y4_<feature>` 로 감싸고, default 는 OFF:

```c
#ifdef CONFIG_Y4_AMDV
/* Y4-fork: D1a AMD-V safety check (see docs/amdv_safety.md §S2). */
if (!intercept_floor_holds(vcpu)) {
    return seL4_InvalidArgument;
}
#endif
```

`CONFIG_Y4_*` 가 OFF 일 때 generated assembly 가 upstream 과 **byte-equal**.

#### 3.2.1 byte-equal 의 결정성 강제 (D)

reproducible build 강제로 nondeterminism (timestamp, `__FILE__` macro 등)
제거:

```cmake
# Y4 fork 의 모든 빌드에 적용
add_compile_definitions(
    SOURCE_DATE_EPOCH=$ENV{SOURCE_DATE_EPOCH}
)
add_compile_options(
    -frandom-seed=$ENV{SOURCE_DATE_EPOCH}
    -Wno-builtin-macro-redefined
)
```

추가 검증: `__FILE__` 매크로 의존 코드는 Y4 fork 측에서 새로 도입 X
(grep `__FILE__` 의 새 사용처 0 보장 — CI 측 G5 diff audit 가 검사).

CI 의 byte-equal 게이트:

```sh
diff -r build/sel4-upstream build/sel4-y4-fork-off
# CONFIG_Y4_AMDV=OFF 빌드의 모든 .o / .elf 파일이 upstream 과 byte-equal
```

### 3.3 Build flag 신설

새 cmake option 은 항상 `Y4_*` prefix.  **`CONFIG_Y4_AMDV` 는 master
flag — sub-feature 분리 X (C):**

```cmake
config_option(
    Y4AMDVEnabled
    Y4_AMDV
    "Enable Y4's AMD-V safety extensions (D1a + ARCH-II' capsule cluster)."
    DEFAULT OFF
    DEPENDS "KernelSel4ArchX86_64"
)
```

`Y4_AMDV=OFF` 일 때 빌드 결과는 upstream 과 동일.

근거: capsule cluster 가 모든 4 cap 종류 (SVMVCPU/SVMNPT/SVMMsrBitmap/
SVMIoBitmap) + 6 syscall (Configure/Run/Migrate/ChangeParent/RebaseTsc/
PollNestedRequest) 에 의존 — 부분 enable 의 의미 0.  단일 master flag
가 capsule cluster 의 atomic 진입점.

### 3.4 새 cap 객체 종류 / generated table extension (E = a')

`#ifdef CONFIG_Y4_AMDV` block 안에서 generated table 확장.  bitfield
정의 파일 변경은 **`*.y4-modified.bf` extension file** 로 — 기존 `*.bf`
파일 변경 0:

```
src/api/objecttype.bf            (upstream, 변경 0)
src/api/objecttype.y4-modified.bf  (Y4 추가 — SVMVCPU / SVMNPT / SVMMsrBitmap / SVMIoBitmap)
```

cmake 측에서 `CONFIG_Y4_AMDV=ON` 일 때만 `*.y4-modified.bf` 를 generator
input 에 포함.  generated 산출물 (`objecttype.h` 등) 의 conditional
include block 도 `#ifdef CONFIG_Y4_AMDV` 로 감쌈.

cap_get_capType / capability lookup table 같은 generated header 도
동일 패턴: 기존 generator script 변경 0, Y4 측이 `*.y4-modified.bf`
input file 만 추가.

### 3.5 Y4 측 Verus 명세 짝 PR (F)

각 새 syscall / cap 종류 도입 시 같은 PR 안에 Y4 측 `proofs/verus/src/
amdv/` 의 대응 invariant **statement** 추가 (AV1~AV20 catalog,
`amdv_safety.md` §5).

PR 의존:
- **seL4 mainline PR-1** 자체에는 Verus artifact 미포함 — Apache-2.0
  / Z3 (LGPL/Apache mixed) 의존성을 mainline 에 도입 X
- **Y4 PR-3** (Verus 명세 + paper artifact) 가 짝 PR — PR-1 의 새
  syscall 마다 invariant statement 1:1 대응

CONTRIBUTING.md §5 의 "신규 privileged path 는 Verus 명세 동반" 강제와
정합.

### 3.6 Negative test — timing-equal (G6, optional in v1.0)

byte-equal (G5) 외 추가 negative test 후보:

| 게이트 | 조건 | 강제 |
|---|---|---|
| **G6 — timing-equal** | `KernelDebugBuild=ON` 의 timing trace (sel4bench 또는 등가) 가 upstream 과 ±5% 안.  `CONFIG_Y4_AMDV=OFF` 일 때 모든 syscall latency 의 분포 동일성 | **v1.0 optional**.  Phase C 종반 microbench 후 v1.x patch 에서 **강제** |

근거: byte-equal 은 instruction-level 동일성 보장하지만 cache/branch
predictor 영향 측정 X.  side-channel 보존을 위해 timing-equal 도 강제
필요.  단 v1.0 시점에서는 microbench 자료 부재 → optional.

---

## 4. Contribute-back 의 분리

Y4 fork 의 패치 중:

| 카테고리 | mainline contribute |
|---|---|
| **새 syscall + cap 종류** (예: D1d 의 SVMVCPU/SVMNPT 등) | ◎ — `Y4_*` prefix 제거 + cmake option 이름 변경 (예: `KernelSVM`) 후 PR.  isabelle proof 와 함께 — 단 isabelle 트랙 완료 책임은 seL4 팀과 협의 |
| **Y4-VMM 의 Verus 증명** | △ — 별도 artifact (논문 + Y4 repo 링크).  seL4 팀이 isabelle 으로 재증명할지 별개 |
| **CONFIG_Y4_\* gate 자체** | ✗ — mainline PR 에서는 conditional 제거, 무조건 활성화 (또는 cmake option 으로) |

Strictly Additive 원칙 덕에 mainline PR 도 자연스럽게 additive — 같은
diff 를 `Y4_*` prefix 만 떼고 제출 가능.

---

## 5. Forbidden 변경 (절대 금지)

Y4 fork 가 절대 하지 않는 것:

1. 기존 syscall signature 변경 (인자 추가/제거/타입 변경)
2. 기존 cap 종류의 권한 비트 의미 변경
3. 기존 build option 의 default 변경
4. 기존 verification 정리의 가정/결론 변경
5. seL4 의 boot path 핵심 (page table 셋업, root cnode 초기화) 변경
6. seL4 의 IPC fast path / slow path 결정 로직 변경
7. seL4 의 scheduler tick 주기 변경
8. 기존 `CONFIG_*` (Y4 prefix 없는 것) 추가 — 모든 신규는 `Y4_*` prefix

위 8 항목 중 어느 하나라도 PR 에 들어오면 G5 게이트가 자동 reject.

---

## 6. Y4 fork repo 형상

### 6.1 채택 — (F3) overlay patch directory

| 옵션 | 의미 | 채택 |
|---|---|:---:|
| (F1) `third_party/sel4` 의 Y4 branch | `third_party/sel4/` 가 Y4 의 `y4-fork-15.0.0` 브랜치 가리킴 | ✗ |
| (F2) 별도 repo `y4-sel4-fork` | Y4 자체 repo 운영, `third_party/sel4` 가 그것을 submodule 로 | ✗ (승급 후보) |
| **(F3) overlay patch directory** | upstream seL4 그대로 + `third_party/sel4-patches/*.patch` 파일들 + 빌드 시점에 apply.  fork repo 운영 부담 0 | **◎** |

**근거 (2026-05-04 결정):** 패치 분량이 시작 시점 ~수백 LoC C + 2 generated
bf 파일 — (F3) 가 가장 가볍고 upstream rebase 시 patch refresh 만으로
충분.  patch 깊이 임계 도달 시 (F2) 로 승급 (§6.7).

### 6.2 patch directory 위치

**`third_party/sel4-patches/`** — `third_party/sel4` (submodule) 의
sibling.  patch 와 upstream pin 의 짝이 한 디렉터리에 노출.

```
Y4/
├── third_party/
│   ├── sel4/                         # submodule, upstream 그대로
│   ├── sel4-patches/                 # Y4 fork patch directory
│   │   ├── 001-cap-types-svm.patch
│   │   ├── 002-syscall-vcpu-configure.patch
│   │   ├── ...
│   │   └── README.md                 # patch 정렬 + apply order 설명
│   ├── limine/                       # 기존
│   └── scudo/                        # 기존
```

### 6.3 Patch 파일 명칭 / 정렬

**`NNN-<topic>.patch` 형식** — sequence 정렬 + topic prefix:

```
001-cap-types-svm.patch              # SVMVCPU/SVMNPT/SVMMsrBitmap/SVMIoBitmap
002-syscall-vcpu-configure.patch     # Configure / Run
003-syscall-vcpu-migrate.patch       # Migrate / ChangeParent
004-syscall-vcpu-rebase-tsc.patch    # RebaseTsc
005-syscall-poll-nested-request.patch
006-vmrun-wrapper-7-step.patch       # S7 atomic sequence
007-mandatory-mask-check.patch       # S2 16-bit
008-svmpt-cap-derived.patch          # S3 cap derivation
...
```

apply order = sequence number 정순.  patch 추가/제거 시 번호 재정렬:
- 추가: 마지막 번호 + 1 사용
- 제거: 빈 번호 그대로 유지 (압축 X — git history 의 reference 보존)
- 압축은 v2 (incompatible) 단계에서만

### 6.4 Patch apply 메커니즘

**`git am --3way` + `tools/sel4-fork-apply.sh` script.**

```sh
#!/bin/bash
# tools/sel4-fork-apply.sh
set -euo pipefail
cd third_party/sel4
git checkout "$(cat ../sel4-pin.txt)"     # upstream pin
for p in ../sel4-patches/[0-9]*.patch; do
    git am --3way --keep-cr "$p"
done
```

근거:
- `git am --3way` 의 3-way merge 가 upstream rebase 시 conflict 자동
  해결 (대부분의 conflict)
- sign-off chain 보존 (git format-patch 형식의 patch metadata)
- patch 자체는 git 의 이메일 형식 (Subject / Author / Date 메타데이터
  보존)

### 6.5 Upstream rebase 정책

**분기별 (3 개월) upstream tag 갱신 + patch refresh.**

```
Q1, Q2, Q3, Q4 의 첫 평일에 자동 trigger (CI cron):
1. upstream sel4 의 latest stable tag 확인
2. third_party/sel4-pin.txt 갱신
3. tools/sel4-fork-apply.sh 자동 실행
4. CI green (G1-G5 + G6 optional) 시 자동 머지 PR 생성
5. 자동 refresh fail 시 maintainer review (사용자 호출)
```

근거: 정기 cycle + CI 자동화 + maintainer 부담 ↓.  hot fix (security
CVE) 는 ad-hoc 추가.

### 6.6 CI 측 patch 검증 (sel4-fork-check.sh 의 일부)

```sh
# tools/sel4-fork-check.sh 의 patch 검증 단계
1. 모든 patch 가 third_party/sel4-pin.txt 의 upstream 에 clean apply
   (git am --3way --check)
2. apply 후 G1 (sel4test) / G2 (unit test) / G3 (y4-roottask boot) /
   G4 (Verus 50+ verified) / G5 (diff audit byte-equal) 통과
3. *.y4-modified.bf 파일 존재 시 generator 가 정상 동작 (cmake build
   의 generated header 가 expected layout)
```

### 6.7 (F2) 로 승급 임계

다음 임계 중 하나라도 도달 시 (F2) 별도 repo 로 승급:

| 임계 | 값 |
|---|---|
| Patch 분량 | **≥ 5000 LoC C** (모든 patch 합산) |
| Patch 파일 수 | **≥ 30 개** |
| Upstream rebase 자동 refresh 실패율 | **≥ 30%** (직전 4 분기 평균) |

승급 시점에 별도 repo `y4-sel4-fork` 신설 + Y4 의 third_party 가
submodule 로 전환.  patch directory 는 archive 보존 (drift detection
용).

### 6.8 Mainline contribute-back 시 patch 변환

`tools/sel4-mainline-export.sh` automation 으로 patch directory →
mainline PR multi-commit series 자동 변환:

```sh
#!/bin/bash
# tools/sel4-mainline-export.sh
set -euo pipefail
output_dir="${1:?usage: $0 <output_dir>}"
mkdir -p "$output_dir"
for p in third_party/sel4-patches/[0-9]*.patch; do
    out="$output_dir/$(basename "$p")"
    sed -e 's/Y4_AMDV/SVM/g' \
        -e 's/Y4AMDVEnabled/KernelSVM/g' \
        -e 's|/\* Y4-fork: |/* |g' \
        "$p" > "$out"
done
```

contribute-back 진입 장벽 ↓ — patch directory 가 그대로 mainline PR
의 source.

> §4 contribute-back 의 분리 정합: 본 automation 산출물 = §4 표의
> "새 syscall + cap 종류" 카테고리.  isabelle proof 와의 짝은 PR-4
> (`y4-verus2isabelle` 산출물) 가 담당.

---

## 7. 동결 정책

본 정책은 v0 draft.  `v1.0 frozen` 조건:
- **§1 Strictly Additive Fork contract** 사용자 sign-off — 6 row 보장
  표 (기존 syscall / cap / build option / user program / verification
  artifact / 보안 보장) + 4 원칙 (새 syscall / 새 cap / 새 build option
  default OFF / 기존 path 변경 금지) + §1.1 라이선스 정합 (D1a BSD-2 +
  Y4 capsule Apache-2.0)
- **§3 패치 형식 표준** 사용자 sign-off
- **§6 fork repo 형상** 결정

frozen 후 변경은 v1.x patch (additive 보장 강화) 또는 v2 (정책 자체
변경 — 매우 드물 것).

`tools/sel4-fork-check.sh` (G1-G5 자동화 CI 스크립트) 는 본 doc v1.0
frozen 직후 작성 — Phase C 진입 차단 7 단계 (`phase_plan.md`) 중 일부:

```
본 doc v1.0 frozen
  → tools/sel4-fork-check.sh 작성 (G1 sel4test, G2 unit test,
    G3 y4-roottask boot, G4 Verus 50+ verified, G5 diff audit)
  → CI green
  → PR-1 (D1a raw-SVM C 패치) 진입
```
