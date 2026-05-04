<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 — 용어 사전 (WaveTensor 측 정의 기반)

본 문서는 Y4 가 호스트하는 WaveTensor 가속기의 핵심 용어를
**WaveTensor 로컬 저장소(`/home/ybi/WaveTensor/`)의 RTL 및 spec 메모에서
추출한 정의 그대로** 정리한다. Y4 의 capability 스키마, HIU ABI 문서,
Verus/Coq 명세는 모두 이 사전을 참조한다.

원본 출처는 각 항목 끝에 `source:` 줄로 명시한다. WaveTensor 측에서 정의가
바뀌면 본 사전을 우선 갱신한 뒤 의존 문서(architecture.md, hiu_abi.md,
proofs/)에 전파한다.

---

## 1. HIU (Host Interface Unit)

WaveTensor 가속기와 호스트 사이의 단일 진입점 모듈. 주소 변환·DMA·shadow
region·partitioned TLB·XChaCha20 마스킹·TRNG 출력까지 **호스트–가속기
경계에 걸리는 모든 보안/통신 primitive 를 한 모듈에 모은 것**.

RTL 모듈 시그니처(파라미터 + 주요 포트):

| 파라미터 | 기본값 | 의미 |
|---|---:|---|
| `ADDR_WIDTH` | 64 | 가상/물리 주소 폭 |
| `DATA_WIDTH` | 32 | DMA 데이터 폭 |
| `MAX_REGIONS` | 16 | shadow region 슬롯 수 |
| `TLB_ENTRIES` | 16 | partition 당 TLB entry 수 |
| `NUM_PARTITIONS` | 4 | partitioned TLB 분할 수 (cache-attack 격리) |
| `CHACHA_ROUNDS` | 20 | XChaCha20 라운드 수 |
| `DUMMY_CYCLES` | 8 | TRANSLATE 후 상수시간 dummy 지연 |

핵심 입력 신호:
- `virt_addr [63:0]`, `dma_req`, `dma_data_in [31:0]`, `interface_type [1:0]`
- `ats_req`, `pri_req` (PCIe ATS / PRI)
- `shadow_write_*` 4종 + `shadow_index [3:0]` — shadow region 등록
- **`context_switch`** — 1-cycle 펄스, 모든 partitioned TLB / shadow region 강제 무효화
- `partition_id [1:0]` — 현재 사이클의 TLB 파티션 선택
- `chacha_nonce [191:0]`, `chacha_key [255:0]` — XChaCha20 키/논스 입력

핵심 출력:
- `phys_addr [63:0]`, `dma_data_out [31:0]`, `dma_ack`, `access_granted`
- `ats_resp`, `pri_resp`
- TRNG: `rng_raw [7:0] + valid`, `rng_word [63:0] + valid`, `trng_unhealthy`

FSM 상태(요지): `IDLE → SHADOW_UPDATE | TRANSLATE → DUMMY_DELAY →
CHACHA_H_INIT → CHACHA_H_MASK → CHACHA_INIT → CHACHA_MASK → CHACHA_FINAL
→ DMA_EXEC → GRANT_ACCESS → IDLE`. `context_switch` 가 들어오면 어느
상태에서나 `FLUSH_TLB` 로 강제 천이.

> source: `WaveTensor/HIU.v` lines 4–46 (모듈 헤더), 84–98 (FSM 상태 정의),
> 112–306 (메인 시퀀서).

---

## 2. Partitioned TLB

HIU 내부의 4-way partitioned TLB. **공유 TLB 가 아니라 partition_id 별로
물리적으로 분리된 4 개의 16-entry 테이블**. 같은 가상 주소가 partition 마다
다른 물리 주소로 매핑될 수 있고, partition 사이의 lookup hit/miss 가
서로의 timing 에 영향을 주지 않는다 → cross-tenant cache-timing 채널 차단.

```verilog
reg [ADDR_WIDTH-1:0] partitioned_tlb_virt [0:NUM_PARTITIONS-1][0:TLB_ENTRIES-1];
reg [ADDR_WIDTH-1:0] partitioned_tlb_phys [0:NUM_PARTITIONS-1][0:TLB_ENTRIES-1];
reg                  partitioned_tlb_valid [0:NUM_PARTITIONS-1][0:TLB_ENTRIES-1];
```

총 64 entry (4 × 16). Y4 는 `partition_id` 를 lease capability 의 attribute
로 노출하고, 동시 활성 lease 수의 hard cap 을 4 로 강제한다.

> source: `WaveTensor/HIU.v` lines 49–51, 152–156 (조회 경로).

---

## 3. Shadow region

