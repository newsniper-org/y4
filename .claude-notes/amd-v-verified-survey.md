<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# AMD-V (SVM) 를 지원하는 formally-verified microkernel / hypervisor 후보 — 사전 조사

문제:
- seL4 15.0.0 의 `KernelVTX` 는 Intel VT-x 전용. AMD SVM 코드 0.
- 현 개발 호스트는 AMD Ryzen APU.
- Y4 base 는 "verified microkernel + 그 위 specialization" 정책
  (CLAUDE.md §6.5 "Verified base").  base 자체 가 verified 가 아니면
  Phase E 인증 트랙의 핵심 가치가 흔들림.

본 메모는 **"AMD-V 게스트 호스팅을 verified-base 위에서 가능하게 할
대안"** 을 카테고리별로 정리. 사실 확인은 모두 *공개 문헌 + 프로젝트
저장소* 기준.  검증 강도, 라이선스, 라이브성, AMD-V 채택 의사를 비교.

---

## 카테고리 A — seL4 fork / 패치로 SVM 추가

### A1. seL4 mainline AMD-V 작업

상태: **mainline 미머지.** seL4 GitHub 이슈/PR 트래커를 한 번 훑어
보았을 때 (작성 시점 기준) AMD-V 추가 patch 가 산발적으로 제안되었으나
머지된 것은 없음.  seL4 팀은 verified configuration 우선 (Intel VT-x
도 verified 빌드에서는 OFF) — AMD-V 추가 시 verification 작업 재개
필요.

검증 강도: **현재 0** (코드 미존재). 추가 시 비-verified 모드.

라이선스: BSD-2-Clause.

채택 의사를 평가: 직접 작성 시 **수천 줄 + verification 미적용**.
verification 받으려면 다년 작업.

### A2. NICTA/Data61 academic forks

여러 학위논문/논문에서 seL4 에 SMP, RT, hypervisor 확장 시도. 그 중
AMD 측 작업은 거의 없음 (대부분 ARM hyp 모드 / Intel VT-x 확장).

상태: 학술 prototype. **production 없음.**

---

## 카테고리 B — verified hypervisor 직접 (microkernel 우회)

### B1. CertiKOS

- 출처: Yale (Zhong Shao 그룹).
- verified concurrent OS kernel.  **Intel x86 only** — AMD-V 미지원.
- 라이선스: 일부 BSD-style + 일부 학술 라이선스.  코드 공개 제한적.
- 비고: 2016 OSDI 논문 ("Building Certified Concurrent OS Kernels").
  research artifact; **production 없음**.  가속기 통합 사례 없음.

⇒ **AMD-V 측면에서는 비후보.**

### B2. Verisoft / VAMOS / VAMP

- 출처: 독일 Saarland 대학 + 산업 컨소시엄 (BMW, Infineon 등).
- VAMP CPU model 위 verified hypervisor (Hyper-V variant).
- 검증: Isabelle/HOL.
- 상태: 2010 즈음 종료. 코드 공개 제한.  **AMD-V 통합 작업
  보고된 적 없음** (VAMP 자체가 Intel 모델).

⇒ 비후보.

### B3. Komodo (Microsoft Research)

- ARM TrustZone 기반 verified enclave 모니터.  x86 무관.

⇒ AMD-V 측면 비후보.

### B4. Hyperkernel (UW) — **사실 확인 완료**

- 출처: University of Washington UNSAT 그룹 (Luke Nelson 등),
  "Hyperkernel: Push-Button Verification of an OS Kernel", SOSP 2017.
  Repo: https://github.com/uw-unsat/hyperkernel.
- 검증: **Z3 SMT solver, LLVM IR 수준에서 모든 syscall + trap handler
  의 functional correctness 검증**.  flow: kernel 을 LLVM IR 로
  빌드 → Irpy 가 LLVM IR 을 Python 으로 번역 → `hv6/spec/main.py` 가
  Z3 호출.  loop / 재귀 제한 설계로 push-button 패턴.  툴 의존:
  LLVM 5.0.0 + Z3 4.5.0.
- **AMD-V (SVM) + Intel VT-x + Intel VT-d 모두 코드 트리에 존재 — 직접
  확인됨**:
  - `kernel/svm.c`, `kernel/svm.h` — AMD-V
  - `kernel/vmx.c`, `kernel/vmx.h` — Intel VT-x
  - `kernel/intel_iommu.c` — Intel VT-d
- 라이선스: **Apache License 2.0** (UW 자체 작성 부분).  외부 reuse
  코드 (xv6, sv6, FreeBSD, Linux, NetBSD) 는 각자 라이선스.  Y4 측에서
  사용 시 mixed-license 관리 필요.
- 아키텍처: x86_64 only.  Intel + AMD 둘 다.
- 시스템 종류: "OS kernel with hypervisor capabilities" — xv6 파생
  ("hv6").  microkernel 디자인.
- 활성도: **연구 plagground, production 비권장** (README 명시:
  "This is a playground ... Don't use it for production").  master
  branch 7 커밋, release 0 개.  **Issue 1 / PR 0** — 사실상 휴면
  상태.  Verification 1회 ~30분.
