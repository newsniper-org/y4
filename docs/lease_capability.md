<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Lease Capability — 스키마 v0.1 (draft, not frozen)

본 문서는 `architecture.md` §"Lease 의 OS-level 모델링" 의 lease 를
**seL4 capability** 로 모델링한 첫 스키마 초안이다. 모든 attribute 는
장치-대면 attribute(`partition_id` · `key` · `nonce` · `xlate_entries`)는
`docs/hiu_abi.md` v0.1 §3 의 구성 표면과 1:1 매핑된다 — 즉 lease 는 HIU 상태의
호스트-측 type-safe 표현. 나머지 attribute 는 호스트 전용이다. 소유 페이지 집합은
Y4 가 cap 밖에서 추적한다.

> **v0 → v0.1 (2026-10-02).** WaveTensor 정본(부록 E.2 · E.9) 정합. shadow slot →
> 파티션별 변환 항목, `context_switch` 5-step → 정본 E.9 6항 회전, TRNG 의존 제거
> (H1-6 · H4-5 · Y4 확정 답 질문 3 ⓐⓑ). 장치는 lease 객체를 모른다 — 파티션 단위
> 표면만 안다.

용어는 [`./glossary.md`](./glossary.md) 의 정의에 의존. ABI 는
[`./hiu_abi.md`](./hiu_abi.md) 와 함께 동결.

---

## 1. capability 객체

```rust
// 실제 정의는 Phase C (hiu_abi v1.0 frozen 후) 에 kernel/ 에 land. 본 문서는 spec.
pub struct LeaseCap {
    pub partition_id: PartitionId,        // 실재 파티션 수 ≤ 4 (정본 E.2 인코딩 상한)
    pub lease_serial: LeaseSerial,        // 파티션별 단조 일련번호 — 키 유도 입력
    pub key: MaskKey,                     // 256-bit — sealed, no read-out (0xE80 WO)
    pub nonce: MaskNonce,                 // 192-bit — never reused (0xEA4 WO)
    pub xlate_entries: XlateEntrySet,     // ⊆ 실재 색인 — 파티션마다 ≤ 16 (0xC00, 색인 = VPN 하위 비트)
    pub expiry: Timestamp,                // wall-clock or wave-aligned deadline
    pub isa_conformance: IsaLevel,        // WT64v1-base | WT64v1+C | WT64v2 ...
    pub heartbeat_period: Duration,       // tenant must renew within
    pub revocation_token: RevocationToken,// hypervisor-side single-use
}

pub enum PartitionId { P0, P1, P2, P3 } // 실재 파티션 수는 형상 파라미터 — 상한 4
pub struct LeaseSerial(u64);
pub struct MaskKey([u8; 32]);     // sealed type — no Debug, no Serialize
pub struct MaskNonce([u8; 24]);   // sealed type — issued by Y4 NonceAllocator
pub struct XlateEntrySet(u16);    // bitmask over the partition's real entry indices (≤ 16; 16 = 측정 형상)
pub enum IsaLevel { Wt64v1Base, Wt64v1C, Wt64v2 }
```

**sealed type 의 의미:** `MaskKey` 와 `MaskNonce` 는 (Verus 명세에서)
어떤 게스트 IPC 경로로도 read-out 되지 않음을 invariant 로 강제. Y4 는
이 두 필드를 HIU 마스킹 창(`0xE80`/`0xEA4`, WO) 기입으로만 소비한다. 장치도
되읽기를 주지 않는다(읽기 0, 정본 E.9 5항).

---

## 2. invariant (Verus / Coq spec hooks)

본 절의 모든 항목은 Phase C 의 `proofs/` 에 1:1 대응 spec 으로
머지된다. 여기서는 자연어로 명시한다.

### I1. partition disjointness

> 임의의 두 활성 lease `L1, L2` 가 서로 다른 `partition_id` 를 가진다:
> `L1 ≠ L2 ⇒ L1.partition_id ≠ L2.partition_id`.

따름 정리: 동시에 보유되는 lease 수 ≤ 실재 파티션 수 ≤ 4 (정본 E.2 5항 인코딩
상한). 단 패브릭 안에서 진행하는 일은 어느 순간에도 한 테넌트의 것이다 —
드레인 후 교체(시간 분할, `glossary.md` §3–§4).

### I2. 변환 항목의 파티션 귀속

