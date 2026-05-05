<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 seL4 fork 정책

Y4 가 seL4 mainline (`third_party/sel4`, 현재 핀 15.0.0) 에 패치를
얹는 fork 를 운용할 때 **반드시** 만족해야 하는 호환성 contract.

본 정책은 D1d (AMD-V 안전장치) 를 비롯해 **모든 seL4 fork 변경**에
적용.  D1d 자체의 안전장치 카탈로그는 `docs/amdv_safety.md`.

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
| **기존 verification artifact** | seL4 의 Isabelle/HOL proof 가 Y4 fork 에서도 그대로 유효 (기존 정리 invalidate 금지) |
| **기존 보안 보장** | seL4 의 capability 격리, IPC 결정성, scheduler 공정성 등 모두 유지 |

Y4 fork 의 변경은 **순수 additive** (적용 가능한 부분에 한해):
- 새 syscall 추가 — OK
- 새 cap 객체 종류 추가 — OK
- 새 build option 추가 (default OFF) — OK
- 기존 코드 path 의 행동 변경 — **금지**

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

### 새 파일

기존 SPDX 헤더 + 추가 줄:

```c
/* SPDX-License-Identifier: BSD-2-Clause */
/* Y4-fork: This file is added by the Y4 fork.
 * Copyright (c) 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors.
 * Upstream seL4 is BSD-2-Clause; this file follows the same license.
 * See docs/sel4_fork_policy.md.
 */
```

### 기존 파일 변경

각 변경 블록을 `#ifdef CONFIG_Y4_<feature>` 로 감싸고, default 는 OFF:

```c
#ifdef CONFIG_Y4_AMDV
/* Y4-fork: D1d AMD-V safety check (see docs/amdv_safety.md §S2). */
if (!intercept_floor_holds(vcpu)) {
    return seL4_InvalidArgument;
}
#endif
```

`CONFIG_Y4_*` 가 OFF 일 때 generated assembly 가 upstream 과 byte-equal
이도록 강제 (CI 측 `diff -r build/sel4-upstream build/sel4-y4-fork-off`
검증).

### Build flag 신설

새 cmake option 은 항상 `Y4_*` prefix:

```cmake
config_option(
    Y4AMDVEnabled
    Y4_AMDV
    "Enable Y4's AMD-V safety extensions (D1d)."
    DEFAULT OFF
    DEPENDS "KernelSel4ArchX86_64"
)
```

`Y4_AMDV=OFF` 일 때 빌드 결과는 upstream 과 동일.

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

옵션 (확정 결정 시 본 §갱신):

| 옵션 | 의미 |
|---|---|
| **(F1) third_party/sel4 의 Y4 branch** | `third_party/sel4/` 가 Y4 의 `y4-fork-15.0.0` 브랜치 가리킴.  upstream rebase 정책 명시 |
| **(F2) 별도 repo `y4-sel4-fork`** | Y4 가 자체 repo 운영, `third_party/sel4` 가 그것을 submodule 로 |
| **(F3) overlay patch directory** | upstream seL4 그대로 + `tools/sel4-patches/*.patch` 파일들 + 빌드 시점에 apply.  fork repo 운영 부담 0 |

(F3) 가 가장 가벼움 — 패치 자체가 git diff 형태로 트리에 보관, upstream
rebase 시 patch refresh 만.  그러나 patch 깊이가 깊어지면 (F2) 로 승급
권고.

---

## 7. 동결 정책

본 정책은 v0 draft.  `v1.0 frozen` 조건:
- §1 contract 7 줄 사용자 sign-off
- §3 패치 형식 사용자 sign-off
- §6 fork repo 형상 결정

frozen 후 변경은 v1.x patch (additive 보장 강화) 또는 v2 (정책 자체
변경 — 매우 드물 것).