- 코드 분량: C 91.8%, Python 5.6%, C++ 1.0%, Asm 0.6%.  rough scale 은
  중규모 microkernel (정확 LoC 불명).
- 게스트 호스팅: README 에 "VT-x (VMX) and VT-d (IOMMU) support" 명시,
  Intel E1000 NIC 테스트.  단 **무엇을 게스트 OS 로 호스트하는지**
  문서 부족 — sample guest 가 있는지, Linux/BSD 부팅 가능한지 코드
  레벨 추가 확인 필요.
- 후속 작업: https://unsat.cs.washington.edu/projects/hyperkernel/

⇒ **놀라운 후보** — verification + AMD-V 양립이 documented 로 존재하는
   **유일한 발견된 사례**.  단 휴면 상태 + production 비권장이 큰
   trade-off.

평가 갱신:

| 측면 | 결과 |
|---|---|
| AMD-V 지원 | ◎ 확인 (`kernel/svm.c`) |
| Verification | ◎ Z3 SMT push-button (LLVM IR 수준) |
| 라이선스 (UW 자체) | ◎ Apache-2.0 |
| 라이선스 (외부 reuse) | △ mixed (xv6/Linux/etc.) |
| 활성도 | ✗ 사실상 휴면 (7 커밋, README 가 "playground" 라 자칭) |
| 게스트 OS hosting 사례 | △ documented 부족 |
| Y4 정합 (HIU 같은 비표준 device 통합) | ? — verification 범위 확장 시 push-button 패턴 작동 여부 미확인 |
| 도구 의존 | ⚠ LLVM 5.0.0 (현재 22.x) + Z3 4.5.0 (구버전) — 현 도구체인과 격차 큼 |

추가 확인 필요 (남은 항목):
1. **Hyperkernel 의 Z3 spec 범위 — kernel core 만? hypervisor extension 도?**
   `hv6/spec/main.py` 직접 읽어 syscall 별 verification 구조 파악
2. **휴면 상태 → fork + 자체 유지 비용** — 활성 프로젝트가 아니므로 Y4
   가 본격 채택하려면 자체 fork + LLVM/Z3 버전 업데이트 + 신규 syscall
   추가 시 verification framework 확장이 우리 책임
3. **xv6 파생 의 한계** — 학술 prototype 의 OS 모델 (단순 syscall set,
   minimal driver, single-CPU 가능성 등) 이 Y4 의 SMP-first 결정 (C2)
   과 호환되는지
4. **HIU 같은 비표준 PCIe device 의 verification spec 작성 부담**
5. **게스트 OS 호스팅 능력의 실제 범위** — Linux 부팅 가능? BSD?
   Alpine 첫 게스트 PoC 가능성?

### B5. SeKVM (Columbia, 2021)

- Linux KVM 의 hypervisor core 를 ARM 위에서 *security-critical
  invariants* 만 Coq 로 검증.
- **ARM only** — AMD-V 측면 비후보.

### B6. CASH (Computer-Aided Security Hardening)

- 2024 류 OSDI/SOSP 발표 — TEE/hypervisor 부분 검증.  특정 ISA 의존
  보고 다양.  세부는 추가 조사 필요.

---

## 카테고리 C — 비-verified 이지만 AMD-V production-ready

### C1. Xen Project

- production-grade.  AMD-V 완전 지원.
- 검증: **없음** (코드만, formal proof 무).
- 라이선스: GPLv2 (hypervisor) → Y4 main tree 와 **라이선스 충돌**.
- Y4 정책상 "GPL capsule isolation" 으로 우회 가능하지만, *base 가 GPL*
  이면 그 위 specialization 도 영향.

⇒ **라이선스 차단** + **verified 아님**.

### C2. Bhyve (FreeBSD)

- BSD-2-Clause.  AMD-V (SVM) 완전 지원.  Phase D 의 형상별 driver
  포팅에서 BSD source 로 1순위 우선권.
- 검증: 없음.
- 채택 시: Y4 가 bhyve 위에 얹히는 것이 아니라, bhyve 의 SVM 핸들링
  코드를 **Y4-VMM (`y4-hypercall`) 안으로 알고리즘 포팅** 가능.
  GPL 회피 + BSD 호환.

⇒ **production AMD-V 코드의 가장 깨끗한 reference**.  단 verification
   없음.

### C3. NetBSD's NVMM (Native Virtual Machine Monitor)

- BSD-2-Clause.  AMD-V (SVM) + Intel VT-x 둘 다.  user-space 부분이
  많아 포팅 분리 깨끗.
- 검증: 없음.

⇒ bhyve 와 같은 카테고리.  포팅 reference 로 활용.

### C4. illumos KVM port

- CDDL.  Y4 와 라이선스 비호환 (CDDL ↔ Apache-2.0 일방).

⇒ 비후보.

### C5. Tock OS hypervisor mode

- Tock 자체는 hypervisor 가 아님.  Cortex-M 기반 IoT OS.

