---
name: Y4 formal-first verification rule
description: Specifications and proofs land before the privileged code they describe; Verus is the 1st-tier tool, Coq for invariants Verus cannot express.
type: feedback
---

**Rule:** Specifications and proofs must land **before** the privileged code they describe — not as follow-up work. PRs that introduce new privileged paths without their Verus/Coq artifacts do not merge.

**Why:** Y4 inherits seL4's verified microkernel base; the value of that base is destroyed if the Y4 specialization layer is unverified — the trust boundary collapses to wherever the unverified code starts. The user explicitly chose "선 정식증명 → 후 구현" as the verification stance (see `docs/architecture.md` §"Verification 방식"). Two industries that drive Y4's revenue (medical FDA 510(k), aviation DO-178C) will not certify post-hoc proofs.

**Tool priority:**
1. **Verus** — Rust-native, 1st-tier choice. Most Y4 code is Rust, so Verus annotations sit inline with the code being verified.
2. **Rocq (Coq)** — manual theories for high-level invariants Verus cannot express (e.g., system-wide capability transitivity, side-channel non-interference). **Isabelle/HOL** — auto-emitted from Verus via the `verus-fork` submodule (`-V emit-isabelle` → `proofs/isabelle/`, R7.7) as the seL4 L4.verified inbound contract (pipeline ready; emitted per-AV `.thy` = 0 as of 2026-10-02 — R7.11 user run pending). **Lean 4** — excluded for now (R1 = a', revisit at adsmt v1.1.x).
3. **Kani** — bounded model checking complement, useful for ruling out specific failure modes before Verus is asked to prove the general invariant.
4. **Frama-C / Why3** — only for the C portion (driver ports), if any.

**Exceptions to the formal-first gating:**
- Bug fixes that do not change specified behavior.
- Build tooling, CI configuration, dependency bumps.
- Documentation.
- Capsule ports of upstream code that already carries its own verified upstream trail (e.g., Tock-verified capsules) — but the integration glue still needs proofs.

**How to apply:**
- When suggesting a new Y4 module / capability / scheduling primitive, the recommendation must include the Verus signature (or Coq proposition) before the implementation skeleton.
- Do not approve "we'll verify later" PRs.
- When porting Linux drivers as 3rd-tier-fallback capsules, the integration boundary (the ABI between the GPL capsule and the Apache-2.0 main tree) needs its own non-interference proof — even if the GPL capsule itself is unverified.
