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
  - adsmt: `newsniper-org/adsmt` `testing` branch (현 시점 v1.0.0-rc.2)
  - adsmt-contrib: `newsniper-org/adsmt-contrib` `testing` branch (현
    시점 v1.0.0)
- **Pin 형식 (예정 — P-redesign.2 sign-off 시점)**: `unified-toolkit-
  pin.toml` 의 `channel = "testing"` 필드 + build-time commit hash
  capture (reproducible build 위해)
- **Cargo dep mechanism**: `Cargo.toml [patch.crates-io]` 로 adsmt +
  adsmt-contrib 의 testing branch 를 path 또는 git dep 으로 redirect
  (adsmt 측 fork `Honey-Be/oxiz` 의 패턴과 동일)
- **Transitive deps**:
  - OxiZ — adsmt 측 fork (`Honey-Be/oxiz`) `feat/enable-writer` branch
    의 transitive 추종
  - logicutils — adsmt 의 `external/logicutils/` submodule 의 transitive
    추종 (v1.0 통합 후 adsmt 안)

**Stable release 도달 시 옵션** (P-redesign.8 sign-off 시점 결정):
- (i) testing channel 계속 (rolling 유지) — Y4 측 work 가 항상 latest
- (ii) stable channel 으로 전환 — Y4 spec v1.x patch 의 freeze 보장
- (iii) hybrid — paper artifact submission 시점에만 stable, development
  는 testing

**Tracker cross-ref**: `.claude-notes/trackers/adsmt-integration-tracker.md`
§10.6.