⇒ 비후보.

---

## 카테고리 D — 새 model + 부분 verification 전략

### D1. Y4-VMM 자체 작성 + Verus refinement

`y4-hypercall` 안에서 SVM 명령어 (`vmrun`, `vmload`, `vmsave`,
`clgi`/`stgi`, VMCB 구조체) 를 직접 발화 + 그 위 control flow 를 Verus
로 검증.

- AMD APM (vol 2 §15) 가 SVM 의 ISA 명세 — 비교적 작은 표면.
- VMCB 의 trap 처리 + nested page table 셋업이 핵심.
- Verus 로 "vmrun → exit → state restore" 의 invariant 만 증명해도
  의미 있음 (full functional correctness 까지는 다년 작업이지만 부분
  보장 가능).

비용: 중-대 (~수천 LoC + Verus 명세 비슷).
verification 정도: **부분** (가장 critical 한 invariant 만).
seL4 의존: 가능 (seL4 위 root task / capsule 로 동작) 또는 우회 가능.

⇒ **현실적 best path 후보 1 순위.**  seL4 base (verified microkernel) +
   Y4-VMM (부분 verified) 조합으로 verified-base 정체성 보존.

### D2. NOVA microhypervisor — **재평가, 라이선스 회피 가능**

- 출처: Udo Steinberg (TU Dresden 출신).  Bedrock Systems 와 Genode 의
  base 중 하나.
- AMD-V + Intel VT-x 둘 다 지원.
- 검증: mainline NOVA 자체 는 unverified.  후속 NOVA-PSL (Proof System
  Language) 기반 부분 검증 작업 보고.
- 라이선스: GPL-2 — 그러나 LICENSE 파일 상단에 **hypercall API
  exemption** 명시:

  > 사용자 프로그램이 hypercall API 를 사용하는 것은 "derived work"
  > 로 간주하지 않으며 GPL 제한에서 벗어난다.

  (확인 출처: `https://raw.githubusercontent.com/udosteinberg/NOVA/refs/heads/release/LICENSE`)

  ⇒ **Linux 의 syscall 경계와 동일한 모델**.  Y4 가 NOVA 의 hypercall
     ABI 만 사용하면 GPL 전염 X — Y4 는 Apache-2.0 유지.  Y4 는
     "NOVA 위 userland" 가 됨.

⇒ **라이선스 차단 해제** (재평가).  **유의:** Y4 의 정체성이 "verified
   base 위 specialization" 인데 NOVA 자체는 unverified.  base verified
   원칙은 손상.  대신 AMD-V 즉시 가능 + 라이선스 깨끗.

### D3. Genode + base-hw

- Genode framework 위 base-hw (Genode 자체 microkernel).  AMD-V 지원.
- 라이선스: AGPLv3 → Y4 와 비호환.

⇒ 비후보.

### D4. Muen separation kernel

- Komobi/secunet 의 SPARK/Ada 작성 separation kernel.
- **AMD-V 미지원** (사용자 확인) — Intel VT-x/VT-d 전용. seL4 와 같은
  ISA 한계.
- SPARK proof 로 일부 정리 검증.
- 라이선스: GPL-3 → 라이선스 차단.

⇒ **AMD-V 측면 비후보** + 라이선스 차단.

---

---

## 카테고리 E — "untrusted hypervisor" 모델 (SEV-SNP / SVSM)

### E1. AMD SEV-SNP + SVSM (Secure VM Service Module)

- 출처: AMD SEV-SNP 의 boot/runtime 보호 + Linux/COCONUT 등의 SVSM 구현.
- 핵심 아이디어: **하이퍼바이저를 신뢰하지 않음 (Untrusted Hypervisor)**.
  하드웨어가 게스트 메모리를 암호화 (SEV-SNP) + 페이지 무결성 보장 +
  attestation 제공.  SVSM 은 **VMPL0 (가장 높은 권한 VM 레벨) 에서
  돌아가는 작은 service module** 로, vTPM / boot policy / page state
  관리 등을 게스트 OS 와 분리해 제공.
- 검증: SVSM 자체는 보통 Rust 작성 (COCONUT-SVSM 사례).  Y4 가 SVSM 을
  자체 작성하면 Verus 검증 가능.
- 2025–2026 동향: Linux mainline 에 SVSM ABI 통합 진행, AMD 가 적극
  push.  PRO 시리즈 / EPYC 9xxx 시리즈에서 SEV-SNP + SVSM 정착.
- **라이선스 / verification / Y4 정합 모두 깨끗**:
  - SVSM 은 Y4 가 직접 작성 가능 (Apache-2.0).
  - 하이퍼바이저는 KVM/Xen/Hyper-V 어느 것이든 — Y4 는 그 위 게스트.
  - 실리콘 보안 모델이 "verified base" 의 역할을 일부 대체.

