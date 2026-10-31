---
name: shared-stamp-store-trap
description: Y4 `just ci` gates shared one logicutils stamp store, so the first gate's `stamp record` made every later gate with the same inputs "up-to-date" without running — lint/test were hollow; fixed 2026-10-31 with a per-gate store.
metadata:
  type: feedback
---

`stamp` (logicutils) maps **file path → signature** in one store; `freshcheck --method=hash` only asks "did these files change since they were last recorded *in this store*". Y4's `_gate` used one store (`.cache/stamps`) for fmt-check, lint and test, which all hash the same Cargo.toml/*.rs list — so after fmt-check ran and recorded, lint and test were judged fresh and skipped even though they never ran on the new inputs. Found 2026-10-31: `lint.ok`/`test.ok` were hours older than inputs they claimed to cover. Fixed in the justfile: each gate uses `{{stamp_store}}/gate-<name>/`.

**Why:** a hollow gate is worse than no gate — it prints "up-to-date" and rc=0. Same family as [[just_first_recipe_trap]] (the Verus gate ran zero proofs for 84 days).

**How to apply:** any new hash-gated recipe must get its own store (pass through `_gate`, never call `freshcheck`/`stamp record` against the shared root). When a gate says "up-to-date", sanity-check that its `.ok` sentinel is newer than the inputs (`ls --time-style=+%T -la .cache/stamps/*.ok`). Note `freshcheck … $deps` needs word-splitting — run it from bash/sh, not zsh/fish, or it silently sees one bogus argument.
