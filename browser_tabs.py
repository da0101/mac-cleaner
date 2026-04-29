#!/usr/bin/env python3
"""Chrome tab inventory and safe tab-close helpers."""

from __future__ import annotations

import json
import subprocess
from collections import defaultdict
from typing import Any
from urllib.parse import urlparse


OPTIONAL_MEDIA_DOMAINS = {"youtube.com", "x.com", "twitter.com", "reddit.com", "facebook.com", "instagram.com", "tiktok.com"}
CONVERTER_HINTS = ("youtube to mp3", "ytmp3", "y2mate", "mp3 converter", "video converter", "converter", "downloader")
CONVERTER_DOMAINS = ("ytmp3", "y2mate", "savefrom", "ssyoutube", "flvto", "onlinevideoconverter", "mp3juices")


JXA_CHROME_TABS = r'''
(() => {
  let chrome;
  try {
    chrome = Application("Google Chrome");
  } catch (e) {
    return JSON.stringify({available:false,error:String(e),tabs:[]});
  }
  try {
    if (!chrome.running()) {
      return JSON.stringify({available:false,error:"Google Chrome is not running",tabs:[]});
    }
    const tabs = [];
    chrome.windows().forEach((w, wi) => {
      let activeIndex = 0;
      try { activeIndex = w.activeTabIndex(); } catch (e) {}
      w.tabs().forEach((t, ti) => {
        let title = "";
        let url = "";
        let tabId = "";
        try { title = String(t.title()); } catch (e) {}
        try { url = String(t.url()); } catch (e) {}
        try { tabId = String(t.id()); } catch (e) {}
        tabs.push({
          browser: "chrome",
          window_index: wi + 1,
          tab_index: ti + 1,
          tab_id: tabId,
          title,
          url,
          active: activeIndex === ti + 1
        });
      });
    });
    return JSON.stringify({available:true,error:"",tabs});
  } catch (e) {
    return JSON.stringify({available:false,error:String(e),tabs:[]});
  }
})()
'''


def get_chrome_tabs(runner=subprocess.run) -> dict[str, Any]:
    """Return open Chrome tab metadata via JXA/AppleScript."""
    try:
        result = runner(
            ["osascript", "-l", "JavaScript", "-e", JXA_CHROME_TABS],
            capture_output=True,
            text=True,
            timeout=10,
        )
    except Exception as exc:
        return {"available": False, "error": str(exc), "tabs": [], "count": 0, "recommendations": []}

    if result.returncode != 0:
        return {
            "available": False,
            "error": (result.stderr or result.stdout or "Unable to inspect Chrome tabs").strip(),
            "tabs": [],
            "count": 0,
            "recommendations": [],
        }

    try:
        payload = json.loads(result.stdout.strip() or "{}")
    except json.JSONDecodeError:
        payload = {"available": False, "error": "Chrome tab script returned invalid JSON.", "tabs": []}

    tabs = [normalize_tab(tab) for tab in payload.get("tabs", [])]
    recommendations = recommend_tabs(tabs)
    return {
        "available": bool(payload.get("available", False)),
        "error": payload.get("error", ""),
        "tabs": tabs,
        "count": len(tabs),
        "recommendations": recommendations,
    }


def normalize_tab(tab: dict[str, Any]) -> dict[str, Any]:
    url = str(tab.get("url") or "")
    parsed = urlparse(url)
    domain = parsed.netloc.lower()
    if domain.startswith("www."):
        domain = domain[4:]
    safe_url = ""
    if parsed.scheme and parsed.netloc:
        safe_url = f"{parsed.scheme}://{parsed.netloc}{parsed.path}"
    elif url.startswith("chrome://"):
        safe_url = url.split("?", 1)[0]

    return {
        "browser": "chrome",
        "window_index": int(tab.get("window_index") or 0),
        "tab_index": int(tab.get("tab_index") or 0),
        "tab_id": str(tab.get("tab_id") or ""),
        "title": str(tab.get("title") or "Untitled")[:180],
        "url": safe_url[:300],
        "domain": domain or parsed.scheme or "unknown",
        "active": bool(tab.get("active", False)),
    }