> lease `L` 이 기입하는 모든 변환 항목은 `L.partition_id` 의 창
> (`0xC00 + (L.partition_id ≪ 7) + (i ≪ 3)`, `i ∈ L.xlate_entries`) 안에 있다.

파티션마다 항목 16 — 임대 간 합집합 상한(옛 「≤ 16 across all leases」)은 없다.
임대 간 분리는 Y4 불변식이 아니라 장치 쪽 세 가지가 준다: 주소 인코딩
`addr[8:7]`, 없는 파티션 주소 기입의 거절 + `[6]`, 파티션 간 저장 비공유 규범
(SEC-4) (H4-4 · H4-5, `glossary.md` §3). Y4 가 증명하는 것은 「자기 파티션 창
밖에 쓰지 않는다」 하나다.

### I3. `(키, 논스)` 쌍 유일성 (정본 E.9 2항 — 장치 비강제 호스트 의무)

> Y4 의 `NonceAllocator` 가 한 번이라도 발급한 nonce 는 다시 발급되지
> 않는다 (단조 증가 카운터 + 호스트-식별자 prefix). 그리고 키는 호스트 마스터
> 시크릿에서 `(partition_id, lease_serial)` 로 유도되므로 임대마다 새 키다.
> ⇒ 발급된 모든 `(key, nonce)` 쌍은 서로 다르다. 빈 임대(§3.3)도 같은 경로
> (`lease_serial` 하나 + `NonceAllocator` 값 하나 소비)를 쓰므로 모든 회전이 이 결론
> 안에 든다.
>
> 부속: 마스크가 유효한 두 파티션의 논스는 서로 다르다 — 정본 E.9 2항 첫 절(「논스는
> 파티션마다 다르다」, 개정 후에도 규범). 전역 단조 `NonceAllocator` 가 함께
> 만족한다. `FAULT_STATUS[12]` 는 이 절의 장치 쪽 부분 검사일 뿐이다.
>
> 신선도 근거: `host_secret` 은 **부팅마다 새로 짓는다**(임대는 재부팅을 넘지 않는다).
> 그래서 카운터와 `lease_serial` 이 부팅마다 다시 시작해도 쌍이 반복되지 않는다.

정본은 유일해야 하는 것을 논스 단독이 아니라 `(키, 논스)` 쌍으로 두고, 그 유일성·
신선도를 「장치가 강제할 수단이 없는 호스트 ABI 의무」로 둔다(키는 WO, 장치는 이력을
지지 않는다). I3 위반은 확장 논스 스트림 암호(XChaCha)의 보안 가정을 깨뜨려 cross-
tenant 주소 복원을 가능하게 한다 — Coq 측 high-level 보안 정리의 전제.
**장치 엔트로피(TRNG)에 의존하지 않는다** — 정본에 호스트가 읽을 난수 워드가 없다.

### I4. atomic-rotate 시퀀스 (lease release / hand-off)

lease `L_old` 를 회수하고 `L_new` 를 같은 `partition_id = p` 에 발급할 때, Y4
가 발행하는 명령 시퀀스는 정본 E.9 6항 4단계 + 항목 기입으로 **단일 capability
invocation 안에서 직렬화** 된다 (`hiu_abi.md` §2.3, H1-3 · H1-6):

```
0. cap_table.invalidate(L_old.revocation_token)
1. drain(p)                         // 정본 ⑴ = 3.7.2 드레인 의무 (기계에 드레인 모드 없음)
   a. quiesce(p)                    //   투입 정지 — 게스트 IPC 차단 (step 0 과 함께)
   b. if read(0xE04)[0]: fill_entry(); write(0xE10, 1)   // 걸린 미적중 정지 해소 (B.5 미결 · Y4 처분)
   c. drain_completion_ring()       //   CQ_HEAD 전진 — DRAIN_ALLZERO 는 CQ_OCC 포함
   d. abort_lost_reassembly()       //   0xF40 태그 지정 소거 (DRAIN[4] 미완 문맥)
   e. wait_until(read(0x37C) == 1)  //   DRAIN_ALLZERO — 정의역 여섯 + 완료 큐 (유계)
2. mmio.write(0xE00, p)             // XLATE_FLUSH — 유효 비트만; p 를 래치한 LO 스테이징 유효도 하강 (⑵)
3. mmio.write(0xEA0, p)             // MASK_PART 창 (⑶)
   mmio.write(0xE80..0xE9C, L_new.key)      // 8 워드, WO
   mmio.write(0xEA4..0xEB8, L_new.nonce)    // 6 워드, WO
4. wait_until(read(0xEC0)[9] == 1)  // 재키잉 수락 가능 — 아니면 발화가 불수락 (⑷)
   mmio.write(0xEBC, p)             // MASK_REKEY 발화
   wait_until(read(0xEC0) bit p == 1)       // MASK_STATUS — 완료 관측
   // 모든 wait 는 유계. 시간 초과 · 0xF80[13]/[12] 는 ⑶ 재시도 또는 파티션 fail-closed (P10)
5. for i in L_new.xlate_entries:    // Y4 선택: ⑷ 뒤. 정본 귀결: ⑵ 뒤 첫 기입은 LO
       mmio.write(entry_lo(p, i), frame)    // 스테이징 — 장치에 하나뿐 → LO→HI 쌍 직렬화
       mmio.write(entry_hi(p, i), V|perm|tag)
```

