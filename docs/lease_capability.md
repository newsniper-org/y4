<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Lease Capability — 스키마 v0 (draft, not frozen)

본 문서는 `architecture.md` §"Lease 의 OS-level 모델링" 의 lease 를
**seL4 capability** 로 모델링한 첫 스키마 초안이다. 모든 attribute 는
`docs/hiu_abi.md` v0 의 MMIO 슬롯과 1:1 매핑된다 — 즉 lease 는 HIU 상태의
호스트-측 type-safe 표현.

용어는 [`./glossary.md`](./glossary.md) 의 정의에 의존. ABI 는
[`./hiu_abi.md`](./hiu_abi.md) 와 함께 동결.

---

## 1. capability 객체

```rust
// 실제 정의는 Phase B 에서 kernel/ 에 land. 본 문서는 spec.
pub struct LeaseCap {
    pub partition_id: PartitionId,        // 0..=3 — hard cap from HIU
    pub key: ChachaKey,                   // 256-bit — sealed, no read-out
    pub nonce: ChachaNonce,               // 192-bit — never reused
    pub shadow_slots: ShadowSlotSet,      // ⊆ {0..=15}, ≤ 16 across all leases
    pub expiry: Timestamp,                // wall-clock or wave-aligned deadline
    pub isa_conformance: IsaLevel,        // WT64v1-base | WT64v1+C | WT64v2 ...
    pub heartbeat_period: Duration,       // tenant must renew within
    pub revocation_token: RevocationToken,// hypervisor-side single-use
}

pub enum PartitionId { P0, P1, P2, P3 }
pub struct ChachaKey([u8; 32]);   // sealed type — no Debug, no Serialize
pub struct ChachaNonce([u8; 24]); // sealed type — issued by Y4 NonceAllocator
pub struct ShadowSlotSet(u16);    // bitmask over 16 HIU slots
pub enum IsaLevel { Wt64v1Base, Wt64v1C, Wt64v2 }
```

**sealed type 의 의미:** `ChachaKey` 와 `ChachaNonce` 는 (Verus 명세에서)
어떤 게스트 IPC 경로로도 read-out 되지 않음을 invariant 로 강제. Y4 는
이 두 필드를 HIU MMIO write 로만 소비한다.

---

## 2. invariant (Verus / Coq spec hooks)

본 절의 모든 항목은 Phase B–C 의 `proofs/` 에 1:1 대응 spec 으로
머지된다. 여기서는 자연어로 명시한다.

### I1. partition disjointness

> 임의의 두 활성 lease `L1, L2` 가 서로 다른 `partition_id` 를 가진다:
> `L1 ≠ L2 ⇒ L1.partition_id ≠ L2.partition_id`.

따름 정리: 동시 활성 lease 수 ≤ 4. (HIU `NUM_PARTITIONS=4` hard cap.)

### I2. shadow slot disjointness

> 임의의 두 활성 lease 의 `shadow_slots` 가 disjoint:
> `L1 ≠ L2 ⇒ L1.shadow_slots & L2.shadow_slots == 0`.

따름 정리: 활성 lease 들의 `shadow_slots` 합집합 cardinality ≤ 16.

### I3. nonce uniqueness (XChaCha20 보안 가정)

> Y4 의 `NonceAllocator` 가 한 번이라도 발급한 nonce 는 다시 발급되지
> 않는다. (단조 증가 카운터 + 호스트-식별자 prefix 로 강제.)

I3 위반은 XChaCha20 의 IND-CPA 가정을 깨뜨려 cross-tenant 데이터 복원이
가능해진다 — 본 invariant 는 Coq 측 high-level 보안 정리의 전제.

### I4. atomic-rotate 시퀀스 (lease release / hand-off)

lease `L_old` 를 회수하고 `L_new` 를 같은 `partition_id` 에 발급할 때, Y4
가 발행하는 명령 시퀀스는 다음 5-step 으로 **단일 capability invocation
안에서 직렬화** 된다:

```
1. cap_table.invalidate(L_old.revocation_token)
2. mmio.write(HIU_CTRL.context_switch = 1)         // §hiu_abi 2.2
3. mmio.write(HIU_CHACHA_KEY,   L_new.key)
4. mmio.write(HIU_CHACHA_NONCE, L_new.nonce)
5. for slot in L_new.shadow_slots: SHADOW_UPDATE(slot, range)
```

invariant: 게스트는 step 1 직후부터 step 5 완료 전까지 **HIU 와의 모든
직접 상호작용이 차단** 된다 (Y4 가 IPC 경계에서 lease cap 검증 실패 처리).

### I5. TRNG-unhealthy ⇒ lease quiescence

> `HIU_RNG_WORD_STATUS.trng_unhealthy == 1` 이 관측되면, `NonceAllocator`
> 는 **신규 lease 발급을 중단** 하고 모든 활성 lease 를 atomic-rotate
> (I4) 로 키 갱신한다.

(주의: TRNG 가 unhealthy 인 동안 새 nonce 는 host-side CSPRNG fallback
으로 생성. 이 fallback 도 Verus 명세 내에서 정의된 RNG 를 사용해야 함.)