⇒ **재평가 매우 1순위 후보**.  단, 위 카테고리 A–D 와 **다른 패러다임** —
   "Y4 가 hypervisor" 모델을 포기하고 "Y4 는 SEV-SNP 게스트 + SVSM
   provider" 로 재정의하는 것을 의미.  WaveTensor HIU 통합 / lease
   capability 모델을 SEV-SNP context 안에서 어떻게 표현할지가 새
   설계 문제.

확인 필요:
1. WaveTensor HIU 의 PCIe device 가 SEV-SNP guest 에서 passthrough
   가능한지 (TDISP / SEV-TIO 표준)
2. 사용자 랩탑의 Ryzen APU 가 SEV-SNP 지원 SKU 인지 (`lscpu | grep
   sev_snp`).  보통 EPYC 서버 SKU 만 지원, 데스크탑 Ryzen 은 미지원
   가능성 높음
3. SVSM 자체 작성 분량 (COCONUT-SVSM 코드 규모 reference)
4. WaveTensor 가속기를 SEV-TIO trusted device 로 추가 등록할 수
   있는지 (firmware / SPDM 인증)

⚠ **본 후보의 큰 trade-off**: Y4 의 정체성이 "Type-1 hypervisor" 에서
"SEV-SNP guest + SVSM" 으로 바뀜.  architecture.md 의 핵심 설계 (게스트
호스팅, lease scheduler 의 게스트 위 동작 등) 가 모두 다시 그려져야 함.
재정의 비용 큼, 그러나 "verified-equivalent + AMD-V" 양립의 가장
현실적인 길일 수 있음.

🚨 **본 작성 시점 사용자 호스트에서 SEV-SNP 사용 불가능 확인**:

```
$ lscpu  →  flags = ... svm ... (sev / sev_es / sev_snp / sme 모두 없음)
```

사용자의 현 Ryzen APU 는 일반 desktop/mobile SKU (PRO 시리즈도 SEV-SNP
는 EPYC 9004 + 일부 8000 시리즈 한정) → **SEV-SNP 하드웨어 미보유**.

⇒ E1 경로는 **SEV-SNP 지원 EPYC 또는 SP6 워크스테이션 확보 후**에만
   진입 가능.  현 호스트에서는 (α)/(δ)/(ε) 와 같은 카테고리 — 하드웨어
   투자 또는 emulation.

---

## 카테고리 F — Atmosphere (Verus + Rust 신규 후보, 2026-05-04 추가)

### F1. Atmosphere

- 출처: SOSP '24 발표 — formally-verified microkernel/hypervisor.
  **Rust 작성 + Verus 검증**.  PLOS '25 (Programming Languages and
  Operating Systems workshop) 후속 논문에서 **x86_64 hardware
  virtualization 확장 (AMD-V 포함)**.
- 검증: **Verus** — Y4 가 이미 사용 중인 도구체인과 동일.  Y4 측 50+
  invariant + Atmosphere 측 invariant 가 같은 도구로 통합 가능.
- 라이선스: **확인 필요** (학술 prototype, 보통 MIT 또는 Apache-2.0).
- 활성도: SOSP '24 + PLOS '25 = **현재 활성** (Hyperkernel 의 휴면
  상태 와 대조).
- 가속기 통합: ?  WaveTensor HIU 같은 비표준 device 추가 시 Verus
  명세 확장 — Y4 측 spec 패턴 (`proofs/verus/src/...`) 과 자연 정합.
- AMD Ryzen APU 호환: 사용자 정보로는 **AMD-V 지원** → 현 호스트에서
  즉시 진입 가능.

⇒ **잠재 1 순위 후보 — D1d 결정 재평가 필요**.

#### F1 매력 (D1d 대비)

| 측면 | D1d (seL4 + Y4-VMM) | F1 (Atmosphere base) |
|---|---|---|
| Base verified | seL4 (Isabelle/HOL) | **Atmosphere (Verus)** |
| AMD-V verified | △ Y4-VMM 자체 (Verus) — base 는 무지 | **◎ Atmosphere AMD-V 확장 자체가 Verus 검증** |
| 검증 도구 일관성 | seL4 Isabelle + Y4 Verus — bridge 도구 필요 (verus_to_isabelle.md) | **Verus 단일 — bridge 불필요** |
| Y4 의 Rust ecosystem 정합 | 정합 (seL4 fork 의 C 패치 + Y4-VMM Rust) | **완벽 정합** (Atmosphere 가 Rust) |
| Contribute-back 분량 | seL4 mainline PR (C 패치) + 별도 Verus artifact | Atmosphere upstream PR (같은 언어, 같은 도구) |
| Y4 정체성 영향 | seL4 base 유지 — 변화 X | base 가 seL4 → Atmosphere — Phase B 의 50 invariant 는 의미 그대로, trusted boundary 재정렬 |
| Phase B 작업 재사용 | 100% (seL4 위 그대로) | proofs/verus 의 invariant 그대로, boot/ + kernel/ 의 seL4 의존 부분 재작성 |
| Production 성숙도 | seL4 = 매우 성숙 (15.0.0 release) | Atmosphere = SOSP '24 기준, **학술 prototype 가능성 ↑** |
| 위험 | 알려진 trade-off | **알 수 없음 — 사실 확인 전** |