invariant:
- 게스트는 step 0 직후부터 step 5 완료 전까지 **HIU 와의 모든 직접 상호작용이
  차단**된다(Y4 가 IPC 경계에서 lease cap 검증 실패 처리). 이 차단과 step 1 의 드레인
  대기가 **함께** 정본 ⑴ 의무의 Y4 쪽 이행이다 — 차단만으로는 패브릭에 이미 들어간
  일을 멈추지 못한다.
- 시퀀스는 **파티션 `p` 에만** 건다. 다른 살아 있는 임대의 변환·마스크 유효는 건드리지
  않는다. 전역 flush 는 없다.
- step 1–4 의 순서를 어기면 요청이 미준비 마스크로 통과하고 `FAULT_STATUS[11]` 이
  선다(정지 보호 없음). Verus 명세가 순서를 강제한다.
- 회전마다 에포크를 올리는 호스트 의무(규약 미정, B.5)는 `hiu_abi.md` §5.3-6 데이터
  경로 묶음에 달려 있다 — 아직 step 이 아니다.

### I5. 엔트로피 건강성 신호의 처분 (Y4 정책)

> `FAULT_STATUS[17]`(`0xF80`, 엔트로피원 건강성 검사 실패 — sticky, `0xF84` W1C)
> 이 관측되면, Y4 는 **장치 난수 뱅크(OPREF `src_kind = 2`)를 쓰는 게스트
> 워크로드**에 그 사실을 알리고 정책에 따라 격리·보고한다.

키·논스는 I3 대로 장치 엔트로피에 의존하지 않으므로 `[17]` 이 lease 발급이나
회전을 막지 않는다. 이 좁힘의 근거는 정본 E.9 1항(호스트가 키를 기입, 장치는 생성하지
않음)이다. 정본 안의 옛 해설 문장(`03-hiu.typ:32-33` · `05-security.typ:376` — 장치
엔트로피가 마스킹을 먹인다)은 임대별 호스트 키 이전의 것이며 WaveTensor 쪽 정합 대상이다. 이 처분은 **Y4 정책이지 정본 요구가 아니다.** 완료
디스크립터 CD_TAG1 `[27]` poison 은 값 단위 신호다. (재구현에는 엔트로피원 유닛이
아직 없어 둘 다 상수 0 이다.)

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
                      │  - derive key/nonce     │
                      │  - reserve xlate entries│
                      │  - emit LeaseCap token  │
                      └──────┬──────────────────┘
                             │ atomic-rotate (I4) on partition entry
                             ▼
                      ┌─────────────────────────┐
                      │ HIU 구성 표면 (4 KiB)   │
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
2. `NonceAllocator.next()` (I3) — 단조 카운터 + 호스트 식별자 prefix. 장치 엔트로피
   불사용.
3. `KeyDeriver.derive(host_secret, partition_id, lease_serial) -> MaskKey` — 호스트
   마스터 시크릿(boot-time, 캡슐 외부 노출 불가)에서 도출.
4. `XlateAllocator.reserve(partition_id, req_attrs.entry_count)` — 그 파티션의 색인
   0..15 중에서. 미적중 정지가 영구가 되지 않도록 임대 메모리는 기본적으로
   pre-pin 하고 항목을 미리 기입한다(`hiu_abi.md` §2.2).
5. `RevocationTokenStore.issue()` — 단조 카운터 + HMAC.
6. `LeaseCap` 조립 → atomic-rotate (I4) → tenant 에게 cap 핸들 반환.