### I6. expiry monotonicity

> `LeaseCap.expiry` 는 lease 객체 lifetime 동안 단조 비감소이며, expiry
> 도달 시 atomic-rotate (I4) 가 1 회 자동 트리거된다.

---

## 3. lease lifecycle

```
                      ┌─────────────────────────┐
                      │   Tenant (M)            │
                      └──────┬──────────────────┘
                             │ acquire(req_attrs)
                             ▼
                      ┌─────────────────────────┐
                      │ Y4 LeaseManager         │
                      │  - allocate partition   │
                      │  - draw nonce/key       │
                      │  - reserve shadow slots │
                      │  - emit LeaseCap token  │
                      └──────┬──────────────────┘
                             │ atomic-rotate (I4) on partition entry
                             ▼
                      ┌─────────────────────────┐
                      │ HIU MMIO (BAR0)         │
                      └──────┬──────────────────┘
                             │
        ┌────────────────────┴─────────────────┐
        │                                      │
        ▼                                      ▼
  ┌──────────────┐                      ┌──────────────┐
  │ Tenant uses  │── heartbeat renew ──>│ Y4 watchdog  │
  │ the lease    │<── expiry / revoke ──│              │
  └──────┬───────┘                      └──────┬───────┘
         │ release(cap) | timeout | revoke      │
         └──────────────┬───────────────────────┘
                        ▼
                  atomic-rotate (I4)
                        │
                        ▼
             partition free, lease cap dead
```

### 3.1 acquire

`LeaseManager.acquire(req_attrs) -> Result<LeaseCap, AcquireError>`:
1. 빈 partition 선택 (없으면 `Err(NoPartition)`).
2. `NonceAllocator.next()` (I3) — TRNG 정상 시 HIU `rng_word`, 비정상
   시 fallback CSPRNG.
3. `KeyDeriver.derive(nonce, host_secret) -> ChachaKey` — 호스트 마스터
   시크릿(boot-time, 캡슐 외부 노출 불가)에서 도출.
4. `ShadowAllocator.reserve(req_attrs.region_count)` — bitmap-grain.
5. `RevocationTokenStore.issue()` — 단조 카운터 + HMAC.
6. `LeaseCap` 조립 → atomic-rotate (I4) → tenant 에게 cap 핸들 반환.

### 3.2 use

게스트의 모든 HIU 접근은 cap 검증 경로를 거친다. cap 의 `partition_id`
와 현재 HIU `partition_id` 가 일치해야만 MMIO write 가 허용된다.

### 3.3 release / expiry / revoke

세 경로 모두 동일한 atomic-rotate (I4) 로 수렴 — `L_new` 자리에는 빈
slot ("idle lease", 모든 shadow 미등록 + nonce 폐기) 가 들어간다.

---

## 4. 비-목표 (out of scope for v0)

- **Wave-aligned preemption 알고리즘 자체** — `wave_number` 관측 + 어느
  wave 경계에서 atomic-rotate 를 발화할지의 scheduler 디자인은 Phase C
  의 `kernel/scheduler` 영역. 본 lease 스키마는 expiry 가 도래했을 때
  atomic-rotate 를 보장한다는 invariant 만 진다.
- **Multi-Pod / Multi-HIU** — 현 스펙은 Pod 1 = HIU 1 가정. 다중 HIU 는
  v2 후보.
- **Cross-host migration** — 한 lease 를 다른 호스트 Y4 인스턴스로
  이주시키는 것은 키/논스 재발급이 동반되어 본 schema 외부 연산.
- **Quota / fair-share** — partition 4 개를 어떤 정책으로 분배하는가는
  policy 레이어. 본 schema 는 mechanism.

---

## 5. v0 미해결 항목

1. (§1) `IsaLevel` 에 WT64v2 자리잡기 — WT64v1 lock 이후 v2 가 등장하면
   변경. 현재는 enum 의 미래 변종으로 reserved.
2. (§2 I3) host-side CSPRNG fallback 의 구체 구현 — Verus 가능한
   구현체(예: chacha20 Rust crate 의 verified port) 선택.
3. (§3.1 step 3) `KeyDeriver` 가 HKDF-SHA256 vs HKDF-SHA3 vs BLAKE3 중
   무엇을 쓸지 — 합의 후 고정.
4. (§3.2) cap 검증 경로의 IPC overhead 측정 — Phase C mock 으로 estimate.
5. (§I4) atomic-rotate 의 IRQ-disable / preempt-disable 경계 — seL4
   notification 모델과 정합성 확인.

---

## 6. 다음 단계

- 본 v0 → 양측 sign-off (Y4 측 검토 + WaveTensor 측 RTL 측면 검토) →
  `v1.0 frozen` 마킹.
- Phase B 시작 시 본 spec 을 `proofs/lease/spec.rs` (Verus) 로 옮겨
  invariant I1–I6 의 machine-checked 버전 작성.
- `kernel/lease/` Rust 구현은 그 spec 이 통과한 후에만 land
  (formal-first 원칙).
