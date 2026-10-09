<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Lease Capability — 스키마 v0.3 (draft, not frozen)

본 문서는 `architecture.md` §"Lease 의 OS-level 모델링" 의 lease 를
**seL4 capability** 로 모델링한 첫 스키마 초안이다. 장치-대면 attribute
(`partition_id` · `key` · `nonce` · `dva_map`)는 `docs/hiu_abi.md` v0.3 §3 의 구성
표면과 매핑된다 — 즉 lease 는 HIU 상태의 호스트-측 type-safe 표현. 나머지 attribute
는 호스트 전용이다. 소유 페이지 집합은 Y4 가 cap 밖에서 추적한다.

> **v0.2 → v0.3 (2026-10-09).** WaveTensor 둘째 답과 Y4 둘째 답(둘 다 확정) 반영. I7 의 확인을
> `0xE14` 되읽기 (A, D) 로 바꿨다. 「V=0 는 에지 뒤 판정분에만 효력」의 귀결로 I2 에 프레임 재사용
> 규칙(회수 대기 · 드레인 뒤 재사용)을 더했고, 전역 구성 기입을 전환 창에 묶었다. 부팅 초기화를
> 「마스크 유효 중화」로 단순화했다. 장치 reset 은 증명된 경로가 있을 때만 한다.

> **v0.1 → v0.2 (2026-10-05).** WaveTensor §3.10 v1.0 묶음 제안에 대한 Y4 확정 답 반영.
> 항목 집합 · 할당기 모델(임의 색인 예약)을 「임대별 장치 VA 사상(권위) + 장치 항목은
> 캐시」로 바꿨다(형상 비노출 표면 — Y4 제안). 요청 결속 `0xE14` ACTIVE_PART(WT 제안)
> 와 스케줄 전환 불변식 I7 을 더했다. `[12]` 처분 · 드레인 루프 · ⑷ 완료 관측 · 부팅
> 순서를 고쳤다.

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
    pub dva_map: DvaMap,                  // 임대별 장치 VA 사상 — 권위. 장치 항목(0xC00)은 그 캐시
    pub expiry: Timestamp,                // wall-clock or wave-aligned deadline
    pub isa_conformance: IsaLevel,        // WT64v1-base | WT64v1+C | WT64v2 ...
    pub heartbeat_period: Duration,       // tenant must renew within
    pub revocation_token: RevocationToken,// hypervisor-side single-use
}

pub enum PartitionId { P0, P1, P2, P3 } // 실재 파티션 수는 0xFF8 HIU_SHAPE[7:0] — 상한 4
pub struct LeaseSerial(u64);
pub struct MaskKey([u8; 32]);     // sealed type — no Debug, no Serialize
pub struct MaskNonce([u8; 24]);   // sealed type — issued by Y4 NonceAllocator
pub struct DvaMap { /* VPN(VA 44b) → (frame: PA < 2^44, perm: R/W/X) — 4 KiB 페이지 */ }
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

따름 정리: 동시에 보유되는 lease 수 ≤ 실재 파티션 수(`0xFF8[7:0]`) ≤ 4 (창 인코딩
`addr[8:7]` 상한). 단 패브릭 안에서 진행하는 일은 어느 순간에도 한 임대 —
`0xE14` ACTIVE_PART 에 결속된 임대 — 의 것이다(시간 분할, I7).

### I2. 변환 항목의 파티션 귀속과 내용

> lease `L` 을 위해 기입하는 모든 변환 항목 커밋은 `L.partition_id` 의 창 기저
> (`0xC00 + (L.partition_id ≪ 7)`) 안에 있다. 그리고 V=1 커밋의 (VPN, 프레임, 권한)은
> `L.dva_map` 의 원소이거나 거부 항목(권한 0)이다.

장치 항목은 `dva_map` 의 캐시다(형상 비노출 표면 — `hiu_abi.md` §1.3, Y4 제안). 파티션당
용량 · 연관도는 형상이고 정합성은 그에 기대지 않는다. 임대 간 분리는 Y4 불변식이 아니라
장치 쪽 셋이 준다: 창 인코딩 `addr[8:7]`, 없는 파티션 기저 기입의 거절 + `[6]`, 파티션 간
비축출(3.6.1 결과 3 · 결과 규범 (c)). Y4 가 증명하는 것은 「자기 파티션 창 밖에 쓰지 않고,
사상에 없는 프레임을 V=1 · 권한 ≠ 0 으로 커밋하지 않는다」다. 사상 판정은 *커밋 시점의*
`dva_map` 으로 하고, 장부 갱신과 커밋은 `xlate_staging` 락 아래에서 한다.

