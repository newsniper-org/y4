---
name: WaveTensor 용어 — Y4 ABI 경계 정의 (RTL 추출)
description: HIU/partitioned TLB/shadow region/XChaCha20/context_switch/lease/TRNG/Pod-Cluster-PE/wave 의 RTL 기반 정확 정의. Y4 의 capability 스키마와 hiu_abi.md 의 입력.
type: reference
originSessionId: 78ff80c3-5421-425a-9e23-3da166ef2bb9
---
WaveTensor 측 정의를 Y4 가 어떻게 ABI 경계로 받아들이는지의 캐시.
정전(canonical) 사전은 `Y4/docs/glossary.md`. 본 메모리는 다음 대화 진입
시 즉시 참조 가능한 핵심 수치만 보관 — 상세는 사전에서 lookup.

## 핵심 수치 (RTL 파라미터, `WaveTensor/HIU.v` lines 4–46)

- `ADDR_WIDTH = 64`, `DATA_WIDTH = 32`
- `MAX_REGIONS = 16` (shadow region 슬롯)
- `TLB_ENTRIES = 16`, `NUM_PARTITIONS = 4` → 총 64 TLB entry
- `CHACHA_ROUNDS = 20`, `DUMMY_CYCLES = 8`
- `chacha_nonce` 폭 = 192-bit, `chacha_key` 폭 = 256-bit (XChaCha20 표준)
- TRNG: `rng_raw` 8-bit, `rng_word` 64-bit, RCT threshold 41, APT window 64 / threshold 51

## ABI 경계 표면 (변경 시 Y4 가 영향받음)

1. HIU 모듈 포트 (위 시그널 묶음)
2. `context_switch` semantic — 1 cycle 펄스, 모든 partition + shadow 무효화 강제
3. TRNG 출력 채널 (`rng_word`, `rng_word_valid`, `trng_unhealthy`)

WT64v1 opcode 표 자체는 Y4 의 ABI 가 **아님** — 게스트 SDK 영역.

## Y4 가 강제해야 할 invariant (hypervisor capability 측)

- 동시 활성 lease 수 ≤ 4 (NUM_PARTITIONS hard cap)
- lease 별 `(partition_id, chacha_key, chacha_nonce)` unique binding
- nonce 재사용 금지 (XChaCha20 보안 가정)
- `context_switch` raise → 새 (key, nonce, partition_id) 바인딩 → shadow 등록의 atomic 3-step
- `trng_unhealthy` latch → 신규 lease 발급 중단 + 활성 lease 키 rotation

## 출처 위치 (다음 대화에서 빠르게 찾으려면)

- HIU 전체: `/home/ybi/WaveTensor/HIU.v`
- WT64v1 spec: `/home/ybi/WaveTensor/.claude-memos/wt64v1_spec.md`
- Lease 원안: `/home/ybi/WaveTensor/.claude-memos/remote_accelerator_access.md`
- 토폴로지: `/home/ybi/WaveTensor/Pod.v`, `/home/ybi/WaveTensor/Cluster.v`
- Y4 측 사전: `/home/ybi/Y4/docs/glossary.md`