#### F1 사실 확인 필요

1. **공식 repo URL** — 사용자 추가 정보 요청
2. **라이선스**
3. **PLOS '25 AMD-V 확장 코드의 머지 상태** (mainline / branch / fork)
4. **Atmosphere 의 가속기 / 비표준 PCIe device 통합 가능성**
5. **production 성숙도** (회귀 스위트 / 사용 사례)
6. **SMP 지원** (Y4 의 C2 SMP-first 와 호환?)
7. **guest OS hosting** — Linux / BSD / Alpine 부팅 가능?

위 7 항목 확인 후 **D1d 또는 F1 채택** 결정.  Phase B 작업 재사용도가
F1 의 핵심 결정 요인 — Atmosphere 위에서 Y4 의 capsules / ipc / alloc
이 재컴파일 정도로 재사용 가능하면 F1 의 매력 ↑.

#### Phase D 의 R-α / R-γ / S9 호환

S9 의 nested 차단 + auto-redirect 정책 (R-α/R-γ) 은 Atmosphere base
에서도 그대로 유의미.  D1d 의 14 안전장치 + S14 + S9.4 hook 가 재사용
가능 (단 syscall ABI 의 cap-typed 객체 이름이 Atmosphere 의 ABI 에
맞춰 변경).

#### D1d sign-off 영향

amdv_safety.md 의 sign-off review 는 **F1 사실 확인 후 결정**까지 보류.
- F1 채택 시: amdv_safety.md 의 14 안전장치 catalog 는 Atmosphere base
  로 ABI 만 재정렬 — content 는 90%+ 재사용
- D1d 유지 시: 현 review 재개

S11–S14 review 는 F1 결과 확정 후 재개.

---

## 평가 매트릭스

| 후보 | AMD-V | verified | 라이선스 OK | Y4 정합 |
|---|:---:|:---:|:---:|:---:|
| A1 seL4 mainline (현재) | ✗ | ◎ (base) | ◎ | ◎ |
| A1' seL4 + 자체 SVM 패치 | ◎ (작성 시) | ✗ (변경 부분) | ◎ | ○ |
| B1 CertiKOS | ✗ | ◎ | △ | ✗ |
| B2 Verisoft | ✗ | ◎ | △ | ✗ |
| **B4 Hyperkernel** | **◎ (svm.c / vmx.c / intel_iommu.c 확인됨)** | **◎ (Z3 push-button, LLVM IR)** | **◎ Apache-2.0 (UW 자체) — 외부 reuse 부분은 mixed** | **△ (휴면 + LLVM 5.0/Z3 4.5 구도구 의존 + xv6 파생 → 자체 fork 비용 큼)** |
| C1 Xen | ◎ | ✗ | ✗ (GPL — exemption 미확인) | ✗ |
| C2 Bhyve (reference) | ◎ | ✗ | ◎ | ○ (포팅 ref) |
| C3 NVMM (reference) | ◎ | ✗ | ◎ | ○ (포팅 ref) |
| D1 Y4-VMM 자체 + Verus 부분 | ◎ | △ (부분) | ◎ | ◎ |
| **D2 NOVA (재평가)** | ◎ | △ | **◎ (hypercall API exemption 으로 GPL 회피)** | ○ (Y4 가 "userland" 됨, base verified 손상) |
| D4 Muen | ✗ (사용자 확인) | △ (SPARK) | ✗ (GPL) | ✗ |
| **E1 SEV-SNP + SVSM** | ◎ (silicon) | ◎ (silicon attestation + SVSM 자체 검증) | ◎ | △ (Y4 정체성 재정의 필요) |
| **F1 Atmosphere (SOSP '24 / PLOS '25)** | **◎ (사용자 정보 — 사실 확인 필요)** | **◎ (Verus, Y4 와 동일 도구)** | **? (확인 필요)** | **◎ (Rust + Verus 완벽 정합) — 단 production 성숙도 확인 필요** |

---

## 결과 요약 (사용자 입력 + lscpu + Atmosphere 추가 반영)

**현 시점 5 개의 길:**

| 길 | 핵심 | base verified | AMD-V | Y4 정체성 | 차단 |
|---|---|:---:|:---:|---|---|
| **1. seL4 + Y4-VMM 자체 (D1)** | bhyve/NVMM 알고리즘 reference, Verus 부분 검증 | ◎ (seL4) | ○ (Y4-VMM 안) | 보존 | 작업 분량 큼 |
| **2. NOVA + Y4 (D2 재평가)** | NOVA 의 hypercall API exemption 으로 GPL 회피, Y4 가 NOVA 위 userland | ✗ (NOVA unverified) | ◎ | "userland" 로 강등 — Type-1 정체성 손상 | base verified 손상 |
| **3. Hyperkernel (B4 사실 확인 완료)** | Z3 push-button + svm.c/vmx.c/intel_iommu.c 모두 존재 + Apache-2.0 | ◎ (Z3 SMT, LLVM IR) | ◎ (UW 자체) | xv6 파생 microkernel — 정체성 변경 (seL4 → hv6 derivative). **휴면 상태 + LLVM 5.0/Z3 4.5 구도구**가 큰 trade-off |
| **4. SEV-SNP + SVSM (E1)** | silicon 보안 + SVSM 자체 작성 | ◎ (silicon attestation) | ◎ | "SNP guest + SVSM" 으로 재정의 | **현 호스트 미지원** — EPYC 등 별도 하드웨어 |
| **5. Atmosphere base (F1 — 신규, 사실 확인 필요)** | Rust + Verus + AMD-V 통합 (PLOS '25) | ◎ Verus | ◎ (PLOS '25) | base 가 seL4 → Atmosphere, Phase B invariant 재사용 가능 | **production 성숙도 + 라이선스 + repo 활성도 사실 확인 필요** |

