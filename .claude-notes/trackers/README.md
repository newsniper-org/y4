<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# `.claude-notes/trackers/` — 진행 중 추적 파일

본 디렉터리는 Y4 의 spec 진행 중에 **지속 갱신** 되는 ledger / tracker
파일 묶음.  새 정보 (CVE / 학술 논문 / venue deadline / 위협 발견 등)
가 도착할 때마다 row 추가 / cell 갱신.

`.claude-notes/` 의 sibling 파일 (예: `amd-v-verified-survey.md`) 와의
구분:

| 종류 | 위치 | 성격 |
|---|---|---|
| **Tracker / ledger** (지속 갱신) | `.claude-notes/trackers/` (본 디렉터리) | 새 정보 도착 시 row 추가, status 갱신, cell 수정 |
| **Design memo / decision archive** (갱신 종료) | `.claude-notes/` | 결정 시점의 record, 채택 후 갱신 종료, historical reference |
| **Completed work archive** | `.claude-notes/_completed/` | 종료된 work item 보관 |

## 현재 / 예정 tracker 파일

(Phase C 진입 후 실제 작성 — 본 README 는 placeholder + 정책 ledger.)

| 파일 | 역할 | 신설 시점 | Cross-ref |
|---|---|---|---|
| `power-prior-art-ledger.md` | `power_arch.md` §6.7 의 prior art ledger 의 갱신 — 새 학술 논문 / CVE / 산업 도입 발견 시 row 추가 + §6.1 의 8 학술적 차별점 의 prior art 부재 주장 재평가 | Phase C 진입 후 | `power_arch.md` §6.7 / §8.6 |
| `power-paper-venue-tracker.md` | `power_arch.md` §6.4 의 venue 후보 (SOSP workshop / PLOS / IEEE S&P / SOSP main / OSDI / USENIX Security / EuroSys / ASPLOS / HOTOS) deadline 추적 + paper draft → submission timeline 정합 | Phase C 종반 paper draft 시점 | `power_arch.md` §6.4 / §8.7 |
| `power-threat-ledger.md` | `power_safety.md` §1.4 의 v1.x 위협 ledger — 새 CVE / 학술 논문 발견 시 row 추가 + §1.2 의 12 항목 catalog 갱신 + §3 의 안전장치 (S15~S23) 의 mitigation 영향 재평가 | Phase C 진입 후 | `power_safety.md` §1.4 / §7.3 |
| (추가 tracker 들 — Phase C 진입 후 발견 시 추가) | — | — | — |

## 정책

### git tracking
**Git-tracked** — `vmm_arch.md` §5.4 의 `.claude-notes/` 정책 정합.
design 흔적 보존이 contribute-back paper / 코드 리뷰 / 산업 도입 시
audit reference.

### 갱신 권한
- Y4 contributor (host operator / lease holder 의 OS 측 사용자 권한과
  무관) — git push 가능한 모든 사용자
- Claude Code 가 sign-off cycle 또는 microbench 측정 시 자동 갱신 가능

### Tracker → Archive 전이
tracker 가 갱신 종료될 시점 (예: paper 게시 후 venue tracker 갱신 종료):
1. 본 디렉터리에서 `.claude-notes/_completed/` 로 이동
2. cross-ref doc 갱신 (file path 갱신)
3. 본 README 의 표 갱신

### 파일 명칭 convention
- `<domain>-<kind>.md` 형식: `<domain>` = power / amdv / vmm / wavetensor /
  etc.  `<kind>` = ledger / tracker / survey / etc.
- 예: `power-prior-art-ledger.md` (domain=power, kind=prior-art-ledger)
