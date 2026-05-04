<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 ↔ HIU ABI (v0 — draft, not frozen)

본 문서는 Y4 가 WaveTensor 가속기의 HIU 모듈에 의존하는 **유일한 ABI 표면**
을 정의한다. 본 문서가 동결(frozen)되어야 Y4 의 mock-HIU 구현
(Phase B) 과 lease capability 런타임 (Phase C) 이 시작될 수 있다.

용어와 RTL 출처는 [`./glossary.md`](./glossary.md) 를 참조. 본 문서는 그
사전을 기반으로 한 단계 위 — **호스트가 보는 레지스터 맵 + 시간 계약 +
동결 정책** — 을 명시한다.

---

## 0. 동결 정책

- 본 ABI 의 동결(`v1.0 frozen`) 은 **Y4 측 + WaveTensor 측 양쪽
  sign-off** 로만 발생한다. 둘 중 한 쪽 단독 변경은 무효.
- 동결 후 변경은 다음 둘 중 하나:
  - **`v1.x` patch** — backwards-compatible 추가만 가능 (예: reserved 슬롯
    채우기, 새 health flag 비트 추가).
  - **`v2`** — incompatible 변경. mock 과 RTL 양쪽이 동시 갱신되어야 함.
- v0 은 draft. 본 단계에서는 누구든 변경 가능, 변경 시 commit message 에
  `hiu-abi: <change summary>` prefix 필수.

> **현재 상태:** v0 — draft. 양측 sign-off 전.

---

## 1. MMIO 레지스터 맵 (호스트 측 view)

HIU 의 RTL 포트(`virt_addr`, `dma_req`, `chacha_*`, `shadow_write_*` 등)는
**호스트 측에서 PCIe BAR 의 단일 4 KiB 페이지**로 직렬화된다. 한 페이지에
모두 들어가도록 슬롯을 배치 — IOMMU page-grain 격리와 정합.

베이스 오프셋은 `BAR0 + 0x0000`. 주소는 little-endian.

| Offset | 폭 | 이름 | 접근 | 의미 |
|--------|---:|------|:----:|------|
| `0x000` | 64 | `HIU_VIRT_ADDR` | RW | 다음 DMA 의 `virt_addr` 입력 |
| `0x008` | 32 | `HIU_DMA_DATA_IN` | RW | 다음 DMA write payload |
| `0x00C` | 32 | `HIU_DMA_DATA_OUT` | RO | 직전 DMA read payload (`dma_ack` cycle 에 latch) |
| `0x010` | 32 | `HIU_CTRL` | RW | bit0=`dma_req`, bit1=`ats_req`, bit2=`pri_req`, bit3=`shadow_write_en`, bit4=`context_switch`, bit[7:6]=`partition_id`, bit[9:8]=`interface_type` |
| `0x014` | 32 | `HIU_STATUS` | RO | bit0=`dma_ack`, bit1=`access_granted`, bit2=`ats_resp`, bit3=`pri_resp`, bit31=`trng_unhealthy` |
| `0x018` | 64 | `HIU_SHADOW_START` | WO | 다음 SHADOW_UPDATE 의 `shadow_write_start` |
| `0x020` | 64 | `HIU_SHADOW_END` | WO | 다음 SHADOW_UPDATE 의 `shadow_write_end` |
| `0x028` | 32 | `HIU_SHADOW_INDEX` | WO | bit[3:0]=`shadow_index ∈ [0,15]` |
| `0x040` | 256 | `HIU_CHACHA_KEY` | WO | XChaCha20 256-bit key (8 × 32-bit) |
| `0x060` | 192 | `HIU_CHACHA_NONCE` | WO | XChaCha20 192-bit nonce (6 × 32-bit) |
| `0x080` | 64 | `HIU_PHYS_ADDR` | RO | 직전 TRANSLATE 결과 (마스킹 전) |
| `0x100` | 8 | `HIU_RNG_RAW` | RO | TRNG Phase 1 8-bit raw byte (read clears `valid`) |
| `0x101` | 8 | `HIU_RNG_RAW_STATUS` | RO | bit0=`rng_raw_valid` |
| `0x108` | 64 | `HIU_RNG_WORD` | RO | TRNG Phase 3 64-bit conditioned word |
| `0x110` | 8 | `HIU_RNG_WORD_STATUS` | RO | bit0=`rng_word_valid` (read-to-clear), bit1=`trng_unhealthy` (sticky) |
| `0xFFC` | 32 | `HIU_ABI_VERSION` | RO | `(major << 16) | minor`, v0 = `0x0000_0000`, v1.0 frozen 시 `0x0001_0000` |

**예약 영역:** 본 표에 없는 모든 오프셋은 reserved, write 무시 / read 0.

**access mode 약자:** RW = read+write, RO = read-only, WO = write-only
(read 시 0 또는 마지막 write 값 — 미정의).

**v0 미해결:**
- `HIU_CHACHA_KEY/NONCE` 가 capability-protected 영역인지 (Y4 가 게스트
  에게 노출하지 않음), 별도 BAR 로 분리할지.
- ATS/PRI 결과 데이터(요청 ID, completion code) 의 실제 폭 — 현 RTL 은
  resp 단일 비트만 노출.

---

## 2. 시간 계약 (timing contracts)

### 2.1 DMA 단발 사이클

호스트가 `HIU_VIRT_ADDR` + `HIU_DMA_DATA_IN` + `HIU_CTRL.dma_req=1` 을
순서대로 write 하면 HIU 는 다음을 보장한다:

1. `dma_req` 캡처 직후 1 cycle 안에 IDLE → TRANSLATE 천이.
2. TRANSLATE → DUMMY_DELAY (`DUMMY_CYCLES=8`) → CHACHA_H_INIT → ... →
   CHACHA_FINAL → DMA_EXEC → GRANT_ACCESS → IDLE.
3. `dma_ack` 펄스는 DMA_EXEC cycle 에 1 cycle hi.
4. `access_granted` 는 DMA_EXEC 에서 hi, GRANT_ACCESS 끝에 lo.

전체 latency (TLB hit 기준): `1 + 8 + 10 + 10 + 1 + 1 + 1 ≈ 32 cycle`.
TLB miss + shadow hit 시 동일 — TRANSLATE 가 항상 DUMMY_DELAY 로 진행하므로
**상수시간** 보장.

### 2.2 `context_switch` atomicity

`HIU_CTRL.context_switch = 1` write 는 **동일 사이클**에:
- 모든 4 × 16 partitioned TLB entry 의 `valid=0`.
- 모든 16 shadow region 의 `valid=0`.
- FSM 강제 `FLUSH_TLB` 진입 (현재 상태 무관).

호스트 측 의무: write 직전·직후 키/논스 갱신을 **하나의 lease-rotation
시퀀스로 묶어** 발행 — 자세한 순서는 `lease_capability.md` §atomic-rotate.

### 2.3 SHADOW_UPDATE

`HIU_SHADOW_START`, `HIU_SHADOW_END`, `HIU_SHADOW_INDEX` 셋업 후
`HIU_CTRL.shadow_write_en=1` write → 1 cycle 후 등록 완료. 다음 사이클부터
TRANSLATE fallback 에 반영.

`shadow_index ∈ [0, 15]` 외 값은 RTL 에서 무시(`{1'b0, shadow_index} < 5'd16`
체크) — 호스트는 이 동작에 의존하지 않고 자체 검증한다.

### 2.4 TRNG 채널

- `HIU_RNG_RAW`: read 시 현재 byte 반환, `rng_raw_valid` 클리어. 다음 valid
  까지 평균 ~16 cycle (8 RO × von Neumann ~50 % 통과).
- `HIU_RNG_WORD`: read 시 64-bit word 반환, `rng_word_valid` 클리어. 다음
  valid 까지 평균 ~128 cycle.
- `trng_unhealthy` 는 sticky — 한 번 1 이 되면 reset 까지 유지. 호스트는
  poll 또는 인터럽트 (Phase B 에서 결정) 로 감지.

---

## 3. Lease attribute → HIU 상태 매핑

Y4 가 lease capability 를 발급할 때, 그 attribute 는 다음과 같이 HIU
레지스터로 직접 매핑된다.

| Lease attribute | HIU 레지스터 | 비고 |
|---|---|---|
| `partition_id ∈ {0,1,2,3}` | `HIU_CTRL[7:6]` | 동시 활성 lease 수 hard cap=4 |
| `chacha_key (256-bit)` | `HIU_CHACHA_KEY` | tenant 가 직접 보지 않음 |
| `chacha_nonce (192-bit)` | `HIU_CHACHA_NONCE` | nonce 재사용 금지 |
| `shadow_slots ⊆ [0,15]` | `HIU_SHADOW_*` 반복 write | 슬롯 union ≤ 16 |
| (가속기 명령) `wave_number` | (HIU 외부, ISA-level) | wave-aligned preempt 시 사용 |

**전이(回收) 시퀀스 (lease release):**

1. Y4: 호스트 사이드 cap-table 에서 token 무효화.
2. Y4: `HIU_CTRL.context_switch=1` write — 동일 사이클에 TLB/shadow flush.
3. Y4: 새 lease 의 `(key, nonce, partition_id)` 를 HIU 에 write — 또는
   slot 비워둠.
4. Y4: 다음 SHADOW_UPDATE 시퀀스로 새 lease 의 region 등록.

이 4-step 은 **단일 capability invocation 안에서 atomic** — 게스트가
중간 상태를 관찰할 수 없도록 Y4 가 IPC 경계에서 직렬화한다.

---

## 4. 인터럽트 / poll 정책 (v0 미해결)

현재 RTL 은 별도 인터럽트 라인을 노출하지 않는다. 호스트 측 옵션:

- (a) `HIU_STATUS` polling — Phase B mock 에서 단순.
- (b) PCIe MSI-X 한 벡터를 합성 단계에서 추가 — RTL 변경 필요.
- (c) `HIU_STATUS.dma_ack` rising edge 를 PCIe controller 의 message
  signaled completion 으로 묶기 — 보드 의존.

**v0 결정:** Phase B mock 은 (a) 로 시작. RTL 변경 (b) 은 v1.x patch
후보. 본 결정은 v1.0 frozen 전에 재검토.

---

## 5. v0 미해결 항목 (frozen 전 처리 대상)

1. (§1) `HIU_CHACHA_KEY/NONCE` 의 capability 보호 경계 — 같은 BAR 면
   IOMMU page-grain 으로 충분한지.
2. (§1) ATS / PRI 응답 폭이 단일 비트만으로 충분한지.
3. (§4) 인터럽트 vs polling 결정.
4. (§2.1) 32-cycle latency 가 TLB miss 시에도 같은지 RTL 측 확인 (현
   readout 으로는 YES, 그러나 양측 sign-off 시 재확인).
5. (§3) `shadow_slots` 의 lease 간 분배 알고리즘 — Y4 측 결정이지만 RTL
   측에서 추가 hint port 가 필요한지.

위 5 항목이 양쪽 합의로 닫히면 본 문서를 `v1.0 frozen` 으로 바꾸고,
`HIU_ABI_VERSION` 레지스터 값을 `0x0001_0000` 으로 고정한다.