> **I2 부속 — 프레임 재사용 (`hiu_abi.md` §2.7).** V=0 와 flush 는 커밋 에지 뒤에 판정되는
> 조회에만 효력이 있고, 승인 단 소유권 판정은 프레임 전역이다. 그래서:
> - 사상에서 뺀 프레임(remap 의 옛 프레임 포함)은 V=0 에지 시각과 함께 `pending_reclaim` 에
>   들어간다. `pending_reclaim` 의 프레임은 어느 살아 있는 임대의 `dva_map` 에도 없다.
> - 그 프레임의 재사용(다른 임대 사상 · `OWN_CLR` · 할당기 반환)은 그 에지 *뒤의*
>   `DRAIN_ALLZERO` 관측 뒤에만 한다. 단 결속되지 않은 파티션의 해제는 결속 이탈 판정(D=1)이
>   그 파티션의 일 0 을 보장하므로 드레인 관측 없이 다음 전환 창의 `OWN_CLR` 뒤에 재사용한다(재결속
>   전에 V=0 완료). 프레임마다 {사상됨 · 회수 대기 · 호스트 소유} 상태를 진다.
> - 처리는 다음 자연 드레인 지점(I4 step 2b · I7 step 3)에서 일괄로 하고, `pending_reclaim` 이
>   상한을 넘으면 강제 전환 · 드레인을 한다. 지연분은 임대 메모리 예산에 계상한다.

### I3. `(키, 논스)` 쌍 유일성 (정본 E.9 2항 — 장치 비강제 호스트 의무)

> Y4 의 `NonceAllocator` 가 한 번이라도 발급한 nonce 는 다시 발급되지
> 않는다 (단조 증가 카운터 + 호스트-식별자 prefix). 그리고 키는 호스트 마스터
> 시크릿에서 `(partition_id, lease_serial)` 로 유도되므로 임대마다 새 키다.
> ⇒ 발급된 모든 `(key, nonce)` 쌍은 서로 다르다. 빈 임대(§3.3)와 부팅 초기화도 같은 경로
> (`lease_serial` 하나 + `NonceAllocator` 값 하나 이상 — 재키잉 재시도마다 하나 — 소비)를
> 쓰므로 모든 회전이 이 결론 안에 든다.
>
> 부속: 마스크가 유효한 두 파티션의 논스는 서로 다르다 — 정본 E.9 2항 첫 절(「논스는
> 파티션마다 다르다」, 개정 후에도 규범). 전역 단조 `NonceAllocator` 가 한 부팅 안에서
> 함께 만족한다. 부팅을 건너서는 `hiu_abi.md` §2.5 의 초기화 순서(드레인 → 전 파티션 flush →
> 파티션마다 새 키·논스 기입으로 마스크 유효 하강 → `0xEC0[7:0]` == 0 확인 — 재키잉은 임대
> 발급 때)가 전제다. 이 순서 뒤에는 어느 파티션도 마스크 유효가 아니므로 옛 부팅의 잔여
> 논스와 겹칠 수 없다. `FAULT_STATUS[12]` 는 이 절의
> 장치 쪽 부분 검사일 뿐이고, **`[12]` 발화 = 장치 안에 이 부속의 위반이 실재한다 →
> 재시도 없이 원인을 귀속한다**(`hiu_abi.md` §2.6): Y4 쪽이면 `LeaseInvariant`, 장치 쪽 ·
> 귀속 불가면 `DeviceFault`.
>
> **논스 장부:** `NonceLedger` 는 실재 파티션마다 `(현재 기입 논스, 기입 부팅, 마스크 유효)` 를
> 진다. 불변식: 마스크 유효인 두 파티션의 장부 논스는 서로 다르다(이 부속의 Y4 쪽 형태).
> `[12]` 귀속은 이 장부와 장치 보고(`0xEC4` 캡처 — Y4 제안)를 대조한다.
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
가 발행하는 명령 시퀀스는 정본 E.9 6항 4단계 + 선택적 사전 채움으로 **단일 capability
invocation 안에서 직렬화** 된다 (`hiu_abi.md` §2.3, H1-3 · H1-6):

