---
name: 드라이버/SDK 개발환경 · Rust 툴체인 · Y4 ABI 선행조건 방침 (사용자 2026-08-30)
description: 사용자 방침 축자 보존 + Y4 국소 실측 델타. 권장 개발환경 5종(열린 목록)·2024 Edition·stable >= 1.97 하한·asm Rust 포팅(착수 시기 미정)·Y4 ABI 확정이 드라이버/SDK 의 선행 조건(!!). MSRV 정합은 verus-fork 작업 후 재판정으로 유보. (2026-09-14 갱신: Y4 ABI v0 draft = docs/y4_abi.md 착수 — §6-4/§6-5 해소, v1.0 frozen 은 남음 — §8.)
type: project
originSessionId: 01QzUmzau38A2GzjbGNoFRTf
modified: 2026-09-14T05:11:33.301Z
---
2026-08-30 사용자가 드라이버 / SDK 개발 방침을 제시했다.  방침은 Y4 ·
wavetensor-drivers · wavetensor-sdk · WaveTensor 네 저장소에 동시에
걸리며, 본 메모는 그중 **Y4 국소 델타**를 기록한다.

## 1. 사용자 방침 — 축자 (해석으로 대체하지 말 것)

```
* 드라이버 및 SDK 개발을 위한 권장 개발환경들
    - Linux
        -- Alpine Linux with kernel version 7.x + musl libc + LLVM: for musl ABI build
        -- LineageOS 23.2 (based on AOSP 16): for Android(Bionic ABI) build
        -- openSUSE Slowroll with kernel version 7.x + LLVM: for glibc ABI build
    - NetBSD version 11.0
    - DragonFlyBSD version 6.4.2
    ...
* 현재 Python(3.x)으로 작성되어있는 어셈블러는 pure Rust로 port할 예정. 단, 착수 시기는 아직 미정.
* 모든 Rust 코드들은 2024 Edition, stable toolchain version >= 1.97을 기준으로 함.
* Y4(로컬 저장소 위치: `~/Y4/`)용 드라이버 및 SDK 개발을 위해서는 구체적인 Y4 ABI부터 먼저 확정해야 함!!
```

## 2. 문면이 말하는 것과 말하지 않는 것

- **`...` 는 목록이 열려 있다는 선언이다.** 위 5종을 「전부」로 적으면
  방침을 왜곡한다.  「현재까지 명시된 것」으로만 인용하고, 항목이 추가될
  자리를 남겨 둘 것.
- **「권장 개발환경」이지 「지원 타깃」이 아니다.**  *어디서 개발하는가*
  와 *무엇 위에서 도는가* 는 별개 축이다.  다만 musl / Bionic / glibc 를
  명시로 가른 것은 **빌드 타깃을 셋으로 갈랐다**는 읽기를 허용한다.  그
  읽기가 맞는지는 §6 미결 — 지금 단정하지 말 것.
- **`>= 1.97` 은 하한이지 핀이 아니다.**  「1.97 로 고정」이 아니다.
- **asm 포팅은 「예정」이고 착수 시기 미정.**  「진행 중」·「다음 단계」로
  승격하지 말 것.  시기 미정 자체가 방침의 일부다.  (Y4 는 Python
  어셈블러 의존 0 — 이 축은 WaveTensor `asm/wavetensor_asm.py` 소관.)
  **「착수 시기 미정」 ≠ 「동결」** — 사용자 추가 판정 2026-08-30 축자:
  「Python 어셈블러는 내가 명시적으로 지시하기 전까지는 동결하지
  말도록.」  즉 포팅 착수 전까지 Python 쪽은 계속 개량된다.
- **`!!` 는 강조다.**  Y4 ABI 확정은 **선행 조건**이고, 그 전에 Y4 용
  드라이버 / SDK 를 짓는 것은 방침 위반이다 — 블로커.

## 3. Y4 국소 실측 (2026-08-30, HEAD `d404040`, working tree clean)

