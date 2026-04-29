#!/usr/bin/env python3
"""Background AI recommendation engine for mac-cleaner.

The model is advisory only. This module validates every recommendation against
local safety facts before the server can display it or use it to trigger safe
RAM-only actions.
"""

from __future__ import annotations

import json
import os
from datetime import datetime
from pathlib import Path
from typing import Any

try:
    from dotenv import load_dotenv
except Exception:  # pragma: no cover - optional dependency path
    load_dotenv = None

try:
    from google import genai
except Exception:  # pragma: no cover - optional dependency path
    genai = None


MODEL = os.getenv("GEMINI_MODEL", "gemini-2.5-flash")
PROJECT_DIR = Path(__file__).parent
ALLOWED_ACTIONS = {"close_app", "review_app", "purge_ram", "clean_storage", "do_nothing"}
ALLOWED_RISKS = {"safe", "review", "danger"}
ALLOWED_PRIORITIES = {"high", "medium", "low"}
PROTECTED_LABELS = {
    "terminal",
    "iterm",
    "zsh",
    "bash",
    "vs code",
    "cursor ide",
    "claude code",
    "codex",
    "gemini cli",
    "node.js",
    "python",
}

RESPONSE_SCHEMA = {
    "type": "object",
    "properties": {
        "summary": {
            "type": "string",
            "description": "One concise sentence explaining the current optimization opportunity.",
        },
        "recommendations": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["close_app", "review_app", "purge_ram", "clean_storage", "do_nothing"],
                    },
                    "target_type": {
                        "type": "string",
                        "enum": ["process", "storage", "memory", "none"],
                    },
                    "target_id": {
                        "type": "string",
                        "description": "Process label or scanner item name from the snapshot.",
                    },
                    "target_label": {"type": "string"},
                    "priority": {"type": "string", "enum": ["high", "medium", "low"]},
                    "risk": {"type": "string", "enum": ["safe", "review", "danger"]},
                    "expected_savings_bytes": {"type": "integer"},
                    "reason": {
                        "type": "string",
                        "description": "Short explanation based only on the snapshot.",
                    },
                },
                "required": [
                    "action",
                    "target_type",
                    "target_id",
                    "target_label",
                    "priority",
                    "risk",
                    "expected_savings_bytes",
                    "reason",
                ],
            },
        },
    },
    "required": ["summary", "recommendations"],
}

TAB_RESPONSE_SCHEMA = {
    "type": "object",
    "properties": {
        "summary": {"type": "string"},
        "recommendations": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "window_index": {"type": "integer"},
                    "tab_index": {"type": "integer"},
                    "tab_id": {"type": "string"},
                    "priority": {"type": "string", "enum": ["high", "medium", "low"]},
                    "reason": {"type": "string"},
                },
                "required": ["window_index", "tab_index", "tab_id", "priority", "reason"],
            },
        },
    },
    "required": ["summary", "recommendations"],
}


def build_snapshot(system: dict[str, Any], ram_summary: dict[str, Any], storage_items: list[dict[str, Any]]) -> dict[str, Any]:
    """Return a compact, model-safe snapshot with no paths unless needed for labels."""
    processes = []
    for proc in ram_summary.get("processes", [])[:15]:
        processes.append({
            "label": proc.get("label", "unknown"),
            "category": proc.get("category", "other"),
            "can_close": bool(proc.get("can_close", False)),
            "memory_bytes": int(proc.get("mem_bytes", 0) or 0),
            "memory_human": proc.get("mem", ""),
            "count": int(proc.get("count", 1) or 1),
            "suggestion": proc.get("suggestion", ""),
        })

    storage = []
    for item in storage_items[:15]:
        storage.append({
            "name": item.get("name", "unknown"),
            "category": item.get("category", "unknown"),
            "safety": item.get("safety", "review-required"),
            "cleanable": bool(item.get("cleanable", False)),
            "size_bytes": int(item.get("size", 0) or 0),
            "size_human": item.get("size_human", ""),
            "reason": item.get("reason", ""),
        })

    return {
        "system": {
            "available_ram": int(system.get("available_ram", 0) or 0),
            "free_ram": int(system.get("free_ram", 0) or 0),
            "total_ram": int(system.get("total_ram", 0) or 0),
            "disk_available": system.get("disk_available", ""),
            "disk_percent": system.get("disk_percent", 0),
        },
        "ram": {
            "closeable_ram": int(ram_summary.get("closeable_ram", 0) or 0),
            "closeable_ram_human": ram_summary.get("closeable_ram_human", ""),
            "processes": processes,
        },
        "storage": storage,
    }


