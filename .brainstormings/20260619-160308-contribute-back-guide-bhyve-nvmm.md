<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Contribute-back 가이드 — Y4 의 virt 구현체 → bhyve / NVMM 역방향 기여
created: 2026-06-19T16:03:08+09:00   # KST (UTC+9)
status: guide (대비용 — 실제 contribute 시 docs/ 승격).  구현 전 준비
scope: 경로 B (spec clean-room, cross-platform §3.2) 로 구현한 Y4 의 virt
       backend (SVM + 다른 arch) 를 차후 bhyve / NVMM 에 contribute-back
       할 때의 license / 언어 / 절차 가이드
refs:
  - .brainstormings/20260618-231936-cross-platform-strategy.md §3.2 (경로 B)
  - docs/vmm_arch.md §1.1 (bhyve/NVMM = AMD-V reference, BSD-2)
  - docs/licensing.md (Y4 Apache-2 single-license)
  - CLAUDE.md §3 (single-license Apache-2.0) / §6.6 (formal-first)
  - ~/y4-upstream-refs/{bhyve,nvmm}/ (확보 2026-06-19)
---

# Contribute-back 가이드 — Y4 → bhyve / NVMM

## 0. 목적

기존 흐름은 **bhyve/NVMM (BSD-2) → Y4 가 algorithm port** (가져오기).
본 가이드는 **역방향** — Y4 가 경로 B (spec clean-room) 로 구현한 virt
backend (특히 bhyve/NVMM 에 *없는* arch, 또는 Verus 가 발견한 개선) 를
**bhyve/NVMM 에 contribute-back** (돌려주기) 할 때의 대비.

핵심 메시지 3가지:
1. **clean-room (경로 B) 으로 구현하면 contribute-back 이 쉬워진다** —
   Y4 독립 저작물이라 license 자유 (§3).
2. **가장 가치 있는 기여 = Verus 가 발견한 spec 모호성 / soundness** —
   코드/license 무관, 항상 가능 (§2.B).
3. **Rust → C 는 algorithm/spec transfer** 로 (코드 직접 아님, §4).

## 1. 왜 contribute-back

- **상호주의** — Y4 가 bhyve/NVMM 의 BSD-2 알고리즘을 가져왔으니
  (vmm_arch §1.1), 개선/신규 arch 를 돌려주는 게 BSD 생태계 예의 +
  Y4 의 학술적 가시성 (paper §6 contribute-back venue 와 정합).
- **유지보수 분산** — Y4 단독 유지보다 upstream 병합 시 공동 유지.
- **검증의 역수출** — Y4 의 Verus-verified 구현이 발견한 것이
  bhyve/NVMM 의 C 구현 개선 (§2.B).

## 2. 무엇이 contribute-back 가치

### 2.A bhyve/NVMM 에 없는 arch virt
- **POWER virt** — bhyve/nvmm 둘 다 powerpc vmm 부재 (cross-platform
  §3.1).  Y4 가 경로 B (PAPR clean-room) 로 구현 → bhyve 에 `sys/powerpc/
  vmm/` 신규 기여 가치 ↑↑.
- **ARCv3 virt** — 마찬가지 부재.
- (단 NVMM 은 x86 only 설계라 arch 추가가 구조적으로 큼 — bhyve 가
  multi-arch 라 더 수용적)

### 2.B Verus 가 발견한 spec 모호성 / soundness (★ 최고 가치)
- Y4 가 spec clean-room + Verus 로 구현하며 발견한 것:
  - spec (AMD APM / ARM ARM / RISC-V H / PAPR) 의 모호한 corner case
  - soundness 이슈 (특정 intercept / nested page 조합의 미정의 동작)
  - bhyve/NVMM 의 C 구현에 *잠재* 한 동일 버그 (formal 이 찾은 것)
- **코드/license 무관** — bug report / spec clarification 으로 기여
  (FreeBSD Bugzilla / NetBSD PR).  Y4 의 formal-first 가 주는 고유 가치
  — 누구도 production VMM 을 formal verify 안 했으니 (real-time §, paper
  §6.1 차별점) Y4 만 발견 가능한 것.

