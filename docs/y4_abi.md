<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 게스트 ↔ 하이퍼바이저 ABI (v0 — draft, not frozen)

본 문서는 **게스트 OS/워크로드가 Y4 하이퍼바이저를 호출·소비하는 유일한
계약** — 「Y4 ABI」— 을 정의한다.  지금까지 이 ABI 는 전용 문서 없이
`amdv_safety.md`(§VMMCALL) · `lease_capability.md` · `y4-hypercall` repo 계획 ·
서비스 capability 발제들에 **흩어져 있었고**, 그 부재가 드라이버/SDK 개발환경의
**블로커**였다(방침 2026-08-30, `MEMORY/y4_toolchain_and_abi_policy.md`).  본
문서(v0)가 그 조각들을 하나로 통합한다.

> ★ **`hiu_abi.md` 와 혼동 금지.**  `hiu_abi.md`("Y4 ↔ HIU ABI")는 Y4 가
> **WaveTensor 가속기 HIU** 에 의존하는 *가속기-대면* ABI 다.  본 문서는
> *게스트-대면* ABI — 두 ABI 는 별개이며 각자 독립적으로 동결된다(§5).

## 0. 범위와 경계

| ABI | 당사자 | 방향 | 이 문서? |
|-----|--------|------|:--------:|
| **Y4 ABI** (본 문서) | 게스트 ↔ Y4 하이퍼바이저 | 게스트-대면 | ✅ |
| `hiu_abi.md` | Y4 ↔ WaveTensor HIU | 가속기-대면 | ✗ |
| seL4 syscall ABI | Y4 ↔ seL4 | Y4 *아래*(재사용·verified) | ✗ |
| capsule ABI (`licensing.md` §2) | Y4 ↔ 격리 드라이버(GPL capsule 등) | 드라이버-대면 | ✗ |

**계층 위치**:

```
        게스트 OS / 워크로드
              │   ← Y4 ABI (본 문서: hypercall + capability + 가상 디바이스)
   ┌──────────┴───────────────────────────────┐
   │ Y4 하이퍼바이저                            │
   │  orchestrator · capsules · ipc · alloc     │
   └──────────┬───────────────────────────────┘
              │   ← seL4 syscall ABI (Y4 아래, verified base — 원칙 5)
        seL4 microkernel
              │
        hardware ──(별개 ABI: hiu_abi.md)── WaveTensor 가속기
```

## 1. 세 표면 (three surfaces)

Y4 ABI 는 세 표면으로 구성된다:

| 표면 | 성격 | §  |
|------|------|----|
| **A. hypercall** | 동기 호출(guest→Y4 trap) | §2 |
| **B. capability** | 권한의 보유·행사(unforgeable handle) | §3 |
| **C. 가상 디바이스/서비스** | 비동기·스트림(paravirt) | §4 |

## 2. hypercall 표면 (surface A)

- guest 가 `VMMCALL`(AMD-V) / `VMCALL`(Intel VT-x) 실행 → `INTERCEPT_VMMCALL`
  vmexit → Y4 **orchestrator 로 trap**(별도 hypercall capsule 없음 —
  orchestrator 의 thin entry, `amdv_safety.md` §VMMCALL / 2026-05-04 결정).
- **호출 규약**: `RAX` = hypercall 번호.  인자·반환 레지스터 배치는 v0 미확정
  (§6, TBD).  orchestrator 가 **화이트리스트 mediation**(S7.2) — 미등록 번호
  거부.
- **구현 위치**: core VMM/hypercall 처리는 **Y4 워크스페이스 안**(orchestrator
  의 thin entry — VMCB/NPT/VMEXIT).  `y4-hypercall` repo 는 **사용자측 CLI/API
  tooling 전용**(`vmm_arch.md` §5 재정의 확정; 현재 디스크 미존재)이지 core
  spec/impl 의 집이 **아니다**.  ★ **게스트↔하이퍼바이저 ABI 스펙의 집 = 본
  문서**(정책 메모 `y4_toolchain_and_abi_policy.md` §6-5 미결 해소).
- ★ **vendor/ISA-neutral**(`cpu_virt_compat.md` / `hw_mechanism_abstraction.md`
  P3): `VMMCALL`↔`VMCALL`(↔RISC-V 등)은 **realization**, "guest→host 동기 trap
  호출" 은 **계약**.
- **버전 discovery**: `Y4_ABI_VERSION` hypercall 이 `major.minor`(예:
  `0x0001_0000`)를 반환.  hiu_abi 가 HW 레지스터로 버전을 노출하는 것과 달리,
  Y4 ABI 는 SW 이므로 **hypercall** 로 discovery(§5).

## 3. capability 표면 (surface B)

- **원칙 2**: 게스트가 받는 모든 cross-tenant primitive 는 **capability**
  (unforgeable handle, seL4 cap 기반)이지 ad-hoc user token 이 아니다.
- 게스트는 handle 로 연산을 **invoke**(hypercall 경유)하며, 대상 자원에 **raw
  접근 불가**(use-without-extraction 계열, `key-management` §2 정합).
- **partition 스코프**(P2): capability 는 `partition_id` 로 스코프 —
  cross-partition capability 접근은 **구조적으로 불가**(design_decisions P2 의
  "하나의 partition 모델, N enforcer").