def fallback_recommendations(snapshot: dict[str, Any], reason: str = "Gemini unavailable") -> dict[str, Any]:
    """Generate deterministic recommendations when Gemini is unavailable or invalid."""
    recommendations = []
    for proc in snapshot.get("ram", {}).get("processes", []):
        if proc.get("can_close") and proc.get("memory_bytes", 0) >= 200 * 1024 * 1024:
            recommendations.append({
                "action": "review_app",
                "target_type": "process",
                "target_id": proc["label"],
                "target_label": proc["label"],
                "priority": "high" if proc["memory_bytes"] >= 700 * 1024 * 1024 else "medium",
                "risk": "review",
                "expected_savings_bytes": proc["memory_bytes"],
                "reason": proc.get("suggestion") or "High memory process; confirm it is not needed.",
            })

    for item in snapshot.get("storage", []):
        if item.get("cleanable") and item.get("safety") == "safe-cache" and item.get("size_bytes", 0) >= 100 * 1024 * 1024:
            recommendations.append({
                "action": "clean_storage",
                "target_type": "storage",
                "target_id": item["name"],
                "target_label": item["name"],
                "priority": "medium",
                "risk": "safe",
                "expected_savings_bytes": item["size_bytes"],
                "reason": item.get("reason") or "Safe cache target from scanner.",
            })

    if not recommendations:
        recommendations.append({
            "action": "do_nothing",
            "target_type": "none",
            "target_id": "",
            "target_label": "No urgent action",
            "priority": "low",
            "risk": "safe",
            "expected_savings_bytes": 0,
            "reason": "No high-confidence optimization target is visible.",
        })

    return {
        "status": "fallback",
        "provider": "local-rules",
        "model": None,
        "generated_at": datetime.now().isoformat(),
        "summary": reason,
        "recommendations": validate_recommendations({"recommendations": recommendations}, snapshot),
    }


def get_ai_recommendations(snapshot: dict[str, Any]) -> dict[str, Any]:
    """Ask Gemini for structured recommendations and validate the result."""
    if load_dotenv is not None:
        load_dotenv(PROJECT_DIR / ".env")

    api_key = os.getenv("GEMINI_API_KEY")
    if not api_key or genai is None:
        return fallback_recommendations(snapshot, "Gemini is not configured; showing local safe recommendations.")

    prompt = (
        "You are a local macOS optimization advisor. Use only this JSON snapshot. "
        "Recommend at most 5 actions. Be conservative. Never suggest closing protected "
        "developer tools or deleting report-only storage. Prefer review_app over close_app "
        "unless the process is clearly optional. Return JSON matching the schema.\n\n"
        f"SNAPSHOT:\n{json.dumps(snapshot, separators=(',', ':'))}"
    )

    try:
        client = genai.Client(api_key=api_key)
        response = client.models.generate_content(
            model=MODEL,
            contents=prompt,
            config={
                "response_mime_type": "application/json",
                "response_json_schema": RESPONSE_SCHEMA,
            },
        )
        raw = json.loads(response.text or "{}")
        recommendations = validate_recommendations(raw, snapshot)
        return {
            "status": "ok",
            "provider": "gemini",
            "model": MODEL,
            "generated_at": datetime.now().isoformat(),
            "summary": str(raw.get("summary") or "AI recommendations ready."),
            "recommendations": recommendations,
        }
    except Exception as exc:
        return fallback_recommendations(snapshot, f"Gemini request failed: {exc}")


