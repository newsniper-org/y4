---
name: wavetensor-read-policy
description: "Y4 세션의 WaveTensor 소스 「읽기 전용 조건부」 허용 방침 (사용자 지시, 2026-10-01) — 6 조건 + 권위 순서 + clean-room + 요청/답장 채널."
metadata:
  node_type: memory
  type: feedback
  originSessionId: 38a83459-9d91-4355-910c-062288dd88aa
  modified: 2026-10-01T04:56:56.377Z
---

Y4 세션은 `/home/ybi/WaveTensor/` 소스를 **직접 읽어도 된다**. 조건은 아래 여섯이다.
출처: WaveTensor 세션이 보낸 `Y4/.local-requests-from/WaveTensor/2026-10-01-status-and-read-access.md` §1 (「사용자 지시」로 명시). 사용자가 그 파일을 확인하라고 한 뒤 이 방침의 기록을 지시했다(2026-10-01).

1. **읽기만 한다.**
   - WaveTensor 트리에서 파일을 쓰거나 고치지 않는다.
   - `git add/commit/checkout/switch/stash/worktree` 를 쓰지 않는다.
   - 트리 안에 산출물을 만드는 명령을 돌리지 않는다: `make rtl-gen` · `make rtl-sim` · `scripts/sby_sweep.sh`(증거 json 을 다시 씀) · 트리 안 metadir 를 쓰는 TLC.
   - 실행이 필요하면 `git -C /home/ybi/WaveTensor archive HEAD | tar -x -C /var/tmp/<고유>` 사본에서 한다.
   - WaveTensor 의 컨테이너 · `/tmp/wt-sby-out` · `/var/tmp/wt*` 는 건드리지 않는다.
2. **권위 순서.**
   - `docs/wt64v1-architecture-guide/`(main.typ → PDF)가 **유일한 정본**이다.
   - `rtl/` 재구현은 오늘의 실물이지 ABI 계약이 아니다. (요청문 §2 보충: `rtl/` 은 SpinalHDL 주 + SV 검증층이다.)
   - `.claude-memos/` 는 비정본이고 volatile 하다.
   - 루트 `*.v` / `tb_*.v` / `test_*.py` 는 **옛 참조 구현(legacy)** 이고 재구현으로 대체되는 중이다. ABI 근거로 쓰지 않는다. 요청문은 `HIU.v` 를 예로 든다(`Pod.v` · `Cluster.v` 도 같은 루트 파일이다 — Y4 측 보충).
3. **인용은 커밋을 고정한다.** `파일:줄` + 커밋 sha + 그 줄의 축자 문구를 함께 적는다. HEAD 와 정본 줄 번호가 자주 움직이기 때문이다.
4. **clean-room 보호.** WaveTensor 재구현은 clean-room 이고, 재구현 계보는 legacy 루트 파일을 읽지 않는다. Y4 세션이 legacy 를 읽었다면 그 내용(코드 · 신호명 · 계층 경로 · 모듈 구조)을 WaveTensor `rtl/` 이나 WaveTensor 세션 메시지로 옮기지 않는다. 판정과 결론만 보낸다. 오염은 *파일 접근이 아니라* 보고서를 통해 들어온다.
5. **쓰기 권한은 넓어지지 않는다.** CLAUDE.md §7 의 「ABI 교차 시에만 편집」은 그대로다. WaveTensor 쪽 변경 요청은 직접 편집 대신 Y4 의 요청 채널 관례로 보내기를 **권한다**(권고): Y4 `.local-requests-to/WaveTensor/` 에 쓰고 WaveTensor `.local-requests-from/Y4/` 로 거울한다.
6. **라이선스 경계.** WaveTensor 는 다중 라이선스다(HW SHL-2.1/CERN-OHL-W-2.0 · SW LGPL-2.1+/BSD-2/Apache-2.0 · 문서 CC-BY-4.0). 읽고 참조하는 것은 자유다. Y4(Apache-2.0)로 **복사**하려면 라이선스 판단이 먼저다.

(§1 밖, 요청문 끝 단락 :122) 답장은 Y4 `.local-replies-to/WaveTensor/` 에 쓰고 WaveTensor `.local-replies-from/Y4/` 로 거울한다.

**Why:** WaveTensor 는 2026-08-20 부터 정본 기준 clean-room RTL 재구현 중이다. legacy 기준 서술이 Y4 문서·메모리에 퍼져 있었다(2026-10-01 정정; [[wavetensor-terms]]).

**How to apply:**
- WaveTensor 사실이 필요하면 정본을 먼저 읽고, `rtl/` 은 「실물 상태」로만 인용한다.
- 공용 트리 대신 Explore/subagent 에 읽기 전용 조건을 명시한다.
- WaveTensor 로 가는 답장 초안에 legacy 신호명·구조를 넣지 않는다. Y4 자기 문서의 옛 용어를 지칭하는 것은 괜찮다. 관련 메모: [[y4-wavetensor-relation]].
