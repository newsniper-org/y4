---
name: Y4 sign-off 후 후속 논의 대기 항목
description: 현재 진행 중인 ARCH-II' sign-off 18 단계 완료 후 사용자가 요청한 후속 논의 주제 목록
type: project
originSessionId: 78ff80c3-5421-425a-9e23-3da166ef2bb9
---
ARCH-II' sign-off 18 단계 (P1.1~P1.6, P2.1~P2.5, P3.1~P3.7) + Phase 4
v1.0 frozen 마킹 모두 완료 **이후** 논의할 항목:

1. **소비전력 관리 (power management) 기능** (사용자 요청 2026-05-05)
   - **Why:** 사용자가 sign-off 흐름 중에 명시적으로 추가 요청. 현재
     spec (S1–S14, vmm_arch.md ARCH-II', amdv_safety.md, sel4_fork_policy.md,
     verus_to_isabelle.md) 어디에도 power 관련 항목 0.  Y4 가 server-farm
     host / 랩톱 / 핸드헬드+독 / 임베디드 SoC 까지 5 형상 host OS 라
     소비전력 관리는 형상별 차이 큼.
   - **How to apply:** sign-off 완료 신호 (Phase 4 frozen 마킹) 직후,
     자체적으로 본 항목을 꺼내어 사용자에게 진입 옵션 제시.  논의 범위
     예시:
     * cpufreq governor (P-state / C-state) 의 capsule 위치 + ACPI/PSP
       와의 관계
     * S5 의 SMT-aware grouping + CPU offline 정책과의 정합 (저전력
       시 SMT pair offline 가능성)
     * deep C-state 시 capsule cluster suspend / lease 의 일시 정지
       의미
     * 형상별 power profile (data center / desktop / handheld / SoC
       battery)
     * RAPL / energy-uj 같은 energy counter 의 audit (S12) 통합
     * power-related side channel (Hertzbleed 등) 차단의 안전장치
       카탈로그 추가 (S15+ 후보)
   - **포지션:** `docs/amdv_safety.md` 의 S1–S14 와 같은 별도 안전장치
     카탈로그 (`docs/power_safety.md`?) 또는 `docs/architecture.md` 의
     subsection 으로 진입 결정.
