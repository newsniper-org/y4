# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

# Y4 의 1차 build orchestration 은 `justfile` (logicutils 와 통합).
# 본 Makefile 은 conventional `make <target>` interface 가 필요한 경우의
# thin facade.  대부분의 target 은 `just` 로 위임.

.PHONY: claude-resume

SESSION_ID_FILE := .claude-recent-session-id

# Resume the most recent Claude Code session whose ID was captured by the
# SessionStart hook in .claude/settings.json (writes to $(SESSION_ID_FILE)).
# Triggered manually after [Ctrl+C] x2 ends a session.
claude-resume:
	@if [ ! -s $(SESSION_ID_FILE) ]; then \
		echo "[claude-resume] $(SESSION_ID_FILE) is empty or missing."; \
		echo "[claude-resume] Has a Claude Code session been started in this repo?"; \
		exit 1; \
	fi
	@sid=$$(cat $(SESSION_ID_FILE)); \
	echo "[claude-resume] resuming session $$sid"; \
	exec claude --resume "$$sid"
