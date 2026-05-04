<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors -->

# Y4 Verus specs

Verus 는 Rust 코드와 정렬되는 모든 명세의 1차 도구다 (CLAUDE.md §6.6).
본 크레이트는 Verus CLI 의 검증 대상이다 — `cargo build` 는 syntax 체크
용이며 검증을 의미하지 않는다.

## 빠른 사용

```sh
just            # = just verify
just verify
```

또는 repo 루트에서 `just verus`.

## Verus 설치

Verus 는 자체 rustc fork 와 자체 표준 라이브러리(`vstd`) 를 가지므로
crates.io / cargo 와 분리되어 있다. 본 크레이트는 그래서 cargo
workspace 멤버가 **아니다** (root `Cargo.toml` 의 `exclude` 항목 참조).

**Arch Linux (현 개발 환경, 권장):**

```sh
yay -S verus-bin                   # AUR 패키지
verus --version                    # smoke check
```

`verus-bin` 은 `/opt/verus/` 에 다음을 설치한다:
- `libverus_builtin.rlib`, `libvstd.rlib`, `vstd.vir` — Verus 가 자동
  로드.
- `vstd/` — 표준 라이브러리 소스 (참고용).

별도 설정 / 환경변수 / Cargo dep 불필요. `use vstd::prelude::*;` +
`verus! { ... }` 매크로만 소스에 쓰면 verus CLI 가 알아서 link.

**기타 OS:**

```sh
git clone https://github.com/verus-lang/verus ~/verus
cd ~/verus/source
./tools/get-z3.sh                  # bundled Z3
./tools/local-install.sh           # rustc fork + verus CLI 빌드
export PATH=~/verus/source/target-verus/release:$PATH
verus --version
```

이 경로일 때도 vstd 는 verus 가 install prefix 안에서 자동 탐색.

## vstd 사용 패턴 (Y4 명세 작성 규칙)

모든 Y4 Verus 명세 파일은 다음 형태를 따른다:

```rust
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

use vstd::prelude::*;

verus! {
    // ---- spec / proof / exec functions here ----
    proof fn my_invariant() ensures /* ... */ { /* ... */ }
}
```

- `use vstd::prelude::*;` 는 매크로 외부에 둔다 (verus 가 import 단계에서
  처리).
- 모든 명세/구현은 `verus! { ... }` 안에. 매크로 밖의 일반 Rust 는 verus
  검증 대상이 아니다.
- 본 크레이트는 `verus --crate-type=lib src/lib.rs` 로 검증 — `--crate-type=lib`
  를 빠뜨리면 verus 가 binary 로 인식해 `main` 을 요구한다.

## 명세 작성 규칙

1. **모듈 경계 = 서브시스템 경계.** `src/{ipc,alloc,kernel,lease}/` 가
   각각 대응 서브시스템의 spec.
2. **`#[verifier::trusted]` 사용은 PR 설명에 정당화 필수.** trusted 는
   증명 boundary 의 끝 — 외부 ABI(예: HIU MMIO write) 만 허용.
3. **명세 PR 이 구현 PR 보다 먼저** 머지된다 (formal-first). 명세 없이
   구현이 들어오면 PR reject.
4. **Phase B step 3** 부터 ipc/alloc 의 첫 실제 명세가 들어온다. 그 전까지
   본 크레이트는 placeholder 만 호스트.

## 미해결 / 보류

- **Verus 버전 핀:** 첫 실제 spec PR 에서 `vstd` 의 git rev 를 결정해
  `Cargo.toml` 에 고정. 그 전까지는 system Verus 사용.
- **CI 자동화:** 현 단계에선 로컬 `just verus` 만. CI 게이트는 Phase B
  step 2 (boot) 직후 GitHub Actions / 자체 러너로 자동화.
