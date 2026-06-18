<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# `.brainstormings/` — design 탐색 기록

본 디렉터리는 sign-off cycle 전의 **brainstorming / 옵션 탐색** 기록.
결정 (`.claude-notes/`) 이나 지속 추적 (`.claude-notes/trackers/`) 과
구분되는, "아직 결정 X — 방향 탐색" 단계의 산출물.

## 성격 구분

| 종류 | 위치 | 단계 |
|---|---|---|
| **Brainstorming** (옵션 탐색) | `.brainstormings/` (본 디렉터리) | 결정 전 — 방향/tradeoff 나열 |
| **Decision archive** (결정 record) | `.claude-notes/` | 채택 후 record, 갱신 종료 |
| **Tracker / ledger** (지속 갱신) | `.claude-notes/trackers/` | 새 정보 도착 시 row 추가 |

brainstorming 이 sign-off cycle 로 승격되면 결정 부분은 `.claude-notes/`
또는 docs/ 의 design memo 로 이전, 본 기록은 historical reference 로 보존.

## 파일 명칭 convention

```
<YYYYMMDD>-<HHMMSS>-<topic-kebab>.md
```

- timestamp = **UTC+9:00 (KST)** 기준 (생성 시점)
- frontmatter 에 `created: <ISO8601 +09:00>` + `status` + `scope` + `refs`

예: `20260618-212218-alloc-frontend-improvements.md`

## git tracking

**Git-tracked** — design 흔적 보존 (`.claude-notes/` 정책 정합).
contribute-back paper / 코드 리뷰 / 산업 도입 시 audit reference.

## 기록 목록

| 파일 | topic | 생성 (KST) | status |
|---|---|---|---|
| [20260618-212218-alloc-frontend-improvements.md](20260618-212218-alloc-frontend-improvements.md) | alloc front-end 개선 8 방향 (capability allocator + small hardening + verification) | 2026-06-18 21:22 | brainstorming |