**Phase E 인증 트랙 측면:** 길 1 / 3 / 4 는 verified-equivalent 유지.
길 2 는 "AMD-V 우선 + verification 야심 일시 포기" 의 명시 트레이드.

**현 호스트 (AMD Ryzen APU, no SEV-SNP) 에서 즉시 가능한 길:**
- 길 1 — D1, **현 호스트에서 즉시 진입 가능**, 하드웨어 무관.
- 길 2 — NOVA on AMD-V, **현 호스트에서 즉시 진입 가능**.
- 길 3 — Hyperkernel, **현 호스트에서 진입 가능하지만 휴면 상태 +
  구도구 의존 + xv6 파생** 의 부담 큼 — 채택 시 self-fork 책임.
- 길 4 — **차단 (호스트 미지원)** → SEV-SNP EPYC 보드 / 서버 확보 후.
- 길 5 — Atmosphere, **사용자 정보로는 즉시 가능**.  사실 확인 후
  결정.  Verus 완벽 정합으로 Y4 의 매력 가장 높음.

추가로 "개발은 emulation, production 은 별도 호스트" 분리 옵션:
- (δ) QEMU TCG 위에서 G1–G3 PoC.  길 1/2/3 모두에 직교 가능 — 코드는
  AMD-V API 로 작성, TCG 가 SVM 명령어를 emulate.

---

## 사용자 결정 입력 + 차단 의존

본 메모는 사용자 결정의 **사전 자료**.  결정에 필요한 추가 사실
확인:

1. ~~**Hyperkernel** repo / 라이선스 / AMD-V 코드 트리~~ — ✅ 완료.
   휴면 + 구도구 의존 부담 확인.
2. **SEV-SNP 지원 호스트 확보 비용 / 일정** — 길 4 의 진입 시점 결정.
3. **bhyve / NVMM 의 SVM 핸들링 코드 분량** — 길 1 의 분량 추정.
4. **NOVA 위에서 lease capability + HIU 통합 가능성** — 길 2 의
   architecture 영향 평가.
5. **Atmosphere repo / 라이선스 / AMD-V 코드 / SMP / guest hosting /
   가속기 통합 가능성** — **길 5 의 viability 직접 검증 필요** (가장
   유력한 신규 후보).

위 5 항목 추가 조사 결과를 본 메모에 누적해 결정 시점에 활용.

---

## 추가 확인 필요 (메모로 남김)

- **NOVA 의 GPL 정확 적용 범위** — NOVA core 만 GPL 이고 hypercall
  ABI 위 layer 는 다른 라이선스일 수 있음. PR 회피 가능성?
- **Muen 의 AMD-V 실제 지원 상태** — SPARK 코드 트리 직접 확인 필요.
- **2025–2026 신규 verified hypervisor 발표** — OSDI/SOSP/ASPLOS
  최근 proceedings 훑어보면 새로운 후보가 있을 수 있음.  본 메모는
  *현 시점 인지 범위* 한정.
- **AMD APM Vol 2 §15 의 SVM 표면 크기** — VMCB 필드 수, exit code
  종류, MSR permissions, IO permissions 등.  D1 의 분량 추정에 직접
  영향.
- **seL4 의 KernelVerificationBuild 와 KernelVTX 동시 ON 가능 여부** —
  코드 보면 `KernelVTX` 가 `NOT KernelVerificationBuild` 의존.  즉
  seL4 verified 빌드는 VTX 미포함.  AMD 도 같은 한계 — verified
  base 와 hypervisor 기능은 seL4 정책상 분리되어 있음.

---

## 결론 (메모용 — 결정은 추후)

본 조사 결과로 미루어, **"완전 verified + AMD-V" 는 현 단계에서
타협 없이 양립 불가**.  세 길 중 골라야 함:

1. verified base 보존 + Y4-VMM 부분 verified  ← Y4 정체성 유지에 최적
2. AMD-V 우선 + base verified 포기 (예: Xen/NOVA 채택, GPL 감수)
   ← Y4 의 "verified base" 원칙 손상
3. Intel 호스트 + 현 seL4 그대로  ← base 무손실, 하드웨어 비용