def get_tab_recommendations(tab_snapshot: dict[str, Any]) -> dict[str, Any]:
    """Ask Gemini to rank Chrome tab cleanup candidates; fall back to local rules."""
    local_candidates = tab_snapshot.get("recommendations", [])
    if not tab_snapshot.get("available"):
        return {
            "status": "unavailable",
            "provider": "chrome",
            "model": None,
            "generated_at": datetime.now().isoformat(),
            "summary": tab_snapshot.get("error") or "Chrome tabs are not available.",
            "recommendations": [],
        }

    if load_dotenv is not None:
        load_dotenv(PROJECT_DIR / ".env")

    api_key = os.getenv("GEMINI_API_KEY")
    if not api_key or genai is None:
        return fallback_tab_recommendations(tab_snapshot, "Gemini unavailable; showing local Chrome tab recommendations.")

    safe_tabs = []
    for tab in tab_snapshot.get("tabs", [])[:80]:
        safe_tabs.append({
            "window_index": tab.get("window_index"),
            "tab_index": tab.get("tab_index"),
            "tab_id": tab.get("tab_id"),
            "title": tab.get("title"),
            "domain": tab.get("domain"),
            "active": tab.get("active"),
        })

    prompt = (
        "You are helping reduce Chrome memory usage. Rank only tabs that look safe to close. "
        "Prefer LOCAL_CANDIDATES, but you may also choose other non-active OPEN_TABS when the title/domain "
        "clearly suggests disposable or ad-heavy browsing such as converters, downloaders, spam, duplicate "
        "media, shopping, social, or blank tabs. Do not invent tabs. Avoid active tabs and work/dev tabs. "
        "Return JSON only.\n\n"
        f"OPEN_TABS:\n{json.dumps(safe_tabs, separators=(',', ':'))}\n\n"
        f"LOCAL_CANDIDATES:\n{json.dumps(local_candidates[:20], separators=(',', ':'))}"
    )

    try:
        client = genai.Client(api_key=api_key)
        response = client.models.generate_content(
            model=MODEL,
            contents=prompt,
            config={
                "response_mime_type": "application/json",
                "response_json_schema": TAB_RESPONSE_SCHEMA,
            },
        )
        raw = json.loads(response.text or "{}")
        recs = validate_tab_recommendations(raw.get("recommendations", []), tab_snapshot)
        return {
            "status": "ok",
            "provider": "gemini",
            "model": MODEL,
            "generated_at": datetime.now().isoformat(),
            "summary": str(raw.get("summary") or "Chrome tab recommendations ready."),
            "recommendations": recs,
        }
    except Exception as exc:
        return fallback_tab_recommendations(tab_snapshot, f"Gemini tab ranking failed: {exc}")


def fallback_tab_recommendations(tab_snapshot: dict[str, Any], reason: str) -> dict[str, Any]:
    return {
        "status": "fallback",
        "provider": "local-rules",
        "model": None,
        "generated_at": datetime.now().isoformat(),
        "summary": reason,
        "recommendations": validate_tab_recommendations(tab_snapshot.get("recommendations", []), tab_snapshot),
    }


def validate_tab_recommendations(recommendations: list[dict[str, Any]], tab_snapshot: dict[str, Any]) -> list[dict[str, Any]]:
    """Return tab recommendations that still match the current local snapshot."""
    by_id = {
        str(tab.get("tab_id")): tab
        for tab in tab_snapshot.get("tabs", [])
        if tab.get("tab_id")
    }
    by_position = {
        (int(tab.get("window_index") or 0), int(tab.get("tab_index") or 0)): tab
        for tab in tab_snapshot.get("tabs", [])
    }

    safe = []
    seen = set()
    for idx, rec in enumerate(recommendations[:12]):
        tab = None
        tab_id = str(rec.get("tab_id") or "")
        if tab_id:
            tab = by_id.get(tab_id)
        if tab is None:
            key = (int(rec.get("window_index") or 0), int(rec.get("tab_index") or 0))
            tab = by_position.get(key)
        if not tab or tab.get("active"):
            continue

        key = f"{tab.get('window_index')}:{tab.get('tab_index')}:{tab.get('tab_id')}"
        if key in seen:
            continue
        seen.add(key)

        priority = _enum(rec.get("priority"), ALLOWED_PRIORITIES, "medium")
        reason = str(rec.get("reason") or rec.get("reason_code") or "Review this tab before closing.")[:220]
        safe.append({
            "id": f"chrome-tab-rec-{idx + 1}",
            "action": "close_tab",
            "browser": "chrome",
            "window_index": tab.get("window_index"),
            "tab_index": tab.get("tab_index"),
            "tab_id": tab.get("tab_id", ""),
            "title": tab.get("title", "Untitled"),
            "domain": tab.get("domain", "unknown"),
            "url": tab.get("url", ""),
            "priority": priority,
            "risk": "review",
            "reason": reason,
            "executable": False,
        })
    return safe


