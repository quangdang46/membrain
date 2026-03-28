#!/usr/bin/env python3
"""Persist Claude Code hook events into Membrain without breaking the session."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from typing import Any


SENSITIVE_KEY_PARTS = (
    "token",
    "secret",
    "password",
    "passwd",
    "authorization",
    "api_key",
    "apikey",
    "cookie",
    "session_token",
    "bearer",
    "private_key",
)

MAX_VALUE_LEN = 240
MAX_CONTENT_LEN = 900


def truncate(value: str, limit: int = MAX_VALUE_LEN) -> str:
    compact = " ".join(value.split())
    if len(compact) <= limit:
        return compact
    return compact[: limit - 3] + "..."


def is_sensitive_key(key: str) -> bool:
    lowered = key.lower()
    return any(part in lowered for part in SENSITIVE_KEY_PARTS)


def sanitize(value: Any, depth: int = 0) -> Any:
    if depth > 3:
        return "<depth-limited>"
    if isinstance(value, dict):
        sanitized: dict[str, Any] = {}
        for key, nested in value.items():
            if is_sensitive_key(key):
                sanitized[key] = "<redacted>"
            else:
                sanitized[key] = sanitize(nested, depth + 1)
        return sanitized
    if isinstance(value, list):
        return [sanitize(item, depth + 1) for item in value[:8]]
    if isinstance(value, str):
        return truncate(value)
    return value


def payload_text(payload: dict[str, Any], *keys: str) -> str | None:
    for key in keys:
        value = payload.get(key)
        if isinstance(value, str) and value.strip():
            return truncate(value)
    return None


def summarize_hook_payload(event: str, payload: dict[str, Any]) -> tuple[str, str]:
    session_id = payload_text(payload, "session_id", "sessionId")
    transcript_path = payload_text(payload, "transcript_path", "transcriptPath")
    cwd = payload_text(payload, "cwd")
    tool_name = payload_text(payload, "tool_name", "toolName")
    reason = payload_text(payload, "reason")
    matcher = payload_text(payload, "matcher")
    prompt = payload_text(payload, "prompt", "user_prompt", "userPrompt")
    notification = payload_text(payload, "message", "notification")

    sanitized_input = sanitize(payload.get("tool_input"))
    sanitized_response = sanitize(payload.get("tool_response"))
    sanitized_payload = sanitize(payload)

    parts = [f"Claude hook event={event}"]
    context_parts = [f"claude_hook event={event}"]

    if tool_name:
        parts.append(f"tool={tool_name}")
        context_parts.append(f"tool={tool_name}")
    if reason:
        parts.append(f"reason={reason}")
        context_parts.append(f"reason={reason}")
    if matcher:
        parts.append(f"matcher={matcher}")
    if session_id:
        parts.append(f"session_id={session_id}")
    if cwd:
        parts.append(f"cwd={cwd}")
        context_parts.append(f"cwd={cwd}")
    if transcript_path:
        parts.append(f"transcript_path={transcript_path}")

    if prompt:
        parts.append(f'prompt="{prompt}"')
    elif notification:
        parts.append(f'notification="{notification}"')

    if sanitized_input not in (None, {}, []):
        parts.append(
            "tool_input=" + truncate(json.dumps(sanitized_input, sort_keys=True), 280)
        )
    if sanitized_response not in (None, {}, []):
        parts.append(
            "tool_response=" + truncate(json.dumps(sanitized_response, sort_keys=True), 280)
        )

    if len(parts) <= 2:
        parts.append(
            "payload=" + truncate(json.dumps(sanitized_payload, sort_keys=True), 320)
        )

    content = truncate("; ".join(parts), MAX_CONTENT_LEN)
    context = truncate(" ".join(context_parts), 240)
    return content, context


def attention_for_event(event: str) -> str:
    return {
        "UserPromptSubmit": "0.95",
        "PostToolUseFailure": "0.85",
        "StopFailure": "0.85",
        "PermissionRequest": "0.75",
        "PostToolUse": "0.55",
        "PreToolUse": "0.35",
        "SessionStart": "0.4",
        "SessionEnd": "0.45",
        "SubagentStop": "0.5",
        "TaskCompleted": "0.6",
    }.get(event, "0.45")


def remember_event(
    membrain_bin: str,
    namespace: str,
    db_path: str | None,
    event: str,
    content: str,
    context: str,
) -> None:
    command = [
        membrain_bin,
        "remember",
        "--namespace",
        namespace,
        "--source",
        "claude_hook",
        "--attention",
        attention_for_event(event),
        "--context",
        context,
        "--quiet",
        content,
    ]
    if db_path:
        command[2:2] = ["--db-path", db_path]
    subprocess.run(
        command,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        timeout=20,
    )


def emit_additional_context(event: str, text: str) -> None:
    print(
        json.dumps(
            {
                "hookSpecificOutput": {
                    "hookEventName": event,
                    "additionalContext": text,
                }
            }
        )
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--event", required=True)
    parser.add_argument("--namespace", default=os.environ.get("MEMBRAIN_NAMESPACE", "default"))
    parser.add_argument("--db-path", default=os.environ.get("MEMBRAIN_DB_PATH"))
    parser.add_argument("--membrain-bin", default=os.environ.get("MEMBRAIN_BIN", "membrain"))
    parser.add_argument("--emit-additional-context")
    args = parser.parse_args()

    try:
        raw = sys.stdin.read()
        payload = json.loads(raw) if raw.strip() else {}
        if not isinstance(payload, dict):
            payload = {"payload": sanitize(payload)}
    except Exception:
        payload = {}

    try:
        content, context = summarize_hook_payload(args.event, payload)
        remember_event(
            membrain_bin=args.membrain_bin,
            namespace=args.namespace,
            db_path=args.db_path,
            event=args.event,
            content=content,
            context=context,
        )
        if args.emit_additional_context:
            emit_additional_context(args.event, args.emit_additional_context)
    except Exception:
        return 0

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