def recommend_tabs(tabs: list[dict[str, Any]], max_items: int = 12) -> list[dict[str, Any]]:
    """Recommend low-risk tab cleanup candidates from local rules."""
    recommendations = []
    by_url = defaultdict(list)
    by_title_domain = defaultdict(list)

    for tab in tabs:
        by_url[tab["url"]].append(tab)
        by_title_domain[(tab["domain"], tab["title"].strip().lower())].append(tab)

    for group in list(by_url.values()) + list(by_title_domain.values()):
        if len(group) < 2:
            continue
        for tab in group[1:]:
            if tab["active"]:
                continue
            recommendations.append(tab_recommendation(tab, "duplicate", "Duplicate tab appears more than once."))

    for tab in tabs:
        title = tab["title"].strip().lower()
        url = tab["url"].lower()
        if tab["active"]:
            continue
        if title in {"new tab", "untitled"} or url in {"chrome://newtab/", "chrome://new-tab-page/"}:
            recommendations.append(tab_recommendation(tab, "empty", "Unused new/blank tab."))
        elif looks_like_converter_or_downloader(tab):
            recommendations.append(tab_recommendation(tab, "converter", "Likely ad-heavy media converter/downloader tab; review if not needed."))
        elif tab["domain"] in OPTIONAL_MEDIA_DOMAINS:
            recommendations.append(tab_recommendation(tab, "attention", "Likely optional media/social tab; review if not needed."))

    deduped = {}
    for rec in recommendations:
        key = tab_key(rec)
        deduped.setdefault(key, rec)
    return list(deduped.values())[:max_items]


def tab_recommendation(tab: dict[str, Any], reason_code: str, reason: str) -> dict[str, Any]:
    return {
        "browser": "chrome",
        "window_index": tab["window_index"],
        "tab_index": tab["tab_index"],
        "tab_id": tab.get("tab_id", ""),
        "title": tab["title"],
        "domain": tab["domain"],
        "url": tab["url"],
        "reason_code": reason_code,
        "reason": reason,
        "risk": "review",
    }


def tab_key(tab: dict[str, Any]) -> str:
    return f"{tab.get('window_index')}:{tab.get('tab_index')}:{tab.get('tab_id') or tab.get('url')}"


def looks_like_converter_or_downloader(tab: dict[str, Any]) -> bool:
    haystack = " ".join([
        str(tab.get("title", "")),
        str(tab.get("domain", "")),
        str(tab.get("url", "")),
    ]).lower()
    domain = str(tab.get("domain", "")).lower()
    if any(hint in domain for hint in CONVERTER_DOMAINS):
        return True
    if "mp3" in haystack and any(term in haystack for term in ("youtube", "yt", "converter", "download")):
        return True
    return any(hint in haystack for hint in CONVERTER_HINTS)


def close_chrome_tab(window_index: int, tab_index: int, runner=subprocess.run) -> dict[str, Any]:
    """Close a Chrome tab by fresh window/tab indices."""
    script = (
        'tell application "Google Chrome"\n'
        f'  close tab {int(tab_index)} of window {int(window_index)}\n'
        'end tell\n'
    )
    try:
        result = runner(["osascript", "-e", script], capture_output=True, text=True, timeout=10)
        return {
            "success": result.returncode == 0,
            "error": "" if result.returncode == 0 else (result.stderr or result.stdout).strip(),
        }
    except Exception as exc:
        return {"success": False, "error": str(exc)}


def find_matching_tab(snapshot: dict[str, Any], requested: dict[str, Any]) -> dict[str, Any] | None:
    """Find the requested tab in a fresh snapshot before closing."""
    requested_id = str(requested.get("tab_id") or "")
    requested_window = int(requested.get("window_index") or 0)
    requested_tab = int(requested.get("tab_index") or 0)
    requested_title = str(requested.get("title") or "")
    requested_domain = str(requested.get("domain") or "")
    requested_url = str(requested.get("url") or "")

    for tab in snapshot.get("tabs", []):
        if requested_id and tab.get("tab_id") == requested_id:
            return tab

    if requested_id:
        return None

    for tab in snapshot.get("tabs", []):
        if tab.get("window_index") == requested_window and tab.get("tab_index") == requested_tab:
            if requested_title and requested_title != tab.get("title"):
                return None
            if requested_domain and requested_domain != tab.get("domain"):
                return None
            if requested_url and requested_url != tab.get("url"):
                return None
            return tab
    return None