```
// entry_vpn/frame/commit(p) = 0xC00 + (p≪7) + 0x00 / 0x04 / 0x08   (hiu_abi §1.3)
// active_part = Y4 가 마지막으로 적용을 확인한 0xE14 값.  모든 wait/deadline 은 유계.
0. cap_table.invalidate(L_old.revocation_token)
1. drain(p)                         // 정본 ⑴ = 3.7.2 드레인 의무 (기계에 드레인 모드 없음)
   if p == active_part:             //   아니면 결속이 p 를 떠날 때 이미 충족 (I7)
     a. quiesce(p)                  //   투입 정지 — 게스트 IPC 차단 (step 0 과 함께)
     b. until read(0x37C)[0] == 1 or deadline(T_DRAIN):        // DRAIN_ALLZERO — 정의역 여섯 + 완료 큐
          while read(0xE04)[0] == 1 and !deadline(T_DRAIN):   //   정지가 다시 설 수 있으므로 [1] 이 아니라 [0]
              require read(0xE04)[15:8] == p ∧ read(0xE0C) == 0   // 어기면 fail-closed (hiu_abi §1.2.1)
              lock(xlate_staging): fill(p, read(0xE08))          // 보고된 VPN 만 — 옛 사상 또는 거부 채움 (B.5 갈래 ⑴)
          drain_completion_ring()   //   CQ_HEAD 전진 — DRAIN_ALLZERO 는 CQ_OCC 포함
          abort_lost_reassembly()   //   0xF40 태그 지정 소거 (DRAIN[4] 미완 문맥)
     c. write(0xE10, 1)             //   sticky 해제는 루프 뒤 한 번 (항목 기입 → 해제)
     // deadline 도달 → 장치 전체 격리 (결속을 옮길 길 없음 — I7 과 같음). flush 로 진행하지 않음.
     //   reset 은 플랫폼이 증명한 경로가 있을 때만 (hiu_abi §2.6)
2. mmio.write(0xE00, p)             // XLATE_FLUSH — p 의 항목 전부 무효 · p 를 래치한 스테이징 유효 하강 (⑵)
2b. if p == active_part:            // 철회 에지(flush) 뒤 드레인 재관측 — hiu_abi §2.7 규칙 3
      require read(0x37C)[0] == 1   //   아니면 1.b 로 돌아감 (유계)
      reclaim(pending_reclaim ∪ L_old 소유 페이지)   // 드레인 창 — OWN_CLR 먼저
      global_config(L_new)          //   그다음 OWN_SET · LNID/LBID · step (p ≠ active_part 이면 다음 전환 창)
3–4. lock(mask_window):             // 0xEA0 창은 하나 — ⑶→⑷ 전체(재시도 포함)를 장치 전역 직렬화
   require FAULT[6,12,13] == 0      //   기준선 — 서 있으면 앞 사건으로 처분하고 그 비트만 0xF84 로 해제한 뒤 진입
   nonce = L_new.nonce
   for attempt in 0..REKEY_RETRY_MAX:
     3. mmio.write(0xEA0, p)                      // MASK_PART 창 (⑶)
        mmio.write(0xE80..0xE9C, L_new.key)       // 8 워드, WO
        mmio.write(0xEA4..0xEB8, nonce)           // 6 워드, WO
     4. if wait_until(read(0xEC0)[9] == 1, T_REKEY):            // 재키잉 수락 가능 (⑷)
          mmio.write(0xEBC, p)                    // MASK_REKEY 발화
          if wait_until(read(0xEC0)[8] == 0, T_REKEY):          // 엔진 busy 하강 (합의 — 수락 뒤에 수락된 읽기는 완료 전까지 [8]=1)
            f = read(0xF80)                       //   [8]=0 을 본 읽기 *뒤에* 수락된 FAULT 읽기 → 최종 [13]
            if f[12]: return attribute_nonce_dup(p)   //   이탈 — 재시도 · [12] 해제 · 준비 비트 판독 없음 (hiu_abi §2.6, 0xEC4 를 먼저 읽는다)
            if f[6]:  raise Y4Defect              //   값 오류
            if read(0xEC0)[p] == 1 ∧ f[6,12,13] == 0: break   // 성공 (hiu_abi §2.3 ⑷)
        mmio.write(0xF84, read(0xF80) & {13})     //   이 경로가 처분한 비트([13])만 해제 — 다른 임대의 sticky 는 건드리지 않는다
        nonce = NonceAllocator.next()             //   재시도는 같은 키 + 새 논스 (I3)
   else: quarantine(p)                            //   재시도 소진 → 그 파티션 fail-closed
5. lock(xlate_staging):             // 스테이징은 장치에 하나 — 모든 0xC00 기입 열 직렬화
   for (vpn, frame, perm) in prefill(L_new.dva_map):   // 선택 — 상주 보장 없음, 성능 힌트
       mmio.write(entry_vpn(p), vpn); mmio.write(entry_frame(p), frame)
       mmio.write(entry_commit(p), V | perm)
```

