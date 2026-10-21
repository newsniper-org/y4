<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Lease Capability — 스키마 v0.9 (draft, not frozen)

본 문서는 `architecture.md` §"Lease 의 OS-level 모델링" 의 lease 를
**seL4 capability** 로 모델링한 첫 스키마 초안이다. 장치-대면 attribute
(`partition_id` · `key` · `nonce` · `dva_map`)는 `docs/hiu_abi.md` v1.0-rc4 §3 의 구성
표면과 매핑된다 — 즉 lease 는 HIU 상태의 호스트-측 type-safe 표현. 나머지 attribute
는 호스트 전용이다. 소유 페이지 집합은 Y4 가 cap 밖에서 추적한다.

이 문서의 정본(`G/…:줄`) 인용은 따로 표시가 없으면 WaveTensor 동결 sha `b16fc07`(정본 트리 `2dd78f80…` —
`hiu_abi.md` 와 같은 기준) 기준이다. 정본 줄 수가 보존됐으므로 바뀌지 않은 줄의 좌표는 `e88f6f4` 와 같다. 판 이력
블록(「> **v0.x → v0.y**」) 안의 인용은 그 블록이 적은 sha 기준이다. RTL 인용(`rtl/…`)은 따로 표시가 없으면 `22c76bb`
기준이고(`hiu_abi.md` 머리), 착지 표지와 함께 재구현의 현행 사실이며 ABI 출처가 아니다.
의사코드의 `Y4Defect` 는 `Y4Error::LeaseInvariant`(Y4 쪽 원인 — 하이퍼바이저 정지)이고 `DeviceFault` 는
`Y4Error::DeviceFault`(장치 격리)다(`hiu_abi.md` §2.6 · `error.rs`). 창 판정 `window_verdict` 의 결과는 지역 enum
`lease::window::WindowVerdict` 이고 `Y4Error` 변형이 아니다 — 그 가운데 `Y4Defect` 만 `LeaseInvariant` 에 이른다.
step 경로의 처분(`lease::step::StepAction` — `fp_shape_action` · `fp_quiesce_action` · `step_gate_action`)도 지역
enum 이고, `LeaseInvariant` 에 이르는 것은 FP 관문의 `[15]` 와 Y4 step 장부의 어긋남뿐이다
(`step_path_halts_only_on_ledger` — `[15]` 는 `[19]` 선례, `error.rs` `step_fault_origin`).

> **v0.8 → v0.9 (2026-10-22).** `hiu_abi.md` v1.0-rc4 반영 — WaveTensor 동결 sha `b16fc07`(C1 · C2 해소 · C3–C7) ·
> RTL 파도 W6 `efafb14` · 동결 뒤 사실 커밋 `395e5b1` · `22c76bb` 대조, 사용자 결정 2026-10-22 「권고안대로」. 이 블록의
> 정본 인용은 `b16fc07` 기준이다. 바뀐 것:
> - **`[15]` 는 `[19]` 선례로 귀속한다** — `b16fc07` 이 (4) 를 4항의 결속 전제와 재적재 순서로 막았고(「그가 보는
>   `[15]` 는 자기 쪽(드라이버 · 플랫폼)의 적재 · 결속 의무 위반이 낸 (4) 이거나 그 밖의 자기 쪽 의무 위반이다」
>   `e-defined.typ:466`) 테넌트 원인이 없으므로, `StepUnattributed`(기록만)를 지우고 `window_verdict(f, own_refused,
>   step_ledger_ok)` · `dispose_pre_window(f0, step_ledger_ok)` 가 Y4 step 장부가 어긋났으면 `Y4Defect`, 맞으면
>   `DeviceContradiction`(귀속 불가 → `DeviceFault`)으로 판정한다. `act` 의 `RecordAndClear` 를 지웠다. 꼬리 읽기
>   `fp_tail_read` 와 FP 관문도 같은 귀속이고, 관문의 1회 재적재를 지웠다(`StepGate` = `Clean` · `Refused`).
> - **`K_WALK` 연속 소진은 `StepOff`**(그 장치의 step 기능을 끈다 · 운영자 보고 — 장치 격리 아님).
> - `fp_quiesce` 가 기대는 장치 쪽 문장은 C2 로 정본에 섰다(`e-defined.typ:463` FP_STATUS 행) — 「반영 전 읽기
>   정지」는 유계 장치 의무이고 `T_WALK` 폴링이 끊는 대기가 아니다(`hiu_abi.md` §2.7 4).
> - Verus: `step_path_never_halts` · `fault15_only_records` · `window_halts_only_on_5_6` · `fault7_15_never_halt` 를
>   `step_path_halts_only_on_ledger` · `fault15_follows_step_fault_origin` · `window_halts_only_on_y4_evidence` ·
>   `fault7_never_halts` 로 바꿨다.

> **v0.7 → v0.8 (2026-10-18).** WaveTensor 의 rc2 대조 답과 정본 spec 커밋 `f6bc4df`
> (`2026-10-16-reply-rc2-d1d2-spec-f6bc4df.md` — D1 · D2 를 정본 규범으로 답했다)의 Y4 대조 반영(`hiu_abi.md`
> v1.0-rc3 — 사용자 결정 「권고안대로」, U1–U5). 이 블록의 정본 인용은 `f6bc4df` 기준이다. 바뀐 것:
> - **창 순서 — step 의 정지 확인 (major).** v0.7 의 창은 나가는 임대의 선택 열쇠를 `invalidate_tables` 안에서 풀고
>   곧바로 들어오는 임대의 FP_PROG 를 썼다. `0xF08[1]` 을 어디서도 읽지 않았으므로 나가는 임대의 걷기 중에 슬롯을
>   덮을 수 있었다(「걷기 중인 슬롯의 재적재(결과가 다른 프로그램의 것이 된다)」 `e-defined.typ:466` — 드레인은
>   step 걷기를 덮지 않는다, `hiu_abi.md` §2.7 「step 적재 규칙」 4). 정본 4항은 재적재를 결속을 모두 거둔 `0xF04`
>   기입 뒤의 관측에 걸었다(「수락된 `0xF08` 읽기가 `[1]` = 0 을 본 다음이다」 · 「해제 앞의 관측은 그 사이에 시작한 그
>   슬롯의 걷기를 덮지 않는다」 `:208`). I4 2b 와 I7 3 의 창을 같은 순서로 다시 적었다: (가) → `fp_unbind_all`(앞당김 —
>   결속된 선택 열쇠 전부 en = 0) → (I7: 거부 VPN V=0 · `0xE10`) → `reclaim` · LBID/LNID V=0 → `fp_quiesce`(`T_WALK`)
>   → (선택) `[15]` 꼬리 읽기 → `global_config`(OWN_SET 되읽기 · 거절이면 다른 프레임 `K_OWN` 회 · LNID/LBID V=1 ·
>   FP_PROG `fp_load_ok` · FP 관문 재적재 한 번 · FP_BIND en = 1 이 창의 마지막 기입) → `T_FLT` → (나) → `0xE14`.
>   헬퍼 `fp_unbind_all` · `fp_quiesce` · `fp_tail_read` 를 I7 블록 끝에 새로 두고, `invalidate_tables` 에서 FP_BIND 를
>   뺐다.
> - **`[15]` 는 어느 임대에도 귀속하지 않는다 (major).** v0.7 은 FP 관문 안의 `[15]` 를 들어오는 임대의 적재
>   실패(`FpLoadRefused`)로, 관문 밖의 `[15]` 를 결속된 임대의 실행 사건(`FpRunEvent`)으로 귀속했다. 정본은 「시점은
>   판별의 근거가 아니다」 · 「그 (4) 가 어느 결속 · 어느 걷기의 것인지는 v1.0 이 가르지 않는다」(`:466`)이고, 시점으로
>   귀속하면 나가는 임대가 들어오는 임대의 보고를 조절하는 1 비트 채널이 생긴다. 이제 `[15]` 는 `StepUnattributed`
>   (`RecordAndClear` — 운영자 · 플랫폼 기록 뒤 해제, 테넌트 보고 없음)다. FP 관문은 방어용이다 — 1차 `[15]` 면 기록 ·
>   해제하고 같은 이미지를 한 번 다시 싣고, 2차도 1 이면 결속하지 않고 `DeviceFault`(`StepGate::Refused`, U2)다.
>   `global_config` 의 `fp_gate` 반환값과, 관문이 본 `[15]` 의 소실을 `DeviceContradiction` 으로 보던 갈래는 지웠다.
> - **`[7]` 의 원천 — D1 해소.** E.7 6항(`e-defined.typ:1387`)이 OWN_SET 거절을 형상 중립으로 정의했고 「`[7]` 은
>   E.7 6항의 OWN_SET 거절 하나」다(`:527`). 판정 이름 `OwnCapacity` → `OwnSetRefused`. 거절된 프레임은 호스트 풀로
>   돌리고 다른 프레임으로 다시 이양한다(상한 `K_OWN` — 플랫폼 레지스트리, U4). 그래도 모자라면 그 임대에 거친 「풀
>   확장 실패」만 돌려준다(프레임 · 시점 비공개 — §3.1 4). 되읽기의 근거는 E.7 7항(`:1388`)과 E.1.1 5항(`:208`)이고,
>   쓰기 → 읽기 순서가 장치 포트까지 지켜지는 것은 플랫폼 전제다(「끝-끝 순서는 플랫폼의 성질이다」 `:1388`).
>   `global_config` 는 이제 불리언 `own_refused` 하나를 돌려준다(`window_verdict(f, own_refused)`).
> - **step 이미지 · 형상 (U1 · U2).** v1.0 에서 Y4 는 플랫폼이 지정한 이미지만 싣고 테넌트는 선택 열쇠로 고르기만
>   한다(§3.1 4 · §3.2). P · stepMax 는 부팅에서 `0xF0C` FP_SHAPE 로 읽고(`e-defined.typ:463` — `hiu_abi.md` §1.4 ·
>   §2.5 1), 플랫폼 레지스트리는 교차 확인으로 내렸다(§5 10). 이미지는 `fp_image_ok` 꼴만 싣는다(색인 0 부터 연속 ·
>   모든 워드 `[15:14]` = 0 · 마지막 워드 last = 1). 슬롯 장부 `fp`(`FpLedger` — 불변식 `fp_mirror`)가 결속된 슬롯과
>   정지 미관측 슬롯에 FP_PROG 를 쓰지 않게 하고, en = 1 은 관문을 통과한(또는 이미지가 그대로인) 슬롯 · 장부상
>   결속되지 않은 열쇠에만 낸다 — 재결속은 쓰지 않는다.
> - §5 12(D1 · D2)를 닫았다. 이 문서의 창이 기대는 Y4 sign-off 차단 조건은 C2(`[1]` 의 장치 쪽 읽기 순서 한 문장)와
>   C1(동결 sha 서면 지명)이다(`hiu_abi.md` §5.3). §5 5 에 `T_WALK` · `K_WALK` · `K_OWN` 을 더했다.
> - Verus: `lease::step`(새 — `fp_unbind_ok` · `fp_unbind_all_ok` · `fp_load_ok` · `fp_key_bindable` · `fp_mirror` ·
>   `fp_quiesce_action` · `StepGate` · `step_gate` · `fp_shape_verdict` · `step_path_never_halts`) · `lease::window`
>   (`WindowVerdict` 다섯 · `WindowAction::RecordAndClear` · `window_verdict(f, own_refused)` · `pre_window_exhaustive` ·
>   `fault15_only_records`; `OwnCapacity` · `FpLoadRefused` · `FpRunEvent` · `fp_gate_binds` 는 지웠다) · `lease::lnid`
>   (step 술어는 `lease::step` 으로 옮겼다).
>
> 스키마 변경은 없다.