### 2.C spec 해석 개선 / 신규 hardware 지원
- 신규 CPU (Zen6 / ARMv9.x / RVA23) 의 virt feature 를 Y4 가 먼저
  spec 기반 구현 → upstream 기여.

## 3. license 장벽 + 해법 (★ clean-room 이 핵심)

### 3.A 장벽
- **Y4 = Apache-2.0** (single-license, CLAUDE.md §3) / **bhyve = BSD-2,
  NVMM = BSD-2**.
- Apache-2 는 BSD-2 의 *superset 제약* (patent grant §3 + NOTICE §4).
  BSD 프로젝트가 Apache-2 코드 직접 병합 = license 혼합 → **upstream 이
  꺼림** (BSD 단일성 선호).

### 3.B 해법 — clean-room 구현의 license 자유 (경로 B 의 결정적 이점)
- **clean-room (경로 B) 으로 구현하면 bhyve derivative 아님** → Y4
  독립 저작물 → Y4 가 license 자유롭게 설정 가능:
  - (a) **dual-license** 해당 모듈: `Apache-2.0 OR BSD-2-Clause` —
    bhyve/NVMM 은 BSD-2 갈래로 병합 (adsmt 의 triple-license 선례 정합)
  - (b) **BSD-2 로 별도 release** 그 contribution 만 (Y4 본체는
    Apache-2 유지, 기여분만 BSD-2)
- **대조 — algorithm port (bhyve 봄, derivative) 는 어려움**: bhyve
  코드를 보고 port 한 부분은 BSD-2 파생 → Y4 안에서 이미 BSD-2
  attribution.  이건 원래 bhyve 거라 contribute-back 의미 적음 (이미
  거기 있음).  *신규 가치* 는 clean-room 부분.
- **결론**: **contribute-back 의도가 있으면 그 모듈은 clean-room (경로
  B) 으로 구현하라** — license 자유 + 신규성 + Verus 정합.  bhyve port
  (경로 derivative) 는 contribute-back 부적합 (이미 upstream 것).

### 3.C patent grant 주의
- Apache-2 §3 patent grant 를 BSD-2 로 떨굴 때 — Y4 (윤병익) 가 해당
  contribution 의 patent 권리 보유 시 명시적 grant 필요 (BSD-2 는 patent
  조항 없음 → 별도 grant 또는 dual-license 의 Apache 갈래 유지 권장).

## 4. 언어 장벽 (Rust → C)

- Y4 = Rust / bhyve·NVMM = C.  **코드 직접 transfer 불가**.
- **방법 A — algorithm / spec transfer** (권장): Y4 의 **Verus spec +
  설계 문서** 를 제공 → upstream C 구현자가 그 spec 보고 자체 C 구현.
  Y4 의 invariant 가 *정확한 명세* 라 C 구현의 정합 기준 제공.  코드
  아닌 knowledge transfer → license 부담 최소 (단 spec 자체 license).
- **방법 B — C reference impl 동반**: Y4 가 contribute-back 모듈의 C
  포팅을 직접 작성 (Rust→C 수동) + BSD-2.  비용 ↑ 단 upstream 즉시 병합.
- **방법 C — bug report only** (§2.B): Verus 발견물은 코드 아예 없이
  spec clarification / PR 텍스트.

## 5. clean-room 양방향성 (정리)

| Y4 구현 방식 | 저작권 | license 자유 | contribute-back |
|---|---|---|---|
| **경로 B clean-room** (spec 만, bhyve 안 봄) | Y4 독립 | ✅ 자유 (dual/BSD-2 가능) | **쉬움** (신규 + license 자유) |
| algorithm port (bhyve 봄) | BSD-2 derivative | BSD-2 attribution 고정 | 부적합 (이미 upstream) |

→ **clean-room 우선 원칙** (cross-platform §3.2 경로 B) 이 contribute-
back 까지 고려하면 더 강해짐.  bhyve reference 는 *학습/검증 대조용*,
실제 구현은 spec clean-room → license 독립.