| 자리 | 실측값 | 방침 대비 |
|---|---|---|
| `Cargo.toml:36` `[workspace.package] edition` | `"2024"` | ✅ 일치 |
| `Cargo.toml:37` `rust-version` | `"1.94"` | ❌ 하한 1.97 미만 |
| `kernel/Cargo.toml:8-9` | `edition = "2024"` / `rust-version = "1.94"` | ❌ 동일.  `kernel` 은 workspace `exclude` 라 **상속 불가 — 독립 수정 필요** |
| `alloc` / `capsules` / `ipc` / `scudo-sys` | `rust-version.workspace = true` | workspace 값 따라감 |
| `rust-toolchain.toml` | `channel = "stable"` (+ rustfmt/clippy/rust-src, profile minimal) | 버전 핀 아님 — **롤링 별칭** |
| 이 호스트에서 `stable` 이 푸는 값 | `rustc 1.96.0 (ac68faa20 2026-05-25)` / `cargo 1.96.0` | ❌ 오늘 빌드하면 하한 미만으로 빌드된다 |
| rustup 설치 목록 | `1.97.1-x86_64-unknown-linux-gnu` **설치되어 있음** | 그런데 `stable` 별칭이 그것을 고르지 않는다 |
| `verus-fork/rust-toolchain.toml` | `channel = "1.95.0"`, components 에 `rustc-dev` + `llvm-tools` | ❌ 하한 미만.  rustc 내부 API 결합이라 임의 상향 불가.  외부 저장소(`newsniper-org/verus`) 서브모듈이라 **Y4 단독으로 못 정한다** |
| `.gitmodules` `verus-fork` | `branch = backend-pluggable` | 커밋이 아니라 **브랜치 추적** |

**문서 측 MSRV 기재는 넷이고 서로 어긋난다** (실측):

- `CLAUDE.md:106` — `rust-toolchain.toml  channel = stable (MSRV 1.94)`
- `README.md:93` — `rust-toolchain.toml  pins channel = stable (MSRV 1.94)`
- `Cargo.toml:37` + `kernel/Cargo.toml:9` — `1.94`
- `.claude-memories/y4_pending_topics.md:294` — **「Rust toolchain 의무
  1.96」** ← 나머지 넷과 다른 숫자.  새 방침 이전부터 있던 내부 불일치.
- `.brainstormings/20260826-231914-reproducible-build-supply-chain.md`
  §2 — 「기존 pin(1.94/submodule/Cargo.lock)」 로 1.94 를 전제한 논증.

값을 올릴 때 **같은 커밋에서 함께 움직여야 하는 자리가 최소 다섯**이다.

### 3.1 방침의 나머지 축이 Y4 에서 서는 자리

- **CI 없음.**  `.github/` 부재, 최상위 CI 설정 0건.  `justfile:292`
  `tools-check` 는 로컬 도구 존재 확인이지 CI 가 아니다.  **권장
  개발환경 5종 중 Y4 에 결선된 것은 0개.**  즉 이 축의 작업 크기는
  「기존 CI 에 OS 추가」가 아니라 **「CI 를 처음 만든다」** 이다.
- **libc 축에 Y4 가 안 들어간다.**  `kernel/.cargo/config.toml` 타깃 =
  `x86_64-unknown-none`.  musl / Bionic / glibc 셋 중 어느 것도 아니다.
  세 ABI 분기는 호스트측 드라이버 / SDK 빌드를 가르는 축이지 Y4 자체
  빌드를 기술하지 않는다.
- **aarch64 미개방.**  `CLAUDE.md` / `y4_build_decisions.md` D2 —
  "x86_64 first (Phase B). 다른 arch 는 해당 형상 작업 시작 시".
  LineageOS 23.2 (AOSP 16, Bionic) 축은 사실상 aarch64 를 함의한다.
