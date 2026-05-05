---
name: y4-drivers + y4-hypercall + 게스트 호스팅 계획 (2026-05-04)
description: 별도 repo y4-drivers / y4-hypercall, 첫 driver matrix, ACPI/네트/USB/FS 도구 채택, AMD-V 차단 이슈, KernelVTX sub-options.
type: project
originSessionId: 78ff80c3-5421-425a-9e23-3da166ef2bb9
---
## y4-drivers (별도 repo, 로컬 시작)

- **위치:** `/home/ybi/y4-drivers/` (시작), 추후 push.
- **라이선스:** Apache-2.0 main + GPL-capsule mixed (Y4 의 `licensing.md`
  §"Linux driver tier" 정책 적용).
- **워크스페이스:** cargo workspace, **멤버별 driver 1 크레이트**.
- **Naming:** `y4-driver-<name>` (예: `y4-driver-virtio-net`).
- **Mock fixtures:** 별도 멤버 `y4-driver-test-fixtures` — `MockMmio` /
  `MockDma` / `MockIrq` trait 묶음 공유.
- **SPDX 자동 검증:** git pre-commit 에 SPDX 헤더 검사 추가.
- **ISO 동봉 방식:** **동적 로딩**. boot ISO 에 driver binary 가 정적
  포함되지 않고, Y4 가 부팅 후 device discovery → 필요한 driver 만
  load. 인프라는 `kernel/` 의 module loader (Phase B 후속).
- **Y4 의존:** `y4-drivers` 가 `y4-capsules` / `y4-alloc` / `y4-ipc` 를
  git dep 으로 참조. 단방향. Y4 측은 driver 의 존재를 모름.

## 첫 driver matrix (1순위 PR 범위)

"지원 목표" 와 "1차 PR 범위" 분리. **1차 PR**:
- `y4-driver-virtio-net` (`virtio-drivers` crate 이식)
- `y4-driver-virtio-blk` (동일 crate)
- `y4-driver-e1000e` (DragonFly port reference)
- `y4-driver-ahci`
- `y4-driver-nvme` (NVMe 1.x — 2.x 는 장기)
- `y4-driver-xhci` (USB 3.x — `xhci` crate 활용; USB4/Thunderbolt 는 장기)

**장기 로드맵 (현 PR 범위 외):**
- USB4 / Thunderbolt 4 / Alt mode (CIO router, NHI, sec policy)
- NVMe 2.x (zoned namespaces, KV, computational storage)
- Wi-Fi 802.11 a/b/g/n/ac/ad/ax/be — 칩셋 1 개 (iwlwifi 또는 mt76) 부터
- 이동통신 LTE/5G — Quectel 류 USB QMI dongle 1 개 부터

## 상위 stack 도구 채택

| 영역 | 채택 |
|---|---|
| ACPI | `acpi` + `aml` crates (Redox 사용, MIT) 이식 |
| L2/L3 networking | `smoltcp` (BSD-0) — 별도 멤버 `y4-net-smoltcp` |
| USB host | `xhci` crate 등 활용 |
| Filesystem matrix | (아래 별도 표) |

## 파일시스템 매트릭스

| FS | 출처 |
|---|---|
| ext2/3/4 | `ext4-rs` 이식 |
| UFS (FFS2), FAT12/16/32, NFS (server + client) | DragonFlyBSD 또는 NetBSD 드라이버 이식 |
| HAMMER, HAMMER2 | DragonFlyBSD 드라이버 이식 |
| NTFS, exFAT | NetBSD puffs 의 `librefuse` (FUSE 호환 레이어) 이식 + 그 위에 |
| ZFS | FreeBSD 또는 NetBSD 드라이버 이식 |
| ISO 9660 | DragonFly/FreeBSD/NetBSD 중 이식, 또는 더 나은 대안 모색 |
| UDF, NILFS | NetBSD 드라이버 이식 |
| 기타 | librefuse 또는 게스트 측에서 처리 |

## 게스트 호스팅 (VMM 위치 — ARCH-II' 채택 후 재정의 2026-05-04)