> **v0.6 → v0.7 (2026-10-16).** WaveTensor 의 rc1 대조 답(`2026-10-16-reply-rc1-crosscheck.md` — major 1 · minor 16)과
> Y4 검증 반영(`hiu_abi.md` v1.0-rc2 — 사용자 결정 「D1과 D2 둘 다 sign-off 차단 조건들로 걸어놔. 권고안들은 전부
> 수용.」). 이 블록의 인용은 `e88f6f4` 기준이다. 바뀐 것:
> - **창 기준선 — `[6]` 에서 창 거절 집합 W 로 (major).** v0.6 의 창 기준선 `FAULT[6] == 0` 은 창 기입이 거절되지
>   않았음을 보이지 못했다 — LNID 서로소 위반은 `[5]`, FP_PROG 용량 밖은 `[15]` 이고, 재구현에서는 OWN_SET 충돌이
>   전이 없음 + `[7]` 이다. I7 3 과 I4 2b 의 창을 `hiu_abi.md` §2.4 3 대로 다시 적었다: 창 진입 (가) `f0 = read(0xF80) & W
>   → dispose_pre_window` · 회수의 `OWN_CLR` 되읽기 · 나가는 임대의 `invalidate_tables`(LBID · LNID V=0 · FP_BIND en = 0)
>   · 들어오는 임대의 `global_config`(OWN_SET 되읽기로 `refused` 를 모으고, FP 관문 — FP_PROG → `T_FLT` → `[15]` 확인 →
>   깨끗할 때만 FP_BIND) · 창 기준선 (나) `T_FLT` 뒤 `f = read(0xF80) & W → window_verdict(f, refused, fp_gate)`. 정지
>   (`LeaseInvariant`)에 이르는 것은 `[5]` · `[6]` 뿐이다. 확인이 주장하는 범위는 「장치가 거절로 알린 창 기입이
>   없었다」로 좁혔다(겹치는 LBID 구간은 `lbid_write_ok` 만이 막는다). 창 기입 셋(`reclaim` · `invalidate_tables` ·
>   `global_config`)의 정의를 I7 블록 끝에 두었고, §3.1 4 에 되읽기로 확인된 프레임만 사상하고 거절은 풀 확장 실패로
>   보고한다고 적었다. §5 10 에 레지스트리 상한(LNID 등록 용량 · 배달 지점 수 · P · stepMax)을 더하고, 12 에 D1 · D2
>   (Y4 sign-off 차단)를 새로 두었다.
> - **I7 3 — cur 의 LNID V=0 누락 정정.** v0.6 은 cur 의 LBID 만 지우고 next 를 실었다. 글자대로 구현하면 같은 슬롯
>   재기입은 `[6]`, 같은 LNID 의 다른 슬롯은 `[5]` 가 정상 경로에서 선다(「재사상은 해제 후 재기입 두 조작이다」
>   `03-hiu.typ:1057-1058`). 이제 `invalidate_tables(cur)` 가 I4 2b 와 같이 LBID · LNID 를 지우고, 나가는 임대의 선택
>   열쇠를 FP_BIND en = 0 으로 푼다(사용자 판단 — 「`[31]` en (1 = 결속 · 0 = 해제)」 `e-defined.typ:461`; 풀지 않으면
>   다음 임대의 같은 연산이 나가는 임대의 프로그램으로 돈다). §3.3 회수에도 같은 해제를 적었다.
> - **I4 ⑶ 진입 기준선.** v0.6 은 `FAULT[6,12,13]` 을 보고 서 있으면 앞 사건으로 처분 · 해제한 뒤 진입했다. 이제
>   서 있는 `[6]`(⑵ flush 등 Y4 기입의 거절)은 Y4 결함이라 ⑶ 에 진입하지 않는다. 2b 창을 열었으면 이 읽기가 그 창의
>   (나)다 — W 는 `window_verdict` 로 처분하고 `[5]` · `[6]` 이면 ⑶ 에 진입하지 않는다. `[12]` · `[13]` 처분과 ⑷ 성공
>   판정은 그대로다(`hiu_abi.md` §2.3 ⑷).
> - **I7 4 재시도.** 기준선은 `[6]` 만 그대로 본다(WT 제안 1 의 재시도 기준선 확장은 받지 않았다) — 재시도는 창 기입을
>   되풀이하지 않고, (나) 뒤의 W 는 일반 처리기가 맡는다. 주석만 달았다.
> - **락 표지.** `hiu_cfg` 는 `[6]` · W 귀속 락이다 — 창 진입 (가)부터 창 기준선 (나)까지 일반 처리기는 W 를 보유자에게
>   넘긴다(`hiu_abi.md` §2.2).
> - **minor.** m5 — I4 1.a 투입 정지에 v1.0 한계 주석(외부 제시 원천은 멈추지 못한다 — mock 검증, `hiu_abi.md` §2.3 ⑴).
>   m8 — 임대 발급은 실재 파티션(`< 0xFF8[7:0]`)에서만 고른다(§3.1 1)를 적고 I7 4 의 사전조건 주석에 이었다. m9 — v0.6 I3
>   부속은 재키잉이 `[12]` 와 무관하게 완료되므로 이 부속의 위반이 장치 안에 실재하게 된다고 적었는데, 틀렸다. `[12]` 를
>   동반한 재키잉 뒤의 준비 비트는 구현 정의다(`e-defined.typ:527`). 실재하는 것은 발화 시점의 정본 2항 첫 절
>   위반이고(`:1568-1569`), 장부는 `[12]` 를 본 p 를 마스크 유효로 올리지 않는다.
> - 머리에 `Y4Defect` = `Y4Error::LeaseInvariant` 를 적었다(`Y4Error` 변형은 더하지 않는다). Verus: `lease::window`
>   (`W_MASK` · `WindowVerdict` · `window_verdict` · `dispose_pre_window`), `lease::lnid`(`lnid_write_ok` · `fp_prog_ok` ·
>   `fp_bind_ok`), `lease::active_part::probe_outcome`.
>
> 스키마 변경은 없다.