- **제공 capability 종류**:
  - **lease** — `lease_capability.md` 스키마 (accelerator 시간·자원; ⏳ 일부
    `hiu_abi.md` v1.0 의존).
  - **MMIO 접근 / memory region** — 원칙 2·4.
  - **service capability** — 예: WWAN voice/SMS/MCX/data bearer
    (`.brainstormings/20260827-000813-wwan-modem-switch-hub.md` §2/§5).

## 4. 가상 디바이스/서비스 인터페이스 (surface C)

게스트는 **표준 paravirt 가상 디바이스**로 서비스를 소비한다(모뎀/셀룰러 등
raw 하드웨어 세부는 노출하지 않음 — §5):

| 서비스 클래스 | 게스트가 보는 것 | Y4 처리 |
|---|---|---|
| 네트워크/internet | **virtio-net 가상 NIC** | route/NAT (WWAN §3, internet=route) |
| 음성(VoLTE/VoNR) | 로컬 IMS/SIP endpoint | **terminate-then-re-originate**(P6; WWAN §3) |
| SMS/MMS · MCX | SMS 인터페이스 · MCX broker API | terminate → 라우팅/broker |
| 저장 등 | virtio-blk 등 | (v0 TBD) |

- paravirt 모델 = **virtio**(표준, form-factor-agnostic).
- ★ **terminate-then-re-originate**(P6): 정체성/registration 을 실은 서비스는
  Y4 가 종단하고 게스트엔 **로컬 endpoint** 만 재기원 — raw 를 흘리지 않음
  (복제-방지·격리).

## 5. 이 ABI 를 통과하지 않는 것 (TCB 최소화 — 원칙 1·4)

게스트가 **절대 받지 못하는** 것 (경계의 negative space):

- **tenant data 의 Linux-stack 경유**(원칙 1) — 금지.
- **raw USIM-AKA / baseband 제어**(WWAN §5/§6) — 게스트는 로컬 vSIM creds 만.
- **cross-partition 메모리·capability**(P2).
- **seL4 syscall 직접 호출** — Y4 가 전면 mediation.
- **HIU raw 레지스터**(`hiu_abi.md` 는 Y4 *내부* 표면; 게스트에는 lease/service
  capability 로만 노출).
- **direct hardware access**(원칙 4)는 게스트↔Y4 capability *뒤*에서 Y4 가
  수행 — 가속기 경로는 driver stack 미경유.

## 6. 미결 / 이 문서가 여는 것

**v0 미결(설계 필요)**:
- hypercall 번호 체계 + 인자/반환 레지스터 배치(§2).
- capability handle 인코딩(seL4 cap ↔ 게스트 handle 매핑).
- virtio 가상 디바이스 목록 확정(§4).
- service-specific 인터페이스 상세 — 일부는 `hiu_abi.md`/RTL 의존(⏳).

**이 문서가 여는 것**:
- "게스트↔하이퍼바이저 ABI 문서 부재" **블로커 해소**(v0 draft 존재) →
  드라이버/SDK 개발환경 **선행조건 착수 가능**(단, 실제 착수는 v1.0 frozen 후).

## 7. 버전 / freezing 정책

- `Y4_ABI_VERSION`(§2 hypercall).  **semver**: 하위호환 추가 = minor,
  incompatible 변경 = major.
- **frozen 조건**: 게스트-대면 SDK/driver 개발 착수 전 **v1.0 frozen** 필요
  (§6).  frozen 시 `Y4_ABI_VERSION` = `0x0001_0000` 고정.
- `hiu_abi.md`(가속기-대면) frozen 과 **독립** — 서로 다른 ABI.  단 일부 service
  capability(accelerator lease 등)는 `hiu_abi.md` v1.0 에 의존하므로, 그 부분의
  최종 확정은 hiu_abi frozen 이후.

## 8. 검증 (formal-first — 원칙 6)

- Verus 대상: **hypercall 화이트리스트 mediation**(미등록 번호 거부) ·
  **capability unforgeability**(게스트가 handle 위조/승격 불가) · **cross-tenant
  무유출**(partition 경계, P2).
- 착수는 **verus-fork 안정화 이후**(다른 Verus 대상과 동일 타이밍).

## 9. cross-reference

- `docs/amdv_safety.md` §VMMCALL / `INTERCEPT_VMMCALL` — hypercall trap·mediation
- `docs/vmm_arch.md` §5 — VMM repo 구조(core VMM = 워크스페이스 안,
  `y4-hypercall` = 사용자 CLI/API tooling 재정의 확정)
- `docs/glossary.md` §10~11 — RTL↔Y4 ABI 경계 + 「Y4 ABI」중의성 4-표면(A hiu /
  B hypercall / C seL4-syscall / D lease) 분간.  본 문서가 그중 **게스트-대면
  표면(B+D+가상디바이스)**을 스펙으로 확정
- `docs/cpu_virt_compat.md` — `VMMCALL`↔`VMCALL` vendor-neutral (hypercall 계약)
- `docs/lease_capability.md` — lease capability 스키마
- `docs/hiu_abi.md` — **별개** ABI (Y4 ↔ HIU, 가속기-대면)
- `docs/design_decisions.md` — P1(TCB)·P2(partition master key)·P6(terminate-
  then-re-originate)·P11(capability-based everything)
- `docs/hw_mechanism_abstraction.md` — hypercall 명령의 vendor/ISA realization 추상화
- `.brainstormings/20260827-000813-wwan-modem-switch-hub.md` — 게스트-대면 서비스
  인터페이스(§C) 근거
- `CLAUDE.md` §6 원칙 1·2·4·6
