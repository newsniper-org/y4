<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

---
topic: Asterinas reference scan — Rust framekernel OS 에서 Y4 가 참고/차용할 것 (특히 unsafe-confinement = capsule §2.1 선례)
created: 2026-08-13T12:57:42+09:00   # KST (UTC+9)
status: brainstorming (reference scan — 결정: §3 unsafe-confinement/OSDK 참고, §5 framekernel/Linux-ABI/sound-not-proven 배제, §6 clean-room Apache-2.0 차용).  side-channel 발제 다음
scope: https://github.com/asterinas/asterinas 를 Y4 관점에서 스캔 — 무엇을 참고/차용/배제할지.
       설계 탐색이 아니라 외부 프로젝트 차용가치 평가(성격이 다름)
refs:
  - https://github.com/asterinas/asterinas (repo, MPL-2.0)
  - arXiv 2506.03876 / USENIX ATC 2025 "Asterinas: A Linux ABI-Compatible, Rust-Based Framekernel OS with a Small and Sound TCB"
  - .brainstormings/20260812-174402-capsule-fault-isolation-restart.md §2.1 (unsafe-audit gate — 본 스캔의 최대 접점)
  - .brainstormings/20260620-154147-y4-dev-mode-host-execution.md (dev-mode 4-tier — OSDK 참고 접점)
  - .brainstormings/20260619-160308-contribute-back-guide-bhyve-nvmm.md (clean-room 양방향성 — MPL 차용 방식)
  - CLAUDE.md §3(single Apache-2.0) / §6 원칙 1(TCB 최소화)·5(verified base)·6(formal-first)
---

# Asterinas reference scan — 무엇을 참고/차용/배제할까

## §0 프레임

설계 탐색이 아니라 **외부 프로젝트 차용가치 스캔**(성격이 다름).  Asterinas
는 Y4 와 목표가 겹치는 지점(Rust OS, TCB 최소화, `unsafe` 격리)이 많아,
"그대로 채택"과 "무관"의 이분법이 아니라 **부분별 참고/차용/배제**를 판정.

## §1 Asterinas 요약 (fact, 조사 2026-08-13)

- **framekernel** 아키텍처: **single address space** monolith 이나 논리적으로
  2분할 —
  - **OSTD**(privileged framework): **유일하게 `unsafe` 를 쓸 수 있는** 최소
    계층.  모든 low-level·hardware-oriented unsafe 를 **safe API 뒤로
    encapsulate**.
  - **OS services**(de-privileged): **safe Rust 만** 허용.  process/스케줄러/
    FS/네트워크/드라이버를 OSTD 의 safe API 위에서 구현.
- **memory-safety TCB ≈ 14.0%** of codebase (paper) — OSTD 가 작다.
- **soundness** = Rust type safety + **OSTD 에 갇힌 audited unsafe** — **형식
  증명 아님**.
- **OSDK**(`cargo osdk`): Rust kernel/컴포넌트를 일반 Rust 앱처럼 build/run/
  test(in-kernel unit test 포함).
- **Linux ABI**: 210~230+ Linux syscall — Linux 바이너리를 **syscall
  에뮬레이션**으로 native 실행(가상화 아님).
- **license = MPL-2.0**(file-level copyleft), 일부 컴포넌트는 더 관대.
- paper: USENIX ATC 2025 / arXiv 2506.03876.

## §2 Y4 와의 근본 구조 차이 — 왜 통째로 못 가져오나

| 축 | Asterinas | Y4 |
|---|---|---|
| 격리 기반 | Rust type safety (services 는 unsafe 금지) | **하드웨어**(seL4 protection domain) + capability |
| 주소공간 | single-AS monolith | 다중 domain (microkernel/hypervisor) |
| 검증 | sound = type safety + audited unsafe | **Verus 형식 증명**(원칙 6) |
| workload | Linux 바이너리(syscall 에뮬) | guest 는 **HW virt VM**, 자체는 Rust |
| 목표 | Linux 대체 general-purpose OS | WaveTensor accelerator hypervisor |