> **v0.5 → v0.6 (2026-10-16).** WaveTensor spec 커밋 `b81df18`(명확화 S-2 · S-3, HIU-4 사실 정정)과
> RTL 파도 W0–W4 착지 통지(`e88f6f4`) 대조 반영(`hiu_abi.md` v1.0-rc1). 현행 본문의 정본 인용은 모두
> 동결 후보 `e88f6f4` 좌표 · 문면으로 다시 달았다. 바뀐 것:
> - **I7 3–4 (v0.5 절차 결함 정정).** v0.5 는 `[6]` 해제를 if-사슬 뒤 주석에만 두었다. 그래서 A = 0 ∧
>   D = 0 재시도가 서 있는 `[6]` 을 안고 2 로 돌아가 3 의 기준선 검사에서 거짓 `Y4Defect` 를 낼 수
>   있었다. 이제 `[6]` 은 `0xE14` 되읽기 *뒤* · 분기 *앞*에 다룬다 — A = 0 이면 `0xF80[6]` = 1 을 유계
>   폴링으로 확인하고 `0xE14` 몫으로 귀속해 그 비트만 해제한다(소진 = 거절 loud 부재 → `DeviceFault`).
>   정본은 `[6]` 의 가시 시점을 걸지 않으므로 이것은 Y4 정책이다(`hiu_abi.md` §2.4 3). 창의 기준선
>   검사와 I4 ⑷ 완료 확인의 FAULT 읽기도 마지막 구성 기입 뒤 같은 유계 대기를 거친다. 재시도 진입점은 드레인 루프 · 거부 VPN 의 V=0 ·
>   `0xE10` 로 좁혔다 — `hiu_cfg` 는 이미 쥐고 있고 회수 · 전역 구성은 되풀이하지 않는다.
> - **I3 부속 · 논스 장부 (v0.5 공백 정정).** `[12]` 의 정의역은 마스크가 유효한 *다른* 파티션 q ≠ p
>   대 p 의 *새* 논스다(S-3). 재키잉 중인 p 는 키 · 논스 기입으로 이미 마스크 무효이므로, v0.5 의 장부
>   불변식(「마스크 유효인 두 파티션의 장부 논스는 서로 다르다」)만으로는 p 의 새 논스와 q 의 충돌이
>   빠졌다. 불변식 (i) 를 더하고 I4 의사코드에 장부 갱신을 적었다(Verus `lease::nonce`).
> - **RTL 착지.** `[19]` · `0xFF8` · `0xEC4`(W1 `9b6eee6`) · 판정 래치(W2 `aa91e86`) · 형상 비노출 `0xC00`
>   문(W3 `f67e3d6`) · `0xE14` (A, D)(W4 `e88f6f4`)가 재구현에 착지했다. 미결 6 의 「RTL 파도 대기」 표지를
>   걷었다. v0.4 이력의 「오늘 RTL 이 겹치는 LBID 구간을 판정하지 않고 수락」은 v1.0 처분 그대로라 지금도
>   참이다. v1.0 결속은 여전히 격리를 주장하지 않는다(I7 끝).
>
> 스키마 변경은 없다.

> **v0.4 → v0.5 (2026-10-09).** WaveTensor 정정 커밋 `8af46df` 대조 반영(`hiu_abi.md` v0.5). 겹치는 LBID
> 구간의 v1.0 처분(장치는 판정하지 않음 · 비겹침은 호스트 의무 — `03-hiu.typ:1094` · `e-defined.typ:208`)이
> 정본에 착지해 LBID 대행 검증이 정본 호스트 의무가 됐다. 비소유 프레임 접근은 `[19]` 로 갈라졌다(§5-11
> 닫음). 스키마 변경은 없다.

> **v0.3 → v0.4 (2026-10-09).** WaveTensor 정본 spec 커밋 `6a25359` 대조 반영(`hiu_abi.md` v0.4). C1–C4 가
> 정본에 착지해 I7 의 조건 표지를 걷었고, `0xE14` 기입을 RMW 꼴로 바꿨다. 판정 래치의 귀결로 I2 부속에
> 권한 축소 규칙을 더했다. 오늘 RTL 이 겹치는 LBID 구간을 판정하지 않고 수락하고 v1.0 처분 문면이 아직
> 없다는 사실(Y4 요청 — spec 정정 대기)의 귀결로, 전환 · 회전 창의 LBID 대행 검증(`hiu_abi.md` §2.8 —
> Verus 사전조건 착지)을 더했다. `[6]` 귀속 락 `hiu_cfg` 를 I4 · I7 에 넣었다.

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

장치 항목은 `dva_map` 의 캐시다(형상 비노출 표면 — `hiu_abi.md` §1.3, Y4 제안 · 정본 `e-defined.typ:296-313` ·
RTL W3 `f67e3d6` 착지). 파티션당
용량 · 연관도는 형상이고 정합성은 그에 기대지 않는다. 임대 간 분리는 Y4 불변식이 아니라
장치 쪽 셋이 준다: 창 인코딩 `addr[8:7]`, 없는 파티션 기저 기입의 거절 + `[6]`, 파티션 간
비축출(3.6.1 결과 3 `03-hiu.typ:598-600` · 결과 규범 (c)). Y4 가 증명하는 것은 「자기 파티션 창 밖에 쓰지 않고,
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
> - 판정 래치(정본 3.6.2 `03-hiu.typ:729-737` — RTL W2 `aa91e86` 착지) 아래에서는 권한 축소 커밋도 에지 전에 판정된
>   요청에 효력이 없다. 축소에 기대는 동작(CoW · 쓰기 보호 · 이주)은 그 에지 뒤 `DRAIN_ALLZERO` 관측
>   뒤에만 한다(`hiu_abi.md` §2.7 1b).

### I3. `(키, 논스)` 쌍 유일성 (정본 E.9 2항 — 장치 비강제 호스트 의무)

> Y4 의 `NonceAllocator` 가 한 번이라도 발급한 nonce 는 다시 발급되지
> 않는다 (단조 증가 카운터 + 호스트-식별자 prefix). 그리고 키는 호스트 마스터
> 시크릿에서 `(partition_id, lease_serial)` 로 유도되므로 임대마다 새 키다.
> ⇒ 발급된 모든 `(key, nonce)` 쌍은 서로 다르다. 빈 임대(§3.3)와 부팅 초기화도 같은 경로
> (`lease_serial` 하나 + `NonceAllocator` 값 하나 이상 — 재키잉 재시도마다 하나 — 소비)를
> 쓰므로 모든 회전이 이 결론 안에 든다.
>
> 부속: 마스크가 유효한 두 파티션의 논스는 서로 다르다 — 정본 E.9 2항 첫 절(「논스는
> 파티션마다 다르다」, 개정 후에도 규범 — `e-defined.typ:1566-1567`). 전역 단조 `NonceAllocator` 가 한 부팅 안에서
> 함께 만족한다. 부팅을 건너서는 `hiu_abi.md` §2.5 의 초기화 순서(드레인 → 전 파티션 flush →
> 파티션마다 새 키·논스 기입으로 마스크 유효 하강 → `0xEC0[7:0]` == 0 확인 — 재키잉은 임대
> 발급 때)가 전제다. 이 순서 뒤에는 어느 파티션도 마스크 유효가 아니므로 옛 부팅의 잔여
> 논스와 겹칠 수 없다. `FAULT_STATUS[12]` 는 이 절의
> 장치 쪽 부분 검사일 뿐이다. 재키잉 중인 p 는 키 · 논스 기입으로 이미 마스크 무효이고
> (`e-defined.typ:339` · `:343`), `[12]` 는 마스크가 유효한 *다른* 파티션 q ≠ p 만 p 의 새 논스와
> 비교한다(`e-defined.typ:345` — p 자신은 비교에 들지 않는다). 그래서 **`[12]` 발화 = 마스크 유효인
> q ≠ p 가 재키잉 중인 p 의 새 논스와 같은 논스를 진다 — 정본 2항 첫 절의 위반이 발화 시점에 이미
> 실재한다(`e-defined.typ:1568-1569`) → 재시도 없이 원인을 귀속한다**
> (`hiu_abi.md` §2.6): Y4 쪽이면 `LeaseInvariant`, 장치 쪽 · 귀속 불가면 `DeviceFault`. 재키잉의 완료는
> `[12]` 와 무관하게 오지만(`b-gaps.typ:2085`) 그 뒤의 준비 비트는 구현 정의다(「`[12]` 를 동반한 재키잉 뒤의
> 준비 비트는 구현 정의다」 `e-defined.typ:527`). 장부는 `[12]` 를 본 p 를 마스크 유효로 올리지 않으므로(I4 4 —
> 성공은 `[12]` = 0 을 요구한다) 불변식 (ii) 는 장치 준비 비트에 기대지 않는다. `[12]` 의
> 논스 항은 고정 탐침 여섯의 *탐침 축약 아래* 측정이고(3.9 `03-hiu.typ:1539`) 세 비트 이상 · 이웃하지
> 않은 두 비트 차이에만 나타나는 결함은 형식으로도 전수로도 재지 않는다(끝-끝 시험의 무작위 골든 대조
> 표본뿐) — 쌍 유일성은 `[12]` 가 아니라 Y4 장부가 진다.
>
> **논스 장부:** `NonceLedger` 는 실재 파티션마다 `(현재 기입 논스, 기입 부팅, 마스크 유효)` 를
> 진다. 불변식 둘(이 부속의 Y4 쪽 형태):
> (i) 재키잉 발화 시점의 p 의 기입 논스는 장부상 마스크 유효인 모든 q ≠ p 의 논스와 다르다
> — p 는 그때 장부상 마스크 무효이므로 (ii) 만으로는 이 경우가 빠진다. `MASK_REKEY` 발화의 사전조건이다.
> (ii) 마스크 유효인 두 파티션의 장부 논스는 서로 다르다.
> 장부는 키 · 논스 기입 때 `(기입 논스, 이번 부팅, 마스크 유효 = 거짓)` 으로 적고, ⑷ 완료 확인이
> 성공한 뒤에만 마스크 유효 = 참으로 올린다(I4 step 3–4 · `hiu_abi.md` §2.5 4).
> `[12]` 귀속은 이 장부와 장치 보고(`0xEC4` 캡처 — 정본 `e-defined.typ:347`)를 대조한다. 귀속은
> Verus `lease::nonce::nonce_dup_origin` 이고, 장치 보고의 모든 경우에 원인이 정해진다는 것(전체성)은
> `lease::nonce::nonce_dup_origin_total` 이다.
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
// W = 창 거절 집합 {[5] 소유 서로소 위반 · [6] 거절된 구성 기입 · [7] OWN_SET 거절(E.7 6항 — 원천 하나) · [15] step 프로그램 불량(「`[15]` 의 원천」 닫힌 넷)}
//     — lease::window::W_MASK (hiu_abi §2.4 3)
// T_FLT = 구성 기입 거절의 loud(W 의 비트) 가시 대기 상한 — 플랫폼 레지스트리 값 (hw_mechanism_abstraction.md §2; 정본은 사이클 수를 걸지 않는다).
//         「`[15]` 의 원천」 (4) runaway 가 보이기까지의 상한은 아니다 (e-defined.typ:466)
// T_WALK · K_WALK = fp_quiesce 의 [1] = 0 대기 상한 · 연속 소진 상한, K_OWN = OWN_SET 거절 뒤 다른 프레임 재시도 상한 — 플랫폼 레지스트리 값 (hw_mechanism_abstraction.md §2)
// act(v) = lease::window::verdict_action(v): Y4Defect → Y4Error::LeaseInvariant (정지) · DeviceContradiction → DeviceFault (장치 격리)
//          · OwnSetRefused → ReportAndClear ([7] 만 0xF84 로 해제 · K_OWN 소진(L.pool_short)이면 그 임대에 거친 「풀 확장 실패」 · 진행)
//          · Clean → 진행. [15] 는 step_fault_verdict(fp_ok) — fp_ok(장부 정합)이면 DeviceContradiction, 아니면 Y4Defect ([19] 선례 — 테넌트 보고 없음)
//          겹치면 window_reports 의 항을 모두 처리한다
//        step 경로는 lease::step::StepAction — Proceed · SkipLoad (이 창의 적재 생략 — 그 열쇠의 참조는 능력 부재) · StepOff (step 기능 끔 — 장치 격리 아님) · Fail(e).
//          LeaseInvariant 는 FP 관문의 [15] ∧ ¬fp_ok 뿐이다 (step_path_halts_only_on_ledger)
// fp_ok = Y4 step 장부의 자기 점검 — fp_mirror · fp_ledger_wf 가 서고 이번 장치 reset 세대의 모든 FP 기입이 사전조건을 지켰다
// fp = Y4 의 step 장부 lease::step::FpLedger (부팅 hiu_abi §2.5 5e 의 fp_ledger_init 에서 시작) · fp_misses = 정지 확인 연속 소진 수
// P · stepMax = 부팅 0xF0C 판정이 Usable 일 때의 값 (fp_shape_verdict — hiu_abi §1.4 · §2.5 1; NoStep 이면 step 적재를 쓰지 않는다)
// fp_unbind_all · reclaim · invalidate_tables · fp_quiesce · fp_tail_read · global_config 는 I7 아래의 정의
0. cap_table.invalidate(L_old.revocation_token)
1. drain(p)                         // 정본 ⑴ = 3.7.2 드레인 의무 (기계에 드레인 모드 없음)
   if p == active_part:             //   아니면 결속이 p 를 떠날 때 이미 충족 (I7)
     a. quiesce(p)                  //   투입 정지 — 게스트 IPC 차단 (step 0 과 함께). v1.0: 외부 제시 원천은 멈추지 못한다 — mock 검증 (hiu_abi §2.3 ⑴ v1.0 한계)
     b. until read(0x37C)[0] == 1 or deadline(T_DRAIN):        // DRAIN_ALLZERO — 정의역 여섯 + 완료 큐
          while read(0xE04)[0] == 1 and !deadline(T_DRAIN):   //   정지가 다시 설 수 있으므로 [1] 이 아니라 [0]
              require read(0xE04)[15:8] == p ∧ read(0xE0C) == 0   // 어기면 fail-closed (hiu_abi §1.2.1)
              lock(xlate_staging): fill(p, read(0xE08))          // 보고된 VPN 만 — 옛 사상 또는 거부 채움 (B.5 갈래 ⑴)
          drain_completion_ring()   //   CQ_HEAD 전진 — DRAIN_ALLZERO 는 CQ_OCC 포함
          abort_lost_reassembly()   //   0xF40 태그 지정 소거 (DRAIN[4] 미완 문맥)
     c. write(0xE10, 1)             //   sticky 해제는 루프 뒤 한 번 (항목 기입 → 해제)
     // deadline 도달 → 장치 전체 격리 (결속을 옮길 길 없음 — I7 과 같음). flush 로 진행하지 않음.
     //   reset 은 플랫폼이 증명한 경로가 있을 때만 (hiu_abi §2.6)