길 1 의 의미: Phase E 인증 트랙 시 "seL4 microkernel 은 fully verified,
그 위 Y4 specialization 은 Verus 로 부분 verified, AMD-V 핸들링은
Verus 부분 verified" 로 표기.  의료/항공 인증에서 부분 verified 도
대부분 인정 (full verification 은 자체 가치이지 인증 요구가 아님).

길 2 의 라이선스 회피 path: Xen/NOVA 의 hypervisor 코어를 직접
link 하지 않고 capsule binary boundary 두기 → Y4 main tree 는 깨끗.
정책상 가능하지만 "verified base" 원칙은 깨짐.

길 3 은 단순.  AMD vs Intel 의 가속기 워크로드 차이 / WaveTensor 보드
호스트 선택과 결합해서 결정해야 함.

---

⚠ **본 메모는 추가 조사 필요** — 위 "추가 확인 필요" 4 항목.  특히
2025–2026 의 신규 verified hypervisor 발표 (예: 학술 학회 + 산업
컨소시엄) 가 본 매트릭스를 바꿀 수 있음.  본 메모는 *작성 시점의
인지 한계* 를 명시해 두고, 갱신 의무 는 본 메모 활성화 시점 (게스트
호스팅 진입 직전) 에 다시.

---

## 결정 (2026-05-04 확정)

**길 1 (D1) 채택:** seL4 (verified base) 유지 + Y4-VMM 에서 AMD-V 자체
구현 + Verus 부분 검증. bhyve / NVMM (BSD-2) 알고리즘 reference.

### D1 sub-decision: privileged 코드 위치

AMD SVM 명령어 (vmrun / vmload / vmsave / clgi / stgi) 는 ring-0
전용 — root task 에서 inline asm 으로 직접 발화 불가.  세 sub-option
중:

| sub-option | 형태 | 분량 |
|---|---|---|
| **(D1a) seL4 에 raw SVM cap 추가** | 새 syscall (예: `seL4_X86_VCPU_RunSVM`) 만 추가, VMM 로직은 모두 Y4-VMM | 작-중 |
| (D1b) seL4 에 full SVM VCPU object | 기존 Intel `VCPU` cap 의 AMD 버전, ~수천 LoC seL4 측 추가 | 큼 |
| (D1c) seL4 우회 | cap 모델 손상, 비추천 | — |

**(D1a) 권고** — Y4 측 작업 비중 최대 + seL4 측 변경 최소.

### Contribute-back 경로

길 1 의 부산물로 seL4 mainline 에 기여 가능:

| 산출물 | seL4 채택 가능성 |
|---|---|
| AMD-V C 코드 (D1a 의 raw SVM syscall, 또는 D1b 의 full VCPU C 구현) | ◎ 가능 — seL4 PR 형태 |
| Verus 증명 | △ — seL4 의 Isabelle/HOL 과 직접 호환 안 됨. 별도 artifact 로 게시 |
| VMCB state machine spec | ○ — 언어 무관 reference, seL4 팀이 Isabelle 으로 재증명 가능 |
| Bug fix | ◎ |

전략: D1a 의 raw-SVM C 패치를 PR.  Verus 증명은 별도 artifact + 논문 +
Y4 repo 링크.  의의: **"AMD-V 가 push-button verification 가능"** 신호를
seL4 팀에 제공 → seL4 팀의 Isabelle 트랙 follow-up 명분.  Y4 가 첫 발걸음.

### Phase plan 영향

- **Phase C 의 게스트 호스팅 항목**: VMM 위치 = `y4-hypercall` repo
  (확정), AMD-V 핸들링 = D1a 패턴.
- **차단 의존**: seL4 측에 작은 raw-SVM syscall 패치 (D1a) — Y4-VMM 진입
  전에 처리.
- **분리된 워크스트림**:
  1. seL4 측 SVM syscall 패치 (~수백 LoC, C, seL4 빌드 시스템 안)
  2. Y4-VMM Rust 구현 (`y4-hypercall` repo, VMCB / NPT / VMEXIT 처리)
  3. Verus 증명 (state machine + 핵심 invariant)
  4. bhyve/NVMM 알고리즘 reference 분석 (별도 reading)
- **하드웨어**: 현 AMD Ryzen APU 에서 즉시 진입 가능.  SEV-SNP 미지원
  은 무관 (E1 경로 차단된 채로 유지).

### 채택하지 않은 길의 처리

- **길 2 (NOVA)**: 라이선스는 깨끗하지만 verified-base 손상이 trade-off
  로 컸음.  D1 가 가능한 한 미채택.
- **길 3 (Hyperkernel)**: 휴면 + 구도구 + xv6 파생의 self-fork 부담이
  컸음.  D1 가 가능한 한 미채택.  단 Hyperkernel 의 Z3 push-button
  패턴 자체는 Verus 증명 설계 시 영감 자료로 가치 있음.
- **길 4 (SEV-SNP/SVSM)**: 호스트 차단 + Y4 정체성 재정의 비용 컸음.
  미채택.  단 Phase E 인증 트랙 진입 시점에 EPYC 호스트 + SEV-SNP +
  TDISP 가속기 통합 워크스트림 으로 별도 검토 가능.