Asterinas = "**Rust 를 신뢰**해 monolith 성능 + 작은 TCB", Y4 = "**하드웨어
격리 + 형식 증명**".  이는 capsule §2 의 *type-safe capsule vs seL4 domain*
긴장의 **whole-OS 판**이다.  ⟹ **framekernel 아키텍처 전체 채택은 X**(Y4
노선과 상충).  그러나 **부분 차용가치는 크다**.

## §3 (결정) 차용 — tier 1 (직접 참고, 강)

### §3.1 unsafe-confinement = OSTD 모델 ↔ Y4 §2.1 unsafe-audit gate ★★★
Y4 는 capsule §2.1 에서 이미 **"unsafe 는 granted-window HAL shim 에만,
나머지는 safe Rust, lint 로 강제"** 를 결정했다.  Asterinas 는 **정확히 이
패턴을 whole-OS 규모로 구현한 검증된 선례** — OSTD = Y4 의 HAL shim
(`sel4_backend` 류)에 대응, services(safe Rust) = Y4 의 capsule/상위 로직.

- **참고 1 — 강제 수단**: services 는 safe Rust 만(paper).  자연스러운 실현 =
  **service/capsule crate 에 `#![forbid(unsafe_code)]`**, unsafe 는 지정
  framework crate 로 집중.  Y4 §2.1 의 "lint 강제"를 이 **crate-level forbid
  attribute** 로 구체화할 수 있음(Asterinas 방식 검토 후 택).
- **참고 2 — 정량 증거**: TCB 14% 라는 숫자는 "unsafe-confinement 가 실제
  대규모 코드베이스에서 작동함"의 증거.  Y4 원칙 1(TCB 최소화)의 실현
  가능성 근거로 인용 가능.
- **차이 유지**: Asterinas 는 여기서 멈추지만(sound-not-proven), Y4 는 그
  safe-Rust 로직에 **Verus 증명을 더한다**(원칙 6).  즉 Asterinas 의 경계
  *구성*을 참고하되 검증 *깊이*는 Y4 가 더 간다(§5.3).

### §3.2 OSDK dev tooling ↔ Y4 dev-mode(host-Y4, qemu-smoke) ★
`cargo osdk` 의 "kernel 을 일반 Rust 앱처럼 build/run/**in-kernel unit
test**" UX 를, dev-mode 발제(20260620-154147)의 4-tier 개발 계층에 참고.
- 참고 대상 = **UX/패턴**(kernel test 를 마찰 없이), 도구 자체 아님 — Y4 는
  logicutils/justfile 표준(도구 채택 X).
- 특히 OSDK 의 in-kernel ktest 는 Y4 의 qemu-smoke tier 와 host-Y4 tier
  사이를 메우는 참고점.

## §4 차용 — tier 2 (아이디어 참고, 중)

### §4.1 safe hardware abstraction (OSTD 의 safe-API-over-unsafe)
OSTD 가 MMIO/DMA/hardware unsafe 를 safe API 로 감싸는 **API 형태**를 참고 —
Y4 의 HAL shim(§2.1)과 `ResourceKind::Dma`+IOMMU(capsule §4.2)가 정확히
이걸 필요로 한다.  코드가 아니라 **API 설계**(typed MMIO handle, DMA-safe
wrapper)를 참고.  Asterinas 논문/OSTD API 정독은 후속(§7).

### §4.2 TCB 정량화 문화
Asterinas 의 "TCB 14%" 처럼 **Y4 도 TCB 비율을 metric 으로 회계** — 원칙 1
payoff 를 숫자로.  paper/산업 도입 시 차별점(형식 증명 TCB 는 더 강한 주장).

## §5 (결정) 배제 — 명시적 non-adoption + 이유

### §5.1 framekernel single-AS 자체 — 배제
Y4 는 하드웨어 격리(seL4 domain) 노선.  Asterinas 는 services 간 **하드웨어
경계가 없다**(type-safe 라 안전하다는 신뢰).  Y4 는 foreign/GPL(3rd-tier)을
**반드시 하드웨어 격리**하고(capsule §2), safe Rust 로직도 **형식 증명**한다.
⟹ single-AS monolith 채택 X.