2. lock(hiu_cfg)                    // [6] · W 귀속 락 — ⑵ flush 부터 ⑷ 귀속까지 (hiu_abi §2.2): flush 거절의 [6] 도 이 락 안에서 판정
   mmio.write(0xE00, p)             // XLATE_FLUSH — p 의 항목 전부 무효 · p 를 래치한 스테이징 유효 하강 (⑵)
   own_refused = false              //   2b 창이 없으면 그대로
2b. if p == active_part:            // 철회 에지(flush) 뒤 드레인 재관측 — hiu_abi §2.7 규칙 3
      require read(0x37C)[0] == 1   //   아니면 1.b 로 돌아감 (유계)
      f0 = read(0xF80) & W          //   창 진입 (가) — 첫 창 기입 전 (hiu_abi §2.4 3). 여기부터 3–4 의 (나)까지 W 는 hiu_cfg 보유자가 처분 (hiu_abi §2.2)
      act(dispose_pre_window(f0, fp_ok))   //   [5]·[6] → Y4Defect · [7] → DeviceFault (창 밖에서 섰다 — 설명할 OWN_SET 이 없다)
                                    //   · [15] → [19] 선례 (fp_ok → DeviceFault · ¬fp_ok → Y4Defect — L_old 에도 귀속하지 않는다)
      fp_unbind_all()               //   창의 첫 FP 기입 — L_old 가 결속한 선택 열쇠 전부 en=0 (hiu_abi §2.7 「step 적재 규칙」 5)
      reclaim(pending_reclaim ∪ L_old 소유 페이지)   // 드레인 창 — OWN_CLR 먼저 (기입마다 OWN_STATUS 되읽기)
      invalidate_tables(L_old)      //   L_old 의 LBID 엔트리 전부 V=0 (lbid_invalidate_preserves) · LNID 소유 엔트리 V=0 (lnid_invalidate_preserves) — hiu_abi §2.8 규칙 5
      q = fp_quiesce(L_new)         //   L_new 의 적재 계획이 있을 때만 — 마지막 en=0 뒤 [1] = 0 (T_WALK) · 소진 → SkipLoad · K_WALK 창 연속 → StepOff
      fp_tail_read()                //   (선택) T_FLT 뒤 [15] — [19] 선례 귀속
      own_refused = global_config(L_new, q)   //   OWN_SET(되읽기 · K_OWN) · LNID/LBID V=1 (lnid_write_ok · lbid_write_ok — hiu_abi §2.8)
                                    //   · FP_PROG → FP 관문 → FP_BIND en=1 (창의 마지막 기입) (p ≠ active_part 이면 다음 전환 창)
3–4. lock(mask_window):             // hiu_cfg 안에서 0xEA0 창 — ⑶→⑷ 전체(재시도 포함)를 장치 전역 직렬화
   wait_bounded(T_FLT)              //   2 · 2b 의 마지막 구성 기입(flush 또는 창의 마지막 기입) 뒤 유계 대기 — 그 기입 거절 비트도 가시 시점이 정본에 없다
   f = read(0xF80)                  //   ⑶ 진입 기준선 (hiu_abi §2.3 ⑷)
   if p == active_part:             //   2b 창을 열었다 — 이 읽기가 그 창의 기준선 (나) (hiu_abi §2.4 3)
     act(window_verdict(f & W, own_refused, fp_ok))   //   [5]·[6] (· [15] ∧ ¬fp_ok) → Y4Defect: ⑶ 에 진입하지 않는다 · [7] ∧ own_refused → OwnSetRefused · [15] ∧ fp_ok → DeviceFault
                                    //   · 설명 안 되는 [7] · 조용한 거절(own_refused ∧ ¬[7]) → DeviceContradiction → DeviceFault
   elif f[6]: raise Y4Defect        //   서 있는 [6](⑵ flush 등 Y4 기입의 거절)은 Y4 결함 — ⑶ 에 진입하지 않는다
   if f[12] ∨ f[13]:                //   sticky — 앞 사건으로 처분([12] 는 hiu_abi §2.6 귀속을 마친 뒤)하고 그 비트만 0xF84 로 해제한 뒤 진입
     dispose_prior(f & {12, 13})
   nonce = L_new.nonce
   for attempt in 0..REKEY_RETRY_MAX:
     3. mmio.write(0xEA0, p)                      // MASK_PART 창 (⑶)
        mmio.write(0xE80..0xE9C, L_new.key)       // 8 워드, WO
        mmio.write(0xEA4..0xEB8, nonce)           // 6 워드, WO — 키 · 논스 기입이 p 의 마스크 유효를 내린다 (e-defined.typ:339 · :343)
        ledger[p] ← (nonce, this_boot, mask_valid = false)    // NonceLedger (I3)
     4. if wait_until(read(0xEC0)[9] == 1, T_REKEY):            // 재키잉 수락 가능 (⑷)
          require ∀ q ≠ p: ledger[q].mask_valid ⇒ ledger[q].nonce ≠ nonce   // 장부 불변식 (i) — 발화 사전조건 (I3)
          mmio.write(0xEBC, p)                    // MASK_REKEY 발화
          if wait_until(read(0xEC0)[8] == 0, T_REKEY):          // 엔진 busy 하강 (정본 e-defined.typ:347 — 시작된 재키잉 뒤에 수락된 읽기는 완료 전까지 [8]=1)
            wait_bounded(T_FLT)                   //   ⑶ 기입 · 발화 거절의 [6] 가시 대기 — 창 기준선과 같은 원리 (hiu_abi §2.4 3)
            f = read(0xF80)                       //   [8]=0 을 본 읽기 *뒤에* 수락된 FAULT 읽기 → 최종 [13]
            if f[12]: return attribute_nonce_dup(p)   //   이탈 — 재시도 · [12] 해제 · 준비 비트 판독 없음 (hiu_abi §2.6, 0xEC4 를 먼저 읽는다 · lease::nonce::nonce_dup_origin)
            if f[6]:  raise Y4Defect              //   값 오류
            if read(0xEC0)[p] == 1 ∧ f[6,12,13] == 0:          // 성공 (hiu_abi §2.3 ⑷)
              ledger[p].mask_valid ← true; break  //   장부의 마스크 유효는 완료 확인 뒤에만 참
        mmio.write(0xF84, read(0xF80) & {13})     //   이 경로가 처분한 비트([13])만 해제 — 다른 임대의 sticky 는 건드리지 않는다
        nonce = NonceAllocator.next()             //   재시도는 같은 키 + 새 논스 (I3)
   else: quarantine(p)                            //   재시도 소진 → 그 파티션 fail-closed
   unlock(mask_window); unlock(hiu_cfg)
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
  p 가 결속돼 있지 않으면 step 1 자체가 필요 없다 — 이 분기는 3.7.2 「드레인 의무의 범위」
  (`03-hiu.typ:1215-1245`)에 기댄다.