---

## 갱신 이력

- **첫 작성** (2026-05-04): 카테고리 A–D 초안.  Hyperkernel 을 단순 OS
  로 분류 (오류).  NOVA 를 GPL 라이선스 차단으로 분류 (오류 — exemption
  미확인).  Muen AMD-V 지원 미확인.
- **사용자 입력 + 확인 반영** (2026-05-04): NOVA 의 hypercall API
  exemption 확인 (LICENSE 파일 직접 fetch).  Muen AMD-V 미지원 확인.
  Hyperkernel 재평가 1순위 후보로 격상 (사용자 정보).  카테고리 E
  (SEV-SNP + SVSM) 신규 추가.  사용자 호스트 lscpu 에서 SEV-SNP
  미지원 확인 → E1 즉시 진입 차단 추가.  최종 4 길 매트릭스로 결론
  재구성.
- **Hyperkernel 사실 확인** (2026-05-04, 사용자 제공 repo URL):
  - Apache-2.0 (UW 자체 작성), AMD SVM + Intel VT-x + VT-d 코드 모두
    존재 (`kernel/svm.c` / `kernel/vmx.c` / `kernel/intel_iommu.c`).
    Z3 + LLVM IR push-button.  사용자 정보 사실 확인.
  - **그러나** 휴면 상태 (master 7 commits, 0 release, 1 issue, 0 PR),
    LLVM 5.0.0 + Z3 4.5.0 구도구 의존 (현재 LLVM 22.x).  README 가
    "playground, don't use for production" 명시.  xv6 파생.
  - ⇒ Y4 채택 시 self-fork + 도구체인 modernization + xv6 base 의
    SMP/현대 OS 기능 확장 책임. 큰 부담.
- **결정 확정** (2026-05-04, Hyperkernel 사실 확인 후): **길 1 (D1) +
  sub-option D1a 채택**.  seL4 측에 raw-SVM syscall 패치 (소규모) +
  Y4-VMM Rust 구현 + Verus 부분 증명.  contribute-back 경로: D1a 의 C
  패치는 즉시 PR 가능, Verus 증명은 별도 artifact.  자세한 결정
  근거는 위 "결정 (2026-05-04 확정)" 섹션.
- **D1d 로 sub-option 강화** (2026-05-04): D1a + 14 안전장치 카탈로그
  (S1–S14) 의무 적용.  spec: `Y4/docs/amdv_safety.md`.  fork 호환성
  contract: `Y4/docs/sel4_fork_policy.md` (Strictly Additive Fork).
  Verus → Isabelle/HOL 번역기: `Y4/docs/verus_to_isabelle.md`.
- **카테고리 F (Atmosphere) 신규 추가** (2026-05-04, 사용자 정보):
  Verus + Rust + AMD-V (PLOS '25 확장) 의 잠재 1 순위 후보.  **D1d
  결정 재평가 트리거**.  amdv_safety.md sign-off review 일시 보류.
- **사실 확인 결과** (2026-05-04, mars-research/atmosphere + microsoft/
  verismo + DOI 10.5555/3691938.3691970 직접 확인):
  - mars-research/atmosphere = SOSP **'25** artifact, Verus + Rust +
    MIT, 562 commits, **AMD-V 코드는 publicly 어디에도 없음** (24 brs
    + sosp25 artifact tree 모두 svm/vmcb/vmrun 0)
  - microsoft/verismo = USENIX OSDI '24 Best Paper, "VeriSMo: A Verified
    Security Module for Confidential VMs", **SEV-SNP 전용 SVSM** (AMD-V
    hypervisor 아님), 2022-2024 prototype, latest verus 와 호환 X
  - "PLOS '25 Verified Isolation for AMD-V SVM in VeriSMo" = DBLP/arXiv
    /USENIX 어디에도 부재 — 다른 AI 출처의 환각 가능성 ↑
  - 결론: F1 (Atmosphere AMD-V) / E1' (VeriSMo) 모두 직접 채택 X.
- **VMM 아키텍처 원점 재설계 — ARCH-II' 채택** (2026-05-04, 사용자
  트리거):
  - D1d 의 monolithic VMM 폐기, **capsule 분해 + VeriSMo 검증 기법
    영감** 으로 대체.  새 디자인 doc: `docs/vmm_arch.md`.
  - 구성: 10 capsule (vmcb / npt / msr-bitmap / io-bitmap / firmware-
    approval / cpuid-emul / npf-handler / audit / nested-request /
    lifecycle) + thin orchestrator (~수백 LoC).
  - S1–S14 안전장치 content 는 그대로 보존, capsule 매핑만 재정렬
    (`docs/vmm_arch.md` §4).
  - VeriSMo 의 **2-layer concurrency 증명 기법** 차용 (코드 import 0,
    영감만, OSDI '24 paper attribution).
  - AMD-V 알고리즘 reference = bhyve / NVMM (BSD-2).
  - 학술 차별점: seL4 + Y4 capsule pattern + VeriSMo 기법 통합 첫
    사례.
