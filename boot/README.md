<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 boot

Y4 의 부팅 chain. 첫 단계인 Phase B step 2 의 목표는 **QEMU 에서
Limine → seL4 → 'Hello, Y4'** 를 띄우는 것.

본 디렉터리는 **Limine boot config + seL4 빌드 컨피그 + cmake 호출
규칙** 을 보관한다. seL4 와 Limine 의 소스 자체는
[`/third_party/sel4`](../third_party/sel4) (`15.0.0` 핀) 와
[`/third_party/limine`](../third_party/limine) (`v12.1.0` 핀) 에 git
submodule 로 들어 있다 — fork 없음, upstream rebase 하면 즉시 따라간다.

## CMake invocation 정책: logicutils-only

Y4 는 cmake 호출을 **`lu-rule` 룰 파일**로만 표현한다. xtask /
cargo-make / CMakePresets / cmake -P / rust-script 모두 미사용
(CLAUDE.md §8). 결정 근거는 `MEMORY/y4_build_decisions.md` 의
"Phase B step 2 추가 결정" 항목.

flow:

```
just sel4-build          (boot/justfile 안의 wrapper)
    └─→ lu-rule --rulefile=sel4.rules build/sel4/x86_64-debug/kernel.elf
            --format=shell
            └─→ shell 이 expanded recipe 실행 → cmake configure + build
                    └─→ kernel.elf 산출
                        └─→ stamp record (다음 freshcheck 가 skip)
```

매트릭스 (예: x86_64 + aarch64 동시 빌드) 는 `lu-par` 가 흡수:

```
just sel4-build-matrix
    └─→ lu-rule --rulefile=sel4.rules --all '<patterns>'
            | lu-par -j {{j}} --progress
```

**원칙:** 형상이 늘면 `boot/sel4.rules` 의 룰 행을 늘리지, justfile
recipe 를 늘리지 않는다.

## 디렉터리 레이아웃

```
boot/
├── README.md               — 본 파일
├── justfile                — lu-rule + lu-par 래퍼. 각 recipe 1–3줄.
├── sel4.rules              — seL4 cmake invocation 규칙 (lu-rule 입력)
├── limine.rules            — Limine 빌드/ISO 어셈블 규칙
├── x86_64-debug.cmake      — seL4 x86_64 debug build 의 initial cache (cmake -C)
├── x86_64-release.cmake    — (Phase B 후반에 추가)
├── limine.conf             — Limine 의 boot entry config (ISO 에 동봉)
└── scripts/
    └── assemble-iso.sh     — xorriso ISO 어셈블 (limine.rules 가 호출)
```

빌드 산출물 (`build/sel4/<arch>-<mode>/`, ISO 등) 은 repo 루트의
`build/` 아래로 — `.gitignore` 가 처리.

## 빠른 사용

```sh
# 환경 점검 (cmake, ninja, gcc, nasm 등)
just deps-check

# x86_64 debug seL4 kernel.elf 빌드 (Phase B step 2 의 첫 마일스톤)
just sel4-build

# Limine bootloader 빌드 (host-side tools + BIOS/UEFI 페이로드)
just limine-build

# QEMU 에서 부팅 — Limine → seL4 → "Hello, Y4"
just qemu-boot
```

각 recipe 는 logicutils sentinel 로 hash-driven incremental.

## 현 상태 (2026-05-04)

- ✅ submodule 핀 확정 (sel4 15.0.0, limine v12.1.0)
- ✅ logicutils-only 호출 framework 골격
- ⏳ 첫 `just sel4-build` 통과 (다음 PR)
- ⏳ Limine 빌드 + ISO 어셈블 (그 다음)
- ⏳ QEMU "Hello, Y4" (그 다음)

## 비-목표 (Phase B step 2 범위 외)

- 멀티 형상 매트릭스 (Phase D 형상 분기 시작 시).
- aarch64 / RISC-V (D2 결정상 x86_64 only).
- Secure Boot 키 enrollment (Phase E 인증 트랙).
- Limine 외 부트로더 (CLAUDE.md §8 의 우선순위 표 참조).