- **Core VMM 위치:** Y4 워크스페이스 내부 — `Y4/vmrun-orchestrator/` (thin
  ~수백 LoC orchestrator) + `Y4/capsules/` 의 10 신규 capsule (vmcb /
  npt / msr-bitmap / io-bitmap / firmware-approval / cpuid-emul /
  npf-handler / audit / nested-request / lifecycle).
- **`y4-hypercall` repo 재정의:** **사용자측 CLI / 도구 repo**.
  Phase D 의 R-α (`/dev/kvm` ioctl 프록시) + R-γ (paravirt agent) +
  S14 firmware-approval CLI 가 여기. core VMM 코드 X.
- **ISO 옵션:** **(γ) Y4 가 호스트, Linux 가 별도 디스크 partition 의 게스트.**
- **첫 게스트 OS:** Alpine Linux mini-rootfs.
- **G-multiboot:** Alpine 의 mini-bootloader 를 게스트 안에 포함.
- **G-disk / G-network / G-display / G-installer:** G3 통과 후 결정 (지금
  연기). 첫 PoC 는 ramdisk + serial console 로 진행.
- **자체 드라이버 게스트 사용 보장:** Y4 의 driver 범위 트리밍을
  보완하는 요건 — 게스트가 자기 PCIe driver 로 hardware 직접 접근. 이
  보장은 IOMMU passthrough + per-device BAR cap + IRQ remap + VFIO-eq
  API 가 모두 갖춰진 **Phase D 후반**에 성립. 그 전 (Phase B/C 첫
  게스트) 는 **virtio paravirtualization only** — 명시 마일스톤으로
  phase_plan 에 박을 것.

## seL4 KernelVTX sub-options 결정

`boot/x86_64-debug.cmake` 에 추가:

| 옵션 | 값 |
|---|---|
| `KernelVTX` | ON |
| `KernelMaxVCPUsPerVM` | 호스트 vCPU 수 |
| `KernelHugePage` | ON (현재 OFF — 변경. seL4 verification 호환 재확인 필요) |
| `KernelIOMMU` | ON |
| `KernelFPU` | xsave |

## AMD-V (SVM) 결정 — ARCH-II' 채택 (2026-05-04 갱신)

**채택: 길 1 + ARCH-II'** — D1a (seL4 raw-SVM syscall) +
**capsule-decomposed VMM** (10 capsule + thin orchestrator) + Verus
부분 증명 + VeriSMo 의 **2-layer concurrency 증명 기법 영감** (코드
import 0).
spec: `Y4/docs/amdv_safety.md` (S1–S14 안전장치) + `Y4/docs/vmm_arch.md`
(capsule 분해 디자인).

**fork 호환성 contract:** Strictly Additive Fork. upstream seL4 회귀 0
fail gate. `Y4/docs/sel4_fork_policy.md`.

bhyve / NVMM (BSD-2) 알고리즘 reference.

**근거 요약 (3 단계):**
- 1차 4 길 비교 (D1 / NOVA / Hyperkernel / SVSM): D1 이 verified-base
  보존 + 현 호스트 즉시 진입 + Y4 정체성 보존 만족.
- 2차 사실 확인 (2026-05-04): Atmosphere AMD-V 코드 publicly 0 (24
  branch + sosp25 artifact tree 모두 svm/vmcb/vmrun 0). VeriSMo 는
  SEV-SNP 전용 SVSM (hypervisor 아님). "PLOS '25 Verified Isolation
  for AMD-V SVM in VeriSMo" 논문 검색 0 — 다른 AI 출처의 환각 가능성
  ↑.
- 3차 원점 재설계 (사용자 트리거): D1d 의 monolithic VMM 폐기.
  ARCH-II' = capsule 분해 (TCB 분산) + VeriSMo 2-layer concurrency
  기법 영감 (코드 import 0, 기법만 paper attribution).

**워크스트림 (이 순서로 완료 후 y4-drivers / capsules 진입):**
1. **`docs/amdv_safety.md` v1.0 frozen** — S1–S14 + VMCB whitelist +
   PR split. 현 v0 draft, S1–S10 sign-off 끝, S11–S14 + §4/§6 미정.