### §5.2 Linux ABI syscall 에뮬레이션 — 배제
Y4 는 guest 를 **HW virt(AMD-V/VT-x) full VM** 으로 호스팅, Linux 바이너리
native 실행이 목표가 아님.  자체 workload 는 Rust.  ⟹ Asterinas 의 Linux-ABI
층은 Y4 목표와 다른 모델 — 채택 X.  (만약 훗날 "VM 없이 Linux 바이너리"가
필요해지면 그때 재검토할 참고점으로만 기록.)

### §5.3 sound-not-proven 을 검증 노선으로 삼기 — 배제
Asterinas soundness = Rust type safety + audited unsafe(형식 증명 아님).
Y4 는 Verus 형식 증명(원칙 6)이 1st-tier.  ⟹ Asterinas 는 **검증 깊이의
참고 대상이 아니다**(Y4 가 더 강함).  단 §3.1 의 unsafe **경계 tooling** 은
검증과 무관하게 참고 유효.

## §6 (결정) 라이선스 — MPL-2.0 차용 방식

- Asterinas = **MPL-2.0**(file-level copyleft).  GPL 보다 약함(파일 단위,
  Apache 파일로 전염 X, 링크 가능) 하지만 **Apache-2.0 은 아님**.
- **코드 직접 import = MPL 파일 유입** → Y4 single-Apache-2.0 정책(CLAUDE §3)
  훼손.  CONTRIBUTING §3 대로 upstream SPDX 보존+Y4 line 이 필요해져
  GPL-capsule 보다는 완화되나 여전히 mixed-license.
- **결정 — 아이디어/아키텍처 패턴 차용(무료) + clean-room Apache-2.0
  재구현**.  MPL 코드 직접 import 지양.  contribute-back 발제(20260619)의
  clean-room 양방향성과 정합 — 아이디어는 참고, 필요시 Y4 개선을 Asterinas
  에 역기여(그쪽은 MPL 로).
- **예외**: `cargo osdk` 는 **build tool**(런타임 링크 아님)이라 *도구로
  사용*하는 것 자체는 license 부담 적음.  단 Y4 는 logicutils 표준이라
  도구 채택보다 **패턴 참고**(§3.2).

## §7 결정 / 미결 요약

**결정 방향(강)**:
- **unsafe-confinement 선례로 Asterinas OSTD 참고**(§3.1) — capsule §2.1 의
  강제 수단을 crate-level `#![forbid(unsafe_code)]` 로 구체화 검토
- OSDK 의 kernel-test UX 참고(§3.2, dev-mode 발제에 반영 여지)
- OSTD safe-HW-abstraction **API 형태** 참고(§4.1) + TCB 정량화 문화(§4.2)
- framekernel single-AS / Linux-ABI / sound-not-proven **배제**(§5)
- 차용은 **clean-room Apache-2.0**, MPL 코드 직접 import 지양(§6)

**미결**:
- capsule §2.1 강제 수단을 `#![forbid(unsafe_code)]`(Asterinas 식) vs 커스텀
  lint 중 무엇으로 할지 — 통합 결정 필요
- OSDK in-kernel ktest 패턴을 Y4 dev-mode 4-tier 에 구체 반영할지
- OSTD safe-HW API 를 얼마나 깊이 참고할지(논문 정독 후)

**후속(비-blocking)**:
- **arXiv 2506.03876 정독** → OSTD API·enforcement 상세·TCB 측정 방법 추출
- (해당 시) capsule §2.1 / dev-mode 발제에 참고 결과 반영

## §8 다음 발제 후보

- **attestation / measured boot** — tenant 가 TCB(+side-channel 방어)를
  암호학적으로 검증
- **scheduler design** — SMT gang / lease switch / RT budget / μarch flush 통합
- Asterinas 논문 정독 후 OSTD API 상세 참고(후속)
