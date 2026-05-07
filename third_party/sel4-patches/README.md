<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 seL4 fork — overlay patch directory

본 디렉터리는 Y4 의 **seL4 fork patch series**.  upstream seL4
(`third_party/sel4/`, pin = `../sel4-pin.txt` 의 commit hash) 위에
overlay merge 형태로 apply.

> **정책 source:** `docs/sel4_fork_policy.md` v1.0 frozen (2026-05-05).
> Strictly Additive Fork — 새 syscall / cap / `CONFIG_Y4_*` build
> option 만 추가, 기존 코드 path 변경 0.  G1~G7 회귀 게이트
> (`tools/sel4-fork-check.sh`) 통과 의무.

## Apply / check / export

```sh
# Apply patch series 위에 sel4 submodule 의 working tree 갱신
just sel4-fork-apply

# G1~G7 회귀 게이트 검증
just sel4-fork-check

# Mainline submission format 변환 (Y4_AMDV → SVM rename)
just sel4-mainline-export <output_dir>
```

자세한 mechanism = `docs/sel4_fork_policy.md` §6.4 (apply) / §6.6 (check)
/ §6.8 (mainline-export).

## Patch 명칭 convention (sel4_fork_policy §6.3)

`NNN-<topic>.patch` 형식 — sequence 정렬 + topic prefix.  apply order
= sequence number 정순.

## Current patch series

(현재 비어 있음 — PR-1 의 P1.1 부터 patch 작성 시작.)

향후 작성 예정 (sel4_fork_policy §6.3 예시):

| # | Topic | Safety / 결정 | 분량 추정 |
|---|---|---|---|
| 001 | cap-types-svm | S1 (4 cap types: SVMVCPU / SVMNPT / SVMMsrBitmap / SVMIoBitmap) | ~수십 LoC + `*.y4-modified.bf` |
| 002 | syscall-vcpu-configure | Configure / Run syscall | ~100 LoC C |
| 003 | syscall-vcpu-migrate | Migrate / ChangeParent (S5.2 / S6.3) | ~100 LoC C |
| 004 | syscall-vcpu-rebase-tsc | RebaseTsc (S8.2) | ~50 LoC C |
| 005 | syscall-poll-nested-request | PollNestedRequest (S9.4 v1.0 stub) | ~50 LoC C |
| 006 | vmrun-wrapper-7-step | S7.1 atomic sequence (clgi/stgi/vmsave/vmload/vmrun) | ~100 LoC C |
| 007 | mandatory-mask-check | S2 16-bit mandatory mask 검증 | ~50 LoC C |
| 008 | svmpt-cap-derived | S3 cap derivation + S3.2 huge-page + S3.3 entry 상한 | ~100 LoC C |

추가 patch (sub-decision 별):
- `*-smt-grouping.patch` (S5.1 / S5.3)
- `*-thread-group-pin.patch` (S6 group pin + S6.1 lifecycle hook)
- `*-tsc-scaling-disable.patch` (S8.3)
- `*-msr-bitmap-create.patch` (S10.2/3 profile + custom whitelist)
- `*-io-bitmap-create.patch` (S11)
- `*-svm-exit-info.patch` (S12 SVMExitInfo struct)
- `*-cap-revoke-hook.patch` (S13 lifecycle cap revoke chain)
- `*-firmware-trap-path.patch` (S14 MSR/SMI/CPUID trap path)
- `100-power-*.patch` (D1a' power MSR/ACPI/SMI mediation, power_safety §5.2)

## 명칭 정책 (sel4_fork_policy §6.3 정합)

- 추가: 마지막 번호 + 1 사용
- 제거: 빈 번호 그대로 유지 (압축 X — git history reference 보존)
- 압축: v2 (incompatible) 단계에서만

## Vendor-neutrality (cpu_virt_compat §5)

`Y4_AMDV` master flag 가 vendor-neutral — `KernelSVM` (Y4 신설) +
`KernelVTX` (mainline 기존) 둘 다 enable, runtime vendor 자동 감지.
Intel VT-x 측 patch 는 v1.x patch 단계 (cpu_virt_compat §6).