- 장치 전역 락 셋: `hiu_cfg`(`[6]` · W 를 예상 · 판정하는 구간 — ⑵–⑷ · I7 step 2 끝의 창 진입 (가) ~ step 4 · 전역 구성
  창 · 부팅 초기화, `hiu_abi.md` §2.2. 창 진입 (가)부터 창 기준선 (나)까지 — I4 2b ~ 3–4 의 ⑶ 진입 기준선 · I7 2 끝 ~ 3 끝 —
  일반 fault 처리기는 `[6]` 뿐 아니라 W 전부를 보유자에게 넘기고 처분하지 않는다. 보유자 자신의 step 경로 — `fp_tail_read`
  · FP 관문 — 는 본 `[15]` 를 그 자리에서 미귀속으로 기록하고 해제한다), `0xEA0` 창(⑶→⑷ 와 그 재시도
  전체 — `hiu_cfg` 안에서), `0xC00` 스테이징(모든 기입 열 — 미적중 채움 포함).
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
않음)이다. 정본도 같은 좁힘을 적는다(`03-hiu.typ:33` 「마스크 키는
엔트로피가 아니라 *호스트가 기입한다*」 · `05-security.typ:376`). 이 처분은 **Y4 정책이지
정본 요구가 아니다.** 완료
디스크립터 CD_TAG1 `[27]` poison 은 값 단위 신호다. (재구현에는 엔트로피원 유닛이
아직 없어 둘 다 상수 0 이다 — 3.9 `03-hiu.typ:1562` · `:1567`.)

### I6. expiry monotonicity

> `LeaseCap.expiry` 는 lease 객체 lifetime 동안 단조 비감소이며, expiry
> 도달 시 atomic-rotate (I4) 가 1 회 자동 트리거된다.

### I7. 스케줄 전환 — 결속 (`0xE14` ACTIVE_PART — 정본 `e-defined.typ:329` · `:332`)

> 투입이 열려 있는 동안 장치의 결속은 스케줄된 임대의 파티션이다:
> `submissions_open ⇒ active_part == scheduled.partition_id ∧ scheduled 는 살아 있고 빈 임대가 아니다`.
> 결속은 드레인된 상태에서만 바뀐다.

전환 열 (`hiu_abi.md` §2.4) — flush · 재키잉 없음:

```
// W · T_FLT · T_WALK · K_WALK · K_OWN · act · fp · P · stepMax 는 I4 와 같다.
// fp_unbind_all · reclaim · invalidate_tables · fp_quiesce · fp_tail_read · global_config 의 정의는 이 블록 끝
1. quiesce(cur)                      // cur = 지금 결속된(스케줄된) 임대 — 투입 정지
2. drain loop (I4 step 1.b 와 같음)  // 미적중은 cur 의 *살아 있는* 사상으로 채움.
                                     // 사상 없는 VPN 은 거부 채움하고 그 VPN 을 denied 에 모은다
   // deadline 도달 → 결속을 옮길 길 없음 → 장치 전체 격리 (reset 은 증명된 경로가 있을 때만)
   lock(hiu_cfg)                     // [6] · W 귀속 락 — step 4 의 귀속 · 해제까지 쥔다 (hiu_abi §2.2)
   f0 = read(0xF80) & W              //   창 진입 (가) — 첫 창 기입(아래 결속 해제 · 거부 VPN V=0 포함) 전 (hiu_abi §2.4 3). 앞의 T_FLT 대기는 두지 않는다
   act(dispose_pre_window(f0, fp_ok))   //   창 전 사건: [5]·[6] → Y4Defect · [7] → DeviceFault (창 밖에서 섰다 — 설명할 OWN_SET 이 없다)
                                     //   · [15] → [19] 선례 (cur 에도 귀속하지 않는다: 「시점은 판별의 근거가 아니다」 e-defined.typ:466)
                                     //   여기부터 3 의 (나)까지 W 는 hiu_cfg 보유자가 처분한다 (hiu_abi §2.2)
3. fp_unbind_all()                   // 앞당김 — cur 가 결속한 선택 열쇠 전부 en=0, 창의 첫 FP 기입 (hiu_abi §2.4 2 · §2.7 「step 적재 규칙」 5)
   lock(xlate_staging):              // DRAIN_ALLZERO 관측 뒤, 재개 전에 한 번에 (hiu_abi §2.2)
     for vpn in denied:
       mmio.write(entry_vpn(cur.partition_id), vpn)
       mmio.write(entry_commit(cur.partition_id), 0)   // V=0 단일 무효화 (VPN 만 요구) — 0x0000_0000 으로 쓴다
   write(0xE10, 1)                   // sticky 해제 — 「항목 기입 → 해제」 (I4 step 1.c)
   reclaim(pending_reclaim)          // 드레인 창 — 회수 대기 프레임 · 미룬 OWN_CLR 먼저 (기입마다 OWN_STATUS 되읽기)
   invalidate_tables(cur)            //   cur 의 LBID 엔트리 전부 V=0 · LNID 소유 엔트리 V=0 (I4 2b 와 같음 — hiu_abi §2.8 규칙 5)
   q = fp_quiesce(next)              //   next 의 적재 계획이 있을 때만 — 마지막 en=0 뒤 [1] = 0 (T_WALK) · 소진 → SkipLoad · K_WALK 창 연속 → StepOff
   fp_tail_read()                    //   (선택) T_FLT 뒤 [15] — [19] 선례 귀속
   own_refused = global_config(next, q)   //   next 의 OWN_SET(되읽기 · K_OWN) · LNID/LBID V=1 · FP_PROG(fp_load_ok) → FP 관문
                                     //   → FP_BIND en=1 (창의 마지막 기입) (hiu_abi §2.7)
   wait_bounded(T_FLT)               //   창의 마지막 기입 뒤 유계 대기 — 기입 거절의 W 비트도 가시 시점이 정본에 없다
   f = read(0xF80) & W               //   창 기준선 (나)
   act(window_verdict(f, own_refused, fp_ok))   //   [5]·[6] → Y4Defect (창 기입 오류)
                                     //   · [7] ∧ own_refused → OwnSetRefused ([7] 해제 · next.pool_short 이면 그 임대에 거친 「풀 확장 실패」 — 프레임 · 시점 비공개)
                                     //   · [15] → [19] 선례: fp_ok → DeviceFault (귀속 불가) · ¬fp_ok → Y4Defect — next 에도 cur 에도 귀속하지 않고 테넌트 보고 없음
                                     //   · 설명 안 되는 [7] · 조용한 거절(own_refused ∧ ¬[7]) → DeviceContradiction → DeviceFault
                                     //   주장 범위는 「장치가 거절로 알린 창 기입이 없었다」까지 — 겹치는 LBID 구간은 장치가 판정 없이 수락하므로 lbid_write_ok 만이,
                                     //   step 의 용량 안 오용 셋(미적재 슬롯 결속 · 적재가 끝나기 전의 결속 · 걷기 중 재적재)은 장치가 판정하지 않으므로
                                     //   슬롯 장부(fp_mirror)와 fp_quiesce 만이 막는다 (e-defined.typ:466)
4. require next.partition_id != cur.partition_id        // Verus 사전조건 — 같은 값 기입은 탐침 전용
                                     //   next < 0xFF8[7:0] 은 §3.1 1 이 보장 — 범위 밖 값의 D = 0 은 드레인 거절과 구별되지 않는다
   mmio.write(0xE14, (read(0xE14) & !0xFF) | next.partition_id)   // RMW — 정렬 32b 전폭 읽기 · 쓰기 (hiu_abi §1.2.1 (C)); [31:8] 되읽은 값 그대로 (E.1.1 의무 3; [9:8] 쓰기 무시)
   r = read(0xE14)                   //   기입과 이 되읽기 사이에 다른 구성 접근을 내지 않는다 — 판정은 r 의 (A, D) 로만
   if r[8] == 0:                     //   A = 0 — [6] 은 되읽기 응답 *뒤에*, 분기보다 먼저 다룬다 (hiu_abi §2.4 3)
     if !wait_until(read(0xF80)[6] == 1, T_FLT): DeviceFault   //   유계 폴링 소진 = 거절 loud 부재 → 장치 격리
     mmio.write(0xF84, 1 << 6)       //   0xE14 몫으로 귀속 · 그 비트만 해제 (A = 1 이면 [6] 을 기대하지 않는다)
   if r[7:0] == next ∧ r[8] == 1 ∧ r[9] == 1: ok        // 결속 · A(적용) · D(드레인 판정)
   elif r[8] == 0 ∧ r[9] == 1 ∧ r[7:0] == cur:          //   값 거절 — 장부 오류 (e-defined.typ:332 3항)
     raise Y4Defect
   elif r[8] == 0 ∧ r[9] == 0 ∧ r[7:0] == cur:          //   유효 값 · 드레인 판정 거절
     v1.0: 재시도 — drain loop(2 의 루프만) → denied V=0 → 0xE10 → wait_bounded(T_FLT) → require FAULT[6] == 0 → 4
           //   hiu_cfg 는 이미 쥐고 있다. fp_unbind_all · reclaim · invalidate_tables · fp_quiesce · global_config 는 되풀이하지 않는다. 유계, 소진 시 장치 격리
           //   재시도 기준선은 [6] 만 본다 — 재시도는 창 기입을 되풀이하지 않는다. (나) 뒤의 W([5]·[7]·[15])는 일반 처리기가 맡는다 (hiu_abi §2.2 · §2.4 3)
           //   — 일반 처리기도 [15] 를 [19] 선례로 귀속한다 (외부 원천이 걷는 열쇠도 Y4 가 결속한 것뿐 — 적합 경로에서 (4) 는 나지 않는다, e-defined.typ:208 · :466)
     링 소비 회로 착지 뒤: raise Y4Defect               //   투입 정지 실패 또는 드레인 오판
   else: DeviceFault                                    //   장치 모순 (A=1 ∧ 결속 ≠ next · A=1 ∧ D=0 · A=0 ∧ 결속 ≠ cur) → 장치 격리
5. active_part ← next.partition_id; unlock(hiu_cfg); resume(next)

// 창 기입 (hiu_abi §2.7 「창 기입의 확인」 · 「step 적재 규칙」 — 모두 hiu_cfg · 드레인 창 안에서; OWN_IDX/OWN_STATUS 쌍은 hiu_cfg 아래에서만 쓴다)
// L.load = L 이 고른 선택 열쇠의 플랫폼 지정 이미지(U1 — 테넌트는 이미지를 대지 않는다) 가운데 장부상 상주하지 않아 실을 (슬롯 s, 이미지 img).
//          슬롯은 장부가 정하고, 이미지 식별은 플랫폼 레지스트리의 것이다 (Verus 모델 밖)
// L.bind = L 의 (선택 열쇠 k, 슬롯 s) — 이미지가 장부상 그대로인 슬롯(Unbound · Quiesced)은 다시 싣지 않고 결속만 한다 (slot_image_ready)
fp_unbind_all():                     // 창의 첫 FP 기입 — 장부상 결속된 열쇠 전부 (결속은 결속된 임대 cur · L_old 의 것뿐이다)
  for k in fp.keys:                  //   선택 열쇠 전 범위(2^16) sweep 은 부팅만 (hiu_abi §2.5 5b)
    mmio.write(0xF04, fp_bind_word(false, 0, k))         // [31] en=0 · [30:24] 예약 0 · [23:16] 슬롯 0 · [15:0] 열쇠 k — fp_unbind_ok (fp_unbind_never_refused — P 와 무관)
                                     //   「en = 0 은 슬롯 필드를 무시하고」 (e-defined.typ:461) — 슬롯 0 은 결정성 때문이다
  fp ← fp_ledger_unbind_all(fp)      //   Bound → Unbound (정지 미관측 — fp_mirror_unbind_ledger_keys: 장부 열쇠에만 en = 0 인 이 기입 열 · 선택자 표 비움). 풀린 열쇠의 참조는 능력 부재 (「결속이 곧 상주 선언이다」 :461)
reclaim(S):                          // 회수 — 장부상 회수 대기 · 장치 소유인 프레임
  for f in S:
    mmio.write(0x20C, f); mmio.write(0x200, f)          // OWN_CLR f → OWN_IDX ← f
    if read(0x204)[0] == 1: DeviceFault                 //   OWN_STATUS 에 1 이 남음 = 장치 모순 (「OWN_CLR 은 형상 · 점유로 거절되지 않는다」 e-defined.typ:1387 — own_readback Contradiction)
    ledger.own[f] ← 호스트 소유                         //   확인된 것만 재사용(다른 임대 사상 · 할당기 반환)
invalidate_tables(L):                // 나가는 임대의 사상표 — 들어오는 임대의 적재보다 먼저 (hiu_abi §2.8 규칙 5). 선택 열쇠는 fp_unbind_all 이 이미 풀었다
  for e in L 의 LBID 엔트리: write V=0      // lbid_invalidate_preserves
  for e in L 의 LNID 소유 엔트리: write V=0 // lnid_invalidate_preserves — V=0 는 용량 밖이어도 그 원인으로 거절되지 않는다 (03-hiu.typ:1072 · lnid_invalidate_never_rejected6).
                                            //   LNID V=1 은 V=0 뒤에만 (03-hiu.typ:1057-1058)
fp_quiesce(L) -> StepAction:         // 재적재 순서와 정지 확인 (hiu_abi §2.7 「step 적재 규칙」 4 — 정본 E.1.1 4항 e-defined.typ:208)
  if L.load == ∅: return Proceed     //   다시 쓸 슬롯이 없으면 기다리지 않는다 — 이미지가 그대로인 슬롯은 Unbound 인 채 결속할 수 있다
  saw = wait_until(read(0xF08)[1] == 0, T_WALK)        //   마지막 en=0 기입 뒤에 수락된 0xF08 읽기 — 「해제 앞의 관측은 그 사이에 시작한 그 슬롯의 걷기를 덮지 않는다」 (:208)
  q = fp_quiesce_action(saw, fp_misses, K_WALK)        //   관측 → Proceed · 소진 → SkipLoad (외부 투입이 이어진다 — :332) · K_WALK 창 연속 소진 → StepOff (장치 격리 아님)
  fp_misses ← fp_quiesce_misses(saw, fp_misses)        //   관측하면 0 으로
  if saw: fp ← fp_ledger_quiesced(fp)                  //   Unbound → Quiesced · Unloaded → 걷기 없음 (fp_mirror_quiesced · fp_quiesced_all_writable)
  if q == StepOff: step_off(); report_operator()      //   이후 창의 step 적재를 모두 생략 — 그 선택 열쇠들의 참조는 능력 부재
  return q                           //   이 관측이 무엇을 덮는가는 정본 C2 (e-defined.typ:463 FP_STATUS 행) — 읽기 수락 앞의 조회는 모두 끝났고 그 뒤 조회는 반영된 표를 본다.
                                     //   0xF08 읽기 하나가 유계로 정지할 수 있다(반영 전 — 장치 의무) — T_WALK 폴링이 끊는 대기가 아니다 (hiu_abi §2.7 4)
fp_tail_read():                      // (선택) 정지 확인 뒤 · 첫 FP_PROG 앞 — 관문 1차가 앞선 걷기의 (4) 를 보지 않게
  wait_bounded(T_FLT)
  if read(0xF80)[15] == 1:
    raise fault_disposition(step_fault_origin(fp_ok))  //   [19] 선례 — fp_ok 이면 DeviceFault, 아니면 Y4Defect (T_FLT 는 (4) 의 상한이 아니므로 그 뒤에 보여도 처분은 같다)
global_config(L, q) -> own_refused:
  own_refused = false
  for f in L 의 대기 OWN_SET:        //   할당기가 고른 프레임 — 소유권 표의 형상 지식 없이 (U4 — 소유권 표의 형상은 판의 일이 아니다, 03-hiu.typ:1689)
    for attempt in 0..=K_OWN:        //   첫 이양 + 다른 프레임 K_OWN 회 = 이양 K_OWN + 1 회 · swap K_OWN 회 (for … else — break 없이 끝나면 else)
      mmio.write(0x208, f); mmio.write(0x200, f)        // OWN_SET f → OWN_IDX ← f — 사이에 f 의 다른 전이 · OWN_IDX 의 다른 기입 없음 (E.7 7항 e-defined.typ:1388)
      if own_readback(true, read(0x204)[0]) == Confirmed:
        ledger.own[f] ← 장치 소유; break                //   커밋을 되읽어 확인한 뒤에야 셈한다 (E.1.1 5항 :208) — 확인된 프레임만 사상한다 (hiu_abi §2.7 1a)
      own_refused = true                                //   Refused — 무효과, 「대상은 호스트 소유로 남으며」 (:1387). 사상하지 않는다
      if attempt < K_OWN: f = pool.swap(f)              //   거절된 프레임은 호스트 풀로 돌리고 다른 프레임을 받는다 (swap 은 K_OWN 회까지)
      else: pool.release(f)                             //   마지막 시도의 거절 프레임은 풀로 돌리기만 하고 새 프레임은 받지 않는다 — 프레임이 새지 않는다
    else: L.pool_short ← true                           //   K_OWN 소진 — (나) 의 OwnSetRefused 가 거친 「풀 확장 실패」로 보고 (hiu_abi §1.3 [7])
  LNID 엔트리 V=1 (lnid_write_ok) · LBID 엔트리 (lbid_write_ok)
  if q == SkipLoad:
    L.bind ← L.bind 에서 L.load 의 슬롯을 뺀 것       //   이 창은 FP_PROG 를 쓰지 않는다 — 그 열쇠의 참조는 능력 부재 (다음 창에서 다시)
  elif L.load ≠ ∅:
    for (s, img) in L.load:
      require fp_load_ok(fp, s, img, P, stepMax)        //   fp_slot_writable(결속 · 걷기 없음) ∧ fp_image_ok — 모든 기입이 fp_prog_ok 라 (1) · (2) 를 내지 않는다 (fp_load_ok_words)
      for i in 0..img.len(): mmio.write(0xF00, fp_image_word(s, img, i))
      fp ← fp_ledger_load(fp, s, img, P, stepMax)       //   Loaded { gated: false } — fp_load_ok 아래에서만 · 아직 결속할 수 없다 (fp_load_then_gate)
    wait_bounded(T_FLT); g = read(0xF80)[15]            //   FP 관문 — FP_PROG · FP_BIND 는 WO 이고 0xF08 에도 적재 결과가 없다
    if step_gate(g) == Refused:                         //   적합 경로에서 날 수 없는 [15] — 다시 싣지 않고 결속하지 않는다
      raise step_gate_action(Refused, fp_ok)            //   [19] 선례 — fp_ok 이면 DeviceFault, 아니면 Y4Defect. 들어오는 임대의 거절로 읽지 않는다
    fp ← fp_ledger_gate_pass(fp)                        //   Clean — Loaded 슬롯이 결속할 수 있게 된다 (fp_mirror_gate_pass)
  for (k, s) in L.bind:                                 //   창의 마지막 기입 — 그 뒤 T_FLT → (나)
    require fp_key_bindable(fp, k, s)                   //   장부상 결속되지 않은 열쇠 · 이미지가 준비된 슬롯 — 재결속은 쓰지 않는다 (fp_rebind_breaks_wf)
    mmio.write(0xF04, fp_bind_word(true, s, k))         //   fp_bind_ok(예약 0 · 슬롯 < P) — [6] 도 「`[15]` 의 원천」 (3) 도 내지 않는다 (fp_key_bindable_word_ok)
    fp ← fp_ledger_bind(fp, k, s)                       //   fp_mirror_bind — 장치가 결속을 받든 떨구든 fp_mirror 는 선다
  return own_refused
```

