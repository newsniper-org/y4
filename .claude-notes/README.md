<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# `.claude-notes/` — Y4 design memo + decision archive

본 디렉터리는 Y4 의 spec 검토 / sign-off cycle 도중 생성된 **design
memo + decision archive** 보관.  Claude Code 가 sign-off cycle 안에서
생성한 ledger / 비교 / 결정 흔적을 git-tracked 형태로 보존 — contribute-
back paper / 코드 리뷰 / 산업 도입 시 audit reference.

> **본 디렉터리는 `.claude-memories/` 와 별개 구조.**
> `.claude-memories/` 는 Claude Code 의 project memory 의 read-only
> mirror (`tools/git-hooks/` 의 pre-commit hook 이 자동 sync, CLAUDE.md
> §5 + §8 정합).  본 `.claude-notes/` 는 사용자 + Claude Code 가 직접
> 편집하는 design 흔적.

## Sub-directory 구조

| 경로 | 성격 | 갱신 빈도 |
|---|---|---|
| `.claude-notes/` (본 디렉터리 root) | **Design memo / decision archive** — 갱신 종료된 historical record | 결정 시점 1 회 작성, 그 후 갱신 거의 없음 |
| `.claude-notes/trackers/` | **Tracker / ledger** — 지속 갱신 파일 묶음 (CVE / 학술 논문 / venue deadline / 위협 발견 등) | 새 정보 도착 시마다 |
| `.claude-notes/_completed/` | **Completed work archive** — 종료된 work item / 갱신 종료된 tracker | 종료 시점 이동 |

## 현재 root 의 design memo 파일

| 파일 | 역할 | 채택 시점 |
|---|---|---|
| `amd-v-verified-survey.md` | AMD-V 검증 base 의 5-way 비교 (D1 seL4 / NOVA / Hyperkernel / SVSM / Atmosphere) + Atmosphere/VeriSMo 사실 확인 ledger + ARCH-II' 채택 결정 record | 2026-05-04 ARCH-II' 채택, 그 후 갱신 종료 (decision archive) |

(추가 design memo 는 sign-off cycle 안에서 발견 시 추가.)

## 정책

### git tracking
**Git-tracked** — `vmm_arch.md` §5.4 의 정책 정합.

### Tracker 와의 구분
- 본 root 의 파일들은 *결정 시점의 record* — ARCH 비교 / sub-decision
  채택 / sign-off 결과의 snapshot
- 갱신이 필요한 tracker 는 `.claude-notes/trackers/` (별도 README)

### Memo / archive 전이
design memo 의 결정이 frozen 되면 — 그대로 root 에 보관.  결정이 v2
(incompatible) 변경으로 outdated 시 — `_completed/` 로 이동 + 후속
memo 신설.
