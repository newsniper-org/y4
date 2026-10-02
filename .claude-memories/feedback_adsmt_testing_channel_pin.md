---
name: Y4 의 adsmt + adsmt-contrib dependency = testing channel pin (rolling)
description: Y4 가 adsmt + adsmt-contrib 둘 다 testing channel 으로 pin; adsmt main + testing 이 rolling release 패턴이므로 Y4 측은 latest stabilisation 작업의 즉시 흡수
type: feedback
originSessionId: 78ff80c3-5421-425a-9e23-3da166ef2bb9
---
Y4 가 adsmt + adsmt-contrib 둘 다 **testing channel pin** — rolling
release 패턴.

**Why** (사용자 명시 2026-06-01): adsmt 측 main + testing 둘 다
rolling release 패턴으로 운영.  Y4 측이 testing channel 을 pin 으로
추종하면 adsmt 의 latest stabilisation 작업 (RC2.x audit cycle 같은)
의 즉시 흡수 가능 — Y4 측 verification workflow 의 evolve 와 정합.

**How to apply**:

- **Pin 대상 (양 repo testing channel)**:
  - adsmt: `newsniper-org/adsmt` `testing` branch
  - adsmt-contrib: `newsniper-org/adsmt-contrib` `testing` branch
  - 현재 해석된 commit 은 **`Y4/unified-toolkit-pin.lock` 이 기록처**다
    (2026-06-09 기준 adsmt `5a1a6ee` = v1.0.0-rc.33, adsmt-contrib
    `cbf7a46`).  이 메모에 「현 시점 버전」을 적지 않는다.
  - ⚠ 단 lock 은 2026-06-09 이후 재생성되지 않았다. 2026-10-02 실측으로 설치된
    adsmt-cli-testing 은 rc.39.1+ (`5f65943`), verus-fork submodule 은
    `c9b9a45ee`. 실제 상태는 `pacman -Q adsmt-cli-testing` / `git submodule
    status` 로 읽는다.
- **Pin 형식 (✅ P-redesign.2, commit 91cd38a)**: `unified-toolkit-
  pin.toml` 의 `channel = "testing"` 필드 + `.lock` 에 build-time commit
  hash capture (reproducible build 위해)
- **Cargo dep mechanism (계획, 미적용)**: `Cargo.toml [patch.crates-io]` 로
  adsmt + adsmt-contrib 의 testing branch 를 redirect 하는 안.  2026-10-02
  실측: Cargo.toml 에 `[patch.crates-io]`·git dep 없음 — adsmt 는 시스템
  바이너리(`/usr/bin/lu-smt`, `adsmt-cli-testing`)로만 소비된다.
- **Transitive deps**:
  - OxiZ — adsmt 측 `external/oxiz` submodule 의 transitive 추종 (현
    lock: `0.2.4-feat/streaming-stdin`; `unified-toolkit-pin.toml` 의
    `fork_branch` 는 아직 `0.2.3-feat/enable-writer` — toml/lock 불일치)
  - logicutils — adsmt 안 absorbed (P5 option 5,
    `state/logicutils-frozen/`); 도구는 `logicutils-testing` 패키지로 설치(`adsmt-meta-testing` 이 끌어옴)

**Stable release 도달 시 옵션** — 2026-06-01 (i) testing 계속으로 결정
([[feedback_adsmt_v1_verification_redesign]]).  ⚠ **충돌 (2026-10-01 발견,
사용자 확인 필요)**: `y4_pending_topics.md` R7.6 항과 `unified-toolkit-
pin.lock` 주석은 「stable v1.0.0 release 시 stable-v1 channel 전환 예정」
이라 적는다(`.claude-notes/trackers/adsmt-integration-tracker.md:133` ·
`y4-sel4-integration-tracker.md:28` 도 R7.6 「사용자 결정」으로 기재).  시간순으로
R7.6(2026-06-03~07)이 (i)(2026-06-01)보다 나중이다.  stable v1.0.0 은 아직
미출시(2026-10-02 최신 tag rc.42.1)라 전환 시점은 미도래.  어느 쪽이 현행인지는
사용자 확인 필요.  당시 옵션:
- (i) testing channel 계속 (rolling 유지) — Y4 측 work 가 항상 latest
- (ii) stable channel 으로 전환 — Y4 spec v1.x patch 의 freeze 보장
- (iii) hybrid — paper artifact submission 시점에만 stable, development
  는 testing

**Tracker cross-ref**: `.claude-notes/trackers/adsmt-integration-tracker.md`
§10.6.