장치는 드레인 술어 D(「옛 결속 아래 받아들인 일이 하나도 남지 않음」, `e-defined.typ:332` 2항)가 참일 때만 다른 값 기입을 적용한다. v1.0 의
드레인 판정 D 는 `DRAIN_ALLZERO`(같은 사이클 접수 포함)이고, 명령 링 항은 v1.0 에 링 소비 회로가 없어
공허하다(정본 `03-hiu.typ:1274-1281` — 링 소비 회로가 착지하는 개정에서 꺼냄 · 해독 단계 항을 더한다).
결과는 되읽기 `[8]` A · `[9]` D 로 보고된다. 같은 값 기입은 fault 없이 A=1 과 D 를 보고하므로 「결속
확인 + 드레인 관측」 탐침으로 쓸 수 있다(A=1 ∧ D=0 이면 드레인 미완 — 유계 재드레인, 그 밖의 결과인 A = 0 · 결속 ≠ cur
는 장치 모순 → `DeviceFault`; Verus `lease::active_part::probe_outcome`, `hiu_abi.md` §2.7 강제 드레인 창). 되읽은 (A, D) 를 이 기입의
결과로 읽는 근거(유일 기입자 · 정렬 32b 전폭 · 사이 reset 없음, 그리고 규범이 아닌 사실로 재구현의
정렬 되읽기가 판정 사이클 뒤 수락된다는 것)와 `[6]` 가시 대기의 처분은
`hiu_abi.md` §2.4 3 이 적는다. (A, D, 결속) 되읽기의 처분 표는 Verus `lease::active_part::switch_outcome`
이고, 모든 조합이 처분을 하나씩 갖는다는 것은 `lease::active_part::switch_outcome_exhaustive`
다. 창 진입 (가) · 창 기준선 (나)의 처분은 Verus `lease::window::dispose_pre_window` · `window_verdict`(망라 —
`pre_window_exhaustive` · `window_verdict_exhaustive`; `Y4Defect` = `LeaseInvariant` 는 `[5]` · `[6]` 이거나 `[15]` ∧ 장부
어긋남뿐이다 — `window_halts_only_on_y4_evidence`, `[7]` 만 선 읽기는 정지에 이르지 않는다 — `fault7_never_halts`,
`[15]` 는 `[19]` 선례 — `fault15_follows_step_fault_origin`)이고, 창 기입의 사전조건은 `lnid_write_ok` · `lbid_write_ok`(`lease::lnid` · `lease::lbid`)와
`fp_unbind_ok` · `fp_load_ok` · `fp_key_bindable`(`lease::step` — 낱말 수준의 `fp_prog_ok` · `fp_bind_ok` 를 함의한다:
`fp_load_ok_words` · `fp_key_bindable_word_ok`)다. step 장부가 장치를 비춘다는 것은 불변식 `fp_mirror`(장치 모델
`FpDevice` — 결속은 덮어쓰기 · 결속되지 않은 열쇠의 해제는 무연산 · 걷기는 조회 때 래치한 슬롯을 계속 쓴다)이고 창의
전이마다 보존된다(`fp_mirror_unbind_ledger_keys` — 창의 `fp_unbind_all`, 부팅 sweep 은 `fp_mirror_unbind_all` · `fp_mirror_quiesced` · `fp_mirror_load` · `fp_mirror_gate_pass` ·
`fp_mirror_bind`). 결속된 슬롯에 FP_PROG 를 쓰지 않는다는 것은 `fp_bound_not_writable` · `fp_prog_target_idle`, 재적재가
정지 확인을 기다린다는 것은 `fp_reload_needs_quiesce` · `fp_quiesced_all_writable` 이다. 정지 확인 · 관문의 처분은
`fp_quiesce_action`(`fp_quiesce_k_consecutive` — `K_WALK` 연속 소진은 `StepOff`) · `step_gate` · `step_gate_action`
(`step_gate_exhaustive`)이고, step 경로가 `LeaseInvariant` 에 이르는 것은 관문의 `[15]` ∧ 장부 어긋남뿐이다
(`step_path_halts_only_on_ledger`). `[7]` · `[15]` 의 원천은 `f6bc4df` 가 문면으로 닫았다(「`[7]` 과 `[15]` 의 원천은
동결 전 호스트 측 대조가 문면으로 닫았다」 `e-defined.typ:527` — D1 · D2 해소). 정지 확인이 무엇을 덮는가는 C2 로
정본에 섰다(`e-defined.typ:463` FP_STATUS 행 「`[1]` 의 읽기 순서」 — 4항 `:208` 이 가리킨다). v1.0 에서는 일의 원천이 외부 제시라 Y4 가 투입을 멈출 수 없으므로, I7 의 격리 주장은 명령 링
소비 회로가 착지한 판에서 한다(`03-hiu.typ:1311-1320`) — v1.0 의 I7 은 mock 검증 대상이다. D 원천
시점(기입 사이클 접수가 판정 사이클 D 에 든다)의 RTL 측정은 합성 서술에 머문다(측정 한정 — 이월,
`hiu_abi.md` §5.3). Y4 는 그 측정에도, 재구현의 판정 사이클 정지(HIU 접수 · 정렬 `0xE14` 접근만 — 3.9
`03-hiu.typ:1556`)에도 기대지 않는다.

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
1. 빈 partition 선택 — partition_id < `0xFF8[7:0]`(실재 파티션, `hiu_abi.md` §1.4) 가운데서 (없으면
   `Err(NoPartition)`). 그래서 `0xE14` 에 쓰는 next 는 언제나 유효 값이다(I7 4 · `hiu_abi.md` §2.4 3).
2. `NonceAllocator.next()` (I3) — 단조 카운터 + 호스트 식별자 prefix. 장치 엔트로피
   불사용.
3. `KeyDeriver.derive(host_secret, partition_id, lease_serial) -> MaskKey` — 호스트
   마스터 시크릿(boot-time, 캡슐 외부 노출 불가)에서 도출.
4. `dva_map` 구성 — 임대 메모리를 장치 VA 공간(v1.0 VA 44b)에 사상한다. 프레임은
   PA < 2^44 풀에서만 내준다(`hiu_abi.md` §3). 미적중 정지가 영구가 되지 않도록 임대
   메모리는 pre-pin 해 두어 언제나 채울 수 있게 한다. 사전 채움은 성능 힌트다
   (`hiu_abi.md` §2.2). 소유권 이양(`OWN_SET`)과 LNID/LBID · step 적재는 전역 구성이라 그 임대의
   전환-진입 창(I7 step 3)에서 한다 — 임대는 전환되기 전에는 돌지 않으므로 지연 손실이 없다. 소유권 이양은
   `OWN_STATUS` 되읽기로 커밋이 확인된 프레임만 셈하고 사상한다(E.1.1 5항 `e-defined.typ:208`). 거절된 프레임은
   무효과로 호스트 소유에 남으므로(E.7 6항 `:1387`) 호스트 풀로 돌리고 다른 프레임으로 다시 이양한다(상한 `K_OWN` —
   플랫폼 레지스트리, 사용자 결정 U4; 소유권 표의 형상 지식은 할당기에 박지 않는다). 그래도 모자라면 그 임대에는 거친
   「풀 확장 실패」만 보고하고 프레임 · 시점은 알리지 않는다 — 프레임 단위 보고는 다른 임대의 점유를 흘린다
   (`hiu_abi.md` §1.3 `[7]` · `y4_abi.md`). step 은 플랫폼이 지정한 이미지만 싣고 테넌트는 선택 열쇠로 고르기만
   한다(U1 — `hiu_abi.md` §2.7 「step 적재 규칙」 1). 정지 확인이 소진된 창에서는 그 적재를 생략하고(`K_WALK` 창
   연속이면 step 기능을 끈다 — `StepOff`), FP 관문이 `[15]` 를 보면 결속하지 않고 `[19]` 선례로 귀속한다(장부가
   맞으면 `DeviceFault`, 어긋나면 `LeaseInvariant`) — 결속되지 않은 열쇠의 참조는 능력 부재다. `[15]` 는 테넌트에
   보고하지 않는다(`hiu_abi.md` §1.3 `[15]`).
