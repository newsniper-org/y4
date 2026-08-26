<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Reproducible build / supply-chain — source→runtime gap 을 닫는 RIM + provenance (여러 발제의 수렴점)
created: 2026-08-26T23:19:14+09:00   # KST (UTC+9)
status: brainstorming (attestation §9 유일 잔여 미결 + key-mgmt §4 + boot §3.1/§6.5 의 수렴점).  결정 다수; 잔여 미결은 deferrable.  §7 transparency log=기존 채택(자체 구현 X) 확정
scope: Y4 의 build 재현성 + supply-chain 무결성.
       reproducible hermetic build / RIM(Y4-owned) / SLSA·in-toto provenance /
       toolchain(Trusting Trust) / dependency 무결성 / transparency log / trust loop
refs:
  - .brainstormings/20260813-132525-attestation-measured-boot.md §4(RIM/quote)·§9(RIM=유일 잔여 미결)
  - .brainstormings/20260813-205326-key-management.md §4(KEK sealing 대조값)·§1(boot-verify pubkey)
  - .brainstormings/20260817-153920-boot-chain-deep-dive.md §3.1(Limine digest 재현)·§6.2/§6.5(per-chunk RIM)·§8(PCR-replay Verus)
  - docs/hw_mechanism_abstraction.md §3(Y4-owned 경계 — RIM/reproducible build 는 소유 쪽)
  - CLAUDE.md §6 원칙 1(TCB 최소화) / §8 D1(rust-toolchain 1.94 stable, submodule pin, logicutils build)
---

# Reproducible build / supply-chain — 감사한 source 가 실제 도는 code 임을 증명

## §0 프레임

**최종 구현 목표 기준.**  본 발제는 여러 선행 발제의 **수렴점**이다 —
attestation §9(RIM = 유일 잔여 미결) · key-mgmt §4(KEK sealing 의 known-good
대조값) · boot §3.1(Limine 의 digest 재현) · §6.5(per-chunk slot RIM).  이들이
공통으로 요구하는 **known-good 측정값의 출처**가 곧 reproducible build + RIM.

이는 외부 하드웨어 메커니즘이 아니라 **Y4-owned deliverable**(hw_mechanism_
abstraction §3 경계의 소유 쪽 — 설계·고정·운영).

## §0.5 원칙 정합

- **TCB 최소화**(원칙 1) → ★ **build/supply-chain 으로 확장**: 무엇을 신뢰해야
  하는지를 런타임을 넘어 **build 신뢰**까지 최소화(§1).
- **formal-first**(원칙 6) → ★ **검증 3모드 경계**: reproducibility=rebuild 재현,
  provenance=서명, PCR-replay=Verus(§9).
- **capability**: release-signing key = Y4-owned key(key-mgmt, platform AK 와
  구별, §8).
- **atomic-free**: build 는 런타임 hot path 아님 — 무관.

## §1 ★ 명제 — reproducible build 가 source→runtime gap 을 닫는다

- **attestation** 은 "이 바이너리가 부팅됐다"를 증명한다.  그러나 그 바이너리가
  **감사한 source 와 같은지**는 증명 못 한다.
- **reproducible build + RIM** 이 그 gap 을 닫는다: 동일 source+toolchain →
  **bit-identical 바이너리** → 그 측정값(RIM) == 감사한 source 의 측정값.
- 합치면: **감사 가능한 code 가 실제 도는 code 임이 증명**되고, key/attestation
  이 거기 bind(§10).
- ★ **build 신뢰 최소화**: **독립 rebuilder(community)**가 RIM 을 재현하면,
  tenant 는 **Y4 의 build machine 을 신뢰할 필요 없이** source + 자기 rebuild 만
  신뢰하면 된다 → TCB 최소화를 **build/supply-chain 으로 확장**(§7).

## §2 reproducible hermetic build

- **이미 있는 pinning 활용**: `rust-toolchain.toml`(stable 1.94) · submodule pin
  (seL4 15.0.0 / Limine v12.1.0) · `Cargo.lock`(workspace).
- **추가 필요**: `SOURCE_DATE_EPOCH`(고정 타임스탬프) · `--remap-path-prefix`
  (경로 제거) · 고정 `CARGO_HOME` · **no-network hermetic env** · deterministic
  link order · embedded build-host 정보 제거.
- **seL4 CMake 재현성**: 고정 compiler + `SOURCE_DATE_EPOCH`.
- **logicutils**(freshcheck/stamp) 는 hash-driven → 재현 친화; per-arch build
  matrix 는 `lu-par`(boot/).

## §3 RIM (Reference Integrity Manifest) — Y4-owned

- reproducible build 로부터 **정확한 측정값 계산**: PCR 8/9(boot §3.1 Limine
  규약 = UAPI) + DRTM 측정값.  **per-chunk**(chunk-K/chunk-P, boot §6.2/§6.5)
  · **per-arch**.
- **RIM = 서명된 manifest**: {version, per-chunk digests, expected PCR/MR,
  kernel-ABI 호환 범위(boot §6.2), source provenance ref}.
- 공개 → **attestation(§4 대조값)** · **KEK sealing(key-mgmt §4 known-good)** 의
  기준.
- hw_mechanism_abstraction §3 경계: RIM 은 **Y4-owned**(추상화 대상 아님).

## §4 build provenance — SLSA / in-toto

- artifact 가 **어떻게 build 됐는지** attest: source commit · builder identity ·
  toolchain · deps · build steps.  **SLSA provenance / in-toto attestation**.
- 목표 **SLSA L3+**(hardened build + non-falsifiable provenance) — 보안 임계성.
- **조합 효과**: provenance = "source X 를 builder Y 가 build"; reproducibility =
  "누구나 재빌드로 재검증"(builder Y 신뢰 제거).  둘이 상보.