- **롤링 참조가 이미 셋.**  `channel = "stable"` ·
  `unified-toolkit-pin.toml` 의 `channel = "testing"  # rolling`
  (adsmt / adsmt-contrib) · `verus-fork` 의 브랜치 추적.  rustc 핀
  정책이 정해지면 같은 원칙이 나머지 둘에도 걸리는지는 §6 미결.

## 4. 사용자 판정 (2026-08-30) — 미결이 아니라 **선행 조건이 명명된 유보**

축자: 「일단 verus-fork 쪽 개발 작업들(custom backends, 최신 Verus
upstream으로부터 rebase 등...) 부터 먼저 해놓고 나서 추후 결정.」

따라서 §3 의 MSRV / 툴체인 충돌 셋은 **지금 맞추지 않는다.**

| 충돌 | 내용 | 처분 |
|---|---|---|
| ① | `Cargo.toml` + `kernel/Cargo.toml` 의 `rust-version = "1.94"` < 하한 `1.97` | 유보 |
| ② | `verus-fork` 의 `1.95.0` 하드 핀 — Verus 는 rustc 내부 API 결합 fork 라 상류가 올리기 전엔 못 따라간다.  「모든 Rust 코드」에 증명기 포크가 드는지는 방침의 열린 지점 | 유보 |
| ③ | **집행하는 기계가 없다** — `stable` 은 핀이 아니라 롤링 참조, `>= 1.97` 도 하한이지 핀이 아니다.  cargo 가 집행하는 것은 `rust-version` 뿐인데 그 값이 1.94 라 1.96 을 통과시킨다 | 유보 해제 시점에 함께 처분 |

**선행 작업** (사용자가 연 목록, `...` 로 열려 있음): verus-fork custom
backends · 최신 Verus upstream 으로부터 rebase.

→ 「미정」이 아니라 **「verus-fork 작업 후 재판정」**.  이 구분을 지울 것.

## 5. 「Y4 ABI」가 무엇을 가리키는가 — 실측 판정

**`docs/hiu_abi.md` 는 그 ABI 가 **아니다**.  그리고 그 ABI 는 문서가
존재하지 않는다.**

Y4 안에서 「ABI」로 불리는 표면은 넷이고 서로 다른 경계다:

- **A. `docs/hiu_abi.md`** — "Y4 ↔ HIU ABI (v0 — draft, not frozen)".
  Y4(호스트)가 가속기 HIU 를 향해 보는 PCIe BAR0 MMIO 레지스터 맵 +
  타이밍 계약.  **방향이 반대다** — Y4 가 *소비*하는 ABI 이지, Y4 위의
  드라이버 / SDK 가 *호출*하는 ABI 가 아니다.  이 판정은 추론이 아니라
  `docs/glossary.md` §10 이 이미 명문화하고 있다("WT64v1 ISA 자체는 Y4 의
  ABI 경계가 **아니다**").
- **B. 게스트 ↔ 하이퍼바이저 hypercall ABI** — **문서 없음.**
  `docs/amdv_safety.md` §S7.2 (576–578행) 가 "hypercall ABI 자체는 별도
  spec (`y4-hypercall` repo)" 로 미룬다.  실측: `/home/ybi/y4-hypercall/`
  **존재하지 않는다.**  게다가 그 repo 는 2026-05-04 ARCH-II' 채택 때
  `docs/vmm_arch.md` §5.3 에서 **「사용자측 CLI / API tooling repo」로
  재정의**되었다 — core spec 의 집이 아니게 되었다.  즉 지정된 소재지가
  무효이거나 최소한 미갱신.
- **C. seL4 fork 측 raw-SVM syscall ABI (D1a)** — `docs/vmm_arch.md`
  §404–407 의 4 객체 + 6 syscall.  `docs/phase_plan.md` Phase C 차단
  의존 목록에서 **5 = (열림)**.  host-side / root-task-facing 이지
  게스트-facing 이 아니다.
- **D. `docs/lease_capability.md`** — "스키마 v0 (draft, not frozen)".
  §3.1 이 `LeaseManager.acquire(req_attrs) -> Result<LeaseCap,
  AcquireError>` 라는 **Rust 함수 시그니처**를 준다 — ABI 가 아니라
  in-process API.  §3.2 는 "게스트의 모든 HIU 접근은 cap 검증 경로를
  거친다"고만 쓰고 **그 경로가 IPC 인지 hypercall 인지 MMIO trap 인지
  명시하지 않는다**.  §5-4 가 "cap 검증 경로의 IPC overhead 측정" 을
  미결로 남긴다.

**결론:** 「Y4용 드라이버 / SDK」가 부를 ABI = **B + D 의 교집합**
(게스트가 lease 를 취득하고 HIU MMIO 를 두드리는 규약).  현재 **어느
파일에도 정의되어 있지 않다.**  사용자 방침의 `!!` 는 정확한 지점을
짚었다.

부수 실측 — 계획 문서가 반복 참조하는 sibling repo 중 **디스크에 없는
것**: `/home/ybi/y4-hypercall/`, `/home/ybi/y4-drivers/`,
`/home/ybi/y4-verus2isabelle/`.  (`/home/ybi/wavetensor-drivers`,
`/home/ybi/wavetensor-sdk` 는 존재.)

### 5.1 블로커의 실제 크기

- v1.0 frozen 6건 (전부 5월): `amdv_safety` · `vmm_arch` ·
  `sel4_fork_policy` · `verus_to_isabelle` · `power_safety` · `power_arch`.
- v0 draft, 동결 안 됨 3건: `hiu_abi.md` (§5 미해결 5항, 양측 sign-off
  필요, `HIU_ABI_VERSION` 여전히 `0x0000_0000`) · `lease_capability.md`
  (§5 미해결 5항) · `cpu_virt_compat.md` (§8 unresolved).
- Phase A → B 트리거(`hiu_abi.md` v1.0 동결)가 **여전히 미도달**.
  Phase C 차단 의존 8항 중 5·6·7·8 전부 (열림).
- 즉 **Y4 는 게스트를 아직 한 번도 띄운 적이 없다** — 「Y4용 드라이버 /
  SDK」의 실행 기반 자체가 미구축이다.

## 6. 미결 — 답하지 말고 등재 (사람 몫)

방침이 답하지 않았고, 그럴듯한 값을 채워 넣는 것이 이 프로젝트가 반복해서
밟은 함정이다.

1. **하한인가 핀인가.**  ⒜ `channel = "stable"` 유지 + 하한은
   `rust-version` 으로만 (재현성은 별도 lock 캡처) ⒝ `channel = "1.97.1"`
   핀 ⒞ 둘 다.  `unified-toolkit-pin.lock` 같은 build-time 해시 캡처
   장치가 rustc 에는 아직 없다.  *(§4 유보 대상 — verus-fork 작업 후.)*
2. **`rust-version` 을 하한과 같은 값으로 올리는가.**  올리면 Y4 크레이트를
   1.94~1.96 에서 빌드하는 길이 닫힌다.  그것이 의도인가, 아니면
   `rust-version`(소비자 MSRV)과 `rust-toolchain`(개발 툴체인)을 서로 다른
   값으로 두는가.  *(§4 유보 대상.)*
3. **verus-fork 의 `1.95.0` 은 위반인가 면제인가.**  새 방침이 ⒜ Y4 자체
   크레이트에만 ⒝ 서브모듈 포함 전 저장소에 ⒞ 검증 툴체인은 명시 예외 —
   셋 중 무엇인가.  *(§4 유보 대상.)*
4. **「Y4용 드라이버 및 SDK」의 「Y4」는 어느 경계인가** — §5 의 B 인가,
   B+D 인가, 넷 전부인가.  이것이 블로커의 크기를 결정한다.
5. **그 spec 은 어디에 사는가** — ⒜ `Y4/docs/guest_abi.md` 신설 ⒝
   `y4-hypercall` 을 실제로 개설하고 spec 도 거기 ⒞ 또 다른 sibling repo.
6. **Y4 는 드라이버 / SDK 의 지원 타깃인가.**  권장 개발환경 목록에 Y4 가
   없는데 마지막 줄은 「Y4용 드라이버 및 SDK」를 전제한다.  ⒜ 타깃이지만
   개발환경은 아님 ⒝ 타깃이자 장차 개발환경 ⒞ 아직 어느 쪽도 아님.
7. **Y4 를 어느 개발환경에서 빌드하는가.**  「Y4 를 5종 전부에서」인지
   「Y4 는 이 축과 무관하고 드라이버 / SDK 만 갈라지는가」인지.  전자면
   CI 를 처음부터 만드는 작업이다.
8. **aarch64 를 언제 여는가.**  D2 의 「해당 형상 작업 시작 시」를 Bionic
   축이 앞당기는가, 아니면 Bionic 은 드라이버 / SDK 전용이고 Y4 는
   x86_64 로 남는가.
   *(부분 입력 — 사용자 추가 판정 2026-08-30 축자: 「Bionic은 드라이버
   및 NDK에서 필요로 하잖나?」  즉 Bionic 축의 소비자는 드라이버
   유저스페이스 + NDK 대면 C API 다.  이것은 **Y4 본체가 aarch64 를
   여는가**에는 답하지 않는다 — 본 항은 열린 채로 둔다.)*
9. **롤링 채널 의존을 이 규율이 함께 잡는가** — adsmt/adsmt-contrib 의
   `testing` 핀과 `verus-fork` 의 브랜치 추적에도 같은 원칙을 적용하는가,
   rustc 만의 결정인가.

## 7. 착지 기록 (2026-08-30) — 무엇을 했고 무엇을 안 했나

**한 것 (커밋 0, 사람 검토 대기):**

- 본 메모 신설 + `MEMORY.md` 색인 1행.
- `docs/phase_plan.md` Phase C 절 **말미 주석**으로 **게스트↔하이퍼바이저
  ABI spec** 항목 신설.  오늘 이 문서에는 그 항목이 **한 줄도 없었다.**

  ⚠️ **정정 (2026-08-30, 같은 날 수리).**  처음에는 이것을 *차단 의존
  목록 안*(5번 앞)에 넣고 「**4-abi 는 5~8 과 병렬 불가**」까지 적었다.
  **그것은 방침이 내리지 않은 일정 결정이었다.**  방침 문면은 「**Y4용
  드라이버 및 SDK** 개발」의 선행 조건이지 Y4 **본체**의 Phase C(첫
  게스트 부팅 · seL4 D1a 패치 · VMM capsule)를 막는다는 말이 아니다.
  근거로 들었던 「ABI 가 syscall 형상을 규정한다」는 인과는 실장자
  판단이며 방침에도 기존 문서에도 없다 — §6-9 를 미결로 세워 두고 같은
  질문에 본문에서 답해 버린 내부 모순이기도 했다.  지금은 목록 밖
  주석으로 내려 **y4-drivers 진입**에 거는 형태이고, 5~8 과의 시간
  관계는 미결로 남겼다.
- `docs/glossary.md` §11 신설 — ABI 표면 **넷**(A·B·C·D + 「ISA 는 경계
  아님」 행)을 한 표로 가름.  §10 이 「ISA 는 Y4 의 ABI 경계가 아니다」
  까지만 말하고 「그러면 게스트 SDK 가 Y4 에게 말하는 표면은 무엇인가」에
  답하지 않던 자리.

  ⚠️ **정정 (같은 날).**  ⑴ 이 줄은 「표면 **셋**」이라 적었으나 표는
  처음부터 **네 행**이었다 — 분모 불일치.  ⑵ 더 무거운 것: §11 본문이
  「드라이버/SDK 가 실제로 부르는 면은 **B + D 의 교집합이다**」라고
  **단정**했다.  그것이 바로 §6-4 가 미결로 세운 질문이다 — 권위가 낮은
  이 메모는 「모른다」, 권위가 높은 `docs/glossary.md` 는 「B+D 다」라고
  말하는 상태였다.  단정을 걷어내고 후보 셋(⒜ B 단독 ⒝ B+D ⒞ B·C·D
  전부)을 나열한 뒤 「사람이 정한다」로 되돌렸다.  형제 저장소 셋도 같은
  질문을 열어 두고 있다 (SDK [Q14] · drivers §7 ⑺ · WaveTensor §6).

**안 한 것 (의도적):**

- `Cargo.toml` / `kernel/Cargo.toml` / `CLAUDE.md:106` / `README.md:93` 의
  MSRV 값 — §4 유보.  사람이 값을 정한 뒤에만, **다섯 자리를 같은
  커밋에서** 움직인다.
- `rust-toolchain.toml` 의 `channel` — §6-1 이 닫히기 전엔 손대지 않는다.
- 개발환경 / CI 문서 — §6-6·§6-7 이 닫힌 뒤.  신설한다면 `docs/` 가
  아니라 `CONTRIBUTING.md` 절이 정합적이고, CI 는 `.github/workflows/`
  신설이 필요하다(오늘 그 디렉터리는 없다).
- `docs/hiu_abi.md` — **적지 말 것.**  새 방침은 이 문서가 정의하는
  경계(Y4→HIU MMIO)와 무관하다.  여기에 툴체인 / 게스트 ABI 이야기를
  섞으면 §0 동결 정책(양측 sign-off)이 오염된다.

**교차 저장소 기재 노후 (Y4 쪽 결함 아님, 보고만):**
`/home/ybi/wavetensor-drivers/.claude-memories/wtdrv_relationships.md:13`
및 그쪽 `CLAUDE.md:167` 이 Y4 를 "Currently paused … as of 2026-07-14"
로 적는다.  Y4 의 마지막 커밋은 **2026-08-27** (`d404040`) 이고 6~8월에
docs/brainstorming 커밋 14건이 있다.  코드는 멈춰 있으나 설계 작업은
계속되었다.

## 8. 갱신 (2026-09-14) — Y4 ABI v0 draft 착수 (사용자 지시)

사용자가 「블로커 해소에 바로 착수」 지시.  §5 가 "문서 없음" 으로 남겼던
게스트↔하이퍼바이저 ABI 를 **`docs/y4_abi.md` (v0 — draft, not frozen)** 로 신설.

- **§6-4 해소**: 「Y4용 드라이버/SDK」가 부르는 경계 = **게스트-대면** — §5 의
  B(hypercall) + D(lease/capability) + 가상 디바이스(virtio NIC / 로컬 IMS
  endpoint 등, WWAN 발제 근거).  사용자 지시로 확정(후보 ⒝ + C-service).
- **§6-5 해소**: 스펙의 집 = **`docs/y4_abi.md`**(후보 ⒜ 신설).
- **정정**: `y4-hypercall` = 사용자측 CLI/API tooling(`vmm_arch.md` §5 재정의
  확정, 디스크 미존재).  core VMM/hypercall = **Y4 워크스페이스 안**(orchestrator
  thin entry) — y4_abi.md §2 에 이대로 기재.
- **정합**: `glossary.md` §11 row B(없음→`y4_abi.md` §2)+해소 note ·
  `phase_plan.md` Phase C 주석(spec 이제 존재) · `CLAUDE.md` §5/§9 등재.
- **남은 것**: `y4_abi.md` **v1.0 frozen** 이 실제 SDK/driver 개발 선행조건(§7).
  §6-1/2/3(MSRV·toolchain)은 여전히 §4 유보(verus-fork 후).  §6-6/7/8/9 열림.
- **브랜치**: main 에 작성 → 사용자 지시로 `fip/isa-subcrates` 에 merge.