5. `RevocationTokenStore.issue()` — 단조 카운터 + HMAC.
6. `LeaseCap` 조립 → atomic-rotate (I4) → tenant 에게 cap 핸들 반환.

### 3.2 use

게스트는 HIU 구성 표면 **어느 구역에도 직접 닿지 않는다** — 4 KiB 한 페이지는 쪼갤
수 없으므로 Y4 가 페이지 전체를 capability 검증 뒤 대행한다(Y4 결정, `hiu_abi.md` §1
머리). 게스트 SDK 가 LNID/LBID 사상표 · step 선택 열쇠(v1.0 은 플랫폼이 지정한 이미지만 싣는다 — 게스트는 이미지를
대지 않는다, U1) · 재조립 소거를 요청하는 형식은
`docs/y4_abi.md` 소관이다. 장치 쪽에서 요청의 파티션은 「어느 임대
아래 도착했는가」가 정한다(정본 §3.6) — 토큰 값이 아니다. 그 결속을 장치에 알리는 표면은
`0xE14` ACTIVE_PART(정본 `e-defined.typ:329`, I7)다. LBID 사상 요청은 Y4 가 대행 검증한다(`hiu_abi.md` §2.8 — v1.0 장치는
겹치는 구간을 판정하지 않고 수락하며 비겹침은 호스트 의무다, 정본 `03-hiu.typ:1094` · `e-defined.typ:208`).

### 3.3 release / expiry / revoke

세 경로 모두 동일한 atomic-rotate (I4) 로 수렴 — `L_new` 자리에는 빈 임대
("idle lease") 가 들어간다: 그 파티션 flush 상태(항목 미기입) + I3 경로(KDF + 카운터)로
새로 지은 키·논스 기입·재키잉(옛 키 덮어쓰기 — 장치 측 키 바이트 소거 규범은 정본에
없다, G1) + 그 파티션 소유 페이지의 `0x20C` 회수(기입마다 `OWN_STATUS` 되읽기)와 LBID · LNID 엔트리 V=0 · 그
임대가 결속한 선택 열쇠의 FP_BIND en = 0 해제(I7 `fp_unbind_all` — 그 슬롯을 다시 쓰는 것은 정지 확인 `fp_quiesce`
뒤다). 회수는 p 가 결속돼
있었으면 I4 step 2b(⑵ flush 뒤 `DRAIN_ALLZERO` 재관측 뒤)에서, 아니면 다음 전환 창(I7 step 3)에서 한다 — 그동안 페이지는 「회수 대기 ·
장치 소유」이고, p 는 flush 됐고 어느 살아 있는 `dva_map` 에도 없으므로 도달할 수 없다.

---

## 4. 비-목표 (out of scope for v0.9)

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

## 5. v0.9 미해결 항목

1. (§1) `IsaLevel` 에 WT64v2 자리잡기 — WT64v1 lock 이후 v2 가 등장하면
   변경. 현재는 enum 의 미래 변종으로 reserved.
2. (§2 I3) 부팅마다 짓는 `host_secret` 의 엔트로피 원천과 CSPRNG 구현 — Verus 가능한
   구현체(예: chacha20 Rust crate 의 verified port) 선택. 호스트 엔트로피 원천
   (RDSEED/TPM 등)은 `hw_mechanism_abstraction.md` 레지스트리 대상.
3. (§3.1 step 3) `KeyDeriver` 가 HKDF-SHA256 vs HKDF-SHA3 vs BLAKE3 중
   무엇을 쓸지 — 합의 후 고정.
4. (§3.2) cap 검증 경로의 IPC overhead 측정 — Phase C mock 으로 estimate.
5. (§I4) atomic-rotate 의 IRQ-disable / preempt-disable 경계 — seL4
   notification 모델과 정합성 확인. 드레인·재키잉 대기 · W 가시 대기(`T_FLT`) · step 정지 확인 대기(`T_WALK`)의 시간
   상한 값과 정지 확인 연속 소진 상한 `K_WALK` · OWN_SET 재시도 상한 `K_OWN` — 플랫폼 레지스트리
   (`hw_mechanism_abstraction.md` §2). 정본은 사이클 수를 걸지 않는다.
6. ~~(§3.1 step 1, I1) 실재 파티션 수를 Y4 가 어떻게 아는가~~ — `0xFF8` HIU_SHAPE
   `[7:0]`(정본 `e-defined.typ:503`, `hiu_abi.md` §1.4)로 닫는다. RTL W1 `9b6eee6` 착지 — 참조 형상
   (파티션 4 × 칸 16)에서 `0x0000_1004`(3.9 `03-hiu.typ:1556`). 판 0 장치도 이 값을 내지만 Y4 는 판 0 에서
   해석하지 않는다(`hiu_abi.md` §1.4).
7. ~~(I4 · I7) `LeaseInvariant` 의 처분~~ — 2026-10-06 사용자 결정으로 닫음: 원인을 먼저
   귀속하고(`hiu_abi.md` §2.6), Y4 쪽 원인만 `LeaseInvariant`(하이퍼바이저 정지), 장치 쪽 ·
   귀속 불가는 새 변형 `DeviceFault`(장치 격리)로 처리한다. `error.rs` 에 `DeviceFault` ·
   `FaultOrigin` · `fault_disposition` 착지(거울 크레이트 포함). Y4 의도 결함과 절차 결함을
   가르려면 겹친 상대 파티션이 필요하다 — `0xEC4` 캡처 워드가 v1.0 에 들어갔다(합의 2026-10-09 · 정본
   `e-defined.typ:347` · RTL W1 `9b6eee6` 착지).
8. ~~(I2 · §3.1) 형상 비노출 표면(Y4 제안)이 받아들여지지 않으면 `dva_map` 모델을 다시 연다~~ —
   2026-10-09 WaveTensor 가 받았다(사용자 결정).
9. (I2 부속) `pending_reclaim` 상한과 강제 드레인 정책의 값 — 임대 메모리 예산과 함께 Phase C 에서
   정한다. WaveTensor 에 파티션별 「변환 펜스」 RO 비트를 v1.x 후보로 제안했다(있으면 전역 드레인 없이
   회수 가능 — B.5 「파티션별 변환 펜스」 행 `b-gaps.typ:2168`).
10. (§3.2 · `hiu_abi.md` §2.8 · §1.2.1 (C)) LBID 슬롯 수 · LNID 등록 용량 · 배달 지점 수 — `0xFF8` 에 없어 플랫폼
   레지스트리 값을 쓴다(장치 알림 워드는 v1.x 후보 — `hiu_abi.md` §5.3 C10). step 프로그램 깊이 P · stepMax 는 `0xF0C`
   FP_SHAPE 가 알린다(`e-defined.typ:463` — D4 (나) 해소; 오늘 RTL 은 배선하지 않았다, `:738`). 플랫폼 레지스트리의
   (P, stepMax)는 판 0 · mock 동안의 출처이자 교차 확인으로 내렸고, 장치 값과 다르면 장치 모순이다(`DeviceFault` —
   사용자 결정 U2, `fp_shape_verdict` · `fp_shape_registry_mismatch`). 엔트리 「길이」의 뜻(B.5 미결)이 닫히면
   `lbid_entry_ok` 를 다시 본다.
11. ~~(`hiu_abi.md` §1.3) 비소유 접근 전용 fault 비트~~ — 2026-10-09 WaveTensor 가 받아 `[19]` 「비소유 프레임
   접근」으로 신설했다(정정 커밋 `8af46df` · 정본 `e-defined.typ:523` · RTL W1 `9b6eee6` 착지). 처분은 `error.rs`
   `nonowned_access_origin`.
12. ~~(I4 2b · I7 3) `[7]` 과 `[15]` 의 원천~~ — 2026-10-18 `f6bc4df` 대조로 닫음: 정본이 규범으로 답했다(D1 — E.7 6항
   `e-defined.typ:1387` · 「`[7]` 은 E.7 6항의 OWN_SET 거절 하나」 `:527`; D2 — 「`[15]` 의 원천」 닫힌 넷 `:464`).
   OWN_SET 거절은 `OwnSetRefused`(다른 프레임 재시도 상한 `K_OWN` · 거친 보고 — U4)이고, `[15]` 는 rc4 에서 `[19]`
   선례 귀속이 됐다(`b16fc07` `:466`). C1 · C2 도 `b16fc07` 로 해소됐다(`hiu_abi.md` §5.3).

---

## 6. 다음 단계

- 본 v0.9 → `hiu_abi.md` v1.0-rc4(인용 기준 동결 sha `b16fc07`)의 WT 대조와 sign-off → Y4 sign-off 발효(`hiu_abi.md`
  §5.4 의 조건) → `v1.0 frozen` 마킹.
- Phase C 진입(hiu_abi v1.0 frozen) 전에 본 spec 을 `proofs/lease/spec.rs` (Verus) 로 옮겨
  invariant I1–I7 의 machine-checked 버전 작성(`hiu_abi.md` §6 목록).
- `kernel/lease/` Rust 구현은 그 spec 이 통과한 후에만 land
  (formal-first 원칙).
