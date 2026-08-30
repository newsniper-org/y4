---
name: just first-recipe trap — the Verus gate was hollow for 84 days
description: `just` with no arguments runs the FIRST recipe in the file, not the one named `default`; when `_check-verus` sat first, `just verus`/`just proofs`/`just ci` ran zero proofs and passed.
type: feedback
---

**`just` 는 이름이 `default` 인 레시피를 우대하지 않는다. 파일의 첫 레시피를 돈다** —
비공개(`_` 접두) 레시피여도 그렇다. 통제 실험(just 1.50)으로 확인:

```
_private:        ← just 가 이걸 돌린다
default: real    ← 이름이 default 여도 안 고른다
real:            ← 증명이 여기 있다
```
무인자 `just` 결과: `FIRST(_private) 이 돌았다`.

`proofs/verus/justfile` 에서 `_check-verus`(바이너리 존재 확인만)가 `default: verify`
위에 있었고, 루트 `justfile` 이 `verus:` → `cd proofs/verus && just` 이므로
**`just verus` → `just proofs` → `just ci` 가 증명을 한 줄도 돌리지 않았다.**

실행으로 확인 (2026-08-30):
- 수리 전 — 무인자 `just`: **rc=0, stdout 0바이트, stderr 0바이트**
- 수리 후 — `verification results:: 54 verified, 0 errors`

**기간과 아이러니:** 파일 도입 시(`1d6dba1`, 2026-05-04)에는 `default: verify` 가
첫 레시피로 **맞았다**. 뒤집은 것은 `bc0b698`(2026-06-07)이고 그 커밋 제목이
**`verification: R7.1~R7.12 sign-off — Verus fork single source of truth`** 다.
sign-off 를 선언한 그 커밋에서 게이트가 비었고 **84일**이 지났다.

**Why:** 「게이트가 돌았고 통과했다」가 아무것도 보증하지 않는 부류. 이름이 게이트인
것과 실제로 재는 것은 다르다. 그리고 바이너리는 **있었다**(2026-06-08 빌드) — 「도구가
없어서 안 돌았겠지」가 아니라 **도구가 있는데도 안 돌았다.**

**How to apply:**
1. **`default` 를 첫 레시피로 두고 그 자리를 지켜라.** `proofs/verus/justfile` 에
   왜 그래야 하는지를 주석으로 못박아 뒀다.
2. **저장소 전수로 확인**했다 — 루트 · `boot/` · `proofs/coq/` 는 처음부터 맞았고
   verus 하나만 어긋나 있었다. 새 justfile 을 만들 때 첫 레시피를 확인하라.
3. **`--dry-run` 출력을 조건 평가로 읽지 마라.** `_check-verus` 의 `if [ ! -x … ]`
   블록이 인쇄되는 것을 「바이너리가 없다」의 증거로 읽었다가 틀렸다 — dry-run 은
   `if` 를 **평가하지 않고 그대로 인쇄**한다. 무엇이 실제로 도는지는 **실행**해서
   재라(또는 dry-run 전량을 보고 뒤따르는 본체 줄의 유무로 갈라라).
4. 수락 기준은 「rc=0」이 아니라 **「기대한 일이 실제로 일어났다는 증거」**다 —
   여기서는 `54 verified` 줄. 출력 0바이트에 rc=0 이면 그것이 신호다.

Related: [[Y4 formal-first verification rule]] (증명이 코드보다 먼저 착지한다는 규율은
그 증명이 **실제로 돌 때만** 뜻이 있다).