2. **`docs/vmm_arch.md` v1.0 frozen** — ARCH-II' 디자인 sign-off. 현
   v0 draft.
3. **`docs/sel4_fork_policy.md` v1.0 frozen** — Strictly Additive Fork.
   현 v0 draft.
4. **`docs/verus_to_isabelle.md` v1.0 frozen + 도구 구현** —
   statement-only `sorry` + `axiom` opt-in hybrid. 별도 repo
   `y4-verus2isabelle` (Apache-2.0). ~1500 LoC Rust. 5–6 와 병렬.
5. **seL4 측 D1a C 패치** (`CONFIG_Y4_AMDV` gate, default OFF). 4 객체
   + 6 syscall + S2–S9 의 microkernel 측 검사. ~수백 LoC.
6. **Y4 측 vmrun-orchestrator + 10 capsule** (`Y4/vmrun-orchestrator/`
   + `Y4/capsules/`) + Verus 명세 (S1–S14 본문) + VeriSMo 영감 2-layer
   concurrency 증명.
7. **`y4-hypercall` 사용자 CLI repo**. R-α / R-γ / firmware-approval CLI.

**Contribute-back 경로:**
- D1a 의 raw-SVM C 패치 → seL4 mainline PR.
- Verus 증명 → 별도 artifact + Isabelle skeleton 자동 export
  (`y4-verus2isabelle` 도구 사용).

**현 호스트에서 즉시 진입 가능 — 하드웨어 비용 0.** SEV-SNP 미보유 무관.

**자세한 분석:** `Y4/.claude-notes/amd-v-verified-survey.md`

## Phase D auto-redirect 전략 (확정 2026-05-04)

S9 (nested SVM 차단) 의 Linux 게스트 워크로드 영향:

| 워크로드 | 영향 |
|---|---|
| Docker (runc), Podman (crun), Buildah, gVisor, DinD, rootless Podman | **영향 없음** (namespace + cgroup 만 사용) |
| **Waydroid** | **영향 없음** — LXC 기반, nested virt 무관. GPU 가속만 Phase D passthrough 의존 |
| VirtualBox / VMware / KVM-in-guest / Firecracker / Kata / Android emulator | **차단**, Phase D 의 auto-redirect 로 흡수 |

Phase D 마일스톤 (확정):
- **R-α `/dev/kvm` ioctl 프록시** (1순위) — KVM-기반 도구 (QEMU /
  Firecracker / Kata / 최신 VBox / Android emulator) 의 KVM ioctl 을
  vsock 으로 Y4-VMM 에 forward → sibling SVMVCPU 생성. S1–S13 자동
  상속. ~1.5k LoC Y4-측 + ~0.5k LoC guest 측 kernel 모듈.
- **R-γ paravirt agent** (2순위) — legacy VBox vboxdrv / VMware vmmon
  같이 KVM 미사용 매니저용 wrapper agent + paravirt sibling-VM-create
  API. 매 매니저별 어댑터.
- **GPU passthrough** — Waydroid 등 그래픽 워크로드 가속.

**비채택:** R-β (nested SVM trap-and-forward) — verification 표면 두
배 + S9 정체성 손상. 명시적 비채택.

자세한 분석: `Y4/docs/amdv_safety.md` §S9.

## 진행 가능 작업 (즉시 unblock)

1. `y4-drivers` repo scaffold (위 결정대로)
2. `y4-driver-test-fixtures` 멤버 + `MockMmio`/`MockDma`/`MockIrq` trait
3. 첫 driver: `y4-driver-virtio-net` (virtio-drivers crate 이식)
4. capsules `kernel/` 비의존 5 개 (PCIe BAR → cap walker → MSI-X
   discovery → fault recovery → 자원 transfer)
5. ACPI / smoltcp / xhci 채택 결정 영구화

## 차단 작업 (결정 대기)

- 게스트 호스팅 진입 (AMD-V 결정 필요)
- KernelHugePage = ON 으로 cmake 변경 (verification build 호환 재검토)
- IOMMU programming capsule (kernel/ MMIO map 의존)