def validate_recommendations(payload: dict[str, Any], snapshot: dict[str, Any]) -> list[dict[str, Any]]:
    """Normalize model output and enforce local safety constraints."""
    process_by_label = {
        str(proc.get("label", "")).lower(): proc
        for proc in snapshot.get("ram", {}).get("processes", [])
    }
    storage_by_name = {
        str(item.get("name", "")).lower(): item
        for item in snapshot.get("storage", [])
    }

    safe = []
    for idx, rec in enumerate(payload.get("recommendations", [])[:5]):
        if not isinstance(rec, dict):
            continue

        action = _enum(rec.get("action"), ALLOWED_ACTIONS, "review_app")
        target_type = _enum(rec.get("target_type"), {"process", "storage", "memory", "none"}, "none")
        target_id = str(rec.get("target_id") or "")
        target_label = str(rec.get("target_label") or target_id or "Recommendation")
        priority = _enum(rec.get("priority"), ALLOWED_PRIORITIES, "medium")
        risk = _enum(rec.get("risk"), ALLOWED_RISKS, "review")
        savings = _int_at_least_zero(rec.get("expected_savings_bytes"))
        reason = str(rec.get("reason") or "Review before taking action.")[:240]
        executable = False
        blocked_reason = ""

        if action == "review_app" and target_type == "process":
            proc = process_by_label.get(target_id.lower()) or process_by_label.get(target_label.lower())
            if proc and _is_protected_process(proc):
                risk = "danger"
                blocked_reason = "Protected developer/system process; manual review only."
                savings = int(proc.get("memory_bytes", savings) or savings)

        elif action == "review_app" and target_type == "storage":
            item = storage_by_name.get(target_id.lower()) or storage_by_name.get(target_label.lower())
            if item and not item.get("cleanable"):
                risk = "danger"
                blocked_reason = "Scanner marks this storage target as report-only."
                savings = int(item.get("size_bytes", savings) or savings)
            elif item and item.get("safety") != "safe-cache":
                risk = "review"
                blocked_reason = "Scanner requires review before cleaning this target."
                savings = int(item.get("size_bytes", savings) or savings)

        elif action == "close_app":
            proc = process_by_label.get(target_id.lower()) or process_by_label.get(target_label.lower())
            if not proc:
                action = "review_app"
                risk = "danger"
                blocked_reason = "AI referenced a process that is not in the current snapshot."
            elif _is_protected_process(proc):
                action = "review_app"
                risk = "danger"
                blocked_reason = "Protected developer/system process; manual review only."
                savings = int(proc.get("memory_bytes", savings) or savings)
            elif not proc.get("can_close"):
                action = "review_app"
                risk = "danger"
                blocked_reason = "Local classifier marks this process as not closeable."
                savings = int(proc.get("memory_bytes", savings) or savings)
            else:
                risk = "review"
                executable = False
                savings = int(proc.get("memory_bytes", savings) or savings)

        elif action == "clean_storage":
            item = storage_by_name.get(target_id.lower()) or storage_by_name.get(target_label.lower())
            if not item:
                action = "review_app"
                target_type = "storage"
                risk = "danger"
                blocked_reason = "AI referenced a storage item that is not in the current scanner snapshot."
            elif not item.get("cleanable"):
                action = "review_app"
                risk = "danger"
                blocked_reason = "Scanner marks this storage target as report-only."
                savings = int(item.get("size_bytes", savings) or savings)
            elif item.get("safety") != "safe-cache":
                risk = "review"
                blocked_reason = "Scanner requires review before cleaning this target."
                savings = int(item.get("size_bytes", savings) or savings)
            else:
                risk = "safe"
                executable = False
                savings = int(item.get("size_bytes", savings) or savings)

        elif action == "purge_ram":
            target_type = "memory"
            target_id = "purge_ram"
            target_label = "Purge inactive RAM"
            risk = "review"
            executable = False

        elif action == "do_nothing":
            target_type = "none"
            risk = "safe"
            savings = 0

        else:
            action = "review_app"
            risk = "review"

        if blocked_reason:
            reason = f"{blocked_reason} {reason}"

        safe.append({
            "id": f"ai-rec-{idx + 1}",
            "action": action,
            "target_type": target_type,
            "target_id": target_id,
            "target_label": target_label,
            "priority": priority,
            "risk": risk,
            "expected_savings_bytes": savings,
            "expected_savings": format_bytes(savings),
            "reason": reason,
            "executable": executable,
        })

    if not safe:
        safe.append({
            "id": "ai-rec-1",
            "action": "do_nothing",
            "target_type": "none",
            "target_id": "",
            "target_label": "No urgent action",
            "priority": "low",
            "risk": "safe",
            "expected_savings_bytes": 0,
            "expected_savings": "0 B",
            "reason": "No valid recommendation passed local safety validation.",
            "executable": False,
        })

    return safe


def format_bytes(size_bytes: int) -> str:
    if size_bytes < 1024:
        return f"{size_bytes} B"
    if size_bytes < 1024 ** 2:
        return f"{size_bytes / 1024:.1f} KB"
    if size_bytes < 1024 ** 3:
        return f"{size_bytes / 1024**2:.1f} MB"
    return f"{size_bytes / 1024**3:.2f} GB"


def _enum(value: Any, allowed: set[str], default: str) -> str:
    value = str(value or "")
    return value if value in allowed else default


def _int_at_least_zero(value: Any) -> int:
    try:
        return max(0, int(value or 0))
    except (TypeError, ValueError):
        return 0


def _is_protected_process(proc: dict[str, Any]) -> bool:
    label = str(proc.get("label", "")).lower()
    category = str(proc.get("category", "")).lower()
    return label in PROTECTED_LABELS or category == "system" or not bool(proc.get("can_close", False))