`MAX_REGIONS = 16` 개의 `(start, end, valid)` 튜플 테이블. TLB miss 시
fallback 으로 조회되며, hit 시 **`virt_addr` 를 그대로 `phys_addr` 로
identity-map** 하여 access_granted 를 올린다.

```verilog
reg [ADDR_WIDTH-1:0] shadow_regions_start [0:MAX_REGIONS-1];
reg [ADDR_WIDTH-1:0] shadow_regions_end   [0:MAX_REGIONS-1];
reg                  shadow_valid         [0:MAX_REGIONS-1];
```

등록은 `shadow_write_en=1` + `shadow_index=k` 로 1 cycle 에 끝남
(SHADOW_UPDATE 상태). `context_switch` 시 모두 무효화.

Y4 측 책임: tenant compartment 생성 시 IOMMU 셋업과 동시에 shadow region
을 등록하여, tenant 가 자기에게 부여된 메모리 범위 외부를 가속기로 DMA
하지 못하게 한다.

> source: `WaveTensor/HIU.v` lines 53–55, 141–148, 159–164.

---

## 4. XChaCha20 masking

DMA 발사 전, **물리 주소(또는 데이터)를 XChaCha20 키스트림과 XOR** 해
가속기 외부 메모리 트래픽이 노출되더라도 평문이 드러나지 않도록 한다.
192-bit nonce + 256-bit key 입력 → HChaCha20 으로 subkey 유도 →
ChaCha20 으로 keystream 생성 → 64-bit 마스크가 `phys_addr` 와 XOR 되어
`masked_phys_addr` 가 DMA 엔진에 전달.

라운드 수: `CHACHA_ROUNDS = 20` (RFC 8439 표준값). FSM 상태:
`CHACHA_H_INIT → CHACHA_H_MASK → CHACHA_INIT → CHACHA_MASK → CHACHA_FINAL`.

키/논스 바인딩 정책 (Y4 책임):
- **tenant 별 (key, nonce) 발급** — Y4 가 lease capability 발급 시 함께 묶음.
- `context_switch` 와 동일 사이클에 새 nonce 적용.
- nonce 재사용 금지 (XChaCha20 의 보안 가정 위반).

> source: `WaveTensor/HIU.v` lines 27–28 (포트), 174–273 (HChaCha/ChaCha
> 시퀀스), 308–324 (quarter-round 함수).

---

## 5. `context_switch` 신호

HIU 의 1-bit 동기 입력. 라이즈 시 **동일 사이클**에 다음을 일으킨다:

1. FSM 강제 `FLUSH_TLB` 진입.
2. 모든 4 × 16 partitioned TLB entry 의 `valid` 비트 클리어.
3. 모든 16 shadow region 의 `valid` 클리어.

Y4 측 책임: lease 전환의 atomic 단계 중 하나. `context_switch` raise →
새 (key, nonce, partition_id) 바인딩 → 새 shadow region 등록의 **3-step
sequence 가 hypervisor capability 의 일부로 보장되어야 한다**.

> source: `WaveTensor/HIU.v` lines 25 (포트), 133 (강제 천이),
> 292–302 (FLUSH_TLB).

---

## 6. Lease (가속기 lease)

ad-hoc daemon-level 객체였던 lease 를 Y4 에서는 **hypervisor capability**
로 격상한다 (architecture.md §"Lease 의 OS-level 모델링").

life-cycle:
1. 클라이언트(M) → daemon RPC 로 lease 요청.
2. Y4 가 capability token 발급, tenant compartment 의 cap-table 에 등록.
3. compartment 가 가속기 MMIO/메모리에 접근할 때마다 Y4 가 cap 검증.
4. token 만료 시 Y4 가 atomic 3-step 청소:
   - (a) compartment teardown,
   - (b) HIU `context_switch` raise,
   - (c) 가속기 상태 sanitize.

lease attribute(최소):
- `partition_id ∈ {0,1,2,3}` (partitioned TLB 슬롯)
- `(chacha_key, chacha_nonce)` — Y4 가 발급, tenant 가 직접 보지 않음
- shadow region 슬롯 인덱스 집합 (≤ 16)
- 만료 시각 / heartbeat 정책

동시 활성 lease 의 hard cap = 4 (partitioned TLB 의 NUM_PARTITIONS).

> source: `docs/architecture.md` §"Lease 의 OS-level 모델링" (lines 262–269);
> `WaveTensor/.claude-memos/remote_accelerator_access.md` §"우리 가속기 측
> 사전 준비".

---

## 7. TRNG (Phase 1 raw + Phase 3 conditioned)

HIU 내부 TRNG 서브시스템. 두 출력 채널을 가진다:

| 출력 | 폭 | 단계 | 비고 |
|---|---:|---|---|
| `rng_raw + rng_raw_valid` | 8-bit + 1-bit | Phase 1 | RO 8 개 → 3-stage inverter chain → von Neumann 디바이어싱 → 8-bit 누적 시프트 |
| `rng_word + rng_word_valid` | 64-bit + 1-bit | Phase 3 | 8 개의 정상 raw 바이트를 패킹 |
| `trng_unhealthy` | 1-bit, sticky | — | RCT 또는 APT 실패 시 reset 까지 latch |

Health check (NIST SP800-90B):
- **RCT** (Repetition Count Test, §4.4.1): threshold = 41 (= 1 + ⌈20/0.5⌉, H_min ≈ 0.5).
- **APT** (Adaptive Proportion Test, §4.4.2): window = 64, threshold = 51 (α = 2⁻²⁰).

`rng_word` 는 WT64v1 ISA 의 `OPREF.src_kind = 2` (TRNG bank) 에 직접
공급된다 — 가속기 ISA 가 TRNG 를 first-class operand source 로 인식.

⚠ Phase 3 의 64-bit 패킹은 SP800-90A approved conditioner 가 아니다
(per-bit min-entropy 보존만, PRF 확장 없음). Phase 3.5 에서 ChaCha20-keyed
conditioner 로 교체 예정.

Y4 측 책임:
- `trng_unhealthy` 가 latch 되면 lease 발급 중단 + 모든 활성 lease 의 키
  rotation 트리거.
- guest OS 가 TRNG 를 직접 보지 못하게 함 (hypervisor capability 통해서만).

> source: `WaveTensor/HIU.v` lines 326–586 (전체 TRNG 블록);
> `WaveTensor/.claude-memos/wt64v1_spec.md` §6 "TRNG 사양" + §"OPREF.src_kind".

---

## 8. Pod / Cluster / PE — 가속기 토폴로지 용어

Y4 자체는 dataflow 토폴로지를 알 필요 없지만, lease attribute 와 IPC 채널
이름이 이 계층을 참조하므로 정의 고정:

- **PE** (Processing Element): 단일 ALU/MUL/DIV 유닛. WT64v1 토큰 in →
  토큰 out 6 cycle 기본.
- **Cluster**: `(PE_ROWS, PE_COLS) ∈ {(2,2),(2,4),(4,2),(4,4)}` 격자의
  L-PE 들 + 1 개의 stripped MATMUL_UNIT + 1 개의 공유 EHDecode 프론트엔드.
- **Pod**: `(CLUSTER_ROWS, CLUSTER_COLS) ∈ {(2,2),(2,4),(4,2),(4,4)}` 격자의
  Cluster 들. Pod 1 개당 HIU 1 개. **참조 구현은 16-PE Pod = 2×2 cluster
  × 2×2 PE**.
- **Wave**: WT64v1 의 dataflow 토큰 시퀀스 단위. `wave_number` 는 토큰의
  tag 안에서 증가하며 `WADV` (0x01) opcode 로 advance. Y4 의 lease
  scheduler 는 wave 경계에 정렬해 preempt 한다 (wave-aligned preemption).

> source: `WaveTensor/Pod.v` lines 4–35, `WaveTensor/Cluster.v` lines 4–47,
> `WaveTensor/.claude-memos/wt64v1_spec.md` §1, §3.1 (`WADV`).

---

## 9. WT64v1 (참고)

WaveTensor 가속기의 v1 ISA. Y4 는 ISA 자체를 해석하지 않지만, lease 발급
시 **ISA conformance level 을 lease attribute 로 노출**한다 (예:
"WT64v1-base only" vs "WT64v1 + WT64v1-C 확장"). 향후 WT64v2 발생 시 lease
스키마는 backwards-compatible 확장으로 처리.

> source: `WaveTensor/.claude-memos/wt64v1_spec.md` §1 + §11–12 (회귀
> 베이스라인 + 변경 정책).

---

## 10. RTL ↔ Y4 ABI 경계

Y4 가 직접 의존하는 WaveTensor 인터페이스 표면은 다음 셋 뿐이다.
이 경계 외부의 변경은 Y4 에 영향을 주지 않는다 — 본 사전과
`docs/hiu_abi.md` 가 그 경계를 정의한다.

1. **HIU 모듈 포트** (§1) — 호스트에서 보이는 MMIO 레지스터 맵으로 변환됨.
2. **`context_switch` semantic** (§5) — flush atomicity 보장 cycle 수.
3. **TRNG 출력 채널** (§7) — `rng_word` 폭, valid 펄스 길이, health flag
   의미.

WT64v1 ISA 자체(opcode 표 등)는 Y4 의 ABI 경계가 **아니다** — ISA 는 게스트
SDK 가 가속기와 직접 말할 때 의미가 있고, Y4 는 capability 검증과 메모리
mapping 만 책임진다.