### 3.2 use

게스트는 HIU 구성 표면 **어느 구역에도 직접 닿지 않는다** — 4 KiB 한 페이지는 쪼갤
수 없으므로 Y4 가 페이지 전체를 capability 검증 뒤 대행한다(Y4 결정, `hiu_abi.md` §1
머리). 게스트 SDK 가 LNID/LBID 사상표·step 적재·재조립 소거를 요청하는 형식은
`docs/y4_abi.md` 소관이다. 장치 쪽에서 요청의 파티션은 「어느 임대
아래 도착했는가」가 정한다(정본 §3.6) — 토큰 값이 아니다.

### 3.3 release / expiry / revoke

세 경로 모두 동일한 atomic-rotate (I4) 로 수렴 — `L_new` 자리에는 빈 임대
("idle lease") 가 들어간다: 그 파티션 flush 상태(항목 미기입) + I3 경로(KDF + 카운터)로
새로 지은 키·논스 기입·재키잉(옛 키 덮어쓰기 — 장치 측 키 바이트 소거 규범은 정본에
없다, G1) + 그 파티션 소유 페이지의 `0x20C` 회수.

---

## 4. 비-목표 (out of scope for v0.1)

- **Wave-aligned preemption 알고리즘 자체** — 「wave-aligned」는 Y4 스케줄러
  정책 이름이다. 정본에서 wave 번호는 호스트가 짓는 불투명 식별자이고 아무것도
  게이트하지 않는다 — 장치 측 근거는 드레인(I4 step 1)이다. 어느 경계에서
  atomic-rotate 를 발화할지의 scheduler 디자인은 Phase C 의 `kernel/scheduler` 영역. 본 lease 스키마는 expiry 가 도래했을 때
  atomic-rotate 를 보장한다는 invariant 만 진다.
- **Multi-Pod / Multi-HIU** — 현 스펙은 Pod 1 = HIU 1 가정. 다중 Pod 는 정본에서
  2026-07-12 철회됐다. 다중 HIU 는 v2 후보.
- **Cross-host migration** — 한 lease 를 다른 호스트 Y4 인스턴스로
  이주시키는 것은 키/논스 재발급이 동반되어 본 schema 외부 연산.
- **Quota / fair-share** — 실재 파티션(≤ 4)을 어떤 정책으로 분배하는가는
  policy 레이어. 본 schema 는 mechanism.

---

## 5. v0.1 미해결 항목

1. (§1) `IsaLevel` 에 WT64v2 자리잡기 — WT64v1 lock 이후 v2 가 등장하면
   변경. 현재는 enum 의 미래 변종으로 reserved.
2. (§2 I3) 부팅마다 짓는 `host_secret` 의 엔트로피 원천과 CSPRNG 구현 — Verus 가능한
   구현체(예: chacha20 Rust crate 의 verified port) 선택. 호스트 엔트로피 원천
   (RDSEED/TPM 등)은 `hw_mechanism_abstraction.md` 레지스트리 대상.
3. (§3.1 step 3) `KeyDeriver` 가 HKDF-SHA256 vs HKDF-SHA3 vs BLAKE3 중
   무엇을 쓸지 — 합의 후 고정.
4. (§3.2) cap 검증 경로의 IPC overhead 측정 — Phase C mock 으로 estimate.
5. (§I4) atomic-rotate 의 IRQ-disable / preempt-disable 경계 — seL4
   notification 모델과 정합성 확인. 드레인·재키잉 대기의 시간 상한 값.
6. (§3.1 step 1, I1) 실재 파티션 수를 Y4 가 어떻게 아는가 — 정본 E.1 에 그 표면이 없다
   (`0xFC0` 구역은 `0xFFC` 만 정의). 보드·어댑터 층 또는 §3.10 요청(`hiu_abi.md` §5.3-3).

---

## 6. 다음 단계

- 본 v0.1 → `hiu_abi.md` §5.3 합의 묶음과 함께 양측 sign-off → `v1.0 frozen` 마킹.
- Phase C 진입(hiu_abi v1.0 frozen) 전에 본 spec 을 `proofs/lease/spec.rs` (Verus) 로 옮겨
  invariant I1–I6 의 machine-checked 버전 작성.
- `kernel/lease/` Rust 구현은 그 spec 이 통과한 후에만 land
  (formal-first 원칙).