invariant:
- 게스트는 step 0 직후부터 step 5 완료 전까지 **HIU 와의 모든 직접 상호작용이
  차단**된다(Y4 가 IPC 경계에서 lease cap 검증 실패 처리). 이 차단과 step 1 의 드레인
  대기가 **함께** 정본 ⑴ 의무의 Y4 쪽 이행이다 — 차단만으로는 패브릭에 이미 들어간
  일을 멈추지 못한다.
- 시퀀스는 **파티션 `p` 에만** 건다. 다른 살아 있는 임대의 변환·마스크 유효는 건드리지
  않는다. 전역 flush 는 없다. step 1 의 드레인 관측(`DRAIN_ALLZERO`)은 장치 전역이지만,
  p 가 결속돼 있지 않으면 step 1 자체가 필요 없다 — 이 분기는 3.7.2 드레인 의무를
  「결속된 파티션의 구성 교체」로 좁히는 정본 개정을 전제한다(Y4 요청).
- 장치 전역 락 둘: `0xEA0` 창(⑶→⑷ 와 그 재시도 전체), `0xC00` 스테이징(모든 기입 열 — 미적중 채움 포함).
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
않음)이다. 정본도 `645bed0` 에서 같은 좁힘을 적는다(`03-hiu.typ:33` 「마스크 키는
엔트로피가 아니라 *호스트가 기입한다*」 · `05-security.typ:376`). 이 처분은 **Y4 정책이지
정본 요구가 아니다.** 완료
디스크립터 CD_TAG1 `[27]` poison 은 값 단위 신호다. (재구현에는 엔트로피원 유닛이
아직 없어 둘 다 상수 0 이다.)

### I6. expiry monotonicity

> `LeaseCap.expiry` 는 lease 객체 lifetime 동안 단조 비감소이며, expiry
> 도달 시 atomic-rotate (I4) 가 1 회 자동 트리거된다.

### I7. 스케줄 전환 — 결속 (`0xE14` ACTIVE_PART — WT 제안 · Y4 조건부 수용(C1 · C2) 2026-10-09)

> 투입이 열려 있는 동안 장치의 결속은 스케줄된 임대의 파티션이다:
> `submissions_open ⇒ active_part == scheduled.partition_id ∧ scheduled 는 살아 있고 빈 임대가 아니다`.
> 결속은 드레인된 상태에서만 바뀐다.

전환 열 (`hiu_abi.md` §2.4) — flush · 재키잉 없음:

```
1. quiesce(cur)                      // cur = 지금 결속된(스케줄된) 임대 — 투입 정지
2. drain loop (I4 step 1.b 와 같음)  // 미적중은 cur 의 *살아 있는* 사상으로 채움.
                                     // 사상 없는 VPN 은 거부 채움하고 그 VPN 을 denied 에 모은다
   // deadline 도달 → 결속을 옮길 길 없음 → 장치 전체 격리 (reset 은 증명된 경로가 있을 때만)
3. lock(xlate_staging):              // DRAIN_ALLZERO 관측 뒤, 재개 전에 한 번에 (hiu_abi §2.2)
     for vpn in denied:
       mmio.write(entry_vpn(cur.partition_id), vpn)
       mmio.write(entry_commit(cur.partition_id), 0)   // V=0 단일 무효화 (VPN 만 요구)
   write(0xE10, 1)                   // sticky 해제 — 「항목 기입 → 해제」 (I4 step 1.c)
   reclaim(pending_reclaim)          // 드레인 창 — 회수 대기 프레임 · 미룬 OWN_CLR 먼저
   global_config(next)               //   그다음 next 의 OWN_SET · LNID/LBID 사상 · step 적재 (hiu_abi §2.7)
   require FAULT[6] == 0             //   창의 전역 구성 기입이 거절되지 않았다 — 서 있으면 Y4Defect (창 기입 오류) · 기준선
4. require next.partition_id != cur.partition_id        // Verus 사전조건 — 같은 값 기입은 탐침 전용
   mmio.write(0xE14, next.partition_id)                 // [7:0] 만 — 상위는 0 ([9:8] 은 쓰기 무시)
   r = read(0xE14)
   if r[7:0] == next ∧ r[8] == 1 ∧ r[9] == 1: ok        // 결속 · A(적용) · D(드레인 판정)
   elif r[8] == 0 ∧ r[9] == 1 ∧ r[7:0] == cur:          //   값 거절 — 장부 오류 (C2 전제)
     raise Y4Defect
   elif r[8] == 0 ∧ r[9] == 0 ∧ r[7:0] == cur:          //   유효 값 · 드레인 판정 거절
     v1.0: 2 로 돌아가 재드레인(거부 채움 · denied V=0 · 0xE10 포함) 후 4 재시도 — 유계, 소진 시 장치 격리
     링 소비 회로 착지 뒤: raise Y4Defect               //   투입 정지 실패 또는 드레인 오판
   else: DeviceFault                                    //   장치 모순 (A=1 ∧ 결속 ≠ next · A=1 ∧ D=0 · A=0 ∧ 결속 ≠ cur) → 장치 격리
   // 거절이면 [6] 을 0xE14 몫으로 귀속해 그 비트만 해제한다
5. active_part ← next.partition_id; resume(next)
```

장치는 「옛 결속 아래 받아들인 일이 하나도 남지 않았을 때」만 다른 값 기입을 적용한다. v1.0 의
드레인 판정 D 는 `DRAIN_ALLZERO`(같은 사이클 접수 포함)이고, 명령 링 항은 v1.0 에 링 저장이 없어
공허하다(합의 — 링 소비 회로 착지 개정에서 꺼냄 · 해독 단계 항을 더한다). 결과는 되읽기 `[8]`
A · `[9]` D 로 보고된다. 같은 값 기입은 fault 없이 A=1 과 D 를 보고하므로 「결속 확인 + 드레인
관측」 탐침으로 쓸 수 있다(A=1 ∧ D=0 이면 드레인 미완). v1.0 에서는 일의 원천이
외부 제시라 Y4 가 투입을 멈출 수 없으므로, I7 의 격리 주장은 명령 링 소비 회로가 착지한
판에서 한다 — v1.0 의 I7 은 mock 검증 대상이다.

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
                      │  - build DVA map        │
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
4. `dva_map` 구성 — 임대 메모리를 장치 VA 공간(v1.0 VA 44b)에 사상한다. 프레임은
   PA < 2^44 풀에서만 내준다(`hiu_abi.md` §3). 미적중 정지가 영구가 되지 않도록 임대
   메모리는 pre-pin 해 두어 언제나 채울 수 있게 한다. 사전 채움은 성능 힌트다
   (`hiu_abi.md` §2.2). 소유권 이양(`OWN_SET`)과 LNID/LBID · step 적재는 전역 구성이라 그 임대의
   전환-진입 창(I7 step 3)에서 한다 — 임대는 전환되기 전에는 돌지 않으므로 지연 손실이 없다.
5. `RevocationTokenStore.issue()` — 단조 카운터 + HMAC.
6. `LeaseCap` 조립 → atomic-rotate (I4) → tenant 에게 cap 핸들 반환.

### 3.2 use

게스트는 HIU 구성 표면 **어느 구역에도 직접 닿지 않는다** — 4 KiB 한 페이지는 쪼갤
수 없으므로 Y4 가 페이지 전체를 capability 검증 뒤 대행한다(Y4 결정, `hiu_abi.md` §1
머리). 게스트 SDK 가 LNID/LBID 사상표·step 적재·재조립 소거를 요청하는 형식은
`docs/y4_abi.md` 소관이다. 장치 쪽에서 요청의 파티션은 「어느 임대
아래 도착했는가」가 정한다(정본 §3.6) — 토큰 값이 아니다. 그 결속을 장치에 알리는 표면은
`0xE14` ACTIVE_PART(합의 2026-10-09, I7)다.

### 3.3 release / expiry / revoke