> **clean-room 절차 권장** (derivative 오염 방지): bhyve/NVMM 코드를
> *직접 본 사람* 과 *clean-room 구현자* 분리 (또는 시점 분리 + 명확한
> "spec 기반" 기록).  Y4 는 ~/y4-upstream-refs/ 를 *대조 검증* (구현 후
> 동작 비교) 용으로만 쓰고, *구현 중* 에는 spec 만 참조 → clean-room
> 보존.  단 1인 개발 (윤병익) 맥락에선 "spec 기반 구현 + 사후 대조"
> 기록으로 충분 (엄격한 clean-room team 분리는 대규모 분쟁 대비, 현
> 단계 과함).

## 6. upstream 절차

| 대상 | 절차 |
|---|---|
| **bhyve (FreeBSD)** | FreeBSD Phabricator (`reviews.freebsd.org`) 리뷰 + Bugzilla.  committer mentor 경유 또는 직접 review request.  `sys/<arch>/vmm/` 신규 = arch maintainer 협의 |
| **NVMM (NetBSD)** | NetBSD GNATS PR (`send-pr`) 또는 GitHub mirror PR + `tech-kern@` mailing list 논의.  NVMM maintainer (Maxime Villard) 협의 |
| **공통** | BSD 스타일 (style(9) / KNF) 준수, mailing list 사전 논의, 작은 단위 분할 |

## 7. DCO / attribution

- Y4 측: DCO sign-off (CONTRIBUTING.md §1) — contribute-back 도 동일.
- upstream 측 attribution: bhyve/NVMM 에 Y4 (윤병익) 기여 명시 + Y4 의
  NOTICE 에 "역방향 기여" 기록 (상호 attribution).
- Verus 발견 bug report: Y4 + 발견 방법 (Verus formal verification) 명시
  — formal-first 의 가시성 (paper).

## 8. 체크리스트 (contribute-back 시)

- [ ] 그 모듈이 **clean-room (경로 B)** 인가? (algorithm port = 부적합)
- [ ] license 갈래 결정: dual (`Apache-2 OR BSD-2`) vs BSD-2 별도
- [ ] patent grant 처리 (§3.C)
- [ ] Rust→C: algorithm/spec transfer (A) vs C reference impl (B) vs
      bug report (C)
- [ ] Verus spec / 설계 문서 동반 (C 구현 정합 기준)
- [ ] upstream 절차 (Phabricator / NetBSD PR) + mailing list 사전 논의
- [ ] DCO + 상호 attribution (Y4 NOTICE 갱신)
- [ ] clean-room 기록 (spec 기반 구현 증빙)

## 9. 결정 / 미결

### 방향 (잠정)
- contribute-back 대상 = clean-room (경로 B) 모듈 + Verus 발견물
- license = dual-license (`Apache-2 OR BSD-2`) 갈래 (adsmt triple 선례)
- 우선 기여 후보: POWER/ARC virt (bhyve 부재) + Verus soundness 발견

### 미결
- **dual-license 정책** — Y4 single-license (Apache-2, CLAUDE.md §3) 와
  "contribute-back 모듈만 dual" 의 정합.  licensing.md 에 예외 조항?
  (현 single-license 원칙의 좁은 예외 — contribute-back 모듈 한정)
- **clean-room 엄격도** — 1인 개발 맥락의 "spec 기반 + 사후 대조" 기록이
  법적 충분한지 (대규모 분쟁 대비 vs 현 단계 실용)
- **C reference impl 부담** — Rust→C 수동 포팅을 Y4 가 감당 vs spec
  transfer 만 (upstream 자체 구현)
- **시점** — Phase C/D 의 virt 구현 완료 + paper 게시 시점에 본격화
  (지금은 대비 가이드만)

### 후속 / 연결
- cross-platform §3.2 경로 B (spec clean-room) 의 **license 측 근거
  보강** — clean-room 이 가져오기(GPL 회피) + 돌려주기(license 자유)
  양방향 이점.
- licensing.md 의 single-license 원칙에 contribute-back dual-license
  예외 검토 (미결) → 실제 contribute 시 docs/ 승격 + sign-off
- paper §6 contribute-back venue 와 정합 (학술 + 코드 양방향 기여)