## §5 toolchain — Trusting Trust 대응

- 컴파일러 backdoor(Thompson "Reflections on Trusting Trust") 위협.
- 방어: **Diverse Double-Compilation(DDC, Wheeler)**(독립 두 컴파일러 결과 비교로
  backdoor 탐지) + **bootstrappable builds**(minimal auditable seed 에서 toolchain
  구성).
- Y4: 현재 toolchain pin → **bootstrappable / DDC-verifiable** 를 forward-looking
  목표로.  Rust bootstrap chain(rustc-built-by-rustc)도 고려.

## §6 dependency 무결성

- `Cargo.lock` pin + checksum · **cargo-vet / cargo-crev**(dep review) ·
  **cargo-deny**(license/advisory) · vendored deps(hermetic).
- submodule pin 은 verified commit hash.  reused base(seL4 BSD-2 / Tock /
  DragonFly / Redox / Limine) 각각 pin + 무결성.

## §7 (결정) transparency log + 독립 rebuilder

- **결정 — 기존 transparency log 채택(Sigstore Rekor 등), 자체 구현·운영 X**
  (원칙 5 reuse; Y4 는 **publish + verify 만**).  log 서비스는 외부 표준이고,
  그 위에 올리는 RIM/provenance **내용**과 verify 정책이 Y4-owned(§11).
- RIM/provenance 를 **append-only transparency log**(Sigstore Rekor / CT-style)에
  공개 → tenant 가 검증하는 RIM 이 **모두가 보는 것과 동일**(split-view/
  equivocation 차단).
- **독립 rebuilder**(Debian reproducible-builds 식)가 공개 RIM 을 rebuild 로
  확인 → community-verifiable, **Y4 build infra 신뢰 감소**(§1).
- attestation 결합: quote 를 **transparency-logged RIM** 과 대조 → split-view
  attestation 차단.

## §8 서명 + key (key-mgmt 결합)

- RIM/provenance = **release-signing key**(offline, hardware RoT rooted)로 서명 →
  **boot-verify pubkey**(boot §4 / key-mgmt §1)의 출처.
- release-signing key = **Y4-owned key**(key-mgmt; platform AK 와 구별).

## §9 ★ 검증 3모드 (formal-first 경계)

하나의 검증 모드로 안 된다 — 층마다 다른 모드:

| 층 | 검증 모드 |
|---|---|
| build 재현성 | **rebuild-and-compare**(empirical / community, 증명 아님) |
| build provenance | **서명 체인**(SLSA / in-toto) |
| runtime PCR-replay | **Verus**(boot §8, verus-fork 안정화 이후) |

경계 명시: reproducibility 는 증명이 아니라 **재현**으로, provenance 는 **서명**
으로, 측정 정합만 **Verus** 로 선다.

## §10 trust loop — 전체 봉합 ★

```
source (감사·리뷰)
  → reproducible hermetic build (§2)
  → bit-identical artifact
  → RIM(§3) + SLSA provenance(§4)
  → 서명 + transparency log (§7·§8)
  → tenant attestation: quote vs logged RIM (attestation §4)
  → KEK sealed to RIM 측정값 (key-mgmt §4)
  → key 는 reproducibly-built·source-audited TCB 아래에서만 unseal
```
⟹ **source→runtime 신뢰 chain 완결**.  세션 전체(verified base · attestation ·
key · boot)의 **supply-chain 마감**.  attestation §3 의 "verified seL4 가 부팅됨"
에 "그 seL4 == 감사한 source 로 재현 가능"이 더해진다.

## §11 Y4-owned 경계 (hw_mechanism_abstraction §3 positive example)

RIM · reproducible build · provenance · release signing = **Y4-owned
deliverable**(설계·운영).  **transparency log 서비스 자체는 기존(Rekor 등)
채택**(§7)이되, 그 위에 올리는 RIM/provenance **내용**과 verify 정책은
Y4-owned.  하드웨어 메커니즘(RoT/measured-boot 명령/confidential-VM/device
attestation)은 **추상화**하지만 이 산출물들은 **Y4 가 소유·고정**한다 —
hw_mechanism_abstraction §3 경계의 **positive example**.

## §12 결정 / 미결 요약

**결정 방향(강)**:
- reproducible hermetic build — 기존 pin(1.94/submodule/Cargo.lock) 확장 +
  SOURCE_DATE_EPOCH/path-remap/hermetic(§2)
- **RIM = Y4-owned per-chunk·per-arch 서명 manifest**(§3), attestation·sealing 의
  known-good
- **SLSA L3+ provenance**(in-toto) + reproducibility 상보(§4)
- **transparency log = 기존 채택(Rekor 등, 자체 구현 X)** + 독립 rebuilder →
  build infra 신뢰 감소(§7)
- dependency: cargo-vet/deny + pin; toolchain: bootstrappable/DDC 지향(§5·§6)
- **검증 3모드**(재현/서명/Verus, §9) · trust loop 완결(§10) · Y4-owned 경계(§11)

**미결(설계 필요) — 지금/나중 결정 무관, deferrable**:
- SLSA 목표 레벨 확정(L3 vs 그 이상) + in-toto layout
- bootstrappable toolchain 도입 시점 + DDC 운영 정책
- RIM 파일 포맷 세부(서명 방식 / per-chunk·per-arch 구성)

**⏳ 선행 의존**:
- attestation 구현 — RIM 소비(quote 대조 / sealing)
- verus-fork 안정화 — PCR-replay Verus(§9, boot §8과 동일 타이밍)

## §13 다음 발제 후보

- observability / telemetry in minimal TCB — TCB 최소화와 관측성의 긴장
- (사용자 지시) WWAN 모뎀 직접 제어 + 상위 OS 서비스 분배