세 경로 모두 동일한 atomic-rotate (I4) 로 수렴 — `L_new` 자리에는 빈 임대
("idle lease") 가 들어간다: 그 파티션 flush 상태(항목 미기입) + I3 경로(KDF + 카운터)로
새로 지은 키·논스 기입·재키잉(옛 키 덮어쓰기 — 장치 측 키 바이트 소거 규범은 정본에
없다, G1) + 그 파티션 소유 페이지의 `0x20C` 회수. 회수는 p 가 결속돼 있었으면 I4 step 1 의
드레인 창에서 곧바로, 아니면 다음 전환 창(I7 step 3)에서 한다 — 그동안 페이지는 「회수 대기 ·
장치 소유」이고, p 는 flush 됐고 어느 살아 있는 `dva_map` 에도 없으므로 도달할 수 없다.

---

## 4. 비-목표 (out of scope for v0.3)

- **Wave-aligned preemption 알고리즘 자체** — 전환 비용은 드레인(wave 경계)이다(I7).
  LNID/LBID · step 표는 전역 구성이라 전환 창에서 적재한다(I7 step 3) — 「전환마다 재적재」
  쪽으로 기운다. 「wave-aligned」는 Y4 스케줄러
  정책 이름이다. 정본에서 wave 번호는 호스트가 짓는 불투명 식별자이고 아무것도
  게이트하지 않는다 — 장치 측 근거는 드레인(I4 step 1 · I7 step 2)이다. 어느 경계에서
  atomic-rotate 를 발화할지의 scheduler 디자인은 Phase C 의 `kernel/scheduler` 영역. 본 lease 스키마는 expiry 가 도래했을 때
  atomic-rotate 를 보장한다는 invariant 만 진다.
- **Multi-Pod / Multi-HIU** — 현 스펙은 Pod 1 = HIU 1 가정. 다중 Pod 는 정본에서
  2026-07-12 철회됐다. 다중 HIU 는 v2 후보.
- **Cross-host migration** — 한 lease 를 다른 호스트 Y4 인스턴스로
  이주시키는 것은 키/논스 재발급이 동반되어 본 schema 외부 연산.
- **Quota / fair-share** — 실재 파티션(≤ 4)을 어떤 정책으로 분배하는가는
  policy 레이어. 본 schema 는 mechanism.

---

## 5. v0.3 미해결 항목

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
6. ~~(§3.1 step 1, I1) 실재 파티션 수를 Y4 가 어떻게 아는가~~ — `0xFF8` HIU_SHAPE
   `[7:0]`(합의 2026-10-09, `hiu_abi.md` §1.4)로 닫는다. 정본 반영 대기.
7. ~~(I4 · I7) `LeaseInvariant` 의 처분~~ — 2026-10-06 사용자 결정으로 닫음: 원인을 먼저
   귀속하고(`hiu_abi.md` §2.6), Y4 쪽 원인만 `LeaseInvariant`(하이퍼바이저 정지), 장치 쪽 ·
   귀속 불가는 새 변형 `DeviceFault`(장치 격리)로 처리한다. `error.rs` 에 `DeviceFault` ·
   `FaultOrigin` · `fault_disposition` 착지(거울 크레이트 포함). Y4 의도 결함과 절차 결함을
   가르려면 겹친 상대 파티션이 필요하다 — `0xEC4` 캡처 워드가 v1.0 에 들어간다(합의 2026-10-09).
8. ~~(I2 · §3.1) 형상 비노출 표면(Y4 제안)이 받아들여지지 않으면 `dva_map` 모델을 다시 연다~~ —
   2026-10-09 WaveTensor 가 받았다(사용자 결정).
9. (I2 부속) `pending_reclaim` 상한과 강제 드레인 정책의 값 — 임대 메모리 예산과 함께 Phase C 에서
   정한다. WaveTensor 에 파티션별 「변환 펜스」 RO 비트를 v1.x 후보로 제안했다(있으면 전역 드레인 없이
   회수 가능).

---

## 6. 다음 단계

- 본 v0.3 → `hiu_abi.md` §5.3 합의 묶음과 함께 양측 sign-off → `v1.0 frozen` 마킹.
- Phase C 진입(hiu_abi v1.0 frozen) 전에 본 spec 을 `proofs/lease/spec.rs` (Verus) 로 옮겨
  invariant I1–I7 의 machine-checked 버전 작성(`hiu_abi.md` §6 목록).
- `kernel/lease/` Rust 구현은 그 spec 이 통과한 후에만 land
  (formal-first 원칙).
